use crate::database::datatype_database::{
    NewGameExecutable, NewGameMetadata, NewGameName, NewGamePath,
};
use crate::database::datatype_database_schema::{
    game_alt_name, game_executable, game_metadata, game_path,
};
use crate::datatype_endpoint::{Executable, GameMetadata, GameMetadataCreate, OS, Path};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use std::collections::HashMap;
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

    pub fn update_game_metadata_by_id(
        &self,
        target_id: &i32,
        game_metadata: &GameMetadata,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let connection = &mut self.pool.get()?;

        connection.immediate_transaction(|connection| {
            let maybe_id: Option<i32> = game_metadata::table
                .filter(game_metadata::id.eq(target_id))
                .select(game_metadata::id)
                .first::<Option<i32>>(connection)?;

            let Some(game_id) = maybe_id else {
                return Ok(false);
            };

            diesel::update(game_metadata::table.filter(game_metadata::id.eq(game_id)))
                .set((
                    game_metadata::id.eq(&game_id),
                    game_metadata::steam_appid.eq(game_metadata.metadata.steam_appid.as_deref()),
                ))
                .execute(connection)?;

            // Replace related collections
            diesel::delete(
                game_alt_name::table.filter(game_alt_name::game_metadata_id.eq(game_id)),
            )
            .execute(connection)?;
            diesel::delete(game_path::table.filter(game_path::game_metadata_id.eq(game_id)))
                .execute(connection)?;
            diesel::delete(
                game_executable::table.filter(game_executable::game_metadata_id.eq(game_id)),
            )
            .execute(connection)?;

            let new_names: Vec<NewGameName> = game_metadata
                .metadata
                .known_name
                .iter()
                .map(|name| NewGameName {
                    name,
                    game_metadata_id: game_id,
                })
                .collect();
            diesel::insert_into(game_alt_name::table)
                .values(&new_names)
                .execute(connection)?;

            let new_paths: Vec<NewGamePath> = game_metadata
                .metadata
                .path_to_save
                .iter()
                .map(|path| NewGamePath {
                    path: &*path.path,
                    operating_system: match path.operating_system {
                        OS::Windows => "windows",
                        OS::Linux => "linux",
                    },
                    game_metadata_id: game_id,
                })
                .collect();
            diesel::insert_into(game_path::table)
                .values(&new_paths)
                .execute(connection)?;

            let new_executables: Vec<NewGameExecutable> = game_metadata
                .metadata
                .executable
                .iter()
                .map(|game_executable| NewGameExecutable {
                    executable: &*game_executable.executable,
                    operating_system: match game_executable.operating_system {
                        OS::Windows => "windows",
                        OS::Linux => "linux",
                    },
                    game_metadata_id: game_id,
                })
                .collect();
            diesel::insert_into(game_executable::table)
                .values(&new_executables)
                .execute(connection)?;

            Ok(true)
        })
    }

    pub fn add_game_metadata(
        &self,
        game_metadata: &GameMetadataCreate,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let connection = &mut self.pool.get()?;

        connection.immediate_transaction(|connection| {
            let new_game = NewGameMetadata {
                steam_appid: game_metadata.steam_appid.as_deref(),
                default_name: game_metadata.default_name.as_deref(),
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

            let new_paths: Vec<NewGamePath> = game_metadata
                .path_to_save
                .iter()
                .map(|path| NewGamePath {
                    path: &*path.path,
                    operating_system: match path.operating_system {
                        OS::Windows => "windows",
                        OS::Linux => "linux",
                    },
                    game_metadata_id: inserted_id,
                })
                .collect();

            diesel::insert_into(game_path::table)
                .values(&new_paths)
                .execute(connection)?;

            let new_executables: Vec<NewGameExecutable> = game_metadata
                .executable
                .iter()
                .map(|game_executable| NewGameExecutable {
                    executable: &*game_executable.executable,
                    operating_system: match game_executable.operating_system {
                        OS::Windows => "windows",
                        OS::Linux => "linux",
                    },
                    game_metadata_id: inserted_id,
                })
                .collect();

            diesel::insert_into(game_executable::table)
                .values(&new_executables)
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
            let maybe_meta: Option<(Option<i32>, Option<String>, Option<String>)> =
                game_metadata::table
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

            let path_rows: Vec<(String, String)> = game_path::table
                .filter(game_path::game_metadata_id.eq(id))
                .select((game_path::path, game_path::operating_system))
                .load(connection)?;
            let mut paths: Vec<Path> = Vec::with_capacity(path_rows.len());
            for (path, os_str) in path_rows {
                let os = match os_str.as_str() {
                    "windows" => OS::Windows,
                    "linux" => OS::Linux,
                    _ => continue,
                };
                paths.push(Path {
                    path,
                    operating_system: os,
                });
            }

            let exec_rows: Vec<(String, String)> = game_executable::table
                .filter(game_executable::game_metadata_id.eq(id))
                .select((
                    game_executable::executable,
                    game_executable::operating_system,
                ))
                .load(connection)?;
            let mut executables: Vec<Executable> = Vec::with_capacity(exec_rows.len());
            for (executable, os_str) in exec_rows {
                let os = match os_str.as_str() {
                    "windows" => OS::Windows,
                    "linux" => OS::Linux,
                    _ => continue,
                };
                executables.push(Executable {
                    executable,
                    operating_system: os,
                });
            }

            Ok(Some(GameMetadata {
                id: Some(id),
                metadata: GameMetadataCreate {
                    known_name: name_rows,
                    steam_appid,
                    default_name,
                    path_to_save: paths,
                    executable: executables,
                },
            }))
        })
    }
    pub fn get_games_metadata(&self) -> Result<Vec<GameMetadata>, Box<dyn std::error::Error>> {
        let connection = &mut self.pool.get()?;

        connection.immediate_transaction(|connection| {
            let metas: Vec<(Option<i32>, Option<String>, Option<String>)> = game_metadata::table
                .select((
                    game_metadata::id,
                    game_metadata::default_name,
                    game_metadata::steam_appid,
                ))
                .load(connection)?;

            let name_rows: Vec<(i32, String)> = game_alt_name::table
                .select((game_alt_name::game_metadata_id, game_alt_name::name))
                .load(connection)?;
            let mut names_by_game: HashMap<i32, Vec<String>> = HashMap::new();
            for (gid, name) in name_rows {
                names_by_game.entry(gid).or_default().push(name);
            }

            let path_rows: Vec<(i32, String, String)> = game_path::table
                .select((
                    game_path::game_metadata_id,
                    game_path::path,
                    game_path::operating_system,
                ))
                .load(connection)?;
            let mut paths_by_game: HashMap<i32, Vec<Path>> = HashMap::new();
            for (gid, path, os_str) in path_rows {
                let os = match os_str.as_str() {
                    "windows" => OS::Windows,
                    "linux" => OS::Linux,
                    _ => continue,
                };
                paths_by_game.entry(gid).or_default().push(Path {
                    path,
                    operating_system: os,
                });
            }

            let exec_rows: Vec<(i32, String, String)> = game_executable::table
                .select((
                    game_executable::game_metadata_id,
                    game_executable::executable,
                    game_executable::operating_system,
                ))
                .load(connection)?;
            let mut execs_by_game: HashMap<i32, Vec<Executable>> = HashMap::new();
            for (gid, executable, os_str) in exec_rows {
                let os = match os_str.as_str() {
                    "windows" => OS::Windows,
                    "linux" => OS::Linux,
                    _ => continue,
                };
                execs_by_game.entry(gid).or_default().push(Executable {
                    executable,
                    operating_system: os,
                });
            }

            let mut games_metadata: Vec<GameMetadata> = Vec::with_capacity(metas.len());
            for (maybe_id, default_name, steam_appid) in metas {
                let id = maybe_id.unwrap();
                let known_name = names_by_game.remove(&id).unwrap_or_default();
                let path_to_save = paths_by_game.remove(&id).unwrap_or_default();
                let executable = execs_by_game.remove(&id).unwrap_or_default();

                games_metadata.push(GameMetadata {
                    id: Some(id),
                    metadata: GameMetadataCreate {
                        known_name,
                        steam_appid,
                        default_name,
                        path_to_save,
                        executable,
                    },
                });
            }
            Ok(games_metadata)
        })
    }
}
