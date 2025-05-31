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
use crate::game::messages::GameMessage::PlayerLeft;
use crate::game::messages::client_safe_ws_message;

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