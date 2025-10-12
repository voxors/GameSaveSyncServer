use crate::datatype_endpoint::{
    Executable, ExecutableCreate, FileHash, GameMetadata, GameMetadataCreate, OS, SavePath,
    SavePathCreate, SaveReference, UploadedFileYaml, UploadedSave,
};
use crate::route_executable::{
    __path_get_game_executables, __path_get_game_executables_by_os, __path_post_game_executable,
};
use crate::route_games::{
    __path_get_game_metadata, __path_get_games_metadata, __path_post_game_metadata,
};
use crate::route_paths::{
    __path_get_game_paths, __path_get_game_paths_by_os, __path_post_game_path,
};
use crate::route_saves::{
    __path_get_game_save_by_uuid, __path_get_game_saves_reference_by_path_id,
    __path_post_game_save_by_path_id,
};
use crate::route_yaml_import::__path_post_ludusavi_yaml;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        get_game_executables,
        get_game_executables_by_os,
        get_game_metadata,
        get_game_paths,
        get_game_paths_by_os,
        get_game_save_by_uuid,
        get_game_saves_reference_by_path_id,
        get_games_metadata,
        post_game_executable,
        post_game_metadata,
        post_game_path,
        post_game_save_by_path_id,
        post_ludusavi_yaml,
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
    ))
)]
pub struct ApiDoc;
