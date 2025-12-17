use askama::Template;
use axum::{
    extract::Form,
    http::{HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Redirect},
};
use serde::Deserialize;

use crate::DATABASE;

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {}

#[derive(Deserialize)]
pub struct LoginForm {
    token: String,
}

pub async fn get_login() -> impl IntoResponse {
    let html = LoginTemplate {}
        .render()
        .unwrap_or_else(|_| "Template rendering error".into());
    Html(html)
}

pub async fn post_login(
    Form(form): Form<LoginForm>,
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
                "auth_token={}; Max-Age=2628000; Path=/; HttpOnly",
                form.token
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
