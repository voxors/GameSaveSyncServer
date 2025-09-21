mod database;
mod datatype_endpoint;

use crate::database::database::GameDatabase;
use crate::datatype_endpoint::{GameMetadata, GameMetadataCreate, SavePath, SavePathCreate, OS};
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
    request_body = GameMetadataCreate,
    responses(
        (status = 201, description = "game metadata created", body = [String])
    )
)]
async fn post_game_metadata(Json(payload): Json<GameMetadataCreate>) -> StatusCode {
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
        (status = 200, description = "get all games metadata", body = [Vec<GameMetadata>])
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
    get,
    path = "/games/{Id}",
    params(
        ("Id" = String, Path, description = "Id of the game")
    ),
    responses(
        (status = 200, description = "game metadata returned, body = [GameMetadata]"),
        (status = 404, description = "game not found")
    )
)]
async fn get_game_metadata(Path(id): Path<i32>) -> Result<Json<GameMetadata>, StatusCode> {
    match DATABASE.get_game_metadata_by_id(&id) {
        Ok(Some(data)) => Ok(Json(data)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error getting game metadata: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    get,
    path = "/games/{Id}/paths",
    params(
        ("Id" = String, Path, description = "Id of the game")
    ),
    responses(
        (status = 200, description = "game paths returned", body = [Vec<SavePath>]),
    )
)]
async fn get_game_paths(Path(id): Path<i32>) -> Result<Json<Vec<SavePath>>, StatusCode> {
    match DATABASE.get_paths_by_game_id(id) {
        Ok(data) => Ok(Json(data)),
        Err(e) => {
            eprintln!("Error getting game paths: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    get,
    path = "/games/{Id}/paths/{OS}",
    params(
        ("Id" = String, Path, description = "Id of the game"),
        ("OS" = String, Path, description = "Operating system (windows or linux)")
    ),
    responses(
        (status = 200, description = "game paths returned", body = [Vec<String>]),
        (status = 400, description = "invalid operating system"),
        (status = 404, description = "game not found")
    )
)]
async fn get_game_paths_by_os(Path((id, os)): Path<(i32, String)>) -> Result<Json<Vec<String>>, StatusCode> {
    let os = match os.as_str() {
        "windows" => OS::Windows,
        "linux" => OS::Linux,
        _ => {
            eprintln!("Unsupported operating system: {}", os);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    match DATABASE.get_paths_by_game_id_and_os(id,os) {
        Ok(data) => Ok(Json(data)),
        Err(e) => {
            eprintln!("Error getting game paths: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    post,
    path = "/games/{Id}/paths",
    params(
        ("Id" = String, Path, description = "Id of the game"),
    ),
    request_body = SavePathCreate,
    responses(
        (status = 201, description = "game path created"),
    )
)]
async fn post_game_path(Path(id): Path<i32>, Json(payload): Json<SavePathCreate>) -> StatusCode {
    match DATABASE.add_game_path(id, &payload) {
        Ok(()) => StatusCode::CREATED,
        Err(e) => {
            eprintln!("Error adding game path: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(post_game_metadata, get_game_metadata, get_games_metadata, get_game_paths, post_game_path, get_game_paths_by_os),
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    Lazy::force(&DATABASE);
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/games", post(post_game_metadata))
        .route("/games", get(get_games_metadata))
        .route("/games/{Id}", get(get_game_metadata))
        .route("/games/{Id}/paths", get(get_game_paths))
        .route("/games/{Id}/paths", post(post_game_path))
        .route("/games/{Id}/paths/{OS}", get(get_game_paths_by_os))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
