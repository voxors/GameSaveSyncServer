// @generated automatically by Diesel CLI.

diesel::table! {
    file_hash (relative_path, game_save_uuid) {
        relative_path -> Text,
        hash -> Text,
        game_save_uuid -> Text,
    }
}

diesel::table! {
    game_alt_name (name, game_metadata_id) {
        name -> Text,
        game_metadata_id -> Integer,
    }
}

diesel::table! {
    game_executable (id) {
        id -> Nullable<Integer>,
        executable -> Text,
        operating_system -> Text,
        game_metadata_id -> Integer,
    }
}

diesel::table! {
    game_metadata (id) {
        id -> Nullable<Integer>,
        default_name -> Text,
        steam_appid -> Nullable<Text>,
    }
}

diesel::table! {
    game_path (id) {
        id -> Nullable<Integer>,
        path -> Text,
        operating_system -> Text,
        game_metadata_id -> Integer,
    }
}

diesel::table! {
    game_save (uuid) {
        uuid -> Text,
        path_id -> Integer,
        time -> Timestamp,
    }
}

diesel::joinable!(file_hash -> game_save (game_save_uuid));
diesel::joinable!(game_alt_name -> game_metadata (game_metadata_id));
diesel::joinable!(game_executable -> game_metadata (game_metadata_id));
diesel::joinable!(game_path -> game_metadata (game_metadata_id));
diesel::joinable!(game_save -> game_path (path_id));

diesel::allow_tables_to_appear_in_same_query!(
    file_hash,
    game_alt_name,
    game_executable,
    game_metadata,
    game_path,
    game_save,
);
