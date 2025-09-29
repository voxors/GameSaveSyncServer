use crate::database::datatype::{
    DbGameExecutable, DbGameMetadata, DbGameName, DbGamePath, DbGameSave,
};
use crate::database::schema::{
    game_alt_name, game_executable, game_metadata, game_path, game_save,
};
use crate::datatype_endpoint::{
    Executable, ExecutableCreate, GameMetadata, GameMetadataCreate, OS, SavePath, SavePathCreate,
    SaveReference,
};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use uuid::Uuid;

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub struct GameDatabase {
    pub pool: DbPool,
}

impl GameDatabase {
    pub fn new(db_path: &str) -> Self {
        let manager = ConnectionManager::<SqliteConnection>::new(db_path);
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
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let connection = &mut self.pool.get()?;

        connection.immediate_transaction(|connection| {
            diesel::insert_into(game_metadata::table)
                .values(DbGameMetadata {
                    id: None,
                    steam_appid: game_metadata.steam_appid.as_deref(),
                    default_name: &game_metadata.default_name,
                })
                .execute(connection)?;

            let inserted_id: Option<i32> = game_metadata::table
                .select(game_metadata::id)
                .order(game_metadata::id.desc())
                .first(connection)?;

            let inserted_id = match inserted_id {
                Some(id) => id,
                None => return Err("Failed to get inserted id".into()),
            };

            diesel::insert_into(game_alt_name::table)
                .values(
                    game_metadata
                        .known_name
                        .iter()
                        .map(|name| DbGameName {
                            name,
                            game_metadata_id: inserted_id,
                        })
                        .collect::<Vec<_>>(),
                )
                .execute(connection)?;

            Ok(())
        })
    }

    pub fn get_game_metadata_by_name(
        &self,
        target_name: &str,
    ) -> Result<Vec<GameMetadata>, Box<dyn std::error::Error + Send + Sync>> {
        let connection = &mut self.pool.get()?;
        let db_games: Vec<(Option<i32>, String, Option<String>)> = game_metadata::table
            .filter(game_metadata::default_name.eq(target_name))
            .select((
                game_metadata::id,
                game_metadata::default_name,
                game_metadata::steam_appid,
            ))
            .load(connection)?;

        let mut games: Vec<GameMetadata> = Vec::with_capacity(db_games.len());
        for (id, default_name, steam_appid) in db_games {
            let known_name: Vec<String> = game_alt_name::table
                .filter(game_alt_name::game_metadata_id.eq(id.unwrap()))
                .select(game_alt_name::name)
                .load(connection)?;

            games.push(GameMetadata {
                id,
                metadata: GameMetadataCreate {
                    known_name,
                    steam_appid,
                    default_name,
                },
            });
        }
        Ok(games)
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
        let db_games: Vec<(Option<i32>, String, Option<String>)> = game_metadata::table
            .select((
                game_metadata::id,
                game_metadata::default_name,
                game_metadata::steam_appid,
            ))
            .load(connection)?;

        let mut games = Vec::with_capacity(db_games.len());
        for (id, default_name, steam_appid) in db_games {
            let known_name: Vec<String> = game_alt_name::table
                .filter(game_alt_name::game_metadata_id.eq(id.unwrap()))
                .select(game_alt_name::name)
                .load(connection)?;

            games.push(GameMetadata {
                id,
                metadata: GameMetadataCreate {
                    known_name,
                    steam_appid,
                    default_name,
                },
            });
        }

        Ok(games)
    }

    pub fn add_game_path(
        &self,
        game_id: i32,
        path: &SavePathCreate,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let connection = &mut self.pool.get()?;

        diesel::insert_into(game_path::table)
            .values(DbGamePath {
                id: None,
                path: &path.path,
                operating_system: &path.operating_system,
                game_metadata_id: game_id,
            })
            .execute(connection)?;
        Ok(())
    }
    pub fn get_paths_by_game_id_and_os(
        &self,
        game_id: i32,
        os: OS,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let connection = &mut self.pool.get()?;
        let paths: Vec<String> = game_path::table
            .filter(game_path::game_metadata_id.eq(game_id))
            .filter(game_path::operating_system.eq(os))
            .select(game_path::path)
            .load(connection)?;
        Ok(paths)
    }

    pub fn get_paths_by_game_id(
        &self,
        game_id: i32,
    ) -> Result<Vec<SavePath>, Box<dyn std::error::Error>> {
        let connection = &mut self.pool.get()?;
        let path_rows: Vec<(Option<i32>, String, OS)> = game_path::table
            .filter(game_path::game_metadata_id.eq(game_id))
            .select((game_path::id, game_path::path, game_path::operating_system))
            .load(connection)?;
        let mut paths: Vec<SavePath> = Vec::with_capacity(path_rows.len());
        for (id, path, os) in path_rows {
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

    pub fn add_game_executable(
        &self,
        game_id: i32,
        executable: &ExecutableCreate,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let connection = &mut self.pool.get()?;
        diesel::insert_into(game_executable::table)
            .values(DbGameExecutable {
                id: None,
                executable: &executable.executable,
                operating_system: &executable.operating_system,
                game_metadata_id: game_id,
            })
            .execute(connection)?;
        Ok(())
    }
    pub fn get_executable_by_game_id_and_os(
        &self,
        game_id: i32,
        os: OS,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let connection = &mut self.pool.get()?;
        let paths: Vec<String> = game_executable::table
            .filter(game_executable::game_metadata_id.eq(game_id))
            .filter(game_executable::operating_system.eq(os))
            .select(game_executable::executable)
            .load(connection)?;
        Ok(paths)
    }

    pub fn get_executable_by_game_id(
        &self,
        game_id: i32,
    ) -> Result<Vec<Executable>, Box<dyn std::error::Error>> {
        let connection = &mut self.pool.get()?;
        let executable_rows: Vec<(Option<i32>, String, OS)> = game_executable::table
            .filter(game_executable::game_metadata_id.eq(game_id))
            .select((
                game_executable::id,
                game_executable::executable,
                game_executable::operating_system,
            ))
            .load(connection)?;
        let mut executables: Vec<Executable> = Vec::with_capacity(executable_rows.len());
        for (id, executable, os) in executable_rows {
            executables.push(Executable {
                id,
                executable: ExecutableCreate {
                    executable,
                    operating_system: os,
                },
            });
        }
        Ok(executables)
    }

    pub fn add_reference_to_save(
        &self,
        uuid: Uuid,
        path_id: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let connection = &mut self.pool.get()?;
        let now = time::OffsetDateTime::now_utc();
        diesel::insert_into(game_save::table)
            .values(DbGameSave {
                uuid: &uuid.to_string(),
                path_id,
                time: time::PrimitiveDateTime::new(now.date(), now.time()),
            })
            .execute(connection)?;
        Ok(())
    }

    pub fn get_reference_to_save_by_path_id(
        &self,
        path_id: i32,
    ) -> Result<Option<Vec<SaveReference>>, Box<dyn std::error::Error>> {
        let connection = &mut self.pool.get()?;

        let save_rows: Vec<(String, time::PrimitiveDateTime)> = game_save::table
            .filter(game_save::path_id.eq(path_id))
            .select((game_save::uuid, game_save::time))
            .load(connection)?;

        if save_rows.is_empty() {
            return Ok(None);
        }

        let mut save_references: Vec<SaveReference> = Vec::with_capacity(save_rows.len());
        for (uuid, time) in save_rows {
            save_references.push(SaveReference {
                uuid,
                path_id,
                time: time.assume_utc().unix_timestamp(),
            })
        }

        Ok(Some(save_references))
    }
}
