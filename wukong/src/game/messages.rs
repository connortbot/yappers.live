use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum GameMessage {
    PlayerJoined { username: String, player_id: String },
    PlayerLeft { username: String, player_id: String },
    PlayerDisconnected { username: String, player_id: String },
    GameStarted { game_type: String },
    ChatMessage { username: String, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WebSocketMessage {
    pub game_id: String,
    pub message: GameMessage,
    pub player_id: String,
    pub auth_token: Option<String>,
}

pub fn client_safe_ws_message(ws_message: WebSocketMessage) -> WebSocketMessage {
    WebSocketMessage {
        game_id: ws_message.game_id,
        message: ws_message.message,
        player_id: ws_message.player_id,
        auth_token: None,
    }
}