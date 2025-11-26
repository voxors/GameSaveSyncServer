use std::path::Path;

use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

use crate::const_var::{DATA_DIR, TMP_DIR};

pub async fn write_bytes_to_data_file(
    tmp_path: impl AsRef<Path>,
    file_path: impl AsRef<Path>,
    bytes: &[u8],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    write_bytes_to_tmp_file(&tmp_path, bytes).await?;
    std::fs::rename(&tmp_path, file_path)?;
    Ok(())
}

pub async fn write_bytes_to_tmp_file(
    tmp_path: impl AsRef<Path>,
    bytes: &[u8],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut tmp_file = File::create(&tmp_path).await?;
    tmp_file.write_all(bytes).await?;
    Ok(())
}

pub async fn create_fs_structure() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(DATA_DIR).await?;
    fs::create_dir_all(TMP_DIR).await?;
    fs::create_dir_all(format!("{}/saves", DATA_DIR)).await?;
    Ok(())
}
