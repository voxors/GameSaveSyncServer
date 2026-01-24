mod auth;
mod configuration;
mod const_var;
mod database;
mod datatype_endpoint;
mod file_system;
mod job_ludusavi;
mod job_scheduler;
mod ludusavi;
mod ludusavi_datatype;
mod openapi;
mod route_configuration;
mod route_executables;
mod route_games;
mod route_health;
mod route_paths;
mod route_registry_paths;
mod route_saves;
mod route_uuid;
mod route_web_configuration;
mod route_web_dashboard;
mod route_web_login;
mod route_yaml_import;

use crate::auth::{bearer_cookie_auth_no_redirect, bearer_cookie_auth_redirect};
use crate::const_var::{DATA_DIR, LOGIN_PATH, MAX_BODY_SIZE, ROOT_API_PATH};
use crate::database::interface::GameDatabase;
use crate::file_system::create_fs_structure;
use crate::job_ludusavi::LudusaviJob;
use crate::job_scheduler::JobScheduler;
use crate::openapi::ApiDoc;
use crate::route_configuration::{get_configuration, put_configuration};
use crate::route_executables::{
    get_game_executables, get_game_executables_by_os, post_game_executable,
};
use crate::route_games::{
    get_game_metadata, get_games_metadata, get_games_metadata_with_paths_if_saves_exists,
    post_game_metadata,
};
use crate::route_health::get_health;
use crate::route_paths::{get_game_paths, get_game_paths_by_os, post_game_path};
use crate::route_registry_paths::{get_game_registries, post_game_registry};
use crate::route_saves::{
    get_game_save_by_uuid, get_game_saves_reference_by_path_id, post_game_save_by_path_id,
};
use crate::route_uuid::get_db_uuid;
use crate::route_web_configuration::configuration_handler;
use crate::route_web_dashboard::dashboard_handler;
use crate::route_web_login::{get_login, post_login};
use crate::route_yaml_import::post_ludusavi_yaml;
use axum::extract::DefaultBodyLimit;
use axum::{Router, routing::get, routing::post};
use const_format::concatcp;
use once_cell::sync::Lazy;
use tower_http::{
    services::ServeDir, trace::TraceLayer, validate_request::ValidateRequestHeaderLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub static DATABASE: Lazy<GameDatabase> = Lazy::new(|| {
    let db_path = concatcp!(DATA_DIR, "/database.sqlite");
    GameDatabase::new(db_path)
});

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".into()),
        )
        .init();

    std::panic::set_hook(Box::new(|panic_info| {
        let location = panic_info
            .location()
            .unwrap_or_else(|| std::panic::Location::caller());

        tracing::error!(
            message = "A thread panicked",
            payload = panic_info.payload_as_str().unwrap_or("not a string"),
            file = location.file(),
            line = location.line(),
            column = location.column(),
        )
    }));

    create_fs_structure().await.unwrap();
    Lazy::force(&DATABASE);

    let mut job_scheduler = JobScheduler::new();
    job_scheduler
        .add_job(LudusaviJob::default(), chrono::Duration::hours(1))
        .await;
    job_scheduler.start_scheduler();

    let api_router = Router::new()
        .route(
            "/configuration/{configuration}",
            get(get_configuration).put(put_configuration),
        )
        .route("/games", get(get_games_metadata).post(post_game_metadata))
        .route(
            "/games/paths/saves",
            get(get_games_metadata_with_paths_if_saves_exists),
        )
        .route("/games/{Id}", get(get_game_metadata))
        .route(
            "/games/{Id}/executables",
            get(get_game_executables).post(post_game_executable),
        )
        .route(
            "/games/{Id}/executables/{OS}",
            get(get_game_executables_by_os),
        )
        .route(
            "/games/{Id}/paths",
            get(get_game_paths).post(post_game_path),
        )
        .route("/games/{Id}/paths/{OS}", get(get_game_paths_by_os))
        .route(
            "/games/{Id}/registry",
            get(get_game_registries).post(post_game_registry),
        )
        .route("/health", get(get_health))
        .route(
            "/paths/{Id}/saves",
            get(get_game_saves_reference_by_path_id),
        )
        .route(
            "/paths/{Id}/saves/upload",
            post(post_game_save_by_path_id).route_layer(DefaultBodyLimit::max(MAX_BODY_SIZE)),
        )
        .route("/saves/{Uuid}", get(get_game_save_by_uuid))
        .route("/uuid", get(get_db_uuid))
        .route(
            "/yaml/ludusavi",
            post(post_ludusavi_yaml).route_layer(DefaultBodyLimit::max(MAX_BODY_SIZE)),
        )
        .layer(ValidateRequestHeaderLayer::custom(
            bearer_cookie_auth_no_redirect,
        ));

    let swagger_router =
        SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi());

    let protected_router = Router::new()
        .route("/", get(dashboard_handler))
        .route("/configuration", get(configuration_handler));
    let login_router = Router::new().route(LOGIN_PATH, get(get_login).post(post_login));
    let web_router = Router::new()
        .merge(login_router)
        .merge(protected_router.layer(ValidateRequestHeaderLayer::custom(
            bearer_cookie_auth_redirect,
        )));

    let app = Router::new()
        .nest(ROOT_API_PATH, api_router)
        .merge(swagger_router)
        .merge(web_router)
        .nest_service(
            "/assets",
            ServeDir::new("frontend/dist/generated")
                .fallback(ServeDir::new("frontend/dist/static")),
        )
        .layer(TraceLayer::new_for_http());

    #[cfg(debug_assertions)]
    let app = app.nest_service("/ts", ServeDir::new("frontend/ts"));

    tracing::info!("Server Starting");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
