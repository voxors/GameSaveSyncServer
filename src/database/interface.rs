use std::collections::HashMap;
use std::error::Error;

use crate::database::datatype::{
    DbApiTokens, DbConfiguration, DbDbInfo, DbFileHash, DbGameExecutable, DbGameGogExtraId,
    DbGameMetadata, DbGameName, DbGamePath, DbGameRegistry, DbGameSave, DbGameSteamExtraId,
};
use crate::database::schema::{
    api_tokens, configurations, db_info, file_hash, game_alt_name, game_executable,
    game_gog_extra_id, game_metadata, game_path, game_registry, game_save, game_steam_extra_id,
};
use crate::datatype_endpoint::{
    Executable, ExecutableCreate, FileHash, GameMetadata, GameMetadataCreate,
    GameMetadataWithPaths, GameRegistry, OS, SavePath, SavePathCreate, SaveReference,
};
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, CustomizeConnection, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use uuid::Uuid;

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Copy, Clone, Debug)]
struct SqliteConnectionCustomizer {}

pub struct GameDatabase {
    pub pool: DbPool,
}

pub struct GameFull {
    pub game_metadata: GameMetadataCreate,
    pub executables: Vec<ExecutableCreate>,
    pub paths: Vec<SavePathCreate>,
    pub registries: Vec<GameRegistry>,
}

