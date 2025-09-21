-- Your SQL goes here
CREATE TABLE game_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    default_name TEXT,
    steam_appid TEXT
    );

CREATE TABLE game_alt_name (
   name TEXT NOT NULL,
   game_metadata_id INTEGER NOT NULL,
   PRIMARY KEY (name, game_metadata_id),
   FOREIGN KEY (game_metadata_id)
       REFERENCES game_metadata(id)
    );


CREATE TABLE game_path (
    path TEXT NOT NULL,
    operating_system TEXT NOT NULL,
    game_metadata_id INTEGER NOT NULL,
    PRIMARY KEY (path, operating_system, game_metadata_id),
    FOREIGN KEY(game_metadata_id) REFERENCES game_metadata(id)
    );

CREATE TABLE game_executable (
    executable TEXT NOT NULL,
    operating_system TEXT NOT NULL,
    game_metadata_id INTEGER NOT NULL,
    PRIMARY KEY (executable, operating_system, game_metadata_id),
    FOREIGN KEY(game_metadata_id) REFERENCES game_metadata(id)
    );
