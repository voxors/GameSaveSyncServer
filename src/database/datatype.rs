use crate::database::schema::{
    game_alt_name, game_executable, game_metadata, game_path, game_save,
};
use crate::datatype_endpoint::OS;
use diesel::{Insertable, Queryable, Selectable};

#[derive(Insertable, Selectable, Queryable, PartialEq)]
#[diesel(table_name = game_metadata)]
pub struct NewGameMetadata<'a> {
    pub id: Option<i32>,
    pub steam_appid: Option<&'a str>,
    pub default_name: &'a str,
}

#[derive(Insertable, Selectable, Queryable, PartialEq)]
#[diesel(table_name = game_alt_name)]
pub struct NewGameName<'a> {
    pub name: &'a str,
    pub game_metadata_id: i32,
}

#[derive(Insertable, Selectable, Queryable, PartialEq)]
#[diesel(table_name = game_executable)]
pub struct NewGameExecutable<'a> {
    pub executable: &'a str,
    pub operating_system: &'a OS,
    pub game_metadata_id: i32,
}

#[derive(Insertable, Selectable, Queryable, PartialEq)]
#[diesel(table_name = game_path)]
pub struct NewGamePath<'a> {
    pub path: &'a str,
    pub operating_system: &'a OS,
    pub game_metadata_id: i32,
}

#[derive(Insertable, Selectable, Queryable, PartialEq)]
#[diesel(table_name = game_save)]
pub struct NewGameSave<'a> {
    pub uuid: &'a str,
    pub path_id: i32,
    pub time: time::PrimitiveDateTime,
}