fn add_game_metadata(
    connection: &mut SqliteConnection,
    game_metadata: &GameMetadataCreate,
) -> Result<i32, Box<dyn Error + Send + Sync>> {
    diesel::insert_into(game_metadata::table)
        .values(DbGameMetadata {
            id: None,
            steam_appid: game_metadata.steam_appid.clone(),
            default_name: game_metadata.default_name.clone(),
            install_dir: game_metadata.install_dir.clone(),
            gog: game_metadata.gog.clone(),
            flatpak_id: game_metadata.flatpak_id.clone(),
            lutris_id: game_metadata.lutris_id.clone(),
            epic_cloud: game_metadata.epic_cloud,
            gog_cloud: game_metadata.gog_cloud,
            origin_cloud: game_metadata.origin_cloud,
            steam_cloud: game_metadata.steam_cloud,
            uplay_cloud: game_metadata.uplay_cloud,
            ludusavi_managed: game_metadata.ludusavi_managed,
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

    add_game_metadata_additional_info(connection, game_metadata, inserted_id)?;

    Ok(inserted_id)
}

fn add_game_metadata_additional_info(
    connection: &mut SqliteConnection,
    game_metadata: &GameMetadataCreate,
    inserted_id: i32,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    diesel::insert_into(game_alt_name::table)
        .values(
            game_metadata
                .known_name
                .as_ref()
                .iter()
                .flat_map(|names| {
                    names.iter().map(|name| DbGameName {
                        name: name.to_string(),
                        game_metadata_id: inserted_id,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .execute(connection)?;
    diesel::insert_into(game_gog_extra_id::table)
        .values(
            game_metadata
                .gog_extra
                .as_ref()
                .iter()
                .flat_map(|gog_extras| {
                    gog_extras.iter().map(|gog_extra| DbGameGogExtraId {
                        id: *gog_extra,
                        game_metadata_id: inserted_id,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .execute(connection)?;
    diesel::insert_into(game_steam_extra_id::table)
        .values(
            game_metadata
                .steam_extra
                .as_ref()
                .iter()
                .flat_map(|steam_extras| {
                    steam_extras.iter().map(|steam_extra| DbGameSteamExtraId {
                        id: *steam_extra,
                        game_metadata_id: inserted_id,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .execute(connection)?;
    Ok(())
}

fn update_game_metadata(
    connection: &mut SqliteConnection,
    db_game_id: i32,
    game_metadata: &GameMetadataCreate,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    diesel::update(game_metadata::table)
        .filter(game_metadata::id.eq(db_game_id))
        .set(DbGameMetadata {
            id: Some(db_game_id),
            steam_appid: game_metadata.steam_appid.clone(),
            default_name: game_metadata.default_name.clone(),
            install_dir: game_metadata.install_dir.clone(),
            gog: game_metadata.gog.clone(),
            flatpak_id: game_metadata.flatpak_id.clone(),
            lutris_id: game_metadata.lutris_id.clone(),
            epic_cloud: game_metadata.epic_cloud,
            gog_cloud: game_metadata.gog_cloud,
            origin_cloud: game_metadata.origin_cloud,
            steam_cloud: game_metadata.steam_cloud,
            uplay_cloud: game_metadata.uplay_cloud,
            ludusavi_managed: game_metadata.ludusavi_managed,
        })
        .execute(connection)?;

    diesel::delete(game_alt_name::table.filter(game_alt_name::game_metadata_id.eq(db_game_id)))
        .execute(connection)?;
    diesel::delete(
        game_gog_extra_id::table.filter(game_gog_extra_id::game_metadata_id.eq(db_game_id)),
    )
    .execute(connection)?;
    diesel::delete(
        game_steam_extra_id::table.filter(game_steam_extra_id::game_metadata_id.eq(db_game_id)),
    )
    .execute(connection)?;

    add_game_metadata_additional_info(connection, game_metadata, db_game_id)?;

    Ok(())
}

impl CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for SqliteConnectionCustomizer {
    fn on_acquire(&self, connection: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        connection.batch_execute("PRAGMA busy_timeout = 2000;")?;
        connection.batch_execute("PRAGMA journal_mode = WAL;")?;
        connection.batch_execute("PRAGMA synchronous = NORMAL;")?;
        connection.batch_execute("PRAGMA wal_autocheckpoint = 1000;")?;
        connection.batch_execute("PRAGMA wal_checkpoint(TRUNCATE);")?;
        Ok(())
    }

    fn on_release(&self, _conn: SqliteConnection) {}
}

impl GameDatabase {
    pub fn new(db_path: &str) -> Self {
        let manager = ConnectionManager::<SqliteConnection>::new(db_path);
        let pool = Pool::builder()
            .connection_customizer(Box::new(SqliteConnectionCustomizer {}))
            .build(manager)
            .expect("Failed to create pool");

        pool.get()
            .expect("Failed to get DB connection")
            .run_pending_migrations(MIGRATIONS)
            .expect("Failed to run database migrations");

        let db = Self { pool };

        match db
            .get_database_uuid()
            .expect("unable to get uuid the db at uuid initial validation")
        {
            Some(_) => (),
            None => {
                db.add_database_uuid(Uuid::new_v4())
                    .expect("unable to add initial uuid");
            }
        }

        let api_tokens = db
            .get_api_tokens()
            .expect("unable to get api_tokens the db at api_tokens initial validation");

        if api_tokens.is_empty() {
            let uuid = Uuid::new_v4();
            db.add_api_tokens(vec![uuid])
                .expect("unable to add initial api tokens");

            tracing::info!("Initial API token : {uuid}");
        }

        db
    }

    pub fn add_games_full(&self, games: Vec<GameFull>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let connection = &mut self.pool.get()?;

        for game in games {
            connection.immediate_transaction(|conn| {
                let inserted_id = add_game_metadata(conn, &game.game_metadata)?;

                if !game.executables.is_empty() {
                    diesel::insert_into(game_executable::table)
                        .values(
                            game.executables
                                .iter()
                                .map(|executable| DbGameExecutable {
                                    id: None,
                                    executable: executable.executable.clone(),
                                    operating_system: executable.operating_system,
                                    game_metadata_id: inserted_id,
                                })
                                .collect::<Vec<_>>(),
                        )
                        .execute(conn)?;
                }

                if !game.paths.is_empty() {
                    diesel::insert_into(game_path::table)
                        .values(
                            game.paths
                                .iter()
                                .map(|path| DbGamePath {
                                    id: None,
                                    path: path.path.clone(),
                                    operating_system: path.operating_system,
                                    game_metadata_id: inserted_id,
                                })
                                .collect::<Vec<_>>(),
                        )
                        .execute(conn)?;
                }

                if !game.registries.is_empty() {
                    diesel::insert_into(game_registry::table)
                        .values(
                            game.registries
                                .iter()
                                .map(|registry| DbGameRegistry {
                                    path: registry.path.clone(),
                                    game_metadata_id: inserted_id,
                                })
                                .collect::<Vec<_>>(),
                        )
                        .execute(conn)?;
                }

                Ok::<(), Box<dyn Error + Send + Sync>>(())
            })?
        }

        Ok(())
    }

    pub fn update_games_full(
        &self,
        games: Vec<(i32, GameFull)>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let connection = &mut self.pool.get()?;

        for (db_game_id, game) in games {
            connection.immediate_transaction(|conn| {
                update_game_metadata(conn, db_game_id, &game.game_metadata)?;

                for executable in game.executables {
                    diesel::insert_into(game_executable::table)
                        .values(DbGameExecutable {
                            id: None,
                            executable: executable.executable.clone(),
                            operating_system: executable.operating_system,
                            game_metadata_id: db_game_id,
                        })
                        .on_conflict((
                            game_executable::executable,
                            game_executable::operating_system,
                            game_executable::game_metadata_id,
                        ))
                        .do_update()
                        .set((
                            game_executable::executable.eq(executable.executable.clone()),
                            game_executable::operating_system.eq(executable.operating_system),
                        ))
                        .execute(conn)?;
                }

                for path in game.paths {
                    diesel::insert_into(game_path::table)
                        .values(DbGamePath {
                            id: None,
                            path: path.path.clone(),
                            operating_system: path.operating_system,
                            game_metadata_id: db_game_id,
                        })
                        .on_conflict((
                            game_path::path,
                            game_path::operating_system,
                            game_path::game_metadata_id,
                        ))
                        .do_update()
                        .set((
                            game_path::path.eq(path.path.clone()),
                            game_path::operating_system.eq(path.operating_system),
                        ))
                        .execute(conn)?;
                }

                for registry in game.registries {
                    diesel::insert_into(game_registry::table)
                        .values(DbGameRegistry {
                            path: registry.path.clone(),
                            game_metadata_id: db_game_id,
                        })
                        .on_conflict((game_registry::path, game_registry::game_metadata_id))
                        .do_update()
                        .set(game_registry::path.eq(registry.path))
                        .execute(conn)?;
                }
                Ok::<(), Box<dyn Error + Send + Sync>>(())
            })?
        }

        Ok(())
    }

    pub fn add_games_metadata(
        &self,
        games_metadata: Vec<&GameMetadataCreate>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let connection = &mut self.pool.get()?;

        for game_metadata in games_metadata {
            connection.immediate_transaction(|connection| {
                add_game_metadata(connection, game_metadata)?;
                Ok::<(), Box<dyn Error + Send + Sync>>(())
            })?;
        }
        Ok(())
    }

    pub fn get_game_metadata_by_name(
        &self,
        target_name: &str,
    ) -> Result<Vec<GameMetadata>, Box<dyn Error + Send + Sync>> {
        let connection = &mut self.pool.get()?;
        let db_games: Vec<DbGameMetadata> = game_metadata::table
            .filter(game_metadata::default_name.eq(target_name))
            .select(DbGameMetadata::as_select())
            .load(connection)
            .optional()?
            .unwrap_or_default();

        let mut games: Vec<GameMetadata> = Vec::with_capacity(db_games.len());
        for db_game in db_games {
            let known_name: Option<Vec<String>> = game_alt_name::table
                .filter(game_alt_name::game_metadata_id.eq(db_game.id.unwrap()))
                .select(game_alt_name::name)
                .load(connection)
                .optional()?;

            let gog_extra: Option<Vec<i64>> = game_gog_extra_id::table
                .filter(game_gog_extra_id::game_metadata_id.eq(db_game.id.unwrap()))
                .select(game_gog_extra_id::id)
                .load(connection)
                .optional()?;

            let steam_extra: Option<Vec<i64>> = game_steam_extra_id::table
                .filter(game_steam_extra_id::game_metadata_id.eq(db_game.id.unwrap()))
                .select(game_steam_extra_id::id)
                .load(connection)
                .optional()?;

            games.push(GameMetadata {
                id: db_game.id,
                metadata: GameMetadataCreate {
                    known_name,
                    steam_appid: db_game.steam_appid,
                    default_name: db_game.default_name,
                    install_dir: db_game.install_dir,
                    gog: db_game.gog,
                    flatpak_id: db_game.flatpak_id,
                    lutris_id: db_game.lutris_id,
                    epic_cloud: db_game.epic_cloud,
                    gog_cloud: db_game.gog_cloud,
                    origin_cloud: db_game.origin_cloud,
                    steam_cloud: db_game.steam_cloud,
                    uplay_cloud: db_game.uplay_cloud,
                    gog_extra,
                    steam_extra,
                    ludusavi_managed: db_game.ludusavi_managed,
                },
            });
        }
        Ok(games)
    }

    pub fn get_game_metadata_by_id(
        &self,
        target_id: &i32,
    ) -> Result<Option<GameMetadata>, Box<dyn Error>> {
        let connection = &mut self.pool.get()?;

        connection.immediate_transaction(|connection| {
            let maybe_meta: Option<DbGameMetadata> = game_metadata::table
                .filter(game_metadata::id.eq(target_id))
                .select(DbGameMetadata::as_select())
                .first(connection)
                .optional()?;

            let meta = match maybe_meta {
                Some(meta) => meta,
                None => return Ok(None),
            };

            let id = match meta.id {
                Some(id) => id,
                None => return Ok(None),
            };

            let name_rows: Option<Vec<String>> = game_alt_name::table
                .filter(game_alt_name::game_metadata_id.eq(id))
                .select(game_alt_name::name)
                .load(connection)
                .optional()?;

            let gog_extra: Option<Vec<i64>> = game_gog_extra_id::table
                .filter(game_gog_extra_id::game_metadata_id.eq(id))
                .select(game_gog_extra_id::id)
                .load(connection)
                .optional()?;

            let steam_extra: Option<Vec<i64>> = game_steam_extra_id::table
                .filter(game_steam_extra_id::game_metadata_id.eq(id))
                .select(game_steam_extra_id::id)
                .load(connection)
                .optional()?;

            Ok(Some(GameMetadata {
                id: Some(id),
                metadata: GameMetadataCreate {
                    known_name: name_rows,
                    steam_appid: meta.steam_appid,
                    default_name: meta.default_name,
                    install_dir: meta.install_dir,
                    gog: meta.gog,
                    flatpak_id: meta.flatpak_id,
                    lutris_id: meta.lutris_id,
                    epic_cloud: meta.epic_cloud,
                    gog_cloud: meta.gog_cloud,
                    origin_cloud: meta.origin_cloud,
                    steam_cloud: meta.steam_cloud,
                    uplay_cloud: meta.uplay_cloud,
                    gog_extra,
                    steam_extra,
                    ludusavi_managed: meta.ludusavi_managed,
                },
            }))
        })
    }

    pub fn get_games_metadata(&self) -> Result<Vec<GameMetadata>, Box<dyn Error>> {
        let connection = &mut self.pool.get()?;
        let db_games: Vec<DbGameMetadata> = game_metadata::table
            .select(DbGameMetadata::as_select())
            .load(connection)
            .optional()?
            .unwrap_or_default();

        let mut games = Vec::with_capacity(db_games.len());
        for db_game_metadata in db_games {
            let known_name: Option<Vec<String>> = game_alt_name::table
                .filter(game_alt_name::game_metadata_id.eq(db_game_metadata.id.unwrap()))
                .select(game_alt_name::name)
                .load(connection)
                .optional()?;

            let gog_extra: Option<Vec<i64>> = game_gog_extra_id::table
                .filter(game_gog_extra_id::game_metadata_id.eq(db_game_metadata.id.unwrap()))
                .select(game_gog_extra_id::id)
                .load(connection)
                .optional()?;

            let steam_extra: Option<Vec<i64>> = game_steam_extra_id::table
                .filter(game_steam_extra_id::game_metadata_id.eq(db_game_metadata.id.unwrap()))
                .select(game_steam_extra_id::id)
                .load(connection)
                .optional()?;

            games.push(GameMetadata {
                id: db_game_metadata.id,
                metadata: GameMetadataCreate {
                    known_name,
                    steam_appid: db_game_metadata.steam_appid,
                    default_name: db_game_metadata.default_name,
                    install_dir: db_game_metadata.install_dir,
                    gog: db_game_metadata.gog,
                    flatpak_id: db_game_metadata.flatpak_id,
                    lutris_id: db_game_metadata.lutris_id,
                    epic_cloud: db_game_metadata.epic_cloud,
                    gog_cloud: db_game_metadata.gog_cloud,
                    origin_cloud: db_game_metadata.origin_cloud,
                    steam_cloud: db_game_metadata.steam_cloud,
                    uplay_cloud: db_game_metadata.uplay_cloud,
                    gog_extra,
                    steam_extra,
                    ludusavi_managed: db_game_metadata.ludusavi_managed,
                },
            });
        }

        Ok(games)
    }

    pub fn get_games_metadata_and_paths_if_saves_exist(
        &self,
    ) -> Result<Vec<GameMetadataWithPaths>, Box<dyn Error>> {
        let connection = &mut self.pool.get()?;
        let db_games: Vec<(DbGameMetadata, DbGamePath)> = game_metadata::table
            .inner_join(
                game_path::table.on(game_path::game_metadata_id.nullable().eq(game_metadata::id)),
            )
            .inner_join(game_save::table.on(game_save::path_id.nullable().eq(game_path::id)))
            .select((DbGameMetadata::as_select(), DbGamePath::as_select()))
            .distinct()
            .load(connection)?;

        let mut games_map: HashMap<i32, (DbGameMetadata, Vec<DbGamePath>)> = HashMap::new();
        for (metadata, path) in db_games {
            let id = metadata.id.unwrap();
            games_map
                .entry(id)
                .or_insert_with(|| (metadata.clone(), Vec::new()))
                .1
                .push(path);
        }

        let mut games = Vec::with_capacity(games_map.len());
        for (game_id, (db_game_metadata, db_paths)) in games_map {
            let known_name: Option<Vec<String>> = game_alt_name::table
                .filter(game_alt_name::game_metadata_id.eq(game_id))
                .select(game_alt_name::name)
                .load(connection)
                .optional()?;

            let gog_extra: Option<Vec<i64>> = game_gog_extra_id::table
                .filter(game_gog_extra_id::game_metadata_id.eq(game_id))
                .select(game_gog_extra_id::id)
                .load(connection)
                .optional()?;

            let steam_extra: Option<Vec<i64>> = game_steam_extra_id::table
                .filter(game_steam_extra_id::game_metadata_id.eq(game_id))
                .select(game_steam_extra_id::id)
                .load(connection)
                .optional()?;

            games.push(GameMetadataWithPaths {
                game_metadata: GameMetadata {
                    id: db_game_metadata.id,
                    metadata: GameMetadataCreate {
                        known_name,
                        steam_appid: db_game_metadata.steam_appid,
                        default_name: db_game_metadata.default_name,
                        install_dir: db_game_metadata.install_dir,
                        gog: db_game_metadata.gog,
                        flatpak_id: db_game_metadata.flatpak_id,
                        lutris_id: db_game_metadata.lutris_id,
                        epic_cloud: db_game_metadata.epic_cloud,
                        gog_cloud: db_game_metadata.gog_cloud,
                        origin_cloud: db_game_metadata.origin_cloud,
                        steam_cloud: db_game_metadata.steam_cloud,
                        uplay_cloud: db_game_metadata.uplay_cloud,
                        gog_extra,
                        steam_extra,
                        ludusavi_managed: db_game_metadata.ludusavi_managed,
                    },
                },
                paths: db_paths
                    .iter()
                    .map(|db_path| SavePath {
                        id: db_path.id,
                        path: SavePathCreate {
                            path: db_path.path.to_owned(),
                            operating_system: db_path.operating_system,
                        },
                    })
                    .collect(),
            });
        }

        Ok(games)
    }

    pub fn add_game_path(
        &self,
        game_id: i32,
        path: &SavePathCreate,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let connection = &mut self.pool.get()?;

        diesel::insert_into(game_path::table)
            .values(DbGamePath {
                id: None,
                path: path.path.clone(),
                operating_system: path.operating_system,
                game_metadata_id: game_id,
            })
            .execute(connection)?;
        Ok(())
    }

    pub fn get_paths_by_game_id_and_os(
        &self,
        game_id: i32,
        os: OS,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let connection = &mut self.pool.get()?;
        let paths: Vec<String> = game_path::table
            .filter(game_path::game_metadata_id.eq(game_id))
            .filter(game_path::operating_system.eq(os))
            .select(game_path::path)
            .load(connection)
            .optional()?
            .unwrap_or_default();
        Ok(paths)
    }

    pub fn get_paths_by_game_id(&self, game_id: i32) -> Result<Vec<SavePath>, Box<dyn Error>> {
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
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let connection = &mut self.pool.get()?;
        diesel::insert_into(game_executable::table)
            .values(DbGameExecutable {
                id: None,
                executable: executable.executable.clone(),
                operating_system: executable.operating_system,
                game_metadata_id: game_id,
            })
            .execute(connection)?;
        Ok(())
    }
    pub fn get_executable_by_game_id_and_os(
        &self,
        game_id: i32,
        os: OS,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
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
    ) -> Result<Vec<Executable>, Box<dyn Error>> {
        let connection = &mut self.pool.get()?;
        let executable_rows: Vec<(Option<i32>, String, OS)> = game_executable::table
            .filter(game_executable::game_metadata_id.eq(game_id))
            .select((
                game_executable::id,
                game_executable::executable,
                game_executable::operating_system,
            ))
            .load(connection)
            .optional()?
            .unwrap_or_default();
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
        files_hash: Vec<FileHash>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let connection = &mut self.pool.get()?;
        let now = time::OffsetDateTime::now_utc();

        connection.immediate_transaction(|connection| {
            diesel::insert_into(game_save::table)
                .values(DbGameSave {
                    uuid: uuid.to_string(),
                    path_id,
                    time: time::PrimitiveDateTime::new(now.date(), now.time()),
                })
                .execute(connection)?;

            for file_hash in files_hash {
                diesel::insert_into(file_hash::table)
                    .values(DbFileHash {
                        relative_path: file_hash.relative_path,
                        hash: file_hash.hash,
                        game_save_uuid: uuid.to_string(),
                    })
                    .execute(connection)?;
            }
            Ok(())
        })
    }

    pub fn get_reference_to_save_by_path_id(
        &self,
        path_id: i32,
    ) -> Result<Option<Vec<SaveReference>>, Box<dyn Error + Send + Sync>> {
        let connection = &mut self.pool.get()?;

        let save_rows = game_save::table
            .filter(game_save::path_id.eq(path_id))
            .select(DbGameSave::as_select())
            .load(connection)
            .optional()?
            .unwrap_or_default();

        if save_rows.is_empty() {
            return Ok(None);
        }

        let mut save_references: Vec<SaveReference> = Vec::with_capacity(save_rows.len());
        for game_save in save_rows {
            let files_hash_db =
                DbFileHash::belonging_to(&game_save).load::<DbFileHash>(connection)?;

            save_references.push(SaveReference {
                uuid: game_save.uuid.to_string(),
                path_id: game_save.path_id,
                time: game_save.time.assume_utc().unix_timestamp(),
                files_hash: files_hash_db
                    .iter()
                    .map(|files_hash_db| FileHash {
                        relative_path: files_hash_db.relative_path.clone(),
                        hash: files_hash_db.hash.clone(),
                    })
                    .collect(),
            })
        }

        Ok(Some(save_references))
    }

    pub fn get_database_uuid(&self) -> Result<Option<Uuid>, Box<dyn Error + Send + Sync>> {
        let connection = &mut self.pool.get()?;
        let maybe_db_info: Option<DbDbInfo> = db_info::table
            .select(DbDbInfo::as_select())
            .first(connection)
            .optional()?;

        match maybe_db_info {
            Some(db_info) => Ok(Some(Uuid::parse_str(&db_info.db_uuid)?)),
            None => Ok(None),
        }
    }

    pub fn add_database_uuid(&self, uuid: Uuid) -> Result<(), Box<dyn Error + Send + Sync>> {
        let connection = &mut self.pool.get()?;
        diesel::insert_into(db_info::table)
            .values(DbDbInfo {
                id: None,
                db_uuid: uuid.to_string(),
            })
            .execute(connection)?;

        Ok(())
    }

    pub fn get_api_tokens(&self) -> Result<Vec<Uuid>, Box<dyn Error + Send + Sync>> {
        let mut api_tokens = Vec::<Uuid>::new();

        let connection = &mut self.pool.get()?;
        let maybe_api_tokens = api_tokens::table
            .select(DbApiTokens::as_select())
            .load::<DbApiTokens>(connection)
            .optional()?;

        if let Some(db_api_tokens) = maybe_api_tokens {
            db_api_tokens
                .iter()
                .filter_map(|db_api_token| Uuid::parse_str(&db_api_token.api_token).ok())
                .for_each(|uuid| api_tokens.push(uuid));
        }

        Ok(api_tokens)
    }

    pub fn add_api_tokens(&self, uuids: Vec<Uuid>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let db_api_tokens: Vec<DbApiTokens> = uuids
            .iter()
            .map(|uuid| DbApiTokens {
                id: None,
                api_token: uuid.to_string(),
            })
            .collect();

        let connection = &mut self.pool.get()?;
        diesel::insert_into(api_tokens::table)
            .values(db_api_tokens)
            .execute(connection)?;

        Ok(())
    }

    pub fn remove_api_tokens(&self, uuids: Vec<Uuid>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let db_api_tokens: Vec<String> = uuids.iter().map(|uuid| uuid.to_string()).collect();

        let connection = &mut self.pool.get()?;
        diesel::delete(api_tokens::table.filter(api_tokens::api_token.eq_any(&db_api_tokens)))
            .execute(connection)?;

        Ok(())
    }

    pub fn get_configuration_value(
        &self,
        id: &str,
    ) -> Result<Option<DbConfiguration>, Box<dyn Error + Send + Sync>> {
        let connection = &mut self.pool.get()?;
        let maybe_configurations = configurations::table
            .filter(configurations::columns::id.eq(id))
            .select(DbConfiguration::as_select())
            .first::<DbConfiguration>(connection)
            .optional()?;

        Ok(maybe_configurations)
    }

    pub fn update_configuration_value(
        &self,
        id: &str,
        value: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let connection = &mut self.pool.get()?;
        diesel::update(configurations::table.filter(configurations::columns::id.eq(id)))
            .set(configurations::columns::value.eq(value))
            .execute(connection)?;

        Ok(())
    }

    pub fn get_game_registry_by_game_id(
        &self,
        game_id: i32,
    ) -> Result<Vec<GameRegistry>, Box<dyn Error + Send + Sync>> {
        let connection = &mut self.pool.get()?;
        let registries: Option<Vec<GameRegistry>> = game_registry::table
            .filter(game_registry::game_metadata_id.eq(game_id))
            .load::<DbGameRegistry>(connection)
            .optional()?
            .map(|vec_db_game_registry| {
                vec_db_game_registry
                    .iter()
                    .map(|db_game_registry| GameRegistry {
                        path: db_game_registry.path.clone(),
                    })
                    .collect()
            });

        Ok(registries.unwrap_or_default())
    }

    pub fn add_game_registry_path(
        &self,
        game_id: i32,
        registry: &GameRegistry,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let connection = &mut self.pool.get()?;

        diesel::insert_into(game_registry::table)
            .values(DbGameRegistry {
                path: registry.path.clone(),
                game_metadata_id: game_id,
            })
            .execute(connection)?;
        Ok(())
    }
}
