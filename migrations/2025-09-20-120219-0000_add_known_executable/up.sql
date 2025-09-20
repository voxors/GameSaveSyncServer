-- Your SQL goes here
CREATE TABLE game_executable (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    executable TEXT NOT NULL,
    operating_system TEXT NOT NULL,
    game_metadata_id INTEGER NOT NULL,
    FOREIGN KEY(game_metadata_id) REFERENCES game_metadata(id)
    );