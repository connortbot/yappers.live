use std::sync::Arc;
use axum::{
    routing::get,
    Router,
    extract::{Path, State, WebSocketUpgrade},
    response::Response,
};
use axum::extract::ws::{WebSocket, Message};
use futures::{StreamExt, SinkExt};
use crate::lobby::lobby_manager::LobbyManager;

#[utoipa::path(
    get,
    path = "/ws/{lobby_id}/{player_id}",
    responses(
        (status = 200, description = "Websocket connection", body = String)
    )
)]
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path((lobby_id, player_id)): Path<(String, String)>,
    State(lobby_manager): State<Arc<LobbyManager>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, lobby_id, player_id, lobby_manager))
}

async fn handle_socket(
    socket: WebSocket,
    lobby_id: String,
    player_id: String,
    lobby_manager: Arc<LobbyManager>,
) {
    println!("[WS:: Player {} connected to lobby {}]", player_id, lobby_id);

    let broadcaster = match lobby_manager.get_broadcaster(&lobby_id).await {
        Ok(tx) => tx,
        Err(e) => {
            println!("[WS:: Error getting broadcaster: {}]", e);
            return;
        }
    };

    let (mut ws_sender, mut ws_receiver) = socket.split();
    let mut broadcast_rx = broadcaster.subscribe();

    let send_task = tokio::spawn(async move {
        while let Ok(message) = broadcast_rx.recv().await {
            if ws_sender.send(Message::from(message)).await.is_err() {
                break;
            }
        }
    });

    let recv_task = {
        let lobby_id = lobby_id.clone();
        let player_id = player_id.clone();
        let lobby_manager = lobby_manager.clone();
        
        tokio::spawn(async move {
            while let Some(msg_result) = ws_receiver.next().await {
                match msg_result {
                    Ok(Message::Text(text)) => {
                        let text_string = text.to_string();
                        
                        let formatted_message = format!("{}: {}", player_id, text_string);
                        
                        if let Err(e) = lobby_manager.broadcast_to_lobby(&lobby_id, formatted_message).await {
                            println!("[WS] Error broadcasting message: {}", e);
                            break;
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

    println!("[WS:: Player {} disconnected from lobby {}]", player_id, lobby_id);
}

pub fn routes(lobby_manager: Arc<LobbyManager>) -> Router {
    Router::new()
        .route("/ws/{lobby_id}/{player_id}", get(websocket_handler))
        .with_state(lobby_manager)
}