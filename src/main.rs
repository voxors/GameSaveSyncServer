mod database;
mod datatype_endpoint;

use crate::database::database::GameDatabase;
use crate::datatype_endpoint::GameMetadata;
use axum::{
    Json, Router, extract::Path, http::StatusCode, routing::get, routing::post, routing::put,
};
use once_cell::sync::Lazy;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub static DATABASE: Lazy<GameDatabase> = Lazy::new(|| GameDatabase::new());

#[utoipa::path(
    post,
    path = "/games",
    params(),
    request_body = GameMetadata,
    responses(
        (status = 201, description = "game metadata created", body = [String])
    )
)]
async fn post_game_metadata(Json(payload): Json<GameMetadata>) -> StatusCode {
    match DATABASE.add_game_metadata(&payload) {
        Ok(()) => StatusCode::CREATED,
        Err(e) => {
            eprintln!("Error adding game metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[utoipa::path(
    get,
    path = "/games",
    params(),
    responses(
        (status = 200, description = "get all games metadata", body = [String])
    )
)]
async fn get_games_metadata() -> Result<Json<Vec<GameMetadata>>, StatusCode> {
    match DATABASE.get_games_metadata() {
        Ok(data) => Ok(Json(data)),
        Err(e) => {
            eprintln!("Error retrieving game metadata: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    put,
    path = "/games/{internal_name}",
    params(
        ("internal_name" = String, Path, description = "Internal name of the game to update")
    ),
    request_body = GameMetadata,
    responses(
        (status = 204, description = "game metadata updated"),
        (status = 404, description = "game not found")
    )
)]
async fn put_game_metadata(
    Path(internal_name): Path<String>,
    Json(payload): Json<GameMetadata>,
) -> StatusCode {
    match DATABASE.update_game_metadata_by_internal_name(&internal_name, &payload) {
        Ok(true) => StatusCode::NO_CONTENT,
        Ok(false) => StatusCode::NOT_FOUND,
        Err(e) => {
            eprintln!("Error updating game metadata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[utoipa::path(
    get,
    path = "/games/{internal_name}",
    params(
        ("internal_name" = String, Path, description = "Internal name of the game")
    ),
    responses(
        (status = 200, description = "game metadata return"),
        (status = 404, description = "game not found")
    )
)]
async fn get_game_metadata(
    Path(internal_name): Path<String>
) -> Result<Json<GameMetadata>, StatusCode> {
    match DATABASE.get_game_metadata_by_internal_name(&internal_name) {
        Ok(Some(data)) => Ok(Json(data)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error getting game metadata: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(post_game_metadata, get_game_metadata, get_games_metadata, put_game_metadata),
    components(schemas(GameMetadata))
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    Lazy::force(&DATABASE);
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/games", post(post_game_metadata))
        .route("/games", get(get_games_metadata))
        .route("/games/{internal_name}", put(put_game_metadata))
        .route("/games/{internal_name}", get(get_game_metadata))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
