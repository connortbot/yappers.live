use crate::cache::redis_client::RedisClient;
use crate::game::types::Player;
use crate::game::messages::{GameMessage, GameMode};
use crate::error::ErrorResponse;
use async_trait::async_trait;

pub const SERVER_ONLY_AUTHORIZED: &str = "00000000-0000-0000-0000-000000000000";

#[async_trait]
pub trait GameModeManager: Send + Sync {
    async fn init_state_cached(&self, game_id: String, host_player: Player) -> Result<(), ErrorResponse>;
    async fn cleanup_state_cached(&self, game_id: String) -> Result<(), ErrorResponse>;
    
    async fn set_game_settings(&self, game_id: String, max_rounds: u8) -> Result<(), ErrorResponse>;
    
    async fn get_correct_player_source_id(&self, game_id: String, message: GameMessage) -> Result<String, ErrorResponse>;
    async fn handle_message(&self, game_id: String, players: Vec<Player>, message: GameMessage) -> Result<Vec<GameMessage>, ErrorResponse>;
    
    fn get_mode_type(&self) -> GameMode;
}

pub struct GameModeService {
    redis_client: RedisClient,
    mode_type: GameMode,
}

impl GameModeService {
    pub fn new(redis_client: RedisClient, mode_type: GameMode) -> Self {
        Self {
            redis_client,
            mode_type,
        }
    }
    
    pub fn get_redis_client(&self) -> &RedisClient {
        &self.redis_client
    }
}

#[async_trait]
impl GameModeManager for GameModeService {
    async fn init_state_cached(&self, _game_id: String, _host_player: Player) -> Result<(), ErrorResponse> {
        Ok(())
    }

    async fn cleanup_state_cached(&self, _game_id: String) -> Result<(), ErrorResponse> {
        Ok(())
    }

    async fn set_game_settings(&self, _game_id: String, _max_rounds: u8) -> Result<(), ErrorResponse> {
        Ok(())
    }

    async fn get_correct_player_source_id(&self, _game_id: String, _message: GameMessage) -> Result<String, ErrorResponse> {
        Ok(SERVER_ONLY_AUTHORIZED.to_string())
    }

    async fn handle_message(&self, _game_id: String, _players: Vec<Player>, _message: GameMessage) -> Result<Vec<GameMessage>, ErrorResponse> {
        Ok(vec![])
    }

    fn get_mode_type(&self) -> GameMode {
        self.mode_type.clone()
    }
}
