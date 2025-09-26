use axum::extract::Multipart;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

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
