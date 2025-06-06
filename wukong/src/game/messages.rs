use serde::{Deserialize, Serialize};
use ts_rs::TS;
use crate::team_draft::messages::{TeamDraftMessage, TeamDraftTimerReason};
use crate::team_draft::state::TeamDraftManager;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum GameMode {
    TeamDraft,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GameStartedMessage {
    pub game_type: GameMode,
    pub initial_team_draft_state: Option<TeamDraftManager>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct HaltTimer {
    pub end_timestamp_ms: u64,
    pub reason: TimerReason,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum TimerReason {
    TeamDraft(TeamDraftTimerReason),
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BackToLobby {}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum GameMessage {
    PlayerJoined { username: String, player_id: String },
    PlayerLeft { username: String, player_id: String },
    PlayerDisconnected { username: String, player_id: String },
    GameStarted(GameStartedMessage),
    ChatMessage { username: String, message: String },
    
    HaltTimer(HaltTimer),
    BackToLobby(BackToLobby),
    
    TeamDraft(TeamDraftMessage),
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
