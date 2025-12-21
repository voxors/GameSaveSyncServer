use crate::{
    DATABASE,
    const_var::{COOKIE_AUTH_NAME, COOKIE_MAX_AGE},
};
use askama::Template;
use axum::{
    Json,
    http::{HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Redirect},
};
use serde::Deserialize;

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate<'a> {
    title: &'a str,
}

#[derive(Deserialize)]
pub struct LoginForm {
    token: String,
}

pub async fn get_login() -> Result<impl IntoResponse, (StatusCode, String)> {
    match (LoginTemplate { title: "Login" }.render()) {
        Ok(html) => Ok(Html(html)),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}

pub async fn post_login(
    Json(form): Json<LoginForm>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if DATABASE
        .get_api_tokens()
        .unwrap_or_default()
        .iter()
        .any(|token| token.to_string() == form.token)
    {
        let mut response = Redirect::to("/").into_response();
        response.headers_mut().append(
            header::SET_COOKIE,
            HeaderValue::from_str(&format!(
                "{auth}={token}; Max-Age={age}; Path=/; HttpOnly",
                auth = COOKIE_AUTH_NAME,
                token = form.token,
                age = COOKIE_MAX_AGE
            ))
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?,
        );
        Ok(response)
    } else {
        Err((
            StatusCode::UNAUTHORIZED,
            "Invalid token – try again.".into(),
        ))
    }
}
