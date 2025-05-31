use serde::{Serialize, Deserialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetPoolMessage {
    pub pool: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SetCompetitionMessage {
    pub competition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "msg_type")]
pub enum TeamDraftMessage {
    SetPool(SetPoolMessage),
    SetCompetition(SetCompetitionMessage),
}