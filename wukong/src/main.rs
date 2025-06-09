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
mod admin;
mod error;
use error::ErrorResponse;

mod game;
use game::game_manager::GameManager;
mod team_draft;
mod cache;

#[derive(OpenApi)]
#[openapi(
    paths(
        ping,
        game_controller::create_game,
        game_controller::join_game,
        websocket_controller::websocket_handler,
        admin::list_games,
        admin::get_game,
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
    dotenv::dotenv().ok();

    let game_manager = Arc::new(GameManager::new().await);
    
    if let Err(e) = game_manager.clone().start_pubsub().await {
        eprintln!("Failed to start pub/sub routing: {}", e);
        std::process::exit(1);
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(ping))
        .merge(websocket_controller::routes(game_manager.clone()))
        .merge(game_controller::routes(game_manager.clone()))
        .merge(admin::routes(game_manager))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();

    println!("Server running on http://0.0.0.0:8080");
    println!("Swagger UI available at http://0.0.0.0:8080/swagger-ui");
    axum::serve(listener, app).await.unwrap();
}
