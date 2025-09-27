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
    __path_get_game_saves_reference_by_path_id, __path_post_game_save_by_path_id,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    post_game_metadata,
    get_game_metadata,
    get_games_metadata,
    get_game_paths,
    post_game_path,
    get_game_paths_by_os,
    get_game_executables,
    get_game_executables_by_os,
    post_game_executable,
    get_game_saves_reference_by_path_id,
    post_game_save_by_path_id,
))]
pub struct ApiDoc;
