use std::path::Path;

use itertools::Itertools;
use tokio::{fs, io::AsyncReadExt};

use crate::{
    DATABASE,
    datatype_endpoint::{ExecutableCreate, GameMetadataCreate, OS, SavePathCreate},
    ludusavi_datatype::{Game, GameIndex, Os, Tag},
};

type GameFull = (
    GameMetadataCreate,
    Vec<ExecutableCreate>,
    Vec<SavePathCreate>,
);

pub async fn yaml_import(
    yaml_path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut file = fs::File::open(yaml_path).await?;
    let mut yaml_str = String::new();
    file.read_to_string(&mut yaml_str).await?;
    let games: GameIndex = serde_yaml::from_str(&yaml_str)?;

    let db_games_metadata = DATABASE.get_games_metadata();

    DATABASE.add_games_full(
        games
            .iter()
            .filter(|(name, game)| {
                db_games_metadata.iter().flatten().all(|db_game_metadata| {
                    *name != &db_game_metadata.metadata.default_name && game.alias.is_none()
                })
            })
            .map(|(name, game)| extract_datatype_endpoint_from_game_index(name, game))
            .collect::<Vec<GameFull>>(),
    )?;

    Ok(())
}

fn extract_datatype_endpoint_from_game_index(name: &str, game: &Game) -> GameFull {
    (
        GameMetadataCreate {
            default_name: name.to_string(),
            known_name: Vec::new(),
            steam_appid: game
                .steam
                .and_then(|steam| steam.id.map(|id| id.to_string())),
        },
        extract_executable_path_from_game(game),
        extract_save_path_from_game(game),
    )
}

fn extract_executable_path_from_game(game: &Game) -> Vec<ExecutableCreate> {
    game.launch
        .iter()
        .flatten()
        .flat_map(|(executable_path, launch_entries)| {
            launch_entries
                .iter()
                .flat_map(|launch_entry| launch_entry.when.iter())
                .flat_map(|launch_constraints| launch_constraints.iter())
                .filter_map(|launch_constraint| match launch_constraint.os {
                    Some(Os::Linux) => Some(OS::Linux),
                    Some(Os::Windows) => Some(OS::Windows),
                    None => Some(OS::Undefined),
                    _ => None,
                })
                .map(|os| (executable_path.clone(), os))
        })
        .unique()
        .map(|(executable_path, os)| ExecutableCreate {
            executable: executable_path.clone(),
            operating_system: os,
        })
        .collect()
}

fn extract_save_path_from_game(game: &Game) -> Vec<SavePathCreate> {
    game.files
        .iter()
        .flatten()
        .filter(|(_, file_rule)| file_rule.tags.iter().any(|tags| tags.contains(&Tag::Save)))
        .flat_map(|(file_path, file_rule)| {
            file_rule
                .when
                .iter()
                .flat_map(|file_constraints| file_constraints.iter())
                .filter_map(|file_constraint| match file_constraint.os {
                    Some(Os::Linux) => Some(OS::Linux),
                    Some(Os::Windows) => Some(OS::Windows),
                    None => Some(OS::Undefined),
                    _ => None,
                })
                .map(|os| (file_path.clone(), os))
        })
        .unique()
        .map(|(file_path, os)| SavePathCreate {
            path: file_path.clone(),
            operating_system: os,
        })
        .collect()
}
