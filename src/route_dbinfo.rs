use crate::DATABASE;
use crate::const_var::ROOT_API_PATH;
use axum::http::StatusCode;
use const_format::concatcp;

#[utoipa::path(
    get,
    path = concatcp!(ROOT_API_PATH, "/uuid"),
    responses(
        (status = 200, description = "db uuid", body = String),
    )
)]
pub async fn get_db_uuid() -> Result<String, StatusCode> {
    match DATABASE.get_database_uuid() {
        Ok(data) => match data {
            Some(data) => Ok(data.to_string()),
            None => Err(StatusCode::NOT_FOUND),
        },
        Err(e) => {
            tracing::error!("Error getting game paths: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
