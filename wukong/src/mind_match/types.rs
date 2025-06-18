use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use ts_rs::TS;
use std::collections::HashMap;
use crate::cache::key_builder::{KeySchema, KeySegment};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub enum MindMatchPhase {
    Question,
    Answer
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct MindMatchState {
    pub phase: MindMatchPhase,
    pub question: String,
    pub answers: HashMap<String, String>,
}

pub fn get_key_schemas() -> Vec<(&'static str, KeySchema)> {
    vec![
        ("mind_match", KeySchema {
            base_pattern: vec![
                KeySegment::Fixed(vec!["mind_match"]),
                KeySegment::Field("game_id"),
            ],
            allowed_extensions: vec![
                vec![KeySegment::Fixed(vec!["phase"])],
                vec![KeySegment::Fixed(vec!["question"])],
                vec![KeySegment::Fixed(vec!["answers"])],
            ],
        }),
    ]
}
