use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;
use ts_rs::TS;
use crate::game::messages::{GameMessage, TimerReason};
use crate::team_draft::messages::{TeamDraftMessage, TeamDraftTimerReason};
use crate::game::game_manager::Player;
use crate::game::action_timer_manager::ActionTimerManager;

pub const SERVER_ONLY_AUTHORIZED: &str = "00000000-0000-0000-0000-000000000000";


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

    #[serde(skip)]
    #[ts(skip)]
    pub action_timer_manager: ActionTimerManager,
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
                round: 1,
                pool: String::new(),
                competition: String::new(),
                team_size: 3,
                player_to_picks: HashMap::new(),
                starting_drafter_id: String::new(),
                current_drafter_id: String::new(),
            },
            player_points: HashMap::new(),
            action_timer_manager: ActionTimerManager::new(),
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
            TeamDraftMessage::StartDraft { .. } => self.yapper_id.clone(),
            TeamDraftMessage::DraftPick { .. } => self.round_data.current_drafter_id.clone(),
            TeamDraftMessage::AwardingPhase { .. } => SERVER_ONLY_AUTHORIZED.to_string(),
            TeamDraftMessage::AwardPoint { .. } => self.yapper_id.clone(),
            TeamDraftMessage::NextRound { .. } => SERVER_ONLY_AUTHORIZED.to_string(),
            TeamDraftMessage::NextDrafter { .. } => SERVER_ONLY_AUTHORIZED.to_string(),
            TeamDraftMessage::CompleteGame { .. } => SERVER_ONLY_AUTHORIZED.to_string(),
        }
    }

    pub fn requires_action_key(&self, message: TeamDraftMessage) -> bool {
        match message {
            TeamDraftMessage::SetPool { .. } => false,
            TeamDraftMessage::SetCompetition { .. } => false,
            TeamDraftMessage::StartDraft { .. } => false,
            TeamDraftMessage::DraftPick { .. } => true,
            TeamDraftMessage::AwardingPhase { .. } => false,
            TeamDraftMessage::AwardPoint { .. } => true,
            TeamDraftMessage::NextRound { .. } => false,
            TeamDraftMessage::NextDrafter { .. } => false,
            TeamDraftMessage::CompleteGame { .. } => false,
        }
    }
    // Returns messages meant to be broadcasted to game players
    pub fn handle_message(&mut self, players: Vec<Player>, message: TeamDraftMessage) -> Vec<GameMessage> {
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
            TeamDraftMessage::StartDraft(start_draft_msg) => {
                self.phase = TeamDraftPhase::Drafting;
                self.round_data.starting_drafter_id = start_draft_msg.starting_drafter_id.clone();
                self.round_data.current_drafter_id = start_draft_msg.starting_drafter_id.clone();
                self.round_data.player_to_picks.clear();
                for player in &players {
                    self.round_data.player_to_picks.entry(player.id.clone()).or_insert(Vec::new());
                }
                for player in &players {
                    self.player_points.entry(player.id.clone()).or_insert(0);
                }
                
                vec![
                    GameMessage::HaltTimer(
                        crate::game::messages::HaltTimer {
                            duration_seconds: 3,
                            reason: TimerReason::TeamDraft(TeamDraftTimerReason::YapperStartingDraft),
                        }
                    ),
                    GameMessage::TeamDraft(TeamDraftMessage::StartDraft(start_draft_msg.clone())),
                    GameMessage::ActionTimer(
                        crate::game::messages::ActionTimer {
                            duration_seconds: 30,
                            action_key: String::new(), // generated by queue processor n e way
                            default_action: Box::new(GameMessage::TeamDraft(TeamDraftMessage::DraftPick(
                                crate::team_draft::messages::DraftPick {
                                    drafter_id: start_draft_msg.starting_drafter_id,
                                    pick: "I didn't pick!".to_string(),
                                }
                            ))),
                            reason: TimerReason::TeamDraft(TeamDraftTimerReason::WaitingForDraftPick),
                        }
                    ),
                ]
            },
            TeamDraftMessage::DraftPick(draft_pick_msg) => {
                if let Some(picks) = self.round_data.player_to_picks.get_mut(&draft_pick_msg.drafter_id) {
                    picks.push(draft_pick_msg.pick.clone());
                }
                
                let mut messages = vec![
                    GameMessage::TeamDraft(TeamDraftMessage::DraftPick(draft_pick_msg)),
                ];
                
                let all_teams_complete = self.round_data.player_to_picks.values()
                    .all(|picks| picks.len() >= self.round_data.team_size as usize);
                
                if all_teams_complete {
                    self.phase = TeamDraftPhase::Awarding;
                    messages.push(GameMessage::HaltTimer(
                        crate::game::messages::HaltTimer {
                            duration_seconds: 3,
                            reason: TimerReason::TeamDraft(TeamDraftTimerReason::DraftPickShowcase),
                        }
                    ));
                    messages.push(GameMessage::HaltTimer(
                        crate::game::messages::HaltTimer {
                            duration_seconds: 5,
                            reason: TimerReason::TeamDraft(TeamDraftTimerReason::TransitionToAwarding),
                        }
                    ));
                    messages.push(GameMessage::TeamDraft(TeamDraftMessage::AwardingPhase(
                        crate::team_draft::messages::AwardingPhase {}
                    )));
                } else {
                    if let Some(current_index) = players.iter().position(|p| p.id == self.round_data.current_drafter_id) {
                        let mut next_index = (current_index + 1) % players.len();
                        while players[next_index].id == self.yapper_id {
                            next_index = (next_index + 1) % players.len();
                        }
                        self.round_data.current_drafter_id = players[next_index].id.clone();
                        
                        messages.push(GameMessage::HaltTimer(
                            crate::game::messages::HaltTimer {
                                duration_seconds: 3,
                                reason: TimerReason::TeamDraft(TeamDraftTimerReason::DraftPickShowcase),
                            }
                        ));
                        messages.push(GameMessage::TeamDraft(TeamDraftMessage::NextDrafter(
                            crate::team_draft::messages::NextDrafter {
                                drafter_id: self.round_data.current_drafter_id.clone(),
                            }
                        )));
                        messages.push(GameMessage::ActionTimer(
                            crate::game::messages::ActionTimer {
                                duration_seconds: 30,
                                action_key: String::new(),
                                default_action: Box::new(GameMessage::TeamDraft(TeamDraftMessage::DraftPick(
                                    crate::team_draft::messages::DraftPick {
                                        drafter_id: self.round_data.current_drafter_id.clone(),
                                        pick: "I didn't pick!".to_string(),
                                    }
                                ))),
                                reason: TimerReason::TeamDraft(TeamDraftTimerReason::WaitingForDraftPick),
                            }
                        ));
                    }
                }
                
                messages
            },
            TeamDraftMessage::AwardPoint(award_point_msg) => {
                if let Some(points) = self.player_points.get_mut(&award_point_msg.player_id) {
                    *points += 1;
                }
                
                let mut messages = vec![
                    GameMessage::TeamDraft(TeamDraftMessage::AwardPoint(award_point_msg)),
                ];
                
                if self.round_data.round >= self.max_rounds {
                    let final_points = self.player_points.clone();
                    
                    self.phase = TeamDraftPhase::YapperChoosing;
                    self.round_data = Round {
                        round: 1,
                        pool: String::new(),
                        competition: String::new(),
                        team_size: 3,
                        player_to_picks: HashMap::new(),
                        starting_drafter_id: String::new(),
                        current_drafter_id: String::new(),
                    };
                    self.player_points = HashMap::new();
                    
                    messages.push(GameMessage::TeamDraft(TeamDraftMessage::CompleteGame(
                        crate::team_draft::messages::CompleteGame {
                            player_points: final_points,
                        }
                    )));
                } else {
                    let next_yapper_index = (self.yapper_index + 1) % players.len() as u8;
                    if let Some(next_yapper) = players.get(next_yapper_index as usize) {
                        self.yapper_id = next_yapper.id.clone();
                        self.yapper_index = next_yapper_index;
                    }
                    
                    self.phase = TeamDraftPhase::YapperChoosing;
                    self.round_data.round += 1;
                    self.round_data.pool = String::new();
                    self.round_data.competition = String::new();
                    self.round_data.team_size = 3;
                    self.round_data.player_to_picks.clear();
                    self.round_data.starting_drafter_id = String::new();
                    self.round_data.current_drafter_id = String::new();
                    
                    messages.push(GameMessage::TeamDraft(TeamDraftMessage::NextRound(
                        crate::team_draft::messages::NextRound {
                            round: self.round_data.round,
                            team_size: self.round_data.team_size,
                        }
                    )));
                }
                
                messages
            },
            TeamDraftMessage::AwardingPhase(_) => {
                // Server-only message, do nothing
                vec![]
            },
            TeamDraftMessage::CompleteGame(_) => {
                // Server-only message, do nothing
                vec![]
            },
            TeamDraftMessage::NextRound(_) => {
                // Server-only message, do nothing
                vec![]
            },
            TeamDraftMessage::NextDrafter(_) => {
                // Server-only message, do nothing
                vec![]
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




