use std::collections::HashMap;
use tokio::sync::{RwLock, broadcast, mpsc};
use uuid::Uuid;
use chrono;
use rand::{Rng, rng};
use crate::game::messages::{GameMessage, WebSocketMessage, GameMode};
use crate::game::types::{Player, Game};
use crate::game::queue::{GameProcessor, QueuedMessage, BroadcastMessageChunk};
use crate::error::{ErrorResponse, ErrorCode};
use serde_json;
use std::sync::Arc;
use tokio::task::JoinHandle;
use crate::cache::redis_client::RedisClient;
use crate::cache::key_builder::key;
use crate::team_draft::state::TeamDraftService;
use crate::mind_match::state::MindMatchService;
use crate::game::game_mode::GameModeManager;
use futures::StreamExt;

pub const MAX_PLAYERS: u8 = 8;

pub struct GameEntry {
    pub game: Game,
    pub auth_token: String,
}
pub type GameResult<T> = Result<T, ErrorResponse>;

pub struct GameManager {
    redis_client: RedisClient,
    game_broadcasters: RwLock<HashMap<String, broadcast::Sender<String>>>,
    
    // for message processing - just stores the senders
    game_processors: Arc<RwLock<HashMap<String, mpsc::Sender<QueuedMessage>>>>, // game_id -> sender
    processor_handles: RwLock<HashMap<String, JoinHandle<()>>>, // game_id -> processing task handle
    
    // Optional pub/sub task handle (set after construction)
    pubsub_routing_handle: RwLock<Option<JoinHandle<()>>>,

    // Game services
    team_draft_service: TeamDraftService,
    mind_match_service: MindMatchService,
}

impl GameManager {
    pub async fn new() -> Self {
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
            
        let redis_client = RedisClient::new(redis_url).await.unwrap();
        let team_draft_service = TeamDraftService::new(redis_client.clone(), GameMode::TeamDraft);
        let mind_match_service = MindMatchService::new(redis_client.clone(), GameMode::MindMatch);

        let game_processors: Arc<RwLock<HashMap<String, mpsc::Sender<QueuedMessage>>>> = 
            Arc::new(RwLock::new(HashMap::new()));
        
        Self {
            redis_client,
            team_draft_service,
            mind_match_service,
            game_broadcasters: RwLock::new(HashMap::new()),
            game_processors,
            processor_handles: RwLock::new(HashMap::new()),
            pubsub_routing_handle: RwLock::new(None),
        }
    }
    
    pub async fn start_pubsub(self: Arc<Self>) -> GameResult<()> {
        let mut pubsub = self.redis_client.create_shared_pubsub(vec!["game_channel::*".to_string()]).await
            .map_err(|e| ErrorResponse {
                error: ErrorCode::InternalServerError,
                message: format!("Failed to create pub/sub connection: {}", e),
            })?;
        
        let game_manager_clone = self.clone();
        
        // BACKGROUND TASK - route messages from Redis pub/sub
        let pubsub_routing_handle = tokio::spawn(async move {
            while let Some(msg) = pubsub.on_message().next().await {
                let channel = msg.get_channel_name();
                let payload: String = match msg.get_payload() {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                println!("[GameManager] Received broadcast message on channel: {}", channel);
                
                if let Some(_game_id) = channel.strip_prefix("game_channel::") {
                    if let Ok(broadcast_chunk) = serde_json::from_str::<BroadcastMessageChunk>(&payload) {
                        println!("[GameManager] Processing {} broadcast messages for game {}", 
                            broadcast_chunk.messages.len(), broadcast_chunk.game_id);
                        
                        if let Err(e) = game_manager_clone.process_broadcast_messages(broadcast_chunk).await {
                            println!("[GameManager] Error processing broadcast messages: {}", e);
                        }
                    } else {
                        println!("[GameManager] Failed to deserialize broadcast message chunk: {}", payload);
                    }
                }
            }
            println!("[GameManager] Pub/sub routing task shutting down");
        });
        
        *self.pubsub_routing_handle.write().await = Some(pubsub_routing_handle);
        Ok(())
    }

    fn generate_game_code(&self) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNPQRSTUVWXYZ123456789"; // no O or 0
        let mut rng = rng();
        
        (0..6)
            .map(|_| {
                let idx = rng.random_range(0..CHARS.len());
                CHARS[idx] as char
            })
            .collect()
    }

