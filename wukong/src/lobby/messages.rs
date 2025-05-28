use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LobbyMessage {
    PlayerJoined { username: String, player_id: String },
    PlayerLeft { username: String, player_id: String },
    GameStarted { game_type: String },
    ChatMessage { username: String, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub lobby_id: String,
    pub message: LobbyMessage,
}