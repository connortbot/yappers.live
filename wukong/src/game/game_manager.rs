use std::collections::HashMap;
use tokio::sync::{RwLock, broadcast};
use uuid::Uuid;
use chrono;
use serde::Serialize;
use rand::{Rng, rng};
use crate::game::messages::{GameMessage, WebSocketMessage};
use crate::error::{ErrorResponse, ErrorCode};
use serde_json;
use utoipa::ToSchema;
use serde::Deserialize;
use ts_rs::TS;
use crate::team_draft::state::TeamDraftManager;

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

#[derive(Default)]
pub struct GameManager {
    games: RwLock<HashMap<String, Game>>,
    player_to_game: RwLock<HashMap<String, String>>,
    game_broadcasters: RwLock<HashMap<String, broadcast::Sender<String>>>,
    player_to_auth_token: RwLock<HashMap<String, String>>,
    code_to_game: RwLock<HashMap<String, String>>,
}

impl GameManager {
    pub fn new() -> Self {
        Self::default()
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
        let player_to_auth_token = self.player_to_auth_token.read().await;
        let authorized = player_to_auth_token.get(player_id)
            .map(|token| token == auth_token)
            .unwrap_or(false);
        Ok(authorized)
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
            let code_to_game = self.code_to_game.read().await;
            if !code_to_game.contains_key(&code) {
                break code;
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
        let mut player_to_game = self.player_to_game.write().await;
        let mut game_broadcasters = self.game_broadcasters.write().await;
        let mut code_to_game = self.code_to_game.write().await;
        let mut player_to_auth_token = self.player_to_auth_token.write().await;

        if player_to_game.contains_key(&host_id) {
            return Err(ErrorResponse{
                error: ErrorCode::PlayerAlreadyInGame,
                message: "Player already in a game".to_string(),
            });
        }

        player_to_auth_token.insert(host_id.clone(), auth_token.clone());

        let (tx, _) = broadcast::channel(100);
        game_broadcasters.insert(game_id.clone(), tx);

        games.insert(game_id.clone(), game.clone());
        player_to_game.insert(host_id, game_id.clone());
        code_to_game.insert(game_code, game_id);

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
            let code_to_game = self.code_to_game.read().await;
            code_to_game.get(&game_code).cloned()
                .ok_or(ErrorResponse{
                    error: ErrorCode::GameNotFound,
                    message: "Game not found".to_string(),
                })?
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

        let updated_game = {
            let mut games = self.games.write().await;
            let mut player_to_game = self.player_to_game.write().await;

            if player_to_game.contains_key(&player_id) {
                return Err(ErrorResponse{
                    error: ErrorCode::PlayerAlreadyInGame,
                    message: "Player already in a game".to_string(),
                });
            }

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
            player_to_game.insert(player_id.clone(), game_id.clone());
            
            game.clone()
        };

        let mut player_to_auth_token = self.player_to_auth_token.write().await;
        player_to_auth_token.insert(player_id.clone(), auth_token.clone());

        let player_joined = GameMessage::PlayerJoined {
            username: player_username,
            player_id: player_id.clone(),
        };
        let ws_message = WebSocketMessage {
            game_id: game_id.clone(),
            message: player_joined,
            player_id: player_id.clone(),
            auth_token: None, // don't broadcast auth tokens to client
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
        let game_id = {
            let player_to_game = self.player_to_game.read().await;
            player_to_game.get(player_id).cloned()
        };

        if let Some(game_id) = game_id {
            let (should_cleanup_game, player_info) = {
                let mut games = self.games.write().await;
                let mut player_to_game = self.player_to_game.write().await;

                if let Some(game) = games.get_mut(&game_id) {
                    let player_info = game.players.iter()
                        .find(|p| p.id == player_id)
                        .map(|p| (p.username.clone(), p.id.clone()));
                    game.players.retain(|p| p.id != player_id);
                    player_to_game.remove(player_id);

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
        }

        Ok(())
    }

    async fn cleanup_empty_game(&self, game_id: &str) -> GameResult<()> {
        let mut games = self.games.write().await;
        let mut game_broadcasters = self.game_broadcasters.write().await;
        let mut code_to_game = self.code_to_game.write().await;

        let game_code = games.get(game_id).map(|g| g.code.clone());

        games.remove(game_id);
        game_broadcasters.remove(game_id);
        
        if let Some(code) = game_code {
            code_to_game.remove(&code);
        }

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
}