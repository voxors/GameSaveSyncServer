// @generated automatically by Diesel CLI.

diesel::table! {
    api_tokens (id) {
        id -> Nullable<Integer>,
        api_token -> Text,
    }
}

diesel::table! {
    configurations (id) {
        id -> Text,
        value -> Text,
    }
}

diesel::table! {
    db_info (id) {
        id -> Nullable<Integer>,
        db_uuid -> Text,
    }
}

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
    game_gog_extra_id (id, game_metadata_id) {
        id -> BigInt,
        game_metadata_id -> Integer,
    }
}

diesel::table! {
    game_metadata (id) {
        id -> Nullable<Integer>,
        default_name -> Text,
        steam_appid -> Nullable<Text>,
        install_dir -> Nullable<Text>,
        gog -> Nullable<Text>,
        flatpak_id -> Nullable<Text>,
        lutris_id -> Nullable<Text>,
        epic_cloud -> Nullable<Bool>,
        gog_cloud -> Nullable<Bool>,
        origin_cloud -> Nullable<Bool>,
        steam_cloud -> Nullable<Bool>,
        uplay_cloud -> Nullable<Bool>,
        ludusavi_managed -> Nullable<Bool>,
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
    game_registry (path, game_metadata_id) {
        path -> Text,
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

diesel::table! {
    game_steam_extra_id (id, game_metadata_id) {
        id -> BigInt,
        game_metadata_id -> Integer,
    }
}

diesel::joinable!(file_hash -> game_save (game_save_uuid));
diesel::joinable!(game_alt_name -> game_metadata (game_metadata_id));
diesel::joinable!(game_executable -> game_metadata (game_metadata_id));
diesel::joinable!(game_gog_extra_id -> game_metadata (game_metadata_id));
diesel::joinable!(game_path -> game_metadata (game_metadata_id));
diesel::joinable!(game_registry -> game_metadata (game_metadata_id));
diesel::joinable!(game_save -> game_path (path_id));
diesel::joinable!(game_steam_extra_id -> game_metadata (game_metadata_id));

diesel::allow_tables_to_appear_in_same_query!(
    api_tokens,
    configurations,
    db_info,
    file_hash,
    game_alt_name,
    game_executable,
    game_gog_extra_id,
    game_metadata,
    game_path,
    game_registry,
    game_save,
    game_steam_extra_id,
);
