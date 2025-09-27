mod database;
mod datatype_endpoint;
mod file_system;
mod openapi;
mod route_executable;
mod route_games;
mod route_paths;
mod route_saves;

use crate::database::database_interface::GameDatabase;
use crate::file_system::{DATA_DIR, create_fs_structure};
use crate::openapi::ApiDoc;
use crate::route_executable::{
    get_game_executables, get_game_executables_by_os, post_game_executable,
};
use crate::route_games::{get_game_metadata, get_games_metadata, post_game_metadata};
use crate::route_paths::{get_game_paths, get_game_paths_by_os, post_game_path};
use crate::route_saves::{
    get_game_saves_reference_by_path_id, post_game_save_by_path_id,
    post_game_saves_reference_by_path_id,
};
use axum::extract::DefaultBodyLimit;
use axum::{Router, routing::get, routing::post};
use once_cell::sync::Lazy;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub static DATABASE: Lazy<GameDatabase> = Lazy::new(|| {
    let db_path = format!("{}/database.sqlite", DATA_DIR);
    GameDatabase::new(&db_path)
});

#[tokio::main]
async fn main() {
    create_fs_structure().await.unwrap();
    Lazy::force(&DATABASE);
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/games", post(post_game_metadata))
        .route("/games", get(get_games_metadata))
        .route("/games/{Id}", get(get_game_metadata))
        .route("/games/{Id}/paths", get(get_game_paths))
        .route("/games/{Id}/paths", post(post_game_path))
        .route("/games/{Id}/paths/{OS}", get(get_game_paths_by_os))
        .route(
            "/games/{Id}/paths/{Id}/saves",
            get(get_game_saves_reference_by_path_id),
        )
        .route(
            "/games/{Id}/paths/{Id}/saves",
            post(post_game_saves_reference_by_path_id),
        )
        .route(
            "/games/{Id}/paths/{Id}/saves/upload",
            post(post_game_save_by_path_id),
        )
        .layer(DefaultBodyLimit::max(3 * 1024 * 1024 * 1024))
        .route("/games/{Id}/executables", get(get_game_executables))
        .route("/games/{Id}/executables", post(post_game_executable))
        .route(
            "/games/{Id}/executables/{OS}",
            get(get_game_executables_by_os),
        )
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
