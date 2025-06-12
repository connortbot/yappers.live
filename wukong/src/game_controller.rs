use axum::{
    routing::{post},
    Router,
    extract::State,
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use std::sync::Arc;
use crate::game::game_manager::{GameManager};
use crate::game::types::Game;

use crate::error::ErrorResponse;

#[derive(Deserialize, ToSchema)]
pub struct CreateGameRequest {
    pub username: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateGameResponse {
    pub game: Game,
    pub auth_token: String,
}

#[derive(Deserialize, ToSchema)]
pub struct JoinGameRequest {
    pub username: String,
    pub game_code: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct JoinGameResponse {
    pub game: Game,
    pub auth_token: String,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let status_code = StatusCode::BAD_REQUEST;
        let json = Json(self);
        (status_code, json).into_response()
    }
}

#[utoipa::path(
    post,
    path = "/game/create",
    request_body = CreateGameRequest,
    responses(
        (status = 200, description = "Create game", body = CreateGameResponse),
        (status = 400, description = "Bad Request", body = ErrorResponse)
    )
)]
pub async fn create_game(
    State(game_manager): State<Arc<GameManager>>,
    Json(payload): Json<CreateGameRequest>
) -> Result<Json<CreateGameResponse>, ErrorResponse> {
    println!("[POST: /game/create]");
    match game_manager.create_game(payload.username).await {
        Ok(game_entry) => {
            println!("[POST: /game/create]");
            Ok(Json(CreateGameResponse { game: game_entry.game, auth_token: game_entry.auth_token }))
        }
        Err(e) => {
            println!("[POST: /game/create] Error: {}", e);
            Err(e)
        }
    }
}

#[utoipa::path(
    post,
    path = "/game/join",
    request_body = JoinGameRequest,
    responses(
        (status = 200, description = "Join game", body = JoinGameResponse),
        (status = 400, description = "Bad Request", body = ErrorResponse)
    )
)]
pub async fn join_game(
    State(game_manager): State<Arc<GameManager>>,
    Json(payload): Json<JoinGameRequest>
) -> Result<Json<JoinGameResponse>, ErrorResponse> {
    println!("[POST: /game/join]");
    match game_manager.join_game_by_code(payload.username, payload.game_code).await {
        Ok(game_entry) => {
            println!("[POST: /game/join]");
            Ok(Json(JoinGameResponse { game: game_entry.game, auth_token: game_entry.auth_token }))
        }
        Err(e) => {
            println!("[POST: /game/join] Error: {}", e);
            Err(e)
        }
    }
}

pub fn routes(game_manager: Arc<GameManager>) -> Router {
    Router::new()
        .route("/game/create", post(create_game))
        .route("/game/join", post(join_game))
        .with_state(game_manager)
}