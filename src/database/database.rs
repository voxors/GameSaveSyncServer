use crate::database::datatype_database::{
    NewGameExecutable, NewGameMetadata, NewGameName, NewGamePath,
};
use crate::database::datatype_database_schema::{
    game_alt_name, game_executable, game_metadata, game_path,
};
use crate::datatype_endpoint::{
    Executable, GameMetadata, GameMetadataCreate, OS, SavePath, SavePathCreate,
};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use std::fs;

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub struct GameDatabase {
    pub pool: DbPool,
}

impl GameDatabase {
    pub fn new() -> Self {
        let data_dir = "./data/";
        let db_path = format!("{}/database.sqlite", data_dir);

        fs::create_dir_all(data_dir).expect("Failed to create data directory");
        let manager = ConnectionManager::<SqliteConnection>::new(&db_path);
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create pool");

        {
            let mut conn = pool.get().expect("Failed to get DB connection");
            conn.run_pending_migrations(MIGRATIONS)
                .expect("Failed to run database migrations");
        }

        Self { pool }
    }

    pub fn add_game_metadata(
        &self,
        game_metadata: &GameMetadataCreate,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let connection = &mut self.pool.get()?;

        connection.immediate_transaction(|connection| {
            let new_game = NewGameMetadata {
                steam_appid: game_metadata.steam_appid.as_deref(),
                default_name: &game_metadata.default_name,
            };
            diesel::insert_into(game_metadata::table)
                .values(&new_game)
                .execute(connection)?;

            let inserted_id: Option<i32> = game_metadata::table
                .select(game_metadata::id)
                .order(game_metadata::id.desc())
                .first(connection)?;

            let inserted_id = inserted_id.expect("Inserted id is null");

            let new_names: Vec<NewGameName> = game_metadata
                .known_name
                .iter()
                .map(|name| NewGameName {
                    name,
                    game_metadata_id: inserted_id,
                })
                .collect();

            diesel::insert_into(game_alt_name::table)
                .values(&new_names)
                .execute(connection)?;

            Ok(())
        })
    }

    pub fn get_game_metadata_by_id(
        &self,
        target_id: &i32,
    ) -> Result<Option<GameMetadata>, Box<dyn std::error::Error>> {
        let connection = &mut self.pool.get()?;

        connection.immediate_transaction(|connection| {
            let maybe_meta: Option<(Option<i32>, String, Option<String>)> = game_metadata::table
                .filter(game_metadata::id.eq(target_id))
                .select((
                    game_metadata::id,
                    game_metadata::default_name,
                    game_metadata::steam_appid,
                ))
                .first(connection)
                .optional()?;

            let (id, default_name, steam_appid) = match maybe_meta {
                Some(meta) => meta,
                None => return Ok(None),
            };

            let id = match id {
                Some(id) => id,
                None => return Ok(None),
            };

            let name_rows: Vec<String> = game_alt_name::table
                .filter(game_alt_name::game_metadata_id.eq(id))
                .select(game_alt_name::name)
                .load(connection)?;

            Ok(Some(GameMetadata {
                id: Some(id),
                metadata: GameMetadataCreate {
                    known_name: name_rows,
                    steam_appid,
                    default_name,
                },
            }))
        })
    }
    pub fn get_games_metadata(&self) -> Result<Vec<GameMetadata>, Box<dyn std::error::Error>> {
        let connection = &mut self.pool.get()?;
        let ids: Vec<i32> = game_metadata::table
            .select(game_metadata::id)
            .load::<Option<i32>>(connection)?
            .into_iter()
            .flatten()
            .collect();

        let mut games = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(game) = self.get_game_metadata_by_id(&id)? {
                games.push(game);
            }
        }

        Ok(games)
    }
    
    pub fn add_game_path(
        &self,
        game_id: i32,
        path: &SavePathCreate,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let connection = &mut self.pool.get()?;
        let os_str = match path.operating_system {
            OS::Windows => "windows",
            OS::Linux => "linux",
        };
        let new_path = NewGamePath {
            path: &path.path,
            operating_system: os_str,
            game_metadata_id: game_id,
        };
        diesel::insert_into(game_path::table)
            .values(&new_path)
            .execute(connection)?;

        Ok(())
    }
    pub fn get_paths_by_game_id_and_os(
        &self,
        game_id: i32,
        os: OS,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let connection = &mut self.pool.get()?;
        let os_str = match os {
            OS::Windows => "windows",
            OS::Linux => "linux",
        };
        let paths: Vec<String> = game_path::table
            .filter(game_path::game_metadata_id.eq(game_id))
            .filter(game_path::operating_system.eq(os_str))
            .select(game_path::path)
            .load(connection)?;
        Ok(paths)
    }

    pub fn get_paths_by_game_id(
        &self,
        game_id: i32,
    ) -> Result<Vec<SavePath>, Box<dyn std::error::Error>> {
        let connection = &mut self.pool.get()?;
        let path_rows: Vec<(Option<i32>, String, String)> = game_path::table
            .filter(game_path::game_metadata_id.eq(game_id))
            .select((game_path::id, game_path::path, game_path::operating_system))
            .load(connection)?;
        let mut paths: Vec<SavePath> = Vec::with_capacity(path_rows.len());
        for (id, path, os_str) in path_rows {
            let os = match os_str.as_str() {
                "windows" => OS::Windows,
                "linux" => OS::Linux,
                _ => continue,
            };
            paths.push(SavePath {
                id,
                path: SavePathCreate {
                    path,
                    operating_system: os,
                },
            });
        }

        Ok(paths)
    }
}
