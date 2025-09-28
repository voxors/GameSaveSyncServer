use tokio::{fs, io::AsyncReadExt};

use crate::ludusavi_datatype::GameIndex;

pub async fn yaml_import(yaml_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut file = fs::File::open(yaml_path).await?;
    let mut yaml_str = String::new();
    file.read_to_string(&mut yaml_str).await?;

    let games: GameIndex = serde_yaml::from_str(&yaml_str)?;

    for game in games {
        
    }

    Ok(())
}
