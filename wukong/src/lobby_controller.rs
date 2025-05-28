use axum::{
    routing::{post},
    Router,
    Json,
    extract::State,
};
use serde::Deserialize;
use utoipa::ToSchema;
use std::sync::Arc;
use crate::lobby::lobby_manager::{LobbyManager, Lobby};

#[derive(Deserialize, ToSchema)]
pub struct CreateLobbyRequest {
    pub username: String,
}

#[derive(Deserialize, ToSchema)]
pub struct JoinLobbyRequest {
    pub username: String,
    pub lobby_id: String,
}

#[utoipa::path(
    post,
    path = "/lobby/create",
    request_body = CreateLobbyRequest,
    responses(
        (status = 200, description = "Create lobby", body = String)
    )
)]
pub async fn create_lobby(
    State(lobby_manager): State<Arc<LobbyManager>>,
    Json(payload): Json<CreateLobbyRequest>
) -> Result<Json<Lobby>, String> {
    println!("[POST: /lobby/create]");
    match lobby_manager.create_lobby(payload.username).await {
        Ok(lobby) => {
            println!("[POST: /lobby/create]");
            Ok(Json(lobby))
        }
        Err(e) => {
            println!("[POST: /lobby/create] Error: {}", e);
            Err(e.to_string())
        }
    }
}

#[utoipa::path(
    post,
    path = "/lobby/join",
    request_body = JoinLobbyRequest,
    responses(
        (status = 200, description = "Join lobby", body = String)
    )
)]
pub async fn join_lobby(
    State(lobby_manager): State<Arc<LobbyManager>>,
    Json(payload): Json<JoinLobbyRequest>
) -> Result<Json<Lobby>, String> {
    println!("[POST: /lobby/join]");
    match lobby_manager.join_lobby(payload.username, payload.lobby_id).await {
        Ok(lobby) => {
            println!("[POST: /lobby/join]");
            Ok(Json(lobby))
        }
        Err(e) => {
            println!("[POST: /lobby/join] Error: {}", e);
            Err(e.to_string())
        }
    }
}

pub fn routes(lobby_manager: Arc<LobbyManager>) -> Router {
    Router::new()
        .route("/lobby/create", post(create_lobby))
        .route("/lobby/join", post(join_lobby))
        .with_state(lobby_manager)
}