use askama::Template;
use axum::response::{Html, IntoResponse};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    name: &'a str,
}

pub async fn index_handler() -> impl IntoResponse {
    let html = IndexTemplate { name: "Axum User" }
        .render()
        .unwrap_or_else(|_| "Template rendering error".into());
    Html(html)
}
