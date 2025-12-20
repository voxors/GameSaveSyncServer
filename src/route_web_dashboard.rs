use askama::Template;
use axum::response::{Html, IntoResponse};
use reqwest::StatusCode;

struct GameSaveCardDashTemplate<'a> {
    game_title: &'a str,
    date: &'a str,
    paths: Vec<&'a str>,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate<'a> {
    title: &'a str,
    saves: Vec<GameSaveCardDashTemplate<'a>>,
}

pub async fn dashboard_handler() -> Result<impl IntoResponse, (StatusCode, String)> {
    match (DashboardTemplate {
        title: "Dashboard",
        saves: vec![GameSaveCardDashTemplate {
            game_title: "test",
            date: "1970-10-10",
            paths: vec!["/home/potato/lol.txt"],
        }],
    }
    .render())
    {
        Ok(html) => Ok(Html(html)),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}
