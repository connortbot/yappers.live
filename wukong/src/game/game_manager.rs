use std::collections::HashMap;
use tokio::sync::{RwLock, broadcast, mpsc};
use uuid::Uuid;
use chrono;
use serde::Serialize;
use rand::{Rng, rng};
use crate::game::messages::{GameMessage, WebSocketMessage};
use crate::game::queue::{GameProcessor, QueuedMessage};
use crate::error::{ErrorResponse, ErrorCode, REDIS_ERROR};
use serde_json;
use utoipa::ToSchema;
use serde::Deserialize;
use ts_rs::TS;
use crate::team_draft::state::TeamDraftManager;
use std::sync::Arc;
use tokio::task::JoinHandle;
use crate::cache::redis_client::RedisClient;
use crate::cache::key_builder::key;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct Player {
    pub id: String,
    pub username: String,
    // stream?
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct Game {
    pub id: String,
    pub code: String,
    pub host_id: String,
    pub players: Vec<Player>,
    pub max_players: u8,
    pub created_at: i32,
    
    // Game mode managers
    pub team_draft: TeamDraftManager,
}

pub struct GameEntry {
    pub game: Game,
    pub auth_token: String,
}
pub type GameResult<T> = Result<T, ErrorResponse>;

pub struct GameManager {
    redis_client: RedisClient,
    
    games: RwLock<HashMap<String, Game>>,
    game_broadcasters: RwLock<HashMap<String, broadcast::Sender<String>>>,
    
    // for message processing - just stores the senders
    game_processors: RwLock<HashMap<String, mpsc::Sender<QueuedMessage>>>, // game_id -> sender
    processor_handles: RwLock<HashMap<String, JoinHandle<()>>>, // game_id -> processing task handle
}

impl GameManager {
    pub async fn new() -> Self {
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
            
        let redis_client = RedisClient::new(redis_url).await.unwrap();
        Self {
            redis_client,
            games: RwLock::new(HashMap::new()),
            game_broadcasters: RwLock::new(HashMap::new()),
            game_processors: RwLock::new(HashMap::new()),
            processor_handles: RwLock::new(HashMap::new()),
        }
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
            Err(e) => Err(REDIS_ERROR(&e.to_string())),
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
                    return Err(REDIS_ERROR(&e.to_string()));
                }
            }
        };

        let host = Player {
            id: host_id.clone(),
            username: host_username,
        };

        let game = Game {
            id: game_id.clone(),
            code: game_code.clone(),
            host_id: host_id.clone(),
            players: vec![host],
            max_players: 8,
            created_at: chrono::Utc::now().timestamp() as i32,
            team_draft: TeamDraftManager::new(
                host_id.clone(),
                0,
                8,
            ),
        };

        let mut games = self.games.write().await;
        let mut game_broadcasters = self.game_broadcasters.write().await;

        let player_key = key("player_to_game")?.field(&host_id)?.get_key()?;
        if let Ok(Some(_)) = self.redis_client.get(&player_key).await {
            return Err(ErrorResponse{
                error: ErrorCode::PlayerAlreadyInGame,
                message: "Player already in a game".to_string(),
            });
        }

        let auth_key = key("player_auth")?.field(&host_id)?.get_key()?;
        self.redis_client.set(&auth_key, &auth_token).await
            .map_err(|e| REDIS_ERROR(&e.to_string()))?;

        let (tx, _) = broadcast::channel(100);
        game_broadcasters.insert(game_id.clone(), tx);

        games.insert(game_id.clone(), game.clone());
        
        self.redis_client.set(&player_key, &game_id).await
            .map_err(|e| REDIS_ERROR(&e.to_string()))?;
        
        let key_string = key("game_code")?.field(game_code)?.get_key()?;
        
        self.redis_client.set(&key_string, &game_id).await
            .map_err(|e| REDIS_ERROR(&e.to_string()))?;

        Ok(GameEntry {
            game: game,
            auth_token: auth_token,
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

        let updated_game = {
            let mut games = self.games.write().await;

            let game = games.get_mut(&game_id).ok_or(ErrorResponse{
                error: ErrorCode::GameNotFound,
                message: "Game not found".to_string(),
            })?;
            if game.players.len() >= game.max_players as usize {
                return Err(ErrorResponse{
                    error: ErrorCode::GameFull,
                    message: "Game is full".to_string(),
                });
            }

            game.players.push(player);
            
            self.redis_client.set(&player_key, &game_id).await
                .map_err(|e| REDIS_ERROR(&e.to_string()))?;
            
            game.clone()
        };

        let auth_key = key("player_auth")?.field(&player_id)?.get_key()?;
        self.redis_client.set(&auth_key, &auth_token).await
            .map_err(|e| REDIS_ERROR(&e.to_string()))?;

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

        Ok(GameEntry {
            game: updated_game,
            auth_token: auth_token,
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

        let message_json = serde_json::to_string(&ws_message)
            .map_err(|e| ErrorResponse {
                error: ErrorCode::InternalServerError,
                message: format!("Failed to serialize message: {}", e),
            })?;
        
        let broadcaster = self.get_broadcaster(game_id).await?;
        let _ = broadcaster.send(message_json);
        Ok(())
    }

    pub async fn remove_player_from_game(&self, player_id: &str) -> GameResult<()> {
        let player_key = key("player_to_game")?.field(player_id)?.get_key()?;
        let game_id = match self.redis_client.get(&player_key).await {
            Ok(Some(id)) => id,
            Ok(None) => return Ok(()),
            Err(e) => return Err(REDIS_ERROR(&e.to_string())),
        };

        let (should_cleanup_game, player_info) = {
            let mut games = self.games.write().await;

            if let Some(game) = games.get_mut(&game_id) {
                let player_info = game.players.iter()
                    .find(|p| p.id == player_id)
                    .map(|p| (p.username.clone(), p.id.clone()));
                game.players.retain(|p| p.id != player_id);
                
                self.redis_client.del(&player_key).await
                    .map_err(|e| REDIS_ERROR(&e.to_string()))?;

                let auth_key = key("player_auth")?.field(player_id)?.get_key()?;
                self.redis_client.del(&auth_key).await
                    .map_err(|e| REDIS_ERROR(&e.to_string()))?;

                let should_cleanup = game.players.is_empty();
                (should_cleanup, player_info)
            } else {
                (false, None)
            }
        };

        if let Some((username, player_id)) = player_info {
            let player_disconnected = GameMessage::PlayerDisconnected {
                username: username.clone(),
                player_id: player_id.clone(),
            };
            let ws_message = WebSocketMessage {
                game_id: game_id.clone(),
                message: player_disconnected,
                player_id: player_id.clone(),
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
        let mut games = self.games.write().await;
        let mut game_broadcasters = self.game_broadcasters.write().await;

        let game_code = games.get(game_id).map(|g| g.code.clone());

        games.remove(game_id);
        game_broadcasters.remove(game_id);
        
        if let Some(code) = game_code {
            let key_string = key("game_code")?.field(&code)?.get_key()?;
                
            self.redis_client.del(&key_string).await
                .map_err(|e| REDIS_ERROR(&e.to_string()))?;
        }

        self.cleanup_game_processor(game_id).await;

        println!("[GameManager] Cleaned up empty game: {}", game_id);
        Ok(())
    }

    // we just rebroadcast as a chat message, since we assume the client will disconnect, which triggers other code.
    pub async fn handle_player_left(&self, game_id: &str, player_id: &str) -> GameResult<()> {
        let player_info = {
            let games = self.games.read().await;
            games.get(game_id)
                .and_then(|game| game.players.iter().find(|p| p.id == player_id))
                .map(|p| p.username.clone())
        };

        if let Some(username) = player_info {
            let chat_message = GameMessage::ChatMessage {
                username: "System".to_string(),
                message: format!("{} left the game", username),
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
        let games = self.games.read().await;
        Ok(games.values().cloned().collect())
    }
    
    pub async fn get_game(&self, game_id: &str) -> GameResult<Option<Game>> {
        let games = self.games.read().await;
        Ok(games.get(game_id).cloned())
    }

    pub async fn modify_game<F, R>(&self, game_id: &str, f: F) -> GameResult<R>
    where
        F: FnOnce(&mut Game) -> R,
    {
        let mut games = self.games.write().await;
        let game = games.get_mut(game_id).ok_or(ErrorResponse {
            error: ErrorCode::GameNotFound,
            message: "Game not found".to_string(),
        })?;
        Ok(f(game))
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
}