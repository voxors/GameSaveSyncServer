use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level schema: an object with additionalProperties being a "Game" entry.
pub type GameIndex = HashMap<String, Game>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub files: Option<HashMap<String, FileRule>>,
    pub install_dir: Option<serde_yaml::Value>,
    pub launch: Option<HashMap<String, Vec<LaunchEntry>>>,
    pub registry: Option<HashMap<String, RegistryRule>>,
    pub steam: Option<SteamInfo>,
    pub gog: Option<GogInfo>,
    pub id: Option<IdInfo>,
    pub alias: Option<String>,
    pub cloud: Option<CloudInfo>,
    pub notes: Option<Vec<Note>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRule {
    pub tags: Option<Vec<Tag>>,
    pub when: Option<Vec<FileConstraint>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchEntry {
    pub arguments: Option<String>,
    pub working_dir: Option<String>,
    pub when: Option<Vec<LaunchConstraint>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryRule {
    pub tags: Option<Vec<Tag>>,
    pub when: Option<Vec<RegistryConstraint>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct SteamInfo {
    pub id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct GogInfo {
    pub id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdInfo {
    pub flatpak: Option<String>,
    pub gog_extra: Option<Vec<i64>>,
    pub lutris: Option<String>,
    pub steam_extra: Option<Vec<i64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct CloudInfo {
    pub epic: Option<bool>,
    pub gog: Option<bool>,
    pub origin: Option<bool>,
    pub steam: Option<bool>,
    pub uplay: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct FileConstraint {
    pub os: Option<Os>,
    pub store: Option<Store>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct LaunchConstraint {
    pub bit: Option<Bit>,
    pub os: Option<Os>,
    pub store: Option<Store>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub struct RegistryConstraint {
    pub store: Option<Store>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum Bit {
    #[serde(rename = "32")]
    Bit32,
    #[serde(rename = "64")]
    Bit64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Os {
    Dos,
    Linux,
    Mac,
    Windows,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Store {
    Discord,
    Epic,
    Gog,
    Microsoft,
    Origin,
    Steam,
    Uplay,
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Tag {
    Config,
    Save,
}
