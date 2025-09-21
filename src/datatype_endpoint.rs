use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum OS {
    Windows,
    Linux,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SavePathCreate {
    pub path: String,
    pub operating_system: OS,
}
#[derive(Serialize, Deserialize, ToSchema)]
pub struct SavePath {
    pub id: Option<i32>,
    #[serde(flatten)]
    pub path: SavePathCreate,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Executable {
    pub executable: String,
    pub operating_system: OS,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct GameMetadataCreate {
    pub known_name: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(required = false, nullable)]
    pub steam_appid: Option<String>,
    pub default_name: String,
}
#[derive(Serialize, Deserialize, ToSchema, IntoParams)]
pub struct GameMetadata {
    pub id: Option<i32>,
    #[serde(flatten)]
    pub metadata: GameMetadataCreate,
}
