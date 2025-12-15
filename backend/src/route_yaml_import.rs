use axum::{extract::Multipart, http::StatusCode};
use const_format::concatcp;
use tokio::fs;

use crate::{
    const_var::{ROOT_API_PATH, TMP_DIR},
    datatype_endpoint::UploadedFileYaml,
    file_system::write_bytes_to_tmp_file,
    ludusavi::yaml_import,
};

#[utoipa::path(
    post,
    path = concatcp!(ROOT_API_PATH, "/yaml/ludusavi"),
    request_body(
        content = UploadedFileYaml,
        content_type = "multipart/form-data",
        description = "Ludusavi manifest"
    ),
    responses(
        (status = 200, description = "Ludusavi manifest imported", body = String),
    )
)]
pub async fn post_ludusavi_yaml(mut multipart: Multipart) -> StatusCode {
    let tmp_path = format!("{}/{}", TMP_DIR, "ludusavi.yaml");
    let mut err: Option<String> = None;

    let mut file_bytes: Vec<u8> = Vec::new();
    while let Some(field) = multipart.next_field().await.ok().flatten() {
        if let Ok(data) = field.bytes().await {
            file_bytes.extend_from_slice(&data);
        }
    }

    if let Err(e) = write_bytes_to_tmp_file(&tmp_path, &file_bytes).await {
        err = Some(format!("write file failed: {}", e));
    } else if let Err(e) = yaml_import(&tmp_path).await {
        err = Some(format!("yaml import failed: {}", e));
    }

    // Whatever happened, clean up
    let _ = fs::remove_file(&tmp_path).await;
    if let Some(e) = err {
        eprintln!("Error importing ludusavi manifest: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    } else {
        StatusCode::OK
    }
}
