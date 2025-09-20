use crate::database::datatype_database::{NewGameExecutable, NewGameMetadata, NewGameName, NewGamePath};
use crate::database::datatype_database_schema::{game_executable, game_metadata, game_name, game_path};
use crate::datatype_endpoint::{GameMetadata, OS};
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
        game_metadata: &GameMetadata,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let connection = &mut self.pool.get()?;

        connection.immediate_transaction(|connection| {
            let new_game = NewGameMetadata {
                steam_appid: &game_metadata.steam_appid,
            };
            diesel::insert_into(game_metadata::table)
                .values(&new_game)
                .execute(connection)?;

            let inserted_id: Option<i32> = game_metadata::table
                .select(game_metadata::id)
                .order(game_metadata::id.desc())
                .first(connection)?;

            let inserted_id = inserted_id.expect("Inserted id is null");

            for name in &game_metadata.known_name {
                let new_name = NewGameName {
                    name,
                    game_metadata_id: inserted_id,
                };
                diesel::insert_into(game_name::table)
                    .values(&new_name)
                    .execute(connection)?;
            }

            for path in &game_metadata.path_to_save {
                let os_str = match path.operating_system {
                    OS::Windows => "windows",
                    OS::Linux => "linux",
                };
                let new_path = NewGamePath {
                    path: &path.path,
                    operating_system: os_str,
                    game_metadata_id: inserted_id,
                };
                diesel::insert_into(game_path::table)
                    .values(&new_path)
                    .execute(connection)?;
            }

            for executable in &game_metadata.executable {
                let os_str = match executable.operating_system {
                    OS::Windows => "windows",
                    OS::Linux => "linux",
                };
                let new_executable = NewGameExecutable {
                    executable: &executable.executable,
                    operating_system: os_str,
                    game_metadata_id: inserted_id,
                };
                diesel::insert_into(game_executable::table)
                    .values(&new_executable)
                    .execute(connection)?;
            }

            Ok(())
        })
    }
}
