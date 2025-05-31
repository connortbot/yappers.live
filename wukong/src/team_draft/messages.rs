use serde::{Serialize, Deserialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum TeamDraftMessage {
    SetPool { pool: String },
    SetCompetition { competition: String },
}