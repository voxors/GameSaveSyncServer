// @generated automatically by Diesel CLI.

diesel::table! {
    game_alt_name (name, game_metadata_id) {
        name -> Text,
        game_metadata_id -> Integer,
    }
}

diesel::table! {
    game_executable (executable, operating_system, game_metadata_id) {
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

diesel::joinable!(game_alt_name -> game_metadata (game_metadata_id));
diesel::joinable!(game_executable -> game_metadata (game_metadata_id));
diesel::joinable!(game_path -> game_metadata (game_metadata_id));

diesel::allow_tables_to_appear_in_same_query!(
    game_alt_name,
    game_executable,
    game_metadata,
    game_path,
);
