use std::time::{Instant};
use std::sync::Arc;
use tokio::sync::mpsc;
use crate::game::messages::{WebSocketMessage, GameMessage, GameMode, GameStartedMessage, client_safe_ws_message};
use crate::game::game_manager::{GameManager, GameResult};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedMessage {
    #[serde(skip, default = "Instant::now")]
    pub timestamp: Instant,
    pub message: WebSocketMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessageChunk {
    pub game_id: String,
    pub player_id: String, // broadcasted messages are in reaction to this player
    pub messages: Vec<GameMessage>,
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
                    
                    let player_id = ws_message.player_id.clone();
                    let messages = vec![client_safe_ws_message(ws_message).message];
                    game_manager.publish_broadcast_messages(game_id, &player_id, messages).await?;
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
                        match game_manager.get_game(game_id).await {
                            Ok(Some(game)) => {
                                let num_players = game.players.len() as u8;
                                
                                if let Err(e) = game_manager.get_team_draft_service()
                                    .set_game_settings(game_id.to_string(), num_players).await {
                                    println!("[GameProcessor] Error setting team draft game settings: {}", e);
                                }
                                
                                println!("[GameProcessor] Started TeamDraft game with {} players", num_players);
                                
                                let initial_state = crate::team_draft::types::TeamDraftState::new(
                                    game.host_id.clone(),
                                    0, // host starts as yapper at index 0
                                    num_players,
                                );
                                
                                WebSocketMessage {
                                    game_id: ws_message.game_id.clone(),
                                    message: GameStarted(GameStartedMessage {
                                        game_type: GameMode::TeamDraft,
                                        initial_team_draft_state: Some(initial_state),
                                    }),
                                    player_id: player_id.clone(),
                                    auth_token: None,
                                }
                            }
                            Ok(None) => {
                                println!("[GameProcessor] Game {} not found for GameStarted", game_id);
                                client_safe_ws_message(ws_message)
                            }
                            Err(e) => {
                                println!("[GameProcessor] Error getting game for GameStarted: {}", e);
                                client_safe_ws_message(ws_message)
                            }
                        }
                    }
                };
                
                let broadcast_messages = vec![updated_ws_message.message.clone()];
                game_manager.publish_broadcast_messages(game_id, &player_id, broadcast_messages).await?;
            }
            TeamDraft(team_draft_message) => {
                println!("[GameProcessor] Processing TeamDraft message: {:?}", team_draft_message);
                if let Some(auth_token) = &ws_message.auth_token {
                    let required_player_id = match game_manager.get_team_draft_service()
                        .get_correct_player_source_id(game_id.to_string(), team_draft_message.clone()).await {
                        Ok(player_id) => player_id,
                        Err(e) => {
                            println!("[GameProcessor] Error getting required player ID for team draft message: {}", e);
                            return Ok(());
                        }
                    };
                    
                    match game_manager.is_authorized(&required_player_id, auth_token).await {
                        Ok(true) => {
                            let players = match game_manager.get_game(game_id).await {
                                Ok(Some(game)) => game.players,
                                Ok(None) => {
                                    println!("[GameProcessor] Game {} not found for team draft message", game_id);
                                    return Ok(());
                                }
                                Err(e) => {
                                    println!("[GameProcessor] Error getting game for team draft: {}", e);
                                    return Ok(());
                                }
                            };
                            
                            match game_manager.get_team_draft_service()
                                .handle_message(game_id.to_string(), players, team_draft_message.clone()).await {
                                Ok(broadcast_messages) => {
                                    if !broadcast_messages.is_empty() {
                                        println!("[GameProcessor] Publishing {} broadcast messages", broadcast_messages.len());
                                        game_manager.publish_broadcast_messages(game_id, &required_player_id, broadcast_messages).await?;
                                    }
                                }
                                Err(e) => {
                                    println!("[GameProcessor] Error handling team draft message: {}", e);
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
                let player_id = ws_message.player_id.clone();
                let client_safe_ws_message = client_safe_ws_message(ws_message);
                let messages = vec![client_safe_ws_message.message];
                game_manager.publish_broadcast_messages(game_id, &player_id, messages).await?;
            }
        }
        
        Ok(())
    }
} 