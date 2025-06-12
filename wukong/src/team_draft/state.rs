use std::collections::HashMap;
use crate::game::messages::{GameMessage, TimerReason};
use crate::team_draft::messages::{TeamDraftMessage, TeamDraftTimerReason};
use crate::game::types::Player;
use crate::team_draft::types::{TeamDraftPhase, Round, TeamDraftState};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::{ErrorResponse, ErrorCode};
use crate::cache::redis_client::RedisClient;
use crate::cache::key_builder::key;
use serde_json;


pub const SERVER_ONLY_AUTHORIZED: &str = "00000000-0000-0000-0000-000000000000";
pub const DEFAULT_TEAM_SIZE: u8 = 2;
pub const DEFAULT_MAX_ROUNDS: u8 = 3;

pub struct TeamDraftService {
    redis_client: RedisClient,
}

impl TeamDraftService {
    pub fn new(redis_client: RedisClient) -> Self {
        Self { redis_client }
    }

    pub async fn init_state_cached(&self, game_id: String, host_player: Player) -> Result<(), ErrorResponse> {
        let base_key = key("team_draft")?.field(&game_id)?.get_key()?;
        let pattern = format!("{}*", base_key);
        self.redis_client.pdel(&pattern).await?;

        let yapper_id_key = key("team_draft")?.field(&game_id)?.field("yapper_id")?.get_key()?;
        self.redis_client.set(&yapper_id_key, &host_player.id).await?;

        let yapper_index_key = key("team_draft")?.field(&game_id)?.field("yapper_index")?.get_key()?;
        self.redis_client.set(&yapper_index_key, &0u8).await?;

        let max_rounds_key = key("team_draft")?.field(&game_id)?.field("max_rounds")?.get_key()?;
        self.redis_client.set(&max_rounds_key, &DEFAULT_MAX_ROUNDS).await?;

        let phase_key = key("team_draft")?.field(&game_id)?.field("phase")?.get_key()?;
        let phase_json = serde_json::to_string(&TeamDraftPhase::YapperChoosing)?;
        self.redis_client.set(&phase_key, &phase_json).await?;

        let round_key = key("team_draft")?.field(&game_id)?.field("round")?.field("round")?.get_key()?;
        self.redis_client.set(&round_key, &1u8).await?;

        let team_size_key = key("team_draft")?.field(&game_id)?.field("round")?.field("team_size")?.get_key()?;
        self.redis_client.set(&team_size_key, &DEFAULT_TEAM_SIZE).await?;

        let pool_key = key("team_draft")?.field(&game_id)?.field("round")?.field("pool")?.get_key()?;
        self.redis_client.set(&pool_key, "").await?;

        let competition_key = key("team_draft")?.field(&game_id)?.field("round")?.field("competition")?.get_key()?;
        self.redis_client.set(&competition_key, "").await?;

        let starting_drafter_key = key("team_draft")?.field(&game_id)?.field("round")?.field("starting_drafter_id")?.get_key()?;
        self.redis_client.set(&starting_drafter_key, "").await?;

        let current_drafter_key = key("team_draft")?.field(&game_id)?.field("round")?.field("current_drafter_id")?.get_key()?;
        self.redis_client.set(&current_drafter_key, "").await?;

        Ok(())
    }

    pub async fn set_game_settings(&self, game_id: String, max_rounds: u8) -> Result<(), ErrorResponse> {
        let max_rounds_key = key("team_draft")?.field(game_id.clone())?.field("max_rounds")?.get_key()?;
        self.redis_client.set(&max_rounds_key, &max_rounds).await?;

        Ok(())
    }

    pub async fn cleanup_state_cached(&self, game_id: String) -> Result<(), ErrorResponse> {
        let team_draft_base_key = key("team_draft")?.field(&game_id)?.get_key()?;
        let team_draft_pattern = format!("{}*", team_draft_base_key);
        self.redis_client.pdel(&team_draft_pattern).await?;

        Ok(())
    }

