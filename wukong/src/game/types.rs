use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use ts_rs::TS;

use crate::cache::key_builder::{KeySchema, KeySegment};

// types defined for frontend

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct Player {
    pub id: String,
    pub username: String,
    // stream?
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct Game {
    pub id: String,
    pub code: String,
    pub host_id: String,
    pub players: Vec<Player>,
    pub max_players: u8,
    pub created_at: i32,
}

pub fn get_key_schemas() -> Vec<(&'static str, KeySchema)> {
    vec![
        ("game", KeySchema {
            base_pattern: vec![
                KeySegment::Fixed(vec!["game"]),
                KeySegment::Field("game_id"),
            ],
            allowed_extensions: vec![
                // game::id::host
                vec![KeySegment::Fixed(vec!["host_id"])],
                // game::id::code  
                vec![KeySegment::Fixed(vec!["code"])],
                // game::id::players
                vec![KeySegment::Fixed(vec!["players"])],
                // game::id::max_players
                vec![KeySegment::Fixed(vec!["max_players"])],
                // game::id::created_at
                vec![KeySegment::Fixed(vec!["created_at"])],
            ],
        }),
        
        ("player_usernames", KeySchema {
            base_pattern: vec![
                KeySegment::Fixed(vec!["player_usernames"]),
                KeySegment::Field("player_id"),
            ],
            allowed_extensions: vec![],
        }),
    ]
}