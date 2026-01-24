use crate::database::schema::{
    api_tokens, configurations, db_info, file_hash, game_alt_name, game_executable,
    game_gog_extra_id, game_metadata, game_path, game_registry, game_save, game_steam_extra_id,
};
use crate::datatype_endpoint::OS;
use diesel::prelude::{Associations, Identifiable};
use diesel::{Insertable, Queryable, Selectable};

#[derive(Identifiable, Insertable, Clone, Selectable, Queryable, PartialEq, Debug)]
#[diesel(table_name = game_metadata)]
pub struct DbGameMetadata {
    pub id: Option<i32>,
    pub steam_appid: Option<String>,
    pub default_name: String,
    pub install_dir: Option<String>,
    pub gog: Option<String>,
    pub flatpak_id: Option<String>,
    pub lutris_id: Option<String>,
    pub epic_cloud: Option<bool>,
    pub gog_cloud: Option<bool>,
    pub origin_cloud: Option<bool>,
    pub steam_cloud: Option<bool>,
    pub uplay_cloud: Option<bool>,
}

#[derive(Identifiable, Insertable, Selectable, Queryable, PartialEq, Debug)]
#[diesel(primary_key(name, game_metadata_id))]
#[diesel(table_name = game_alt_name)]
pub struct DbGameName {
    pub name: String,
    pub game_metadata_id: i32,
}

#[derive(Insertable, Selectable, Queryable, PartialEq)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DbGameMetadata, foreign_key = game_metadata_id))]
#[diesel(table_name = game_executable)]
pub struct DbGameExecutable {
    pub id: Option<i32>,
    pub executable: String,
    pub operating_system: OS,
    pub game_metadata_id: i32,
}

#[derive(Insertable, Selectable, Queryable, PartialEq)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DbGameMetadata, foreign_key = game_metadata_id))]
#[diesel(table_name = game_path)]
pub struct DbGamePath {
    pub id: Option<i32>,
    pub path: String,
    pub operating_system: OS,
    pub game_metadata_id: i32,
}

#[derive(Identifiable, Insertable, Selectable, Queryable, PartialEq, Debug)]
#[diesel(primary_key(uuid))]
#[diesel(table_name = game_save)]
pub struct DbGameSave {
    pub uuid: String,
    pub path_id: i32,
    pub time: time::PrimitiveDateTime,
}

#[derive(Identifiable, Insertable, Selectable, Queryable, PartialEq, Associations, Debug)]
#[diesel(primary_key(relative_path, game_save_uuid))]
#[diesel(belongs_to(DbGameSave, foreign_key = game_save_uuid))]
#[diesel(table_name = file_hash)]
pub struct DbFileHash {
    pub relative_path: String,
    pub hash: String,
    pub game_save_uuid: String,
}

#[derive(Insertable, Selectable, Queryable, PartialEq)]
#[diesel(primary_key(id))]
#[diesel(table_name = db_info)]
pub struct DbDbInfo {
    pub id: Option<i32>,
    pub db_uuid: String,
}

#[derive(Insertable, Selectable, Queryable, PartialEq)]
#[diesel(primary_key(id))]
#[diesel(table_name = api_tokens)]
pub struct DbApiTokens {
    pub id: Option<i32>,
    pub api_token: String,
}

#[derive(Insertable, Selectable, Queryable, PartialEq)]
#[diesel(primary_key(id))]
#[diesel(table_name = configurations)]
pub struct DbConfiguration {
    pub id: String,
    pub value: String,
}

#[derive(Insertable, Selectable, Queryable, PartialEq)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DbGameMetadata, foreign_key = game_metadata_id))]
#[diesel(table_name = game_steam_extra_id)]
pub struct DbGameSteamExtraId {
    pub id: i64,
    pub game_metadata_id: i32,
}

#[derive(Insertable, Selectable, Queryable, PartialEq)]
#[diesel(primary_key(id))]
#[diesel(belongs_to(DbGameMetadata, foreign_key = game_metadata_id))]
#[diesel(table_name = game_gog_extra_id)]
pub struct DbGameGogExtraId {
    pub id: i64,
    pub game_metadata_id: i32,
}

#[derive(Insertable, Selectable, Queryable, PartialEq)]
#[diesel(primary_key(id, game_metadata_id))]
#[diesel(belongs_to(DbGameMetadata, foreign_key = game_metadata_id))]
#[diesel(table_name = game_registry)]
pub struct DbGameRegistry {
    pub path: String,
    pub game_metadata_id: i32,
}
