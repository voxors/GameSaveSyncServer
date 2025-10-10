use axum::{extract::Multipart, http::StatusCode};
use common::datatype_endpoint::UploadedFile;
use const_format::concatcp;
use tokio::fs;

use crate::{
    const_var::{ROOT_API_PATH, TMP_DIR},
    file_system::write_multipart_to_tmp_file,
    ludusavi::yaml_import,
};

#[utoipa::path(
    post,
    path = concatcp!(ROOT_API_PATH, "/yaml/ludusavi"),
    request_body(
        content = UploadedFile,
        content_type = "multipart/form-data",
        description = "Ludusavi manifest"
    ),
    responses(
        (status = 200, description = "Ludusavi manifest imported", body = String),
    )
)]
pub async fn post_ludusavi_yaml(multipart: Multipart) -> StatusCode {
    let tmp_path = format!("{}/{}", TMP_DIR, "ludusavi.yaml");
    let mut result = write_multipart_to_tmp_file(&tmp_path, multipart).await;

    if result.is_ok() {
        result = yaml_import(&tmp_path).await;
    }
    //Whatever happened, clean up
    let _ = fs::remove_file(tmp_path).await;
    if let Err(e) = result {
        eprintln!("Error importing ludusavi manifest: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    } else {
        StatusCode::OK
    }
}
