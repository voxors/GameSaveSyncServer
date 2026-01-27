use crate::DATABASE;
use crate::const_var::ROOT_API_PATH;
use crate::datatype_endpoint::GameRegistry;
use axum::{Json, extract::Path, http::StatusCode};
use const_format::concatcp;

#[utoipa::path(
    get,
    path = concatcp!(ROOT_API_PATH, "/games/{Id}/registry"),
    params(
        ("Id" = String, Path, description = "Id of the game")
    ),
    responses(
        (status = StatusCode::OK, description = "game registry paths returned", body = [GameRegistry]),
    )
)]
pub async fn get_game_registries(
    Path(id): Path<i32>,
) -> Result<Json<Vec<GameRegistry>>, StatusCode> {
    match DATABASE.get_game_registry_by_game_id(id) {
        Ok(data) => Ok(Json(data)),
        Err(e) => {
            tracing::error!("Error getting game registry paths: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    post,
    path = concatcp!(ROOT_API_PATH, "/games/{Id}/registry"),
    params(
        ("Id" = String, Path, description = "Id of the game"),
    ),
    request_body = GameRegistry,
    responses(
        (status = StatusCode::CREATED, description = "game registry path created"),
    )
)]
pub async fn post_game_registry(
    Path(id): Path<i32>,
    Json(payload): Json<GameRegistry>,
) -> StatusCode {
    match DATABASE.add_game_registry_path(id, &payload) {
        Ok(()) => StatusCode::CREATED,
        Err(e) => {
            tracing::error!("Error adding game registry path: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
