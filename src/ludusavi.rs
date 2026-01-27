use itertools::Itertools;
use std::{collections::HashMap, error::Error, path::Path};
use tokio::{fs, io::AsyncReadExt};

use crate::{
    DATABASE,
    database::interface::GameFull,
    datatype_endpoint::{ExecutableCreate, GameMetadataCreate, GameRegistry, OS, SavePathCreate},
    ludusavi_datatype::{Game, GameIndex, Os, Tag},
};

pub async fn yaml_import(yaml_path: impl AsRef<Path>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut file = fs::File::open(yaml_path).await?;
    let mut yaml_str = String::new();
    file.read_to_string(&mut yaml_str).await?;
    let games: GameIndex = serde_yaml::from_str(&yaml_str)?;

    let db_games_metadata = DATABASE
        .get_games_metadata()
        .map_err(|err| err.to_string())?
        .into_iter()
        .map(|game_metadata| (game_metadata.metadata.clone().default_name, game_metadata))
        .collect::<HashMap<_, _>>();

    let mut game_known_name_hashmap: HashMap<String, Vec<String>> = HashMap::new();
    games
        .iter()
        .filter(|(_, game)| game.alias.is_some())
        .for_each(|(name, game)| {
            game_known_name_hashmap
                .entry(game.alias.clone().unwrap())
                .and_modify(|vec| vec.push(name.clone()))
                .or_insert(vec![name.clone()]);
        });

    let mut game_to_add: Vec<GameFull> = Vec::new();
    let mut game_to_update: Vec<(i32, GameFull)> = Vec::new();

    for (name, game) in games {
        if game.alias.is_none() && !db_games_metadata.contains_key(name.as_str()) {
            game_to_add.push(extract_datatype_endpoint_from_game_index(
                &name,
                &game,
                game_known_name_hashmap.get(name.as_str()).cloned(),
            ));
        } else if let Some(metadata) = db_games_metadata.get(name.as_str())
            && metadata.metadata.ludusavi_managed.unwrap_or(false)
        {
            game_to_update.push((
                metadata.id.unwrap(),
                extract_datatype_endpoint_from_game_index(
                    &name,
                    &game,
                    game_known_name_hashmap.get(name.as_str()).cloned(),
                ),
            ));
        }
    }

    DATABASE.add_games_full(game_to_add)?;
    DATABASE.update_games_full(game_to_update)?;

    Ok(())
}

fn extract_datatype_endpoint_from_game_index(
    name: &str,
    game: &Game,
    known_name: Option<Vec<String>>,
) -> GameFull {
    (
        create_game_metadata_from_name_game_known_name(name, game, known_name),
        extract_executable_path_from_game(game),
        extract_save_path_from_game(game),
        extract_registry_from_game(game),
    )
}

fn create_game_metadata_from_name_game_known_name(
    name: &str,
    game: &Game,
    known_name: Option<Vec<String>>,
) -> GameMetadataCreate {
    GameMetadataCreate {
        default_name: name.to_string(),
        known_name,
        steam_appid: game
            .steam
            .and_then(|steam| steam.id.map(|id| id.to_string())),
        install_dir: game
            .install_dir
            .as_ref()
            .map(|value| {
                value
                    .as_mapping()
                    .and_then(|mapping| mapping.keys().next())
                    .and_then(|key| key.as_str())
                    .map(|str| str.to_string())
            })
            .unwrap_or(None),
        gog: game
            .gog
            .and_then(|gog_info| gog_info.id.map(|id| id.to_string())),
        flatpak_id: game.id.as_ref().and_then(|id| id.flatpak.clone()),
        lutris_id: game.id.as_ref().and_then(|id| id.lutris.clone()),
        epic_cloud: game.cloud.and_then(|cloud| cloud.epic),
        gog_cloud: game.cloud.and_then(|cloud| cloud.gog),
        origin_cloud: game.cloud.and_then(|cloud| cloud.origin),
        steam_cloud: game.cloud.and_then(|cloud| cloud.steam),
        uplay_cloud: game.cloud.and_then(|cloud| cloud.uplay),
        gog_extra: game.id.as_ref().and_then(|id| id.gog_extra.clone()),
        steam_extra: game.id.as_ref().and_then(|id| id.steam_extra.clone()),
        ludusavi_managed: Some(true),
    }
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

fn extract_registry_from_game(game: &Game) -> Vec<GameRegistry> {
    game.registry
        .iter()
        .flatten()
        .filter(|(_, registry_rule)| {
            registry_rule
                .tags
                .iter()
                .any(|tags| tags.contains(&Tag::Save))
        })
        .map(|(registry_path, _)| GameRegistry {
            path: registry_path.clone(),
        })
        .collect()
}
