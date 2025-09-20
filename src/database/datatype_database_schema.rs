// @generated automatically by Diesel CLI.

diesel::table! {
    game_metadata (id) {
        id -> Nullable<Integer>,
        steam_appid -> Text,
    }
}

diesel::table! {
    game_name (id) {
        id -> Nullable<Integer>,
        name -> Text,
        game_metadata_id -> Integer,
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

diesel::joinable!(game_name -> game_metadata (game_metadata_id));
diesel::joinable!(game_path -> game_metadata (game_metadata_id));

diesel::allow_tables_to_appear_in_same_query!(game_metadata, game_name, game_path,);
