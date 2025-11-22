use crate::database::datatype::{
    DbApiTokens, DbDbInfo, DbFileHash, DbGameExecutable, DbGameMetadata, DbGameName, DbGamePath,
    DbGameSave,
};

use crate::database::schema::{
    api_tokens, db_info, file_hash, game_alt_name, game_executable, game_metadata, game_path,
    game_save,
};
use crate::datatype_endpoint::{
    Executable, ExecutableCreate, FileHash, GameMetadata, GameMetadataCreate, OS, SavePath,
    SavePathCreate, SaveReference,
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

        let mut conn = pool.get().expect("Failed to get DB connection");
        conn.run_pending_migrations(MIGRATIONS)
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
            println!("Initial API token : {uuid}")
        }

        db
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
                    steam_appid: game_metadata.steam_appid.clone(),
                    default_name: game_metadata.default_name.clone(),
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
                            name: name.to_string(),
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
        let db_games: Vec<DbGameMetadata> = game_metadata::table
            .filter(game_metadata::default_name.eq(target_name))
            .select(DbGameMetadata::as_select())
            .load(connection)?;

        let mut games: Vec<GameMetadata> = Vec::with_capacity(db_games.len());
        for db_game in db_games {
            let known_name: Vec<String> = game_alt_name::table
                .filter(game_alt_name::game_metadata_id.eq(db_game.id.unwrap()))
                .select(game_alt_name::name)
                .load(connection)?;

            games.push(GameMetadata {
                id: db_game.id,
                metadata: GameMetadataCreate {
                    known_name,
                    steam_appid: db_game.steam_appid,
                    default_name: db_game.default_name,
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

            let name_rows: Vec<String> = game_alt_name::table
                .filter(game_alt_name::game_metadata_id.eq(id))
                .select(game_alt_name::name)
                .load(connection)?;

            Ok(Some(GameMetadata {
                id: Some(id),
                metadata: GameMetadataCreate {
                    known_name: name_rows,
                    steam_appid: meta.steam_appid,
                    default_name: meta.default_name,
                },
            }))
        })
    }

    pub fn get_games_metadata(&self) -> Result<Vec<GameMetadata>, Box<dyn std::error::Error>> {
        let connection = &mut self.pool.get()?;
        let db_games: Vec<DbGameMetadata> = game_metadata::table
            .select(DbGameMetadata::as_select())
            .load(connection)?;

        let mut games = Vec::with_capacity(db_games.len());
        for db_game_metadata in db_games {
            let known_name: Vec<String> = game_alt_name::table
                .filter(game_alt_name::game_metadata_id.eq(db_game_metadata.id.unwrap()))
                .select(game_alt_name::name)
                .load(connection)?;

            games.push(GameMetadata {
                id: db_game_metadata.id,
                metadata: GameMetadataCreate {
                    known_name,
                    steam_appid: db_game_metadata.steam_appid,
                    default_name: db_game_metadata.default_name,
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
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
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
        files_hash: Vec<FileHash>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<Option<Vec<SaveReference>>, Box<dyn std::error::Error + Send + Sync>> {
        let connection = &mut self.pool.get()?;

        let save_rows = game_save::table
            .filter(game_save::path_id.eq(path_id))
            .select(DbGameSave::as_select())
            .load(connection)?;

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

    pub fn get_database_uuid(
        &self,
    ) -> Result<Option<Uuid>, Box<dyn std::error::Error + Send + Sync>> {
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

    pub fn add_database_uuid(
        &self,
        uuid: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let connection = &mut self.pool.get()?;
        diesel::insert_into(db_info::table)
            .values(DbDbInfo {
                id: None,
                db_uuid: uuid.to_string(),
            })
            .execute(connection)?;

        Ok(())
    }

    pub fn get_api_tokens(&self) -> Result<Vec<Uuid>, Box<dyn std::error::Error + Send + Sync>> {
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

    pub fn add_api_tokens(
        &self,
        uuids: Vec<Uuid>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    pub fn remove_api_tokens(
        &self,
        uuids: Vec<Uuid>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let db_api_tokens: Vec<String> = uuids.iter().map(|uuid| uuid.to_string()).collect();

        let connection = &mut self.pool.get()?;
        diesel::delete(api_tokens::table.filter(api_tokens::api_token.eq_any(&db_api_tokens)))
            .execute(connection)?;

        Ok(())
    }
}
