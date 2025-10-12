use crate::database::schema::{
    file_hash, game_alt_name, game_executable, game_metadata, game_path, game_save,
};
use crate::datatype_endpoint::OS;
use diesel::prelude::{Associations, Identifiable};
use diesel::{Insertable, Queryable, Selectable};

#[derive(Insertable, Selectable, Queryable, PartialEq)]
#[diesel(table_name = game_metadata)]
pub struct DbGameMetadata<'a> {
    pub id: Option<i32>,
    pub steam_appid: Option<&'a str>,
    pub default_name: &'a str,
}

#[derive(Insertable, Selectable, Queryable, PartialEq)]
#[diesel(table_name = game_alt_name)]
pub struct DbGameName<'a> {
    pub name: &'a str,
    pub game_metadata_id: i32,
}

#[derive(Insertable, Selectable, Queryable, PartialEq)]
#[diesel(table_name = game_executable)]
pub struct DbGameExecutable<'a> {
    pub id: Option<i32>,
    pub executable: &'a str,
    pub operating_system: &'a OS,
    pub game_metadata_id: i32,
}

#[derive(Insertable, Selectable, Queryable, PartialEq)]
#[diesel(table_name = game_path)]
pub struct DbGamePath<'a> {
    pub id: Option<i32>,
    pub path: &'a str,
    pub operating_system: &'a OS,
    pub game_metadata_id: i32,
}

#[derive(Identifiable, Insertable, Selectable, Queryable, PartialEq, Debug)]
#[diesel(primary_key(uuid))]
#[diesel(table_name = game_save)]
pub struct DbGameSave {
    pub uuid: String,
    pub path_id: i32,
    pub hash: String,
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