    // Certain messages can only be valid from a certain player.
    // Can call this to get the id first, check is_authorized, then handle_message.
    pub async fn get_correct_player_source_id(&self, game_id: String, message: TeamDraftMessage) -> Result<String, ErrorResponse> {
        let yapper_id_key = key("team_draft")?.field(game_id.clone())?.field("yapper_id")?.get_key()?;
        let yapper_id = match self.redis_client.get(&yapper_id_key).await {
            Ok(Some(yapper_id)) => yapper_id,
            _ => return Err(ErrorResponse {
                error: ErrorCode::GameNotFound,
                message: "Invalid game id".to_string(),
            }),
        };
        
        match message {
            TeamDraftMessage::SetPool { .. } => Ok(yapper_id),
            TeamDraftMessage::SetCompetition { .. } => Ok(yapper_id),
            TeamDraftMessage::StartDraft { .. } => Ok(yapper_id),
            TeamDraftMessage::DraftPick { .. } => {
                let current_drafter_key = key("team_draft")?.field(&game_id)?.field("round")?.field("current_drafter_id")?.get_key()?;
                match self.redis_client.get(&current_drafter_key).await {
                    Ok(Some(current_drafter_id)) => Ok(current_drafter_id),
                    Ok(None) => Err(ErrorResponse {
                        error: ErrorCode::GameNotFound,
                        message: "Current drafter not found".to_string(),
                    }),
                    Err(e) => Err(e.into()),
                }
            },
            TeamDraftMessage::AwardingPhase { .. } => Ok(SERVER_ONLY_AUTHORIZED.to_string()),
            TeamDraftMessage::AwardPoint { .. } => Ok(yapper_id),
            TeamDraftMessage::NextRound { .. } => Ok(SERVER_ONLY_AUTHORIZED.to_string()),
            TeamDraftMessage::NextDrafter { .. } => Ok(SERVER_ONLY_AUTHORIZED.to_string()),
            TeamDraftMessage::CompleteGame { .. } => Ok(SERVER_ONLY_AUTHORIZED.to_string()),
        }
    }

