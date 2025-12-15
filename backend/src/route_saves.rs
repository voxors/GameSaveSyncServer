use crate::DATABASE;
use crate::const_var::{ROOT_API_PATH, SAVE_DIR, TMP_DIR};
use crate::datatype_endpoint::{SaveReference, UploadedSave};
use crate::file_system::write_bytes_to_data_file;
use axum::body::Body;
use axum::extract::Multipart;
use axum::response::{IntoResponse, Response};
use axum::{Json, extract::Path, http::StatusCode};
use const_format::concatcp;
use std::fs;
use std::path::PathBuf;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = concatcp!(ROOT_API_PATH, "/paths/{Id}/saves"),
    params(
        ("Id" = String, Path, description = "Id of the path")
    ),
    responses(
        (status = 200, description = "game saves returned", body = [SaveReference]),
    )
)]
pub async fn get_game_saves_reference_by_path_id(
    Path((path_id,)): Path<(i32,)>,
) -> Result<Json<Vec<SaveReference>>, StatusCode> {
    match DATABASE.get_reference_to_save_by_path_id(path_id) {
        Ok(Some(data)) => Ok(Json(data)),
        Ok(None) => Ok(Json(Vec::new())),
        Err(e) => {
            eprintln!("Error getting game saves reference: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    post,
    path = concatcp!(ROOT_API_PATH, "/paths/{Id}/saves/upload"),
    params(
        ("Id" = String, Path, description = "Id of the path")
    ),
    request_body(
        content = UploadedSave,
        content_type = "multipart/form-data",
        description = "save to upload"
    ),
    responses(
        (status = 201, description = "game save created", body = String),
        (status = 404, description = "path not found")
    )
)]
pub async fn post_game_save_by_path_id(
    Path((path_id,)): Path<(i32,)>,
    mut multipart: Multipart,
) -> Result<(StatusCode, String), StatusCode> {
    let uuid = Uuid::new_v4();
    let tmp_path = format!("{}/{}.sav", TMP_DIR, uuid);
    let save_path = format!("{}/{}.sav", SAVE_DIR, uuid);

    let result: Result<(), Box<dyn std::error::Error + Send + Sync>> = async {
        let mut file_hash: Vec<crate::datatype_endpoint::FileHash> = Vec::new();
        let mut file_bytes: Vec<u8> = Vec::new();

        while let Some(field) = multipart.next_field().await? {
            match field.name() {
                Some("file_hash") => {
                    let bytes = field.bytes().await?;
                    let json_str = String::from_utf8(bytes.to_vec())?;
                    file_hash = serde_json::from_str(&json_str)?;
                }
                _ => {
                    let data = field.bytes().await?;
                    file_bytes.extend_from_slice(&data);
                }
            }
        }

        write_bytes_to_data_file(&tmp_path, &save_path, &file_bytes).await?;

        DATABASE.add_reference_to_save(uuid, path_id, file_hash)?;

        if let Some(mut saves_ref) = DATABASE.get_reference_to_save_by_path_id(path_id)? {
            saves_ref.sort_by_key(|save_ref| save_ref.time);
            while saves_ref.len() > 5 {
                let old = saves_ref.remove(0);
                let old_path = format!("{}/{}.sav", SAVE_DIR, old.uuid);
                let _ = std::fs::remove_file(&old_path);
            }
        }

        Ok(())
    }
    .await;

    if let Err(e) = result {
        eprintln!("Error uploading game save: {}", e);
        //Try to clean up
        let _ = fs::remove_file(&tmp_path);
        let _ = fs::remove_file(&save_path);
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    } else {
        Ok((StatusCode::CREATED, uuid.to_string()))
    }
}

#[utoipa::path(
    get,
    path = concatcp!(ROOT_API_PATH, "/saves/{uuid}"),
    params(
        ("uuid" = String, Path, description = "UUID of the game save")
    ),
    responses(
        (status = 200, description = "game save file returned", content_type = "application/octet-stream"),
        (status = 404, description = "save not found")
    )
)]
pub async fn get_game_save_by_uuid(Path((uuid,)): Path<(String,)>) -> impl IntoResponse {
    let file_path = format!("{}/{}.sav", SAVE_DIR, uuid);
    let path_buf = PathBuf::from(&file_path);

    match File::open(&path_buf).await {
        Ok(file) => {
            // Stream the file contents
            let stream = ReaderStream::new(file);
            let body = Body::from_stream(stream);

            // Detect the MIME type (defaults to application/octet-stream)
            let mime = mime_guess::from_path(&path_buf).first_or_octet_stream();

            // Return the file as an attachment
            Response::builder()
                .header("Content-Type", mime.to_string())
                .header(
                    "Content-Disposition",
                    format!("attachment; filename=\"{}.sav\"", uuid),
                )
                .body(body)
                .unwrap_or_else(|_| {
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::empty())
                        .unwrap()
                })
        }
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
    }
}
