use std::error::Error;

use axum::{body::Body, http::StatusCode};

use crate::DATABASE;

#[allow(clippy::result_large_err)]
pub fn bearer_token_auth(
    request_body: &mut axum::http::Request<Body>,
) -> Result<(), axum::http::Response<Body>> {
    let api_tokens = DATABASE.get_api_tokens();

    if authorized(request_body, api_tokens) {
        Ok(())
    } else {
        Err(
            match axum::http::Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::empty())
            {
                Ok(response) => response,
                Err(error) => {
                    eprintln!("Response builder error: {error}");
                    axum::http::Response::new(Body::empty())
                }
            },
        )
    }
}

fn authorized(
    request_body: &mut axum::http::Request<Body>,
    api_tokens: Result<Vec<uuid::Uuid>, Box<dyn Error + Send + Sync>>,
) -> bool {
    request_body
        .headers()
        .iter()
        .filter_map(|(name, value)| {
            if name.as_str().eq_ignore_ascii_case("Authorization") {
                value
                    .to_str()
                    .ok()
                    .and_then(|value| value.strip_prefix("Bearer").map(|value| value.trim()))
            } else {
                None
            }
        })
        .any(|auth_value| {
            api_tokens
                .iter()
                .flatten()
                .any(|api_token| api_token.to_string().trim() == auth_value)
        })
}
