CREATE TABLE IF NOT EXISTS configurations (
    id TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

INSERT INTO configurations VALUES ('max_save_per_game', '5')
