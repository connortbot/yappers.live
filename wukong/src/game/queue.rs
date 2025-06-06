use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use tokio::sync::mpsc;
use crate::game::messages::{WebSocketMessage, GameMessage, GameMode, GameStartedMessage, client_safe_ws_message};
use crate::game::game_manager::{GameManager, GameResult};

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
            BackToLobby { .. } => {
                if let Ok(Some(game)) = game_manager.get_game(game_id).await {
                    if ws_message.player_id != game.host_id {
                        println!("[GameProcessor] Player {} not authorized to send game back to lobby (only host {} can)", ws_message.player_id, game.host_id);
                        return Ok(());
                    }
                    
                    game_manager.broadcast_to_game(game_id, client_safe_ws_message(ws_message)).await?;
                } else {
                    println!("[GameProcessor] Game {} not found for BackToLobby message", game_id);
                }
            }
            PlayerLeft { player_id, .. } => {
                game_manager.handle_player_left(game_id, player_id).await?;
            }
            GameStarted(game_started_msg) => {
                let player_id = ws_message.player_id.clone();
                let updated_ws_message = match &game_started_msg.game_type {
                    GameMode::TeamDraft => {
                        match game_manager.modify_game(game_id, |game| {
                            let num_players = game.players.len() as u8;
                            game.team_draft.set_game_settings(num_players);
                            println!("[GameProcessor] Started TeamDraft game with {} players", num_players);
                            game.team_draft.clone()
                        }).await {
                            Ok(updated_team_draft_state) => {
                                let ws_msg = WebSocketMessage {
                                    game_id: ws_message.game_id.clone(),
                                    message: GameStarted(GameStartedMessage {
                                        game_type: GameMode::TeamDraft,
                                        initial_team_draft_state: Some(updated_team_draft_state),
                                    }),
                                    player_id: player_id.clone(),
                                    auth_token: None,
                                };
                                ws_msg
                            }
                            Err(e) => {
                                println!("[GameProcessor] Error modifying game for GameStarted: {}", e);
                                client_safe_ws_message(ws_message)
                            }
                        }
                    }
                };
                
                let broadcast_messages = vec![updated_ws_message.message.clone()];
                Self::process_broadcast_messages(game_id, &player_id, broadcast_messages, game_manager).await?;
            }
            TeamDraft(team_draft_message) => {
                println!("[GameProcessor] Processing TeamDraft message: {:?}", team_draft_message);
                if let Some(auth_token) = &ws_message.auth_token {
                    let required_player_id = if let Ok(Some(game)) = game_manager.get_game(game_id).await {
                        let required_id = game.team_draft.get_correct_player_source_id(team_draft_message.clone());
                        required_id
                    } else {
                        println!("[GameProcessor] Game {} not found for team draft message", game_id);
                        return Ok(());
                    };
                    
                    match game_manager.is_authorized(&required_player_id, auth_token).await {
                        Ok(true) => {
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
                    };
                    
                    game_manager.broadcast_to_game(game_id, broadcast_ws_message).await?;
                    
                    let current_time_ms = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;
                    
                    if halt_timer.end_timestamp_ms > current_time_ms {
                        let sleep_duration_ms = halt_timer.end_timestamp_ms - current_time_ms;
                        println!("[GameProcessor] Halting for {} ms until timestamp {}", sleep_duration_ms, halt_timer.end_timestamp_ms);
                        tokio::time::sleep(tokio::time::Duration::from_millis(sleep_duration_ms)).await;
                    } else {
                        println!("[GameProcessor] Timer already expired, continuing immediately");
                    }
                }
                _ => {
                    println!("[GameProcessor] Broadcasting regular message: {:?}", game_message);
                    let broadcast_ws_message = WebSocketMessage {
                        game_id: game_id.to_string(),
                        message: game_message.clone(),
                        player_id: player_id.to_string(),
                        auth_token: None,
                    };
                    
                    game_manager.broadcast_to_game(game_id, broadcast_ws_message).await?;
                }
            }
        }
        Ok(())
    }
} 