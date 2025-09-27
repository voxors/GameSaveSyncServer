use const_format::concatcp;

pub const DATA_DIR: &str = "./data";
pub const MAX_BODY_SIZE: usize = 3 * 1024 * 1024 * 1024;
pub const ROOT_API_PATH: &str = "/v1";
pub const SAVE_DIR: &str = concatcp!(DATA_DIR, "/saves");
pub const TMP_DIR: &str = concatcp!(DATA_DIR, "/tmp");
