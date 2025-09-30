use crate::database::schema::{
    game_alt_name, game_executable, game_metadata, game_path, game_save,
};
use crate::datatype_endpoint::OS;
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
#[diesel(belongs_to(DbGameMetadata))]
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

#[derive(Insertable, Selectable, Queryable, PartialEq)]
#[diesel(table_name = game_save)]
pub struct DbGameSave<'a> {
    pub uuid: &'a str,
    pub path_id: i32,
    pub time: time::PrimitiveDateTime,
}
