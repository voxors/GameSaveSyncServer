-- Your SQL goes here
CREATE TABLE game_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    internal_name TEXT NOT NULL,
    steam_appid TEXT
    );

CREATE TABLE game_name (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    game_metadata_id INTEGER NOT NULL,
    FOREIGN KEY(game_metadata_id) REFERENCES game_metadata(id)
    );

CREATE TABLE game_path (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL,
    operating_system TEXT NOT NULL,
    game_metadata_id INTEGER NOT NULL,
    FOREIGN KEY(game_metadata_id) REFERENCES game_metadata(id)
    );

CREATE TABLE game_executable (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    executable TEXT NOT NULL,
    operating_system TEXT NOT NULL,
    game_metadata_id INTEGER NOT NULL,
    FOREIGN KEY(game_metadata_id) REFERENCES game_metadata(id)
    );
