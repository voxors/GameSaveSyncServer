use crate::DATABASE;
use crate::const_var::ROOT_API_PATH;
use axum::{Json, extract::Path, http::StatusCode};
use common::datatype_endpoint::{OS, SavePath, SavePathCreate};
use const_format::concatcp;

#[utoipa::path(
    get,
    path = concatcp!(ROOT_API_PATH, "/games/{Id}/paths"),
    params(
        ("Id" = String, Path, description = "Id of the game")
    ),
    responses(
        (status = 200, description = "game paths returned", body = [Vec<SavePath>]),
    )
)]
pub async fn get_game_paths(Path(id): Path<i32>) -> Result<Json<Vec<SavePath>>, StatusCode> {
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
    path = concatcp!(ROOT_API_PATH,"/games/{Id}/paths/{OS}"),
    params(
        ("Id" = String, Path, description = "Id of the game"),
        ("OS" = OS, Path, description = "Operating system [OS]")
    ),
    responses(
        (status = 200, description = "game paths returned", body = [Vec<String>]),
        (status = 400, description = "invalid operating system"),
        (status = 404, description = "game not found")
    )
)]
pub async fn get_game_paths_by_os(
    Path((id, os)): Path<(i32, OS)>,
) -> Result<Json<Vec<String>>, StatusCode> {
    match DATABASE.get_paths_by_game_id_and_os(id, os) {
        Ok(data) => Ok(Json(data)),
        Err(e) => {
            eprintln!("Error getting game paths: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    post,
    path = concatcp!(ROOT_API_PATH, "/games/{Id}/paths"),
    params(
        ("Id" = String, Path, description = "Id of the game"),
    ),
    request_body = SavePathCreate,
    responses(
        (status = 201, description = "game path created"),
    )
)]
pub async fn post_game_path(
    Path(id): Path<i32>,
    Json(payload): Json<SavePathCreate>,
) -> StatusCode {
    match DATABASE.add_game_path(id, &payload) {
        Ok(()) => StatusCode::CREATED,
        Err(e) => {
            eprintln!("Error adding game path: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
