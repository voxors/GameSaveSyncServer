use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OS {
    Windows,
    Linux,
}

#[derive(Serialize, Deserialize)]
pub struct Path {
    pub path: String,
    pub operating_system: OS,
}

#[derive(Serialize, Deserialize)]
pub struct GameMetadata {
    pub known_name: Vec<String>,
    pub steam_appid: String,
    pub path_to_save: Vec<Path>,
}
