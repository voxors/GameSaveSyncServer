use axum::extract::Multipart;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

use crate::constvar::{DATA_DIR, TMP_DIR};

pub async fn write_file_to_data(
    tmp_path: &str,
    file_path: &str,
    mut multipart: Multipart,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tmp_file = File::create(&tmp_path).await?;
    while let Some(field) = multipart.next_field().await? {
        let data = field.bytes().await?;
        tmp_file.write_all(&data).await?;
    }
    std::fs::rename(tmp_path, file_path)?;
    Ok(())
}

pub async fn create_fs_structure() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(DATA_DIR).await?;
    fs::create_dir_all(TMP_DIR).await?;
    fs::create_dir_all(format!("{}/saves", DATA_DIR)).await?;
    Ok(())
}