    // Returns messages meant to be broadcasted to game players
    pub async fn handle_message(&self, game_id: String, players: Vec<Player>, message: TeamDraftMessage) -> Result<Vec<GameMessage>, ErrorResponse> {
        match message {
            TeamDraftMessage::SetPool(set_pool_msg) => {
                let pool_key = key("team_draft")?.field(&game_id)?.field("round")?.field("pool")?.get_key()?;
                self.redis_client.set(&pool_key, &set_pool_msg.pool).await?;
                
                Ok(vec![
                    GameMessage::TeamDraft(TeamDraftMessage::SetPool(set_pool_msg)),
                ])
            },
            TeamDraftMessage::SetCompetition(set_competition_msg) => {
                let competition_key = key("team_draft")?.field(&game_id)?.field("round")?.field("competition")?.get_key()?;
                self.redis_client.set(&competition_key, &set_competition_msg.competition).await?;
                
                Ok(vec![
                    GameMessage::TeamDraft(TeamDraftMessage::SetCompetition(set_competition_msg)),
                ])
            },
            TeamDraftMessage::StartDraft(start_draft_msg) => {
                let phase_key = key("team_draft")?.field(&game_id)?.field("phase")?.get_key()?;
                let phase_json = serde_json::to_string(&TeamDraftPhase::Drafting)?;
                self.redis_client.set(&phase_key, &phase_json).await?;
                
                let starting_drafter_key = key("team_draft")?.field(&game_id)?.field("round")?.field("starting_drafter_id")?.get_key()?;
                self.redis_client.set(&starting_drafter_key, &start_draft_msg.starting_drafter_id).await?;
                
                let current_drafter_key = key("team_draft")?.field(&game_id)?.field("round")?.field("current_drafter_id")?.get_key()?;
                self.redis_client.set(&current_drafter_key, &start_draft_msg.starting_drafter_id).await?;
                
                let yapper_id_key = key("team_draft")?.field(&game_id)?.field("yapper_id")?.get_key()?;
                let yapper_id: String = self.redis_client.get(&yapper_id_key).await?.unwrap_or_default();
                
                for player in &players {
                    if player.id != yapper_id {
                        let player_picks_key = key("team_draft")?.field(&game_id)?.field("round")?.field("player_to_picks")?.field(&player.id)?.get_key()?;
                        // clear picks
                        self.redis_client.del(&player_picks_key).await?;
                    }
                }
                
                let player_points_key = key("team_draft")?.field(&game_id)?.field("player_points")?.get_key()?;
                for player in &players {
                    self.redis_client.hset(&player_points_key, &player.id, &0u8).await?;
                }
                
                Ok(vec![
                    GameMessage::HaltTimer(
                        crate::game::messages::HaltTimer {
                            end_timestamp_ms: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64 + 3000,
                            reason: TimerReason::TeamDraft(TeamDraftTimerReason::YapperStartingDraft),
                        }
                    ),
                    GameMessage::TeamDraft(TeamDraftMessage::StartDraft(start_draft_msg.clone())),
                ])
            },
            TeamDraftMessage::DraftPick(draft_pick_msg) => {
                let player_picks_key = key("team_draft")?.field(&game_id)?.field("round")?.field("player_to_picks")?.field(&draft_pick_msg.drafter_id)?.get_key()?;
                self.redis_client.rpush(&player_picks_key, &draft_pick_msg.pick).await?;
                
                let mut messages = vec![
                    GameMessage::TeamDraft(TeamDraftMessage::DraftPick(draft_pick_msg.clone())),
                ];
                
                let team_size_key = key("team_draft")?.field(&game_id)?.field("round")?.field("team_size")?.get_key()?;
                let team_size: u8 = self.redis_client.get(&team_size_key).await?
                    .unwrap_or_else(|| DEFAULT_TEAM_SIZE.to_string())
                    .parse()
                    .unwrap_or(DEFAULT_TEAM_SIZE);
                
                let yapper_id_key = key("team_draft")?.field(&game_id)?.field("yapper_id")?.get_key()?;
                let yapper_id: String = self.redis_client.get(&yapper_id_key).await?.unwrap_or_default();
                
                let mut all_teams_complete = true;
                for player in &players {
                    if player.id != yapper_id {
                        let player_picks_key = key("team_draft")?.field(&game_id)?.field("round")?.field("player_to_picks")?.field(&player.id)?.get_key()?;
                        let picks_count = self.redis_client.lrange(&player_picks_key, 0, -1).await?.len() as u8;
                        if picks_count < team_size {
                            all_teams_complete = false;
                            break;
                        }
                    }
                }
                
                if all_teams_complete {
                    let phase_key = key("team_draft")?.field(&game_id)?.field("phase")?.get_key()?;
                    let phase_json = serde_json::to_string(&TeamDraftPhase::Awarding)?;
                    self.redis_client.set(&phase_key, &phase_json).await?;
                    
                    messages.push(GameMessage::HaltTimer(
                        crate::game::messages::HaltTimer {
                            end_timestamp_ms: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64 + 3000,
                            reason: TimerReason::TeamDraft(TeamDraftTimerReason::DraftPickShowcase),
                        }
                    ));
                    messages.push(GameMessage::HaltTimer(
                        crate::game::messages::HaltTimer {
                            end_timestamp_ms: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64 + 8000,
                            reason: TimerReason::TeamDraft(TeamDraftTimerReason::TransitionToAwarding),
                        }
                    ));
                    messages.push(GameMessage::TeamDraft(TeamDraftMessage::AwardingPhase(
                        crate::team_draft::messages::AwardingPhase {}
                    )));
                } else {
                    let current_drafter_key = key("team_draft")?.field(&game_id)?.field("round")?.field("current_drafter_id")?.get_key()?;
                    let current_drafter_id: String = self.redis_client.get(&current_drafter_key).await?.unwrap_or_default();
                    
                    if let Some(current_index) = players.iter().position(|p| p.id == current_drafter_id) {
                        let mut next_index = (current_index + 1) % players.len();
                        while players[next_index].id == yapper_id {
                            next_index = (next_index + 1) % players.len();
                        }
                        let next_drafter_id = &players[next_index].id;
                        self.redis_client.set(&current_drafter_key, next_drafter_id).await?;
                        
                        messages.push(GameMessage::HaltTimer(
                            crate::game::messages::HaltTimer {
                                end_timestamp_ms: SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis() as u64 + 3000,
                                reason: TimerReason::TeamDraft(TeamDraftTimerReason::DraftPickShowcase),
                            }
                        ));
                        messages.push(GameMessage::TeamDraft(TeamDraftMessage::NextDrafter(
                            crate::team_draft::messages::NextDrafter {
                                drafter_id: next_drafter_id.clone(),
                            }
                        )));
                    }
                }
                
                Ok(messages)
            },
            TeamDraftMessage::AwardPoint(award_point_msg) => {
                let player_points_key = key("team_draft")?.field(&game_id)?.field("player_points")?.get_key()?;
                let current_points: Option<String> = self.redis_client.hget(&player_points_key, &award_point_msg.player_id).await?;
                let points: u8 = if let Some(points_str) = current_points {
                    points_str.parse().unwrap_or(0)
                } else {
                    0
                };
                self.redis_client.hset(&player_points_key, &award_point_msg.player_id, &(points + 1)).await?;
                
                let mut messages = vec![
                    GameMessage::TeamDraft(TeamDraftMessage::AwardPoint(award_point_msg)),
                ];
                
                let round_key = key("team_draft")?.field(&game_id)?.field("round")?.field("round")?.get_key()?;
                let current_round: u8 = self.redis_client.get(&round_key).await?
                    .unwrap_or_else(|| "1".to_string())
                    .parse()
                    .unwrap_or(1);
                let max_rounds_key = key("team_draft")?.field(&game_id)?.field("max_rounds")?.get_key()?;
                let max_rounds: u8 = self.redis_client.get(&max_rounds_key).await?
                    .unwrap_or_else(|| DEFAULT_MAX_ROUNDS.to_string())
                    .parse()
                    .unwrap_or(DEFAULT_MAX_ROUNDS);
                
                if current_round >= max_rounds {
                    let final_points: HashMap<String, String> = self.redis_client.hgetall(&player_points_key).await?;
                    let final_points_u8: HashMap<String, u8> = final_points.into_iter()
                        .filter_map(|(k, v)| v.parse().ok().map(|points| (k, points)))
                        .collect();
                    
                    self.init_state_cached(game_id.clone(), players[0].clone()).await?;
                    
                    messages.push(GameMessage::TeamDraft(TeamDraftMessage::CompleteGame(
                        crate::team_draft::messages::CompleteGame {
                            player_points: final_points_u8,
                        }
                    )));
                } else {
                    // Next round - update yapper
                    let yapper_index_key = key("team_draft")?.field(&game_id)?.field("yapper_index")?.get_key()?;
                    let current_yapper_index: u8 = self.redis_client.get(&yapper_index_key).await?
                        .unwrap_or_else(|| "0".to_string())
                        .parse()
                        .unwrap_or(0);
                    let next_yapper_index = (current_yapper_index + 1) % players.len() as u8;
                    
                    let next_yapper_id = if let Some(next_yapper) = players.get(next_yapper_index as usize) {
                        let yapper_id_key = key("team_draft")?.field(&game_id)?.field("yapper_id")?.get_key()?;
                        self.redis_client.set(&yapper_id_key, &next_yapper.id).await?;
                        self.redis_client.set(&yapper_index_key, &next_yapper_index).await?;
                        next_yapper.id.clone()
                    } else {
                        players[0].id.clone() // fallback
                    };
                    
                    // Reset round state
                    let phase_key = key("team_draft")?.field(&game_id)?.field("phase")?.get_key()?;
                    let phase_json = serde_json::to_string(&TeamDraftPhase::YapperChoosing)?;
                    self.redis_client.set(&phase_key, &phase_json).await?;
                    
                    self.redis_client.set(&round_key, &(current_round + 1)).await?;
                    
                    let team_size_key = key("team_draft")?.field(&game_id)?.field("round")?.field("team_size")?.get_key()?;
                    self.redis_client.set(&team_size_key, &DEFAULT_TEAM_SIZE).await?;
                    
                    let pool_key = key("team_draft")?.field(&game_id)?.field("round")?.field("pool")?.get_key()?;
                    self.redis_client.set(&pool_key, "").await?;
                    
                    let competition_key = key("team_draft")?.field(&game_id)?.field("round")?.field("competition")?.get_key()?;
                    self.redis_client.set(&competition_key, "").await?;
                    
                    let starting_drafter_key = key("team_draft")?.field(&game_id)?.field("round")?.field("starting_drafter_id")?.get_key()?;
                    self.redis_client.set(&starting_drafter_key, "").await?;
                    
                    let current_drafter_key = key("team_draft")?.field(&game_id)?.field("round")?.field("current_drafter_id")?.get_key()?;
                    self.redis_client.set(&current_drafter_key, "").await?;
                    
                    // Delete individual player pick lists
                    let yapper_id_key = key("team_draft")?.field(&game_id)?.field("yapper_id")?.get_key()?;
                    let old_yapper_id: String = self.redis_client.get(&yapper_id_key).await?.unwrap_or_default();
                    for player in &players {
                        if player.id != old_yapper_id {
                            let player_picks_key = key("team_draft")?.field(&game_id)?.field("round")?.field("player_to_picks")?.field(&player.id)?.get_key()?;
                            self.redis_client.del(&player_picks_key).await?;
                        }
                    }
                    
                    self.redis_client.del(&player_points_key).await?;
                    
                    messages.push(GameMessage::HaltTimer(
                        crate::game::messages::HaltTimer {
                            end_timestamp_ms: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as u64 + 8000,
                            reason: TimerReason::TeamDraft(TeamDraftTimerReason::WaitingForNextRound),
                        }
                    ));
                    messages.push(GameMessage::TeamDraft(TeamDraftMessage::NextRound(
                        crate::team_draft::messages::NextRound {
                            yapper_id: next_yapper_id,
                            yapper_index: next_yapper_index,
                            round: current_round + 1,
                            team_size: DEFAULT_TEAM_SIZE,
                        }
                    )));
                }
                
                Ok(messages)
            },
            TeamDraftMessage::AwardingPhase(_) => {
                // Server-only message, do nothing
                Ok(vec![])
            },
            TeamDraftMessage::CompleteGame(_) => {
                // Server-only message, do nothing
                Ok(vec![])
            },
            TeamDraftMessage::NextRound(_) => {
                // Server-only message, do nothing
                Ok(vec![])
            },
            TeamDraftMessage::NextDrafter(_) => {
                // Server-only message, do nothing
                Ok(vec![])
            },
        }
    }
}


impl TeamDraftState {
    pub fn new(
        yapper_id: String,
        yapper_index: u8,
        max_rounds: u8,
    ) -> Self {
        Self {
            yapper_id,
            yapper_index,
            max_rounds,
            phase: TeamDraftPhase::YapperChoosing,
            round_data: Round {
                round: 1,
                pool: String::new(),
                competition: String::new(),
                team_size: DEFAULT_TEAM_SIZE,
                player_to_picks: HashMap::new(),
                starting_drafter_id: String::new(),
                current_drafter_id: String::new(),
            },
            player_points: HashMap::new(),
        }
    }
}