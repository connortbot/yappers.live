use std::collections::HashMap;
use crate::game::messages::{GameMessage, TimerReason};
use crate::mind_match::messages::{MindMatchMessage, MindMatchTimerReason};
use crate::game::types::Player;
use crate::mind_match::types::{MindMatchPhase, MindMatchState};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::{ErrorResponse, ErrorCode};
use crate::cache::redis_client::RedisClient;
use crate::cache::key_builder::key;
use serde_json;
use crate::game::game_mode::{GameModeManager, GameModeService};
use crate::game::messages::GameMode;
use async_trait::async_trait;

use crate::game::game_mode::SERVER_ONLY_AUTHORIZED;

pub struct MindMatchService {
    base: GameModeService,
}

impl MindMatchService {
    pub fn new(redis_client: RedisClient, _mode_type: GameMode) -> Self {
        Self {
            base: GameModeService::new(redis_client, GameMode::MindMatch),
        }
    }
}

#[async_trait]
impl GameModeManager for MindMatchService {
    fn get_mode_type(&self) -> GameMode {
        self.base.get_mode_type()
    }

    async fn init_state_cached(&self, game_id: String, host_player: Player) -> Result<(), ErrorResponse> {
        let base_key = key("mind_match")?.field(&game_id)?.get_key()?;
        let pattern = format!("{}*", base_key);
        self.base.get_redis_client().pdel(&pattern).await?;

        let phase_key = key("mind_match")?.field(&game_id)?.field("phase")?.get_key()?;
        let phase_json = serde_json::to_string(&MindMatchPhase::Question)?;
        self.base.get_redis_client().set(&phase_key, &phase_json).await?;

        Ok(())
    }

    async fn set_game_settings(&self, game_id: String, max_rounds: u8) -> Result<(), ErrorResponse> {
        Ok(())
    }

    async fn cleanup_state_cached(&self, game_id: String) -> Result<(), ErrorResponse> {
        let mind_match_base_key = key("mind_match")?.field(&game_id)?.get_key()?;
        let mind_match_pattern = format!("{}*", mind_match_base_key);
        self.base.get_redis_client().pdel(&mind_match_pattern).await?;

        Ok(())
    }

    async fn get_correct_player_source_id(&self, game_id: String, message: GameMessage) -> Result<String, ErrorResponse> {
        if let GameMessage::MindMatch(mind_match_msg) = message {
            match mind_match_msg {
                MindMatchMessage::ShowQuestion { .. } => Ok(SERVER_ONLY_AUTHORIZED.to_string()),
                MindMatchMessage::Answer { .. } => Ok(SERVER_ONLY_AUTHORIZED.to_string()),
            }
        } else {
            Err(ErrorResponse {
                error: ErrorCode::InvalidGameMode,
                message: "Message type not supported for MindMatch mode".to_string(),
            })
        }
    }

    // Returns messages meant to be broadcasted to game players
    async fn handle_message(&self, game_id: String, players: Vec<Player>, message: GameMessage) -> Result<Vec<GameMessage>, ErrorResponse> {
        if let GameMessage::MindMatch(mind_match_msg) = message {
            match mind_match_msg {
                MindMatchMessage::ShowQuestion(_) => {
                    // Server-only message, do nothing
                    Ok(vec![])
                },
                MindMatchMessage::Answer(_) => {
                    // Server-only message, do nothing
                    Ok(vec![])
                },
            }
        } else {
            Err(ErrorResponse {
                error: ErrorCode::InvalidGameMode,
                message: "Message type not supported for TeamDraft mode".to_string(),
            })
        }
    }
}

impl MindMatchState {
    pub fn new() -> Self {
        Self {
            phase: MindMatchPhase::Question,
            question: String::new(),
            answers: HashMap::new(),
        }
    }
}

