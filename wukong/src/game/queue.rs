use std::time::Instant;
use std::sync::Arc;
use tokio::sync::mpsc;
use crate::game::messages::{WebSocketMessage, GameMessage, GameMode, GameStartedMessage, client_safe_ws_message};
use crate::game::game_manager::{GameManager, GameResult};
use crate::team_draft::messages::TeamDraftMessage;

#[derive(Debug, Clone)]
pub struct QueuedMessage {
    #[allow(dead_code)] // Not used yet, but will be useful for metrics/debugging
    pub timestamp: Instant,
    pub message: WebSocketMessage,
}

pub struct GameProcessor {
    pub sender: mpsc::Sender<QueuedMessage>,
}

impl GameProcessor {
    pub fn new() -> (Self, mpsc::Receiver<QueuedMessage>) {
        let (tx, rx) = mpsc::channel(100);
        
        let processor = Self {
            sender: tx,
        };
        
        (processor, rx)
    }
    
    pub async fn process_messages(
        mut rx: mpsc::Receiver<QueuedMessage>,
        game_id: String,
        game_manager: Arc<GameManager>,
    ) {
        println!("[GameProcessor] Starting message processing for game {}", game_id);
        
        while let Some(queued_message) = rx.recv().await {
            println!("[GameProcessor] Processing message for game {}: {:?}", 
                game_id, queued_message.message.message);
            
            if let Err(e) = Self::process_message(&game_id, queued_message.message, &game_manager).await {
                println!("[GameProcessor] Error processing message: {}", e);
            }
        }
        
        println!("[GameProcessor] Message processing for game {} shutting down", game_id);
    }
    
    async fn process_message(
        game_id: &str, 
        ws_message: WebSocketMessage, 
        game_manager: &Arc<GameManager>
    ) -> GameResult<()> {
        use GameMessage::*;
        
        match &ws_message.message {
            PlayerLeft { player_id, .. } => {
                game_manager.handle_player_left(game_id, player_id).await?;
            }
            GameStarted(game_started_msg) => {
                let updated_ws_message = match &game_started_msg.game_type {
                    GameMode::TeamDraft => {
                        match game_manager.modify_game(game_id, |game| {
                            let num_players = game.players.len() as u8;
                            game.team_draft.set_game_settings(num_players);
                            println!("[GameProcessor] Started TeamDraft game with {} players", num_players);
                            game.team_draft.clone()
                        }).await {
                            Ok(updated_team_draft_state) => {
                                WebSocketMessage {
                                    game_id: ws_message.game_id.clone(),
                                    message: GameStarted(GameStartedMessage {
                                        game_type: GameMode::TeamDraft,
                                        initial_team_draft_state: Some(updated_team_draft_state),
                                    }),
                                    player_id: ws_message.player_id.clone(),
                                    auth_token: None,
                                    action_key: None,
                                }
                            }
                            Err(e) => {
                                println!("[GameProcessor] Error modifying game for GameStarted: {}", e);
                                client_safe_ws_message(ws_message)
                            }
                        }
                    }
                };
                
                game_manager.broadcast_to_game(game_id, updated_ws_message).await?;
            }
            TeamDraft(team_draft_message) => {
                println!("[GameProcessor] Processing TeamDraft message: {:?}", team_draft_message);
                if let Some(auth_token) = &ws_message.auth_token {
                    let (required_player_id, requires_action_key) = if let Ok(Some(game)) = game_manager.get_game(game_id).await {
                        let required_id = game.team_draft.get_correct_player_source_id(team_draft_message.clone());
                        let requires_key = game.team_draft.requires_action_key(team_draft_message.clone());
                        (required_id, requires_key)
                    } else {
                        println!("[GameProcessor] Game {} not found for team draft message", game_id);
                        return Ok(());
                    };
                    
                    if requires_action_key && ws_message.action_key.is_none() {
                        println!("[GameProcessor] Action key required but not provided for message: {:?}", team_draft_message);
                        return Ok(());
                    }
                    
                    match game_manager.is_authorized(&required_player_id, auth_token).await {
                        Ok(true) => {
                            if let Some(action_key) = &ws_message.action_key {
                                let action_key_valid = match game_manager.modify_game(game_id, |game| {
                                    if game.team_draft.action_timer_manager.validate_action_key(action_key) {
                                        game.team_draft.action_timer_manager.consume_action_key(action_key).is_ok()
                                    } else {
                                        false
                                    }
                                }).await {
                                    Ok(valid) => valid,
                                    Err(_) => false,
                                };
                                
                                if !action_key_valid {
                                    println!("[GameProcessor] Invalid or expired action key: {}", action_key);
                                    return Ok(());
                                }
                                
                                println!("[GameProcessor] Valid action key used: {}", action_key);
                            }
                            
                            match game_manager.modify_game(game_id, |game| {
                                let players = game.players.clone();
                                println!("[GameProcessor] Handling team draft message: {:?}", team_draft_message);
                                Some(game.team_draft.handle_message(players, team_draft_message.clone()))
                            }).await {
                                Ok(Some(broadcast_messages)) => {
                                    println!("[GameProcessor] Processing broadcast messages: {:?}", broadcast_messages);
                                    Self::process_broadcast_messages(game_id, &required_player_id, broadcast_messages, game_manager).await?;
                                }
                                Ok(None) => {}
                                Err(e) => {
                                    println!("[GameProcessor] Error modifying game for team draft: {}", e);
                                }
                            }
                        }
                        Ok(false) => {
                            println!("[GameProcessor] Player {} not authorized for team draft action", required_player_id);
                        }
                        Err(e) => {
                            println!("[GameProcessor] Error checking authorization for team draft: {}", e);
                        }
                    }
                } else {
                    println!("[GameProcessor] No auth token provided for team draft message");
                }
            }
            _ => {
                let client_safe_ws_message = client_safe_ws_message(ws_message);
                game_manager.broadcast_to_game(game_id, client_safe_ws_message).await?;
            }
        }
        
        Ok(())
    }
    
