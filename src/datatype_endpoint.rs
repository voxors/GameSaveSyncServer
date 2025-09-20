use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum OS {
    Windows,
    Linux,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Path {
    pub path: String,
    pub operating_system: OS,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Executable {
    pub executable: String,
    pub operating_system: OS,
}

#[derive(Serialize, Deserialize, ToSchema, IntoParams)]
pub struct GameMetadata {
    pub known_name: Vec<String>,
    pub steam_appid: String,
    pub path_to_save: Vec<Path>,
    pub executable: Vec<Executable>,
}
