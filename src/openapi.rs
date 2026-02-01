use crate::datatype_endpoint::{
    Executable, ExecutableCreate, FileHash, GameMetadata, GameMetadataCreate, OS, SavePath,
    SavePathCreate, SaveReference, UploadedFileYaml, UploadedSave,
};
use crate::route_configuration::{__path_get_configuration, __path_put_configuration};
use crate::route_executables::{
    __path_get_game_executables, __path_get_game_executables_by_os, __path_post_game_executable,
};
use crate::route_games::{
    __path_get_game_metadata, __path_get_games_default_name, __path_get_games_metadata,
    __path_get_games_metadata_with_paths_if_saves_exists, __path_post_game_metadata,
};
use crate::route_health::__path_get_health;
use crate::route_paths::{
    __path_get_game_paths, __path_get_game_paths_by_os, __path_post_game_path,
};
use crate::route_registry_paths::{__path_get_game_registries, __path_post_game_registry};
use crate::route_saves::{
    __path_get_game_save_by_uuid, __path_get_game_saves_reference_by_path_id,
    __path_post_game_save_by_path_id,
};
use crate::route_uuid::__path_get_db_uuid;
use crate::route_yaml_import::__path_post_ludusavi_yaml;
use utoipa::{
    OpenApi,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        get_configuration,
        get_db_uuid,
        get_game_executables,
        get_game_executables_by_os,
        get_game_metadata,
        get_game_paths,
        get_game_paths_by_os,
        get_game_registries,
        get_game_save_by_uuid,
        get_game_saves_reference_by_path_id,
        get_games_default_name,
        get_games_metadata,
        get_games_metadata_with_paths_if_saves_exists,
        get_health,
        post_game_executable,
        post_game_metadata,
        post_game_path,
        post_game_registry,
        post_game_save_by_path_id,
        post_ludusavi_yaml,
        put_configuration,
    ),
    components(schemas(
        FileHash,
        UploadedSave,
        UploadedFileYaml,
        SavePathCreate,
        SavePath,
        ExecutableCreate,
        Executable,
        GameMetadataCreate,
        GameMetadata,
        SaveReference,
        OS,
    ),),
    security(
        ("bearer_auth" = [])
    ),
    modifiers(&GameSaveSyncSecurityAddon)
)]
pub struct ApiDoc;

pub struct GameSaveSyncSecurityAddon;

impl utoipa::Modify for GameSaveSyncSecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi
            .components
            .get_or_insert_with(utoipa::openapi::Components::new);

        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
    }
}
