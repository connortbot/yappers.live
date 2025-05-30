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
}

pub type GameResult<T> = Result<T, ErrorResponse>;

#[derive(Default)]
pub struct GameManager {
    lobbies: RwLock<HashMap<String, Game>>,
    player_to_game: RwLock<HashMap<String, String>>,
    game_broadcasters: RwLock<HashMap<String, broadcast::Sender<String>>>,
    code_to_game: RwLock<HashMap<String, String>>,
}

impl GameManager {
    pub fn new() -> Self {
        Self::default()
    }

    fn generate_game_code(&self) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut rng = rng();
        
        (0..6)
            .map(|_| {
                let idx = rng.random_range(0..CHARS.len());
                CHARS[idx] as char
            })
            .collect()
    }

    pub async fn create_game(&self, host_username: String) -> GameResult<Game> {
        if host_username.trim().is_empty() {
            return Err(ErrorResponse{
                error: ErrorCode::InvalidInput("Username cannot be empty".to_string()),
                message: "Username cannot be empty".to_string(),
            });
        }

        let game_id = Uuid::new_v4().to_string();
        let host_id = Uuid::new_v4().to_string();

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
        };

        let mut lobbies = self.lobbies.write().await;
        let mut player_to_game = self.player_to_game.write().await;
        let mut game_broadcasters = self.game_broadcasters.write().await;
        let mut code_to_game = self.code_to_game.write().await;

        if player_to_game.contains_key(&host_id) {
            return Err(ErrorResponse{
                error: ErrorCode::PlayerAlreadyInGame,
                message: "Player already in a game".to_string(),
            });
        }

        let (tx, _) = broadcast::channel(100);
        game_broadcasters.insert(game_id.clone(), tx);

        lobbies.insert(game_id.clone(), game.clone());
        player_to_game.insert(host_id, game_id.clone());
        code_to_game.insert(game_code, game_id);

        Ok(game)
    }

    pub async fn join_game_by_code(&self, player_username: String, game_code: String) -> GameResult<Game> {
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

    pub async fn join_game(&self, player_username: String, game_id: String) -> GameResult<Game> {
        if player_username.trim().is_empty() {
            return Err(ErrorResponse{
                error: ErrorCode::InvalidInput("Username cannot be empty".to_string()),
                message: "Username cannot be empty".to_string(),
            });
        }

        let player_id = Uuid::new_v4().to_string();
        let player = Player {
            id: player_id.clone(),
            username: player_username.clone(),
        };

        let updated_game = {
            let mut lobbies = self.lobbies.write().await;
            let mut player_to_game = self.player_to_game.write().await;

            if player_to_game.contains_key(&player_id) {
                return Err(ErrorResponse{
                    error: ErrorCode::PlayerAlreadyInGame,
                    message: "Player already in a game".to_string(),
                });
            }

            let game = lobbies.get_mut(&game_id).ok_or(ErrorResponse{
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

        let player_joined = GameMessage::PlayerJoined {
            username: player_username,
            player_id: player_id,
        };
        let ws_message = WebSocketMessage {
            game_id: game_id.clone(),
            message: player_joined,
        };

        if let Ok(message_json) = serde_json::to_string(&ws_message) {
            if let Err(e) = self.broadcast_to_game(&game_id, message_json).await {
                println!("Failed to broadcast player joined message: {}", e);
            }
        }

        Ok(updated_game)
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

    pub async fn broadcast_to_game(&self, game_id: &str, message: String) -> GameResult<()> {
        let broadcaster = self.get_broadcaster(game_id).await?;
        let _ = broadcaster.send(message);
        Ok(())
    }
}