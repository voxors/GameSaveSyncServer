use crate::configuration::CONFIG_MAP;
use crate::const_var::ROOT_API_PATH;
use crate::datatype_endpoint::ConfigurationForm;
use axum::Json;
use axum::extract::Path;
use axum::http::StatusCode;
use const_format::concatcp;

#[utoipa::path(
    get,
    path = concatcp!(ROOT_API_PATH, "/configuration/{Configuration}"),
    params(
        ("Configuration" = String, Path, description = "Name of the configuration")
    ),
    responses(
        (status = StatusCode::OK, description = "value", body = ConfigurationForm),
        (status = StatusCode::NOT_FOUND, description = "configuration not found")
    )
)]
pub async fn get_configuration(
    Path(configuration): Path<String>,
) -> Result<Json<ConfigurationForm>, StatusCode> {
    let (key, config_info) = match CONFIG_MAP.get_key_value(configuration.as_str()) {
        Some(config_info) => config_info,
        None => {
            tracing::error!("Error updating {}: not found in config map", configuration);
            return Err(StatusCode::NOT_FOUND);
        }
    };

    match config_info.get_value_in_db() {
        Ok(maybe_configuration_form) => match maybe_configuration_form {
            Some(configuration_form) => Ok(Json(configuration_form)),
            None => Err(StatusCode::NOT_FOUND),
        },
        Err(err) => {
            tracing::error!("Error getting value of {}: {}", key, err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    put,
    path = concatcp!(ROOT_API_PATH, "/configuration/{Configuration}"),
    params(
        ("Configuration" = String, Path, description = "Name of the configuration"),
    ),
    request_body = ConfigurationForm,
    responses(
        (status = StatusCode::OK, description = "Configuration saved"),
        (status = StatusCode::NOT_FOUND, description = "configuration not found"),
        (status = StatusCode::BAD_REQUEST, description = "invalid value")
    )
)]
pub async fn put_configuration(
    Path(configuration): Path<String>,
    Json(payload): Json<ConfigurationForm>,
) -> StatusCode {
    let (key, config_info) = match CONFIG_MAP.get_key_value(configuration.as_str()) {
        Some(config_info) => config_info,
        None => {
            tracing::error!("Error updating {}: not found in config map", configuration);
            return StatusCode::NOT_FOUND;
        }
    };

    match config_info.validate(&payload.value) {
        Ok(()) => (),
        Err(err) => {
            tracing::warn!("Invalid value {}: {}", key, err);
            return StatusCode::BAD_REQUEST;
        }
    }

    match config_info.update_value_in_db(&payload.value) {
        Ok(()) => StatusCode::OK,
        Err(err) => {
            tracing::error!("Error updating {}: {}", key, err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
