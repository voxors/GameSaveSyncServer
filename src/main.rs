mod database;
mod datatype_endpoint;

use crate::database::database::GameDatabase;
use crate::datatype_endpoint::GameMetadata;
use axum::{Json, Router, http::StatusCode, routing::post};
use once_cell::sync::Lazy;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub static DATABASE: Lazy<GameDatabase> = Lazy::new(|| GameDatabase::new());

#[utoipa::path(
    post,
    path = "/game_metadata",
    params(
        GameMetadata
    ),
    responses(
        (status = 201, description = "game metadata created", body = [String])
    )
)]
async fn post_game_metadata(Json(payload): Json<GameMetadata>) -> StatusCode {
    match DATABASE.add_game_metadata(&payload) {
        Ok(()) => (),
        Err(e) => {
            eprintln!("Error adding game metadata: {}", e);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }
    StatusCode::CREATED
}

#[derive(OpenApi)]
#[openapi(paths(post_game_metadata), components(schemas(GameMetadata)))]
struct ApiDoc;

#[tokio::main]
async fn main() {
    Lazy::force(&DATABASE);
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/games", post(post_game_metadata))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
