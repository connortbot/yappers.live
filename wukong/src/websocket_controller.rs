use std::sync::Arc;
use axum::{
    routing::get,
    Router,
    extract::{Path, State, WebSocketUpgrade},
    response::Response,
};
use axum::extract::ws::{WebSocket, Message};
use futures::{StreamExt, SinkExt};
use crate::game::game_manager::GameManager;
use crate::game::messages::WebSocketMessage;
use crate::game::messages::GameMessage::{PlayerLeft, TeamDraft, GameStarted};
use crate::game::messages::{client_safe_ws_message, GameMode, GameStartedMessage};

#[utoipa::path(
    get,
    path = "/ws/{game_id}/{player_id}",
    responses(
        (status = 200, description = "Websocket connection", body = String)
    )
)]
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path((game_id, player_id)): Path<(String, String)>,
    State(game_manager): State<Arc<GameManager>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, game_id, player_id, game_manager))
}

async fn handle_socket(
    socket: WebSocket,
    game_id: String,
    player_id: String,
    game_manager: Arc<GameManager>,
) {
    println!("[WS:: Player {} connected to game {}]", player_id, game_id);

    let broadcaster = match game_manager.get_broadcaster(&game_id).await {
        Ok(tx) => tx,
        Err(e) => {
            println!("[WS:: Error getting broadcaster: {}]", e);
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = socket.split();
    let mut broadcast_rx = broadcaster.subscribe();

    let player_id_for_send = player_id.clone();
    let send_task = tokio::spawn(async move {
        while let Ok(message_json) = broadcast_rx.recv().await {
            if ws_sender.send(Message::Text(message_json.clone().into())).await.is_err() {
                println!("[WS:: Error sending message to player {}: {}]", player_id_for_send, message_json);
            }
        }
    });

    let recv_task = {
        let game_id = game_id.clone();
        let player_id = player_id.clone();
        let game_manager = game_manager.clone();
        
        tokio::spawn(async move {
            while let Some(msg_result) = ws_receiver.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<WebSocketMessage>(&text) {
                            Ok(ws_message) => {
                                if ws_message.game_id != game_id {
                                    println!("[WS:: Game ID mismatch: expected {}, got {}]", game_id, ws_message.game_id);
                                    continue;
                                }
                                
                                if let Some(auth_token) = &ws_message.auth_token {
                                    match game_manager.is_authorized(&player_id, auth_token).await {
                                        Ok(false) => {
                                            println!("[WS:: Unauthorized message from player {}]", player_id);
                                            continue;
                                        }
                                        Err(e) => {
                                            println!("[WS:: Error checking authorization: {}]", e);
                                            continue;
                                        }
                                        _ => {}
                                    }
                                }
                                
                                match &ws_message.message {
                                    PlayerLeft { player_id, .. } => {
                                        if let Err(e) = game_manager.handle_player_left(&game_id, player_id).await {
                                            println!("[WS] Error handling player left: {}", e);
                                        }
                                    }
                                    GameStarted(game_started_msg) => {
                                        let updated_ws_message = match &game_started_msg.game_type {
                                            GameMode::TeamDraft => {
                                                if let Ok(Some(mut game)) = game_manager.get_game(&game_id).await {
                                                    let num_players = game.players.len() as u8;
                                                    game.team_draft.set_game_settings(num_players);
                                                    println!("[WS] Started TeamDraft game with {} players", num_players);
                                                    
                                                    WebSocketMessage {
                                                        game_id: ws_message.game_id.clone(),
                                                        message: GameStarted(GameStartedMessage {
                                                            game_type: GameMode::TeamDraft,
                                                            initial_team_draft_state: Some(game.team_draft.clone()),
                                                        }),
                                                        player_id: ws_message.player_id.clone(),
                                                        auth_token: None,
                                                    }
                                                } else {
                                                    client_safe_ws_message(ws_message)
                                                }
                                            }
                                        };
                                        
                                        if let Err(e) = game_manager.broadcast_to_game(&game_id, updated_ws_message).await {
                                            println!("[WS] Error broadcasting game start message: {}", e);
                                        }
                                    }
                                    TeamDraft(team_draft_message) => {
                                        if let Some(auth_token) = &ws_message.auth_token {
                                            if let Ok(Some(mut game)) = game_manager.get_game(&game_id).await {
                                                let required_player_id = game.team_draft.get_correct_player_source_id(team_draft_message.clone());
                                                
                                                match game_manager.is_authorized(&required_player_id, auth_token).await {
                                                    Ok(true) => {
                                                        if let Some(source_player) = game.players.iter().find(|p| p.id == required_player_id).cloned() {
                                                            let broadcast_messages = game.team_draft.handle_message(source_player, team_draft_message.clone());
                                                            for game_message in broadcast_messages {
                                                                let broadcast_ws_message = WebSocketMessage {
                                                                    game_id: game_id.clone(),
                                                                    message: game_message,
                                                                    player_id: required_player_id.clone(),
                                                                    auth_token: None,
                                                                };
                                                                
                                                                if let Err(e) = game_manager.broadcast_to_game(&game_id, broadcast_ws_message).await {
                                                                    println!("[WS] Error broadcasting team draft message: {}", e);
                                                                }
                                                            }
                                                        } else {
                                                            println!("[WS] Required player {} not found in game", required_player_id);
                                                        }
                                                    }
                                                    Ok(false) => {
                                                        println!("[WS] Player {} not authorized for team draft action", required_player_id);
                                                    }
                                                    Err(e) => {
                                                        println!("[WS] Error checking authorization for team draft: {}", e);
                                                    }
                                                }
                                            } else {
                                                println!("[WS] Game {} not found for team draft message", game_id);
                                            }
                                        } else {
                                            println!("[WS] No auth token provided for team draft message");
                                        }
                                    }
                                    _ => {
                                        let client_safe_ws_message = client_safe_ws_message(ws_message);
                                        if let Err(e) = game_manager.broadcast_to_game(&game_id, client_safe_ws_message).await {
                                            println!("[WS] Error broadcasting message: {}", e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                println!("[WS:: Invalid message format from player {}: {}]", player_id, e);
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        println!("[WS:: Player {} closed connection]", player_id);
                        break;
                    }
                    Err(e) => {
                        println!("[WS:: WebSocket error for player {}: {}]", player_id, e);
                        break;
                    }
                    _ => {}
                }
            }
        })
    };

    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }

    println!("[WS:: Player {} disconnected from game {}]", player_id, game_id);
    if let Err(e) = game_manager.remove_player_from_game(&player_id).await {
        println!("[WS:: Error removing player on disconnect: {}]", e);
    }
}

pub fn routes(game_manager: Arc<GameManager>) -> Router {
    Router::new()
        .route("/ws/{game_id}/{player_id}", get(websocket_handler))
        .with_state(game_manager)
}