CREATE TABLE game_registry (
    path TEXT NOT NULL,
    game_metadata_id INTEGER NOT NULL,
    PRIMARY KEY (path, game_metadata_id),
    FOREIGN KEY (game_metadata_id) REFERENCES game_metadata(id)
    );

CREATE TABLE game_steam_extra_id (
    id BigInt NOT NULL,
    game_metadata_id INTEGER NOT NULL,
    PRIMARY KEY (id, game_metadata_id),
    FOREIGN KEY (game_metadata_id) REFERENCES game_metadata(id)
    );

CREATE TABLE game_gog_extra_id (
    id BigInt NOT NULL,
    game_metadata_id INTEGER NOT NULL,
    PRIMARY KEY (id, game_metadata_id),
    FOREIGN KEY (game_metadata_id) REFERENCES game_metadata(id)
    );

ALTER TABLE game_metadata
ADD COLUMN install_dir TEXT;
ALTER TABLE game_metadata
ADD COLUMN gog TEXT;
ALTER TABLE game_metadata
ADD COLUMN flatpak_id TEXT;
ALTER TABLE game_metadata
ADD COLUMN lutris_id TEXT;
ALTER TABLE game_metadata
ADD COLUMN epic_cloud BOOL;
ALTER TABLE game_metadata
ADD COLUMN gog_cloud BOOL;
ALTER TABLE game_metadata
ADD COLUMN origin_cloud BOOL;
ALTER TABLE game_metadata
ADD COLUMN steam_cloud BOOL;
ALTER TABLE game_metadata
ADD COLUMN uplay_cloud BOOL;
