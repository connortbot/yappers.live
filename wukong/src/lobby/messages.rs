use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum LobbyMessage {
    PlayerJoined { username: String, player_id: String },
    PlayerLeft { username: String, player_id: String },
    GameStarted { game_type: String },
    ChatMessage { username: String, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WebSocketMessage {
    pub lobby_id: String,
    pub message: LobbyMessage,
}