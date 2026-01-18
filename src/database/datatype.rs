use crate::database::schema::{
    api_tokens, configurations, db_info, file_hash, game_alt_name, game_executable, game_metadata,
    game_path, game_save,
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
}

#[derive(Identifiable, Insertable, Selectable, Queryable, PartialEq, Debug)]
#[diesel(primary_key(name, game_metadata_id))]
#[diesel(table_name = game_alt_name)]
pub struct DbGameName {
    pub name: String,
    pub game_metadata_id: i32,
}

#[derive(Insertable, Selectable, Queryable, PartialEq)]
#[diesel(table_name = game_executable)]
pub struct DbGameExecutable {
    pub id: Option<i32>,
    pub executable: String,
    pub operating_system: OS,
    pub game_metadata_id: i32,
}

#[derive(Insertable, Selectable, Queryable, PartialEq)]
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
