use crate::{
    DATABASE,
    const_var::{COOKIE_AUTH_NAME, LOGIN_PATH},
};
use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use std::error::Error;

#[allow(clippy::result_large_err)]
pub fn bearer_token_auth(
    request_body: &mut axum::http::Request<Body>,
) -> Result<(), axum::http::Response<Body>> {
    let api_tokens = DATABASE.get_api_tokens();
    if authorized_bearer_token(request_body, api_tokens) {
        Ok(())
    } else {
        Err(
            match axum::http::Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Body::empty())
            {
                Ok(response) => response,
                Err(error) => {
                    tracing::error!("Response builder error: {error}");
                    axum::http::Response::new(Body::empty())
                }
            },
        )
    }
}

fn authorized_bearer_token(
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

#[allow(clippy::result_large_err)]
pub fn cookie_token_auth(
    request_body: &mut axum::http::Request<Body>,
) -> Result<(), axum::http::Response<Body>> {
    let api_tokens = DATABASE.get_api_tokens();
    if authorized_cookie(request_body, api_tokens) {
        Ok(())
    } else {
        Err(Redirect::to(LOGIN_PATH).into_response())
    }
}

fn authorized_cookie(
    request_body: &mut axum::http::Request<Body>,
    api_tokens: Result<Vec<uuid::Uuid>, Box<dyn Error + Send + Sync>>,
) -> bool {
    request_body
        .headers()
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|cookie_header| {
            let token_opt = cookie_header
                .split(';')
                .map(|string| string.trim())
                .find_map(|pair| {
                    let (name, val) = pair.split_once('=')?;
                    if name.eq_ignore_ascii_case(COOKIE_AUTH_NAME) {
                        Some(val.trim())
                    } else {
                        None
                    }
                });
            if let Some(token) = token_opt {
                api_tokens
                    .iter()
                    .flatten()
                    .any(|api_token| api_token.to_string().trim() == token)
            } else {
                false
            }
        })
}
