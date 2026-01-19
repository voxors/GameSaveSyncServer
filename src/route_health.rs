use crate::const_var::ROOT_API_PATH;
use axum::http::StatusCode;
use const_format::concatcp;

#[utoipa::path(
    get,
    path = concatcp!(ROOT_API_PATH, "/health"),
    responses(
        (status = StatusCode::OK, description = "Healthy"),
    )
)]
pub async fn get_health() -> StatusCode {
    StatusCode::OK
}
