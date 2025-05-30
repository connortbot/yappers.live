use axum::{
    routing::get,
    Router,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use tower_http::cors::{CorsLayer, Any};

use std::sync::Arc;
mod game_controller;
mod websocket_controller;
mod error;
use error::ErrorResponse;

mod game;
use game::game_manager::GameManager;

#[derive(OpenApi)]
#[openapi(
    paths(
        ping,
        game_controller::create_game,
        game_controller::join_game,
        websocket_controller::websocket_handler,
    ),
    components(
        schemas(game_controller::CreateGameRequest, game_controller::JoinGameRequest)
    ),
    info(
        title = "Wukong API",
        version = "0.1.0",
        description = "Yappers backend."
    )
)]
struct ApiDoc;


#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "ping", body = String),
        (status = 400, description = "Bad Request", body = ErrorResponse)
    )
)]
async fn ping() -> &'static str {
    "pong"
}

#[tokio::main]
async fn main() {
    let game_manager = Arc::new(GameManager::new());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(ping))
        .merge(websocket_controller::routes(game_manager.clone()))
        .merge(game_controller::routes(game_manager))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();

    println!("Server running on http://0.0.0.0:8080");
    println!("Swagger UI available at http://0.0.0.0:8080/swagger-ui");
    axum::serve(listener, app).await.unwrap();
}
