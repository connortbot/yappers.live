use axum::{
    routing::get,
    Router,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use tower_http::cors::{CorsLayer, Any};

use std::sync::Arc;
mod lobby_controller;
mod websocket_controller;

mod lobby;
use lobby::lobby_manager::LobbyManager;

#[derive(OpenApi)]
#[openapi(
    paths(ping),
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
        (status = 200, description = "ping", body = String)
    )
)]
async fn ping() -> &'static str {
    "pong"
}

#[tokio::main]
async fn main() {
    let lobby_manager = Arc::new(LobbyManager::new());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(ping))
        .merge(websocket_controller::routes(lobby_manager.clone()))
        .merge(lobby_controller::routes(lobby_manager))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();

    println!("Server running on http://0.0.0.0:8080");
    println!("Swagger UI available at http://0.0.0.0:8080/swagger-ui");
    axum::serve(listener, app).await.unwrap();
}
