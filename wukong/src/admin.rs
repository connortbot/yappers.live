use std::sync::Arc;
use axum::{
    routing::get,
    Router,
    extract::{State, Query},
    response::Response,
    Json,
    http::{Request, StatusCode},
    middleware::{self, Next},
    body::Body,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::game::game_manager::{GameManager, Game};
use crate::error::ErrorResponse;

#[derive(Serialize, ToSchema)]
pub struct GamesListResponse {
    pub count: usize,
    pub game_ids: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct GameDetailsResponse {
    pub game: Game,
}

#[derive(Deserialize)]
pub struct GameIdQuery {
    pub id: String,
}

pub async fn admin_auth_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let admin_password = std::env::var("WUKONG_ADMIN_PASSWORD")
        .map_err(|_| {
            println!("[Admin] WUKONG_ADMIN_PASSWORD environment variable not set");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let auth_header = request
        .headers()
        .get("Wukong-Admin")
        .and_then(|h| h.to_str().ok());

    println!("[Admin] Auth header present: {}", auth_header.is_some());
    
    match auth_header {
        Some(password) if password == admin_password => {
            println!("[Admin] Authentication successful");
            Ok(next.run(request).await)
        }
        _ => {
            println!("[Admin] Authentication failed");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

#[utoipa::path(
    get,
    path = "/admin/games",
    responses(
        (status = 200, description = "List of games", body = GamesListResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    )
)]
pub async fn list_games(
    State(game_manager): State<Arc<GameManager>>,
) -> Result<Json<GamesListResponse>, StatusCode> {
    match game_manager.get_all_games().await {
        Ok(games) => {
            let game_ids: Vec<String> = games.iter().map(|g| g.id.clone()).collect();
            let response = GamesListResponse {
                count: games.len(),
                game_ids,
            };
            Ok(Json(response))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[utoipa::path(
    get,
    path = "/admin/game",
    params(
        ("id" = String, Query, description = "Game ID")
    ),
    responses(
        (status = 200, description = "Game details", body = GameDetailsResponse),
        (status = 404, description = "Game not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    )
)]
pub async fn get_game(
    State(game_manager): State<Arc<GameManager>>,
    Query(params): Query<GameIdQuery>,
) -> Result<Json<GameDetailsResponse>, StatusCode> {
    match game_manager.get_game(&params.id).await {
        Ok(Some(game)) => {
            let response = GameDetailsResponse { game };
            Ok(Json(response))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub fn routes(game_manager: Arc<GameManager>) -> Router {
    Router::new()
        .route("/admin/games", get(list_games))
        .route("/admin/game", get(get_game))
        .layer(middleware::from_fn(admin_auth_middleware))
        .with_state(game_manager)
} 