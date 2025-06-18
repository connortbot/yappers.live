use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum MindMatchTimerReason {}


#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ShowQuestionMessage {
    pub player_id: String,
    pub question: String,
}


#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AnswerMessage {
    pub player_id: String,
    pub answer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "msg_type")]
pub enum MindMatchMessage {
    ShowQuestion(ShowQuestionMessage),
    Answer(AnswerMessage),
}
