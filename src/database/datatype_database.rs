use crate::database::datatype_database_schema::{game_metadata, game_name, game_path, game_executable};
use diesel::Insertable;

#[derive(Insertable)]
#[diesel(table_name = game_metadata)]
pub struct NewGameMetadata<'a> {
    pub steam_appid: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = game_name)]
pub struct NewGameName<'a> {
    pub name: &'a str,
    pub game_metadata_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = game_executable)]
pub struct NewGameExecutable<'a> {
    pub executable: &'a str,
    pub operating_system: &'a str,
    pub game_metadata_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = game_path)]
pub struct NewGamePath<'a> {
    pub path: &'a str,
    pub operating_system: &'a str,
    pub game_metadata_id: i32,
}
