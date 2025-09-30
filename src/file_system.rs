use axum::extract::Multipart;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

use crate::const_var::{DATA_DIR, TMP_DIR};

pub async fn write_multipart_to_data_file(
    tmp_path: &str,
    file_path: &str,
    multipart: Multipart,
) -> Result<(), Box<dyn std::error::Error>> {
    match write_multipart_to_tmp_file(tmp_path, multipart).await {
        Ok(_) => {
            std::fs::rename(tmp_path, file_path)?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub async fn write_multipart_to_tmp_file(
    tmp_path: &str,
    mut multipart: Multipart,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut tmp_file = File::create(&tmp_path).await?;
    while let Some(field) = multipart.next_field().await? {
        let data = field.bytes().await?;
        tmp_file.write_all(&data).await?;
    }
    Ok(())
}

pub async fn create_fs_structure() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(DATA_DIR).await?;
    fs::create_dir_all(TMP_DIR).await?;
    fs::create_dir_all(format!("{}/saves", DATA_DIR)).await?;
    Ok(())
}