    pub async fn is_authorized(&self, player_id: &str, auth_token: &str) -> GameResult<bool> {
        let auth_key = key("player_auth")?.field(player_id)?.get_key()?;
        match self.redis_client.get(&auth_key).await {
            Ok(Some(stored_token)) => Ok(stored_token == auth_token),
            Ok(None) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_game_mode_service(&self, game_mode: GameMode) -> &dyn GameModeManager {
        match game_mode {
            GameMode::TeamDraft => &self.team_draft_service,
            GameMode::MindMatch => &self.mind_match_service,
        }
    }

    pub async fn create_game(&self, host_username: String) -> GameResult<GameEntry> {
        if host_username.trim().is_empty() {
            return Err(ErrorResponse{
                error: ErrorCode::InvalidInput("Username cannot be empty".to_string()),
                message: "Username cannot be empty".to_string(),
            });
        }

        let game_id = Uuid::new_v4().to_string();
        let host_id = Uuid::new_v4().to_string();
        let auth_token = Uuid::new_v4().to_string();

        let game_code = loop {
            let code = self.generate_game_code();
            let key_string = key("game_code")?.field(code.clone())?.get_key()?;
            
            match self.redis_client.get(&key_string).await {
                Ok(Some(_)) => {
                    continue;
                }
                Ok(None) => {
                    break code;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        };

        let host_player = Player {
            id: host_id.clone(),
            username: host_username.clone(),
        };

        let player_key = key("player_to_game")?.field(&host_id)?.get_key()?;
        if let Ok(Some(_)) = self.redis_client.get(&player_key).await {
            return Err(ErrorResponse{
                error: ErrorCode::PlayerAlreadyInGame,
                message: "Player already in a game".to_string(),
            });
        }

        self.init_game_cached(&game_id, &game_code, &host_player).await?;

        let auth_key = key("player_auth")?.field(&host_id)?.get_key()?;
        self.redis_client.set(&auth_key, &auth_token).await?;

        self.redis_client.set(&player_key, &game_id).await?;
        
        let key_string = key("game_code")?.field(&game_code)?.get_key()?;
        self.redis_client.set(&key_string, &game_id).await?;

        let mut game_broadcasters = self.game_broadcasters.write().await;
        let (tx, _) = broadcast::channel(100);
        game_broadcasters.insert(game_id.clone(), tx);

        let game = Game {
            id: game_id.clone(),
            code: game_code.clone(),
            host_id: host_id.clone(),
            players: vec![host_player],
            max_players: MAX_PLAYERS,
            created_at: chrono::Utc::now().timestamp() as i32,
        };

        Ok(GameEntry {
            game,
            auth_token,
        })
    }

    pub async fn join_game_by_code(&self, player_username: String, game_code: String) -> GameResult<GameEntry> {
        if player_username.trim().is_empty() {
            return Err(ErrorResponse{
                error: ErrorCode::InvalidInput("Username cannot be empty".to_string()),
                message: "Username cannot be empty".to_string(),
            });
        }

        let game_code = game_code.to_uppercase();

        let game_id = {
            let key_string = key("game_code")?.field(&game_code)?.get_key()?;
                
            self.redis_client.get_required(&key_string, ErrorResponse{
                error: ErrorCode::GameNotFound,
                message: "Game not found".to_string(),
            }).await?
        };

        let current_players = self.get_players_cached(&game_id).await?;
        for player in current_players {
            if player.username == player_username {
                return Err(ErrorResponse{
                    error: ErrorCode::PlayerAlreadyInGame,
                    message: "Username taken.".to_string(),
                });
            }
        }

        self.join_game(player_username, game_id).await
    }

    pub async fn join_game(&self, player_username: String, game_id: String) -> GameResult<GameEntry> {
        if player_username.trim().is_empty() {
            return Err(ErrorResponse{
                error: ErrorCode::InvalidInput("Username cannot be empty".to_string()),
                message: "Username cannot be empty".to_string(),
            });
        }

        let player_id = Uuid::new_v4().to_string();
        let auth_token = Uuid::new_v4().to_string();
        let player = Player {
            id: player_id.clone(),
            username: player_username.clone(),
        };

        let player_key = key("player_to_game")?.field(&player_id)?.get_key()?;
        if let Ok(Some(_)) = self.redis_client.get(&player_key).await {
            return Err(ErrorResponse{
                error: ErrorCode::PlayerAlreadyInGame,
                message: "Player already in a game".to_string(),
            });
        }

        let max_players_key = key("game")?.field(&game_id)?.field("max_players")?.get_key()?;
        let max_players: u8 = match self.redis_client.get(&max_players_key).await? {
            Some(max_str) => max_str.parse().unwrap_or(MAX_PLAYERS),
            None => return Err(ErrorResponse{
                error: ErrorCode::GameNotFound,
                message: "Game not found".to_string(),
            }),
        };

        let current_players = self.get_players_cached(&game_id).await?;
        if current_players.len() >= max_players as usize {
            return Err(ErrorResponse{
                error: ErrorCode::GameFull,
                message: "Game is full".to_string(),
            });
        }

        self.add_player_cached(&game_id, &player).await?;

        let auth_key = key("player_auth")?.field(&player_id)?.get_key()?;
        self.redis_client.set(&auth_key, &auth_token).await?;
        self.redis_client.set(&player_key, &game_id).await?;

        let ws_message = WebSocketMessage {
            game_id: game_id.clone(),
            message: GameMessage::PlayerJoined {
                username: player_username.clone(),
                player_id: player_id.clone(),
            },
            player_id: player_id.clone(),
            auth_token: None,
        };

        if let Err(e) = self.broadcast_to_game(&game_id, ws_message).await {
            println!("Failed to broadcast player joined message: {}", e);
        }

        let updated_players = self.get_players_cached(&game_id).await?;
        let code_key = key("game")?.field(&game_id)?.field("code")?.get_key()?;
        let host_key = key("game")?.field(&game_id)?.field("host_id")?.get_key()?;
        let created_at_key = key("game")?.field(&game_id)?.field("created_at")?.get_key()?;

        let game_code: String = self.redis_client.get_required(&code_key, ErrorResponse{
            error: ErrorCode::GameNotFound,
            message: "Game not found".to_string(),
        }).await?;

        let host_id: String = self.redis_client.get_required(&host_key, ErrorResponse{
            error: ErrorCode::GameNotFound,
            message: "Game not found".to_string(),
        }).await?;

        let created_at: i32 = self.redis_client.get(&created_at_key).await?
            .unwrap_or_else(|| chrono::Utc::now().timestamp().to_string())
            .parse()
            .unwrap_or_else(|_| chrono::Utc::now().timestamp() as i32);

        let updated_game = Game {
            id: game_id.clone(),
            code: game_code,
            host_id: host_id.clone(),
            players: updated_players,
            max_players,
            created_at,
        };

        Ok(GameEntry {
            game: updated_game,
            auth_token,
        })
    }

    pub async fn get_broadcaster(&self, game_id: &str) -> GameResult<broadcast::Sender<String>> {
        let broadcasters = self.game_broadcasters.read().await;
        broadcasters
            .get(game_id)
            .cloned()
            .ok_or(ErrorResponse{
                error: ErrorCode::GameNotFound,
                message: "Game not found".to_string(),
            })
    }

    pub async fn broadcast_to_game(&self, game_id: &str, ws_message: WebSocketMessage) -> GameResult<()> {
        if ws_message.auth_token.is_some() {
            return Err(ErrorResponse {
                error: ErrorCode::InternalServerError,
                message: "Cannot broadcast message with auth token to players - this would be unsafe".to_string(),
            });
        }

        let message_json = serde_json::to_string(&ws_message)?;
        
        let broadcaster = self.get_broadcaster(game_id).await?;
        let _ = broadcaster.send(message_json);
        Ok(())
    }

    pub async fn remove_player_from_game(&self, player_id: &str) -> GameResult<()> {
        let player_key = key("player_to_game")?.field(player_id)?.get_key()?;
        let game_id = match self.redis_client.get(&player_key).await {
            Ok(Some(id)) => id,
            Ok(None) => return Ok(()),
            Err(e) => return Err(e.into()),
        };

        let player_info = self.get_player_cached(&game_id, player_id).await?;

        self.remove_player_cached(&game_id, player_id).await?;

        let auth_key = key("player_auth")?.field(player_id)?.get_key()?;
        self.redis_client.del(&auth_key).await?;

        let remaining_players = self.get_players_cached(&game_id).await?;
        let should_cleanup_game = remaining_players.is_empty();

        if let Some(player) = player_info {
            let player_disconnected = GameMessage::PlayerDisconnected {
                username: player.username,
                player_id: player.id.clone(),
            };
            let ws_message = WebSocketMessage {
                game_id: game_id.clone(),
                message: player_disconnected,
                player_id: player.id,
                auth_token: None,
            };

            let _ = self.broadcast_to_game(&game_id, ws_message).await;
        }

        if should_cleanup_game {
            self.cleanup_empty_game(&game_id).await?;
        }

        Ok(())
    }

    async fn cleanup_empty_game(&self, game_id: &str) -> GameResult<()> {
        let code_key = key("game")?.field(game_id)?.field("code")?.get_key()?;
        let game_code = self.redis_client.get(&code_key).await?;

        let players = self.get_players_cached(game_id).await.unwrap_or_default();
        let player_count = players.len();

        for player in &players {
            let player_to_game_key = key("player_to_game")?.field(&player.id)?.get_key()?;
            let player_auth_key = key("player_auth")?.field(&player.id)?.get_key()?;
            
            if let Err(e) = self.redis_client.del(&player_to_game_key).await {
                println!("[GameManager] Warning: Failed to clean up player_to_game for {}: {}", player.id, e);
            }
            
            if let Err(e) = self.redis_client.del(&player_auth_key).await {
                println!("[GameManager] Warning: Failed to clean up player_auth for {}: {}", player.id, e);
            }
        }

        let mut game_broadcasters = self.game_broadcasters.write().await;
        game_broadcasters.remove(game_id);

        let game_base_key = key("game")?.field(game_id)?.get_key()?;
        let game_pattern = format!("{}*", game_base_key);
        self.redis_client.pdel(&game_pattern).await?;

        GameModeManager::cleanup_state_cached(&self.team_draft_service, game_id.to_string()).await?;
        
        if let Some(code) = game_code {
            let key_string = key("game_code")?.field(&code)?.get_key()?;
            self.redis_client.del(&key_string).await?;
        }

        self.cleanup_game_processor(game_id).await;

        println!("[GameManager] Cleaned up empty game: {} (removed {} players)", game_id, player_count);
        Ok(())
    }

    // we just rebroadcast as a chat message, since we assume the client will disconnect, which triggers other code.
    pub async fn handle_player_left(&self, game_id: &str, player_id: &str) -> GameResult<()> {
        let player_info = self.get_player_cached(game_id, player_id).await?;

        if let Some(player) = player_info {
            let chat_message = GameMessage::ChatMessage {
                username: "System".to_string(),
                message: format!("{} left the game", player.username),
            };
            let ws_message = WebSocketMessage {
                game_id: game_id.to_string(),
                message: chat_message,
                player_id: player_id.to_string(),
                auth_token: None,
            };

            let _ = self.broadcast_to_game(game_id, ws_message).await;
        }

        Ok(())
    }

    pub async fn get_all_games(&self) -> GameResult<Vec<Game>> {
        let game_pattern = "game::*::code";
        let game_keys = self.redis_client.scan_keys(game_pattern).await?;
        
        let mut games = Vec::new();
        let mut seen_game_ids = std::collections::HashSet::new();
        
        for key in game_keys {
            if let Some(game_id) = self.extract_game_id_from_key(&key) {
                if seen_game_ids.insert(game_id.clone()) {
                    if let Ok(Some(game)) = self.get_game(&game_id).await {
                        games.push(game);
                    }
                }
            }
        }
        
        Ok(games)
    }
    
    fn extract_game_id_from_key(&self, key: &str) -> Option<String> {
        let parts: Vec<&str> = key.split("::").collect();
        if parts.len() >= 3 && parts[0] == "game" && parts[2] == "code" {
            Some(parts[1].to_string())
        } else {
            None
        }
    }
    
    pub async fn get_game(&self, game_id: &str) -> GameResult<Option<Game>> {
        let code_key = key("game")?.field(game_id)?.field("code")?.get_key()?;
        let game_code = match self.redis_client.get(&code_key).await? {
            Some(code) => code,
            None => return Ok(None),
        };

        let host_key = key("game")?.field(game_id)?.field("host_id")?.get_key()?;
        let max_players_key = key("game")?.field(game_id)?.field("max_players")?.get_key()?;
        let created_at_key = key("game")?.field(game_id)?.field("created_at")?.get_key()?;

        let host_id: String = self.redis_client.get_required(&host_key, ErrorResponse{
            error: ErrorCode::GameNotFound,
            message: "Game host not found".to_string(),
        }).await?;

        let max_players: u8 = self.redis_client.get(&max_players_key).await?
            .unwrap_or_else(|| MAX_PLAYERS.to_string())
            .parse()
            .unwrap_or(MAX_PLAYERS);

        let created_at: i32 = self.redis_client.get(&created_at_key).await?
            .unwrap_or_else(|| chrono::Utc::now().timestamp().to_string())
            .parse()
            .unwrap_or_else(|_| chrono::Utc::now().timestamp() as i32);

        let players = self.get_players_cached(game_id).await?;

        let game = Game {
            id: game_id.to_string(),
            code: game_code,
            host_id,
            players,
            max_players,
            created_at,
        };

        Ok(Some(game))
    }

    pub async fn ensure_game_processor_with_manager(
        game_manager: Arc<GameManager>, 
        game_id: &str
    ) -> GameResult<()> {
        let mut processors = game_manager.game_processors.write().await;
        let mut handles = game_manager.processor_handles.write().await;
        
        if !processors.contains_key(game_id) {
            let (processor, rx) = GameProcessor::new();
            
            let game_id_clone = game_id.to_string();
            let game_manager_clone = game_manager.clone();
            let handle = tokio::spawn(async move {
                GameProcessor::process_messages(rx, game_id_clone, game_manager_clone).await;
            });
            
            processors.insert(game_id.to_string(), processor.sender);
            handles.insert(game_id.to_string(), handle);
        }
        Ok(())
    }
    
    pub async fn enqueue_message_with_manager(
        game_manager: Arc<GameManager>,
        game_id: &str, 
        message: WebSocketMessage
    ) -> GameResult<()> {
        GameManager::ensure_game_processor_with_manager(game_manager.clone(), game_id).await?;
        
        let processors = game_manager.game_processors.read().await;
        if let Some(sender) = processors.get(game_id) {
            let queued_message = QueuedMessage {
                timestamp: std::time::Instant::now(),
                message,
            };
            
            sender.send(queued_message).await
                .map_err(|_| ErrorResponse {
                    error: ErrorCode::InternalServerError,
                    message: "Game processor channel closed".to_string(),
                })?;
        }
        Ok(())
    }
    
    pub async fn cleanup_game_processor(&self, game_id: &str) {
        let mut processors = self.game_processors.write().await;
        let mut handles = self.processor_handles.write().await;
        
        if let Some(sender) = processors.remove(game_id) {
            drop(sender); // Close the channel - this makes rx.recv() return None
        }
        
        if let Some(handle) = handles.remove(game_id) {
            handle.abort(); // Force stop the task (backup in case it's stuck)
        }
    }

    pub async fn publish_broadcast_messages(&self, game_id: &str, player_id: &str, messages: Vec<GameMessage>) -> GameResult<()> {
        if messages.is_empty() {
            return Ok(());
        }

        let broadcast_chunk = BroadcastMessageChunk {
            game_id: game_id.to_string(),
            player_id: player_id.to_string(),
            messages,
        };

        let message_json = serde_json::to_string(&broadcast_chunk)?;

        let channel = format!("game_channel::{}", game_id);
        self.redis_client.publish(&channel, &message_json).await?;

        println!("[GameManager] Published {} messages to Redis for game {}", broadcast_chunk.messages.len(), game_id);
        Ok(())
    }

    pub async fn process_broadcast_messages(&self, broadcast_chunk: BroadcastMessageChunk) -> GameResult<()> {
        for game_message in broadcast_chunk.messages {
            println!("[GameManager] Processing broadcast message: {:?}", game_message);
            match &game_message {
                GameMessage::HaltTimer(halt_timer) => {
                    let broadcast_ws_message = WebSocketMessage {
                        game_id: broadcast_chunk.game_id.clone(),
                        message: game_message.clone(),
                        player_id: broadcast_chunk.player_id.clone(),
                        auth_token: None,
                    };
                    
                    self.broadcast_to_game(&broadcast_chunk.game_id, broadcast_ws_message).await?;
                    
                    let current_time_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;
                    
                    if halt_timer.end_timestamp_ms > current_time_ms {
                        let sleep_duration_ms = halt_timer.end_timestamp_ms - current_time_ms;
                        println!("[GameManager] Halting for {} ms until timestamp {}", sleep_duration_ms, halt_timer.end_timestamp_ms);
                        tokio::time::sleep(tokio::time::Duration::from_millis(sleep_duration_ms)).await;
                    } else {
                        println!("[GameManager] Timer already expired, continuing immediately");
                    }
                }
                _ => {
                    println!("[GameManager] Broadcasting regular message: {:?}", game_message);
                    let broadcast_ws_message = WebSocketMessage {
                        game_id: broadcast_chunk.game_id.clone(),
                        message: game_message.clone(),
                        player_id: broadcast_chunk.player_id.clone(),
                        auth_token: None,
                    };
                    
                    self.broadcast_to_game(&broadcast_chunk.game_id, broadcast_ws_message).await?;
                }
            }
        }
        Ok(())
    }

    // HELPERS FOR CACHE STATE

    async fn init_game_cached(&self, game_id: &str, game_code: &str, host_player: &Player) -> GameResult<()> {
        let code_key = key("game")?.field(game_id)?.field("code")?.get_key()?;
        self.redis_client.set(&code_key, game_code).await?;
        
        let host_key = key("game")?.field(game_id)?.field("host_id")?.get_key()?;
        self.redis_client.set(&host_key, &host_player.id).await?;

        self.add_player_cached(game_id, host_player).await?;

        let max_players_key = key("game")?.field(game_id)?.field("max_players")?.get_key()?;
        self.redis_client.set(&max_players_key, &MAX_PLAYERS).await?;

        let created_at_key = key("game")?.field(game_id)?.field("created_at")?.get_key()?;
        self.redis_client.set(&created_at_key, chrono::Utc::now().timestamp() as i32).await?;

        // Initialize team draft state
        GameModeManager::init_state_cached(&self.team_draft_service, game_id.to_string(), host_player.clone()).await?;

        Ok(())
    }

    async fn get_players_cached(&self, game_id: &str) -> GameResult<Vec<Player>> {
        let players_key = key("game")?.field(game_id)?.field("players")?.get_key()?;
        
        let player_ids = self.redis_client.lrange(&players_key, 0, -1).await?;
        
        let mut players = Vec::new();
        for player_id in player_ids {
            let username_key = key("player_usernames")?.field(&player_id)?.get_key()?;
            match self.redis_client.get(&username_key).await? {
                Some(username) => {
                    players.push(Player {
                        id: player_id,
                        username,
                    });
                }
                None => {
                    println!("Warning: Player {} found in game {} but no username found", player_id, game_id);
                }
            }
        }
        
        Ok(players)
    }

    async fn add_player_cached(&self, game_id: &str, player: &Player) -> GameResult<()> {
        let players_key = key("game")?.field(game_id)?.field("players")?.get_key()?;
        self.redis_client.rpush(&players_key, &player.id).await?;
        
        let username_key = key("player_usernames")?.field(&player.id)?.get_key()?;
        self.redis_client.set(&username_key, &player.username).await?;
        
        Ok(())
    }

    async fn remove_player_cached(&self, game_id: &str, player_id: &str) -> GameResult<()> {
        let players_key = key("game")?.field(game_id)?.field("players")?.get_key()?;
        self.redis_client.lrem(&players_key, 1, player_id).await?;
        
        let username_key = key("player_usernames")?.field(player_id)?.get_key()?;
        self.redis_client.del(&username_key).await?;
        
        Ok(())
    }

    async fn get_player_cached(&self, game_id: &str, player_id: &str) -> GameResult<Option<Player>> {
        let players_key = key("game")?.field(game_id)?.field("players")?.get_key()?;
        let player_ids = self.redis_client.lrange(&players_key, 0, -1).await?;
        
        if player_ids.contains(&player_id.to_string()) {
            let username_key = key("player_usernames")?.field(player_id)?.get_key()?;
            match self.redis_client.get(&username_key).await? {
                Some(username) => Ok(Some(Player {
                    id: player_id.to_string(),
                    username,
                })),
                None => Ok(None),
            }
        } else {
            Ok(None)
        }
    }
}