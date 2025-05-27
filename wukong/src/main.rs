use axum::{
    routing::get,
    Router,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(hello_world),
    info(
        title = "Yappers API",
        version = "0.1.0",
        description = "API for Yappers multiplayer party games"
    )
)]
struct ApiDoc;


#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Hello message", body = String)
    )
)]
async fn hello_world() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(hello_world))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();

    println!("Server running on http://0.0.0.0:8080");
    println!("Swagger UI available at http://0.0.0.0:8080/swagger-ui");
    axum::serve(listener, app).await.unwrap();
}
