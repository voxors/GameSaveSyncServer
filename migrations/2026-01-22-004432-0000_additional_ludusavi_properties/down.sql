DROP TABLE IF EXISTS game_registry;
DROP TABLE IF EXISTS game_steam_extra_id;
DROP TABLE IF EXISTS game_gog_extra_id;

ALTER TABLE game_metadata
DROP COLUMN install_dir;
ALTER TABLE game_metadata
DROP COLUMN gog;
ALTER TABLE game_metadata
DROP COLUMN flatpak_id;
ALTER TABLE game_metadata
DROP COLUMN lutris_id;
ALTER TABLE game_metadata
DROP COLUMN epic_cloud;
ALTER TABLE game_metadata
DROP COLUMN gog_cloud;
ALTER TABLE game_metadata
DROP COLUMN origin_cloud;
ALTER TABLE game_metadata
DROP COLUMN steam_cloud;
ALTER TABLE game_metadata
DROP COLUMN uplay_cloud;
ALTER TABLE game_metadata
DROP COLUMN ludusavi_managed;