    async fn process_broadcast_messages(
        game_id: &str,
        player_id: &str,
        broadcast_messages: Vec<GameMessage>,
        game_manager: &Arc<GameManager>,
    ) -> GameResult<()> {
        for game_message in broadcast_messages {
            println!("[GameProcessor] Processing broadcast message: {:?}", game_message);
            match &game_message {
                GameMessage::HaltTimer(halt_timer) => {
                    let broadcast_ws_message = WebSocketMessage {
                        game_id: game_id.to_string(),
                        message: game_message.clone(),
                        player_id: player_id.to_string(),
                        auth_token: None,
                        action_key: None,
                    };
                    
                    game_manager.broadcast_to_game(game_id, broadcast_ws_message).await?;
                    
                    println!("[GameProcessor] Halting for {} seconds", halt_timer.duration_seconds);
                    tokio::time::sleep(tokio::time::Duration::from_secs(halt_timer.duration_seconds)).await;
                }
                GameMessage::ActionTimer(turn_timer) => {
                    let action_key = game_manager.modify_game(game_id, |game| {
                        game.team_draft.action_timer_manager.generate_action_key(game_id)
                    }).await?;
                    
                    let turn_timer_with_key = crate::game::messages::ActionTimer {
                        duration_seconds: turn_timer.duration_seconds,
                        action_key: action_key.clone(),
                        default_action: turn_timer.default_action.clone(),
                        reason: turn_timer.reason.clone(),
                    };
                    
                    let turn_timer_ws_message = WebSocketMessage {
                        game_id: game_id.to_string(),
                        message: GameMessage::ActionTimer(turn_timer_with_key),
                        player_id: player_id.to_string(),
                        auth_token: None,
                        action_key: None,
                    };
                    
                    game_manager.broadcast_to_game(game_id, turn_timer_ws_message).await?;
                    
                    if let GameMessage::TeamDraft(team_draft_msg) = turn_timer.default_action.as_ref() {
                        Self::spawn_turn_timer_task(
                            game_id.to_string(),
                            action_key,
                            turn_timer.duration_seconds,
                            team_draft_msg.clone(),
                            game_manager.clone(),
                        );
                    }
                }
                _ => {
                    println!("[GameProcessor] Broadcasting regular message: {:?}", game_message);
                    let broadcast_ws_message = WebSocketMessage {
                        game_id: game_id.to_string(),
                        message: game_message.clone(),
                        player_id: player_id.to_string(),
                        auth_token: None,
                        action_key: None,
                    };
                    
                    game_manager.broadcast_to_game(game_id, broadcast_ws_message).await?;
                }
            }
        }
        Ok(())
    }
    
    fn spawn_turn_timer_task(
        game_id: String,
        action_key: String,
        duration_seconds: u64,
        default_action: TeamDraftMessage,
        game_manager: Arc<GameManager>,
    ) {
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(duration_seconds)).await;
            
            let should_execute_default = match game_manager.modify_game(&game_id, |game| {
                game.team_draft.action_timer_manager.validate_action_key(&action_key)
            }).await {
                Ok(is_valid) => {
                    if is_valid {
                        let _ = game_manager.modify_game(&game_id, |game| {
                            game.team_draft.action_timer_manager.expire_action_key(&action_key);
                        }).await;
                        true
                    } else {
                        false
                    }
                }
                Err(_) => false,
            };
            
            if should_execute_default {
                println!("[GameProcessor] Timer expired, executing default action for game {}", game_id);
                
                let acting_player_id = match game_manager.get_game(&game_id).await {
                    Ok(Some(game)) => {
                        game.team_draft.get_correct_player_source_id(default_action.clone())
                    }
                    _ => {
                        println!("[GameProcessor] Could not get game for default action");
                        return;
                    }
                };
                
                let default_ws_message = WebSocketMessage {
                    game_id: game_id.clone(),
                    message: GameMessage::TeamDraft(default_action),
                    player_id: acting_player_id.clone(),
                    auth_token: Some(crate::team_draft::state::SERVER_ONLY_AUTHORIZED.to_string()),
                    action_key: None,
                };
                
                if let Err(e) = GameManager::enqueue_message_with_manager(game_manager, &game_id, default_ws_message).await {
                    println!("[GameProcessor] Failed to enqueue default action: {}", e);
                }
            }
        });
    }
} 