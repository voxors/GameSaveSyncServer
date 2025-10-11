mod const_var;
mod database;
mod datatype_endpoint;
mod file_system;
mod ludusavi;
mod ludusavi_datatype;
mod openapi;
mod route_executable;
mod route_games;
mod route_paths;
mod route_saves;
mod route_yaml_import;

use crate::const_var::{DATA_DIR, MAX_BODY_SIZE, ROOT_API_PATH};
use crate::database::interface::GameDatabase;
use crate::file_system::create_fs_structure;
use crate::openapi::ApiDoc;
use crate::route_executable::{
    get_game_executables, get_game_executables_by_os, post_game_executable,
};
use crate::route_games::{get_game_metadata, get_games_metadata, post_game_metadata};
use crate::route_paths::{get_game_paths, get_game_paths_by_os, post_game_path};
use crate::route_saves::{
    get_game_save_by_uuid, get_game_saves_reference_by_path_id, post_game_save_by_path_id,
};
use crate::route_yaml_import::post_ludusavi_yaml;
use axum::extract::DefaultBodyLimit;
use axum::{Router, routing::get, routing::post};
use const_format::concatcp;
use once_cell::sync::Lazy;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub static DATABASE: Lazy<GameDatabase> = Lazy::new(|| {
    let db_path = concatcp!(DATA_DIR, "/database.sqlite");
    GameDatabase::new(db_path)
});

#[tokio::main]
async fn main() {
    create_fs_structure().await.unwrap();
    Lazy::force(&DATABASE);
    tracing_subscriber::fmt::init();

    let api_router = Router::new()
        .route("/games", post(post_game_metadata))
        .route("/games", get(get_games_metadata))
        .route("/games/{Id}", get(get_game_metadata))
        .route("/games/{Id}/paths", get(get_game_paths))
        .route("/games/{Id}/paths", post(post_game_path))
        .route("/games/{Id}/paths/{OS}", get(get_game_paths_by_os))
        .route(
            "/paths/{Id}/saves",
            get(get_game_saves_reference_by_path_id),
        )
        .route("/paths/{Id}/saves/upload", post(post_game_save_by_path_id))
        .layer(DefaultBodyLimit::max(MAX_BODY_SIZE))
        .route("/games/{Id}/executables", get(get_game_executables))
        .route("/games/{Id}/executables", post(post_game_executable))
        .route(
            "/games/{Id}/executables/{OS}",
            get(get_game_executables_by_os),
        )
        .route("/saves/{Uuid}", get(get_game_save_by_uuid))
        .route("/yaml/ludusavi", post(post_ludusavi_yaml))
        .layer(DefaultBodyLimit::max(MAX_BODY_SIZE));

    let swagger_router =
        SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi());

    let app = Router::new()
        .nest(ROOT_API_PATH, api_router)
        .merge(swagger_router);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
