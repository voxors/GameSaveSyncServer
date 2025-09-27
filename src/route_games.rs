use crate::DATABASE;
use crate::datatype_endpoint::{GameMetadata, GameMetadataCreate};
use axum::{Json, extract::Path, http::StatusCode};

#[utoipa::path(
    post,
    path = "/games",
    params(),
    request_body = GameMetadataCreate,
    responses(
        (status = 201, description = "game metadata created", body = [String])
    )
)]
pub async fn post_game_metadata(Json(payload): Json<GameMetadataCreate>) -> StatusCode {
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
pub async fn get_games_metadata() -> Result<Json<Vec<GameMetadata>>, StatusCode> {
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
pub async fn get_game_metadata(Path(id): Path<i32>) -> Result<Json<GameMetadata>, StatusCode> {
    match DATABASE.get_game_metadata_by_id(&id) {
        Ok(Some(data)) => Ok(Json(data)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error getting game metadata: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
