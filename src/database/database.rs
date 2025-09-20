use crate::database::datatype_database::{NewGameMetadata, NewGameName, NewGamePath};
use crate::database::datatype_database_schema::{game_metadata, game_name, game_path};
use crate::datatype_endpoint::GameMetadata;
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

        // Ensure directory exists
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

    pub fn add_game_metadata(&self, game_metadata: &GameMetadata) {
        // Insert into game_metadata
        let new_game = NewGameMetadata {
            steam_appid: &game_metadata.steam_appid,
        };
        diesel::insert_into(game_metadata::table)
            .values(&new_game)
            .execute(&mut self.pool.get().unwrap())
            .expect("Error inserting game metadata");

        // Get the inserted id
        let inserted_id: Option<i32> = game_metadata::table
            .select(game_metadata::id)
            .order(game_metadata::id.desc())
            .first(&mut self.pool.get().unwrap())
            .expect("Failed to get inserted id");

        let inserted_id = inserted_id.expect("Inserted id is null");

        // Insert known names
        for name in &game_metadata.known_name {
            let new_name = NewGameName {
                name,
                game_metadata_id: inserted_id,
            };
            diesel::insert_into(game_name::table)
                .values(&new_name)
                .execute(&mut self.pool.get().unwrap())
                .expect("Error inserting game name");
        }

        // Insert paths
        for path in &game_metadata.path_to_save {
            let os_str = match path.operating_system {
                crate::datatype_endpoint::OS::Windows => "windows",
                crate::datatype_endpoint::OS::Linux => "linux",
            };
            let new_path = NewGamePath {
                path: &path.path,
                operating_system: os_str,
                game_metadata_id: inserted_id,
            };
            diesel::insert_into(game_path::table)
                .values(&new_path)
                .execute(&mut self.pool.get().unwrap())
                .expect("Error inserting game path");
        }
    }
}
