use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use ts_rs::TS;
use crate::game::messages::GameMessage;
use crate::team_draft::messages::TeamDraftMessage;
use crate::game::game_manager::Player;

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
    pub pool: String,
    pub competition: String,
    pub team_size: u8,

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
    pub turn_player_id: String,
    pub player_points: HashMap<String, u8>,

}

impl TeamDraftManager {
    pub fn new(
        yapper_id: String,
        yapper_index: u8,
        max_rounds: u8,
    ) -> Self {
        Self {
            yapper_id: yapper_id.clone(),
            yapper_index: yapper_index,
            max_rounds: max_rounds,
            phase: TeamDraftPhase::YapperChoosing,
            round_data: Round {
                pool: String::new(),
                competition: String::new(),
                team_size: 3,
                player_to_picks: HashMap::new(),
            },
            turn_player_id: String::new(),
            player_points: HashMap::new(),
        }
    }

    pub fn set_game_settings(
        &mut self,
        max_rounds: u8,
    ) {
        self.max_rounds = max_rounds;
    }

    // Certain messages can only be valid from a certain player.
    // Can call this to get the id first, check is_authorized, then handle_message.
    pub fn get_correct_player_source_id(&self, message: TeamDraftMessage) -> String {
        match message {
            TeamDraftMessage::SetPool { .. } => self.yapper_id.clone(),
            TeamDraftMessage::SetCompetition { .. } => self.yapper_id.clone(),
        }
    }

    // Returns messages meant to be broadcasted to game players
    pub fn handle_message(&mut self, _source_player: Player, message: TeamDraftMessage) -> Vec<GameMessage> {
        match message {
            TeamDraftMessage::SetPool(set_pool_msg) => {
                self.round_data.pool = set_pool_msg.pool.clone();
                println!("Round data: {:?}", self.round_data);
                vec![
                    GameMessage::TeamDraft(TeamDraftMessage::SetPool(set_pool_msg)),
                ]
            },
            TeamDraftMessage::SetCompetition(set_competition_msg) => {
                self.round_data.competition = set_competition_msg.competition.clone();
                println!("Round data: {:?}", self.round_data);
                vec![
                    GameMessage::TeamDraft(TeamDraftMessage::SetCompetition(set_competition_msg)),
                ]
            },
        }
    }
}

// Game starts
// set the yapper_id and yapper_index

// phase = YapperChoosing
// yapper sets the pool and competition, team_size set to 3 by default
// yapper chooses first player

// phase = Drafting
// player submits pick
// check if all player_to_picks have <team_size> picks, set phase to Awarding
// otherwise, set next player

// phase = Awarding
// yapper awards a point to a player
// set phase to Complete

// phase = Complete
// check if max_rounds is reached, if not then set phase to YapperChoosing




