use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use ts_rs::TS;
use std::collections::HashMap;
use crate::cache::key_builder::{KeySchema, KeySegment};

// types defined for frontend

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub enum TeamDraftPhase {
    YapperChoosing,
    Drafting,
    Awarding,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct Round {
    pub round: u8,
    pub pool: String,
    pub competition: String,
    pub team_size: u8,

    pub starting_drafter_id: String,
    pub current_drafter_id: String,

    pub player_to_picks: HashMap<String, Vec<String>>,
}


#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct TeamDraftManager {
    // Yapper
    pub yapper_id: String,
    pub yapper_index: u8,
    pub max_rounds: u8, // usually, set to number of players
    
    // Phase
    pub phase: TeamDraftPhase,
    pub round_data: Round,

    // Turn
    pub player_points: HashMap<String, u8>,
}

// Key schema definitions for team draft types
pub fn get_key_schemas() -> Vec<(&'static str, KeySchema)> {
    vec![
        ("team_draft", KeySchema {
            base_pattern: vec![
                KeySegment::Fixed(vec!["team_draft"]),
                KeySegment::Field("game_id"),
            ],
            allowed_extensions: vec![
                vec![KeySegment::Fixed(vec!["yapper_id"])],
                vec![KeySegment::Fixed(vec!["yapper_index"])],
                vec![KeySegment::Fixed(vec!["max_rounds"])],
                vec![KeySegment::Fixed(vec!["round_data"])],

                vec![KeySegment::Fixed(vec!["phase"])],
                vec![KeySegment::Fixed(vec!["player_points"])], // hash map

                vec![
                    KeySegment::Fixed(vec!["round"]),
                    KeySegment::Fixed(vec![
                        "round",
                        "pool",
                        "competition",
                        "team_size",
                        "starting_drafter_id",
                        "current_drafter_id",
                        "player_to_picks", // hash map
                    ]),
                ],
            ],
        }),
    ]
}