use crate::const_var::{DATA_DIR, TMP_DIR};
use std::error::Error;
use std::path::Path;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

pub async fn read_file(path: impl AsRef<Path>) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    Ok(std::fs::read(path)?)
}

pub async fn create_empty_file(
    path: impl AsRef<Path>,
) -> Result<File, Box<dyn Error + Send + Sync>> {
    return Ok(File::create(&path).await?);
}

pub async fn append_file(
    file: &mut File,
    bytes: &[u8],
) -> Result<(), Box<dyn Error + Send + Sync>> {
    file.write_all(bytes).await?;
    Ok(())
}

pub async fn write_bytes_to_file(
    path: impl AsRef<Path>,
    bytes: &[u8],
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut tmp_file = File::create(&path).await?;
    tmp_file.write_all(bytes).await?;
    Ok(())
}

pub async fn create_fs_structure() -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(DATA_DIR).await?;
    fs::create_dir_all(TMP_DIR).await?;
    fs::create_dir_all(format!("{}/saves", DATA_DIR)).await?;
    Ok(())
}
