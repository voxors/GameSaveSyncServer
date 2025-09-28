use tokio::{fs, io::AsyncReadExt};

use crate::{
    DATABASE,
    ludusavi_datatype::{Game, GameIndex},
};

fn import_game_into_game_metadata_db(
    (name, game): (String, Game),
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if DATABASE.get_game_metadata_by_name(&name)?.is_empty() {
        DATABASE.add_game_metadata(&crate::datatype_endpoint::GameMetadataCreate {
            known_name: Vec::new(),
            steam_appid: match game.steam {
                Some(info) => info.id.map(|id| id.to_string()),
                None => None,
            },
            default_name: name,
        })?;
    }
    Ok(())
}

pub async fn yaml_import(yaml_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut file = fs::File::open(yaml_path).await?;
    let mut yaml_str = String::new();
    file.read_to_string(&mut yaml_str).await?;

    let games: GameIndex = serde_yaml::from_str(&yaml_str)?;

    for game in games {
        import_game_into_game_metadata_db(game)?;
    }

    Ok(())
}
