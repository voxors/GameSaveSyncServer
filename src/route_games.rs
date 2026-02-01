use crate::DATABASE;
use crate::const_var::ROOT_API_PATH;
use crate::datatype_endpoint::{
    GameDefaultName, GameMetadata, GameMetadataCreate, GameMetadataWithPaths,
};
use axum::{
    Json,
    extract::{Path, Query},
    http::StatusCode,
};
use const_format::concatcp;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    name: String,
}

#[utoipa::path(
    post,
    path = concatcp!(ROOT_API_PATH, "/games"),
    params(),
    request_body = GameMetadataCreate,
    responses(
        (status = StatusCode::CREATED, description = "game metadata created")
    )
)]
pub async fn post_game_metadata(Json(mut payload): Json<GameMetadataCreate>) -> StatusCode {
    if payload.ludusavi_managed.is_none() {
        payload.ludusavi_managed = Some(false);
    }
    match DATABASE.add_games_metadata(vec![&payload]) {
        Ok(()) => StatusCode::CREATED,
        Err(e) => {
            tracing::error!("Error adding game metadata: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[utoipa::path(
    get,
    path = concatcp!(ROOT_API_PATH, "/games"),
    params(),
    responses(
        (status = StatusCode::OK, description = "get all games metadata", body = [GameMetadata])
    )
)]
pub async fn get_games_metadata() -> Result<Json<Vec<GameMetadata>>, StatusCode> {
    match DATABASE.get_games_metadata() {
        Ok(data) => Ok(Json(data)),
        Err(e) => {
            tracing::error!("Error retrieving game metadata: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    get,
    path = concatcp!(ROOT_API_PATH, "/games/search"),
    params(
        ("name" = String, Query, description = "Search string for game name")
    ),
    responses(
        (status = StatusCode::OK, description = "Search games by name", body = [GameDefaultName])
    )
)]
pub async fn get_games_search(
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<GameDefaultName>>, StatusCode> {
    match DATABASE.search_games_by_name(&params.name) {
        Ok(names) => Ok(Json(names)),
        Err(e) => {
            tracing::error!("Error searching games: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    get,
    path = concatcp!(ROOT_API_PATH, "/games/default_name"),
    params(),
    responses(
        (status = StatusCode::OK, description = "get all games defaults name", body = [GameDefaultName])
    )
)]
pub async fn get_games_default_name() -> Result<Json<Vec<GameDefaultName>>, StatusCode> {
    match DATABASE.get_games_default_name() {
        Ok(data) => Ok(Json(data)),
        Err(e) => {
            tracing::error!("Error retrieving game metadata: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    get,
    path = concatcp!(ROOT_API_PATH, "/games/paths/saves"),
    params(),
    responses(
        (status = StatusCode::OK, description = "get all games metadata that has paths with saves", body = [GameMetadataWithPaths])
    )
)]
pub async fn get_games_metadata_with_paths_if_saves_exists()
-> Result<Json<Vec<GameMetadataWithPaths>>, StatusCode> {
    match DATABASE.get_games_metadata_and_paths_if_saves_exist() {
        Ok(data) => Ok(Json(data)),
        Err(e) => {
            tracing::error!("Error retrieving game metadata: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    get,
    path = concatcp!(ROOT_API_PATH, "/games/{Id}"),
    params(
        ("Id" = String, Path, description = "Id of the game")
    ),
    responses(
        (status = StatusCode::OK, description = "game metadata returned, body = [GameMetadata]"),
        (status = StatusCode::NOT_FOUND, description = "game not found")
    )
)]
pub async fn get_game_metadata(Path(id): Path<i32>) -> Result<Json<GameMetadata>, StatusCode> {
    match DATABASE.get_game_metadata_by_id(&id) {
        Ok(Some(data)) => Ok(Json(data)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Error getting game metadata: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
