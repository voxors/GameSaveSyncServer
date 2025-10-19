CREATE TABLE game_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    default_name TEXT NOT NULL,
    steam_appid TEXT
    );

CREATE TABLE game_alt_name (
    name TEXT NOT NULL,
    game_metadata_id INTEGER NOT NULL,
    PRIMARY KEY (name, game_metadata_id),
    FOREIGN KEY (game_metadata_id) REFERENCES game_metadata(id)
    );


CREATE TABLE game_path (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL,
    operating_system TEXT NOT NULL,
    game_metadata_id INTEGER NOT NULL,
    UNIQUE (path, operating_system, game_metadata_id),
    FOREIGN KEY(game_metadata_id) REFERENCES game_metadata(id)
    );

CREATE TABLE game_executable (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    executable TEXT NOT NULL,
    operating_system TEXT NOT NULL,
    game_metadata_id INTEGER NOT NULL,
    UNIQUE (executable, operating_system, game_metadata_id),
    FOREIGN KEY(game_metadata_id) REFERENCES game_metadata(id)
    );

CREATE TABLE game_save (
    uuid TEXT NOT NULL PRIMARY KEY,
    path_id INTEGER NOT NULL,
    time TIMESTAMP NOT NULL,
    FOREIGN KEY(path_id) REFERENCES game_path(id)
    );

CREATE TABLE file_hash (
    relative_path TEXT NOT NULL,
    hash TEXT NOT NULL,
    game_save_uuid TEXT NOT NULL,
    PRIMARY KEY (relative_path, game_save_uuid)
    FOREIGN KEY(game_save_uuid) REFERENCES game_save(uuid)
    );
