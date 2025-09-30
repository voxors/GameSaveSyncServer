use tokio::{fs, io::AsyncReadExt};

use crate::{
    DATABASE,
    datatype_endpoint::{ExecutableCreate, OS, SavePathCreate},
    ludusavi_datatype::{Game, GameIndex, Os, Tag},
};

fn import_game_into_game_metadata_db(
    (name, game): &(String, Game),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if DATABASE.get_game_metadata_by_name(name)?.is_empty() {
        DATABASE.add_game_metadata(&crate::datatype_endpoint::GameMetadataCreate {
            known_name: Vec::new(),
            steam_appid: match game.steam {
                Some(info) => info.id.map(|id| id.to_string()),
                None => None,
            },
            default_name: name.to_string(),
        })?;
    }
    Ok(())
}

fn import_path_into_game_path_db(
    (name, game): &(String, Game),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let id = DATABASE
        .get_game_metadata_by_name(name)?
        .first()
        .and_then(|game_metadata| game_metadata.id)
        .ok_or("Id not found: import path")?;

    let files = match &game.files {
        Some(files) => files,
        None => return Ok(()),
    };

    for (path, file) in files {
        if !file.tags.iter().any(|tags| tags.contains(&Tag::Save)) {
            continue;
        }
        let os_iter = file
            .when
            .iter()
            .flat_map(|file_constraints| file_constraints.iter())
            .filter_map(|file_constraint| match file_constraint.os {
                Some(Os::Linux) => Some(OS::Linux),
                Some(Os::Windows) => Some(OS::Windows),
                None => Some(OS::Undefined),
                _ => None,
            });
        for os in os_iter {
            let exists = DATABASE
                .get_paths_by_game_id_and_os(id, os)?
                .iter()
                .any(|db_path| db_path == path);
            if !exists {
                DATABASE.add_game_path(
                    id,
                    &SavePathCreate {
                        path: path.clone(),
                        operating_system: os,
                    },
                )?;
            }
        }
    }
    Ok(())
}

pub fn import_executable_into_game_executable_db(
    (name, game): &(String, Game),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let id = DATABASE
        .get_game_metadata_by_name(name)?
        .first()
        .and_then(|game_metadata| game_metadata.id)
        .ok_or("Id not found: import path")?;

    let launch = match &game.launch {
        Some(launch) => launch,
        None => return Ok(()),
    };

    for (executable_path, launch_entries) in launch {
        let os_iter = launch_entries
            .iter()
            .flat_map(|launch_entry| launch_entry.when.iter())
            .flat_map(|launch_constraints| launch_constraints.iter())
            .filter_map(|launch_constraint| match launch_constraint.os {
                Some(Os::Linux) => Some(OS::Linux),
                Some(Os::Windows) => Some(OS::Windows),
                None => Some(OS::Undefined),
                _ => None,
            });

        for os in os_iter {
            let exists = DATABASE
                .get_executable_by_game_id_and_os(id, os)?
                .iter()
                .any(|db_exec_path| db_exec_path == executable_path);
            if !exists {
                DATABASE.add_game_executable(
                    id,
                    &ExecutableCreate {
                        executable: executable_path.clone(),
                        operating_system: os,
                    },
                )?;
            }
        }
    }

    Ok(())
}

pub async fn yaml_import(yaml_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut file = fs::File::open(yaml_path).await?;
    let mut yaml_str = String::new();
    file.read_to_string(&mut yaml_str).await?;

    let games: GameIndex = serde_yaml::from_str(&yaml_str)?;

    for game in games {
        import_game_into_game_metadata_db(&game)?;
        import_path_into_game_path_db(&game)?;
        import_executable_into_game_executable_db(&game)?;
    }

    Ok(())
}
