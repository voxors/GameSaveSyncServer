use askama::Template;
use axum::response::{Html, IntoResponse};
use itertools::Itertools;
use reqwest::StatusCode;
use time::OffsetDateTime;

use crate::{
    DATABASE,
    datatype_endpoint::{GameMetadata, SavePath},
};

struct GameSaveCardDashTemplate {
    game_title: String,
    date: String,
    base_path: String,
    paths: Vec<String>,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate<'a> {
    title: &'a str,
    saves: Vec<GameSaveCardDashTemplate>,
}

pub async fn dashboard_handler() -> Result<impl IntoResponse, (StatusCode, String)> {
    let games_metadata_with_paths = match DATABASE.get_games_metadata_and_paths_if_saves_exist() {
        Ok(data) => data,
        Err(err) => return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    };

    let mut saves = Vec::new();
    for (game_metadata, path) in games_metadata_with_paths
        .iter()
        .flat_map(|game_metadata_with_paths| {
            game_metadata_with_paths
                .paths
                .iter()
                .map(|path| (game_metadata_with_paths.game_metadata.clone(), path.clone()))
                .collect::<Vec<(GameMetadata, SavePath)>>()
        })
        .collect::<Vec<(GameMetadata, SavePath)>>()
    {
        let saves_for_path = match DATABASE.get_reference_to_save_by_path_id(path.id.unwrap()) {
            Ok(save_ref) => save_ref,
            Err(err) => return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
        };

        let save_ref = saves_for_path
            .iter()
            .flatten()
            .sorted_by(|item1, item2| item1.time.cmp(&item2.time))
            .last()
            .map(|save_ref| save_ref.to_owned());

        if let Some(save_ref) = save_ref {
            saves.push(GameSaveCardDashTemplate {
                game_title: game_metadata.metadata.default_name,
                date: OffsetDateTime::from_unix_timestamp(save_ref.time)
                    .unwrap()
                    .to_string(),
                base_path: path.path.path.to_owned(),
                paths: save_ref
                    .files_hash
                    .iter()
                    .map(|file_hash| file_hash.relative_path.clone())
                    .collect(),
            });
        }
    }

    match (DashboardTemplate {
        title: "Dashboard",
        saves,
    }
    .render())
    {
        Ok(html) => Ok(Html(html)),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    }
}
