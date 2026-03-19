use crate::DATABASE;
use crate::configuration::MAX_SAVE_PER_GAME_INFO;
use crate::const_var::{ROOT_API_PATH, SAVE_DIR, TMP_DIR};
use crate::datatype_endpoint::{FileHash, SaveReference, UploadedSave, UploadedSaveResponse};
use crate::file_system::{append_file, create_empty_file, read_file, write_bytes_to_file};
use axum::body::Body;
use axum::extract::Multipart;
use axum::response::{IntoResponse, Response};
use axum::{Json, extract::Path, http::StatusCode};
use const_format::concatcp;
use std::error::Error;
use std::path::PathBuf;
use tokio::fs::{self, File};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

async fn cleanup_old_saves(path_id: i32) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(mut saves_ref) = DATABASE.get_reference_to_save_by_path_id(path_id)? {
        saves_ref.sort_by_key(|save_ref| save_ref.time);
        let maybe_max_saves = match MAX_SAVE_PER_GAME_INFO.get_value_in_db() {
            Ok(Some(cfg)) => match cfg.value.parse::<usize>() {
                Ok(value) => Some(value),
                Err(err) => {
                    tracing::error!(
                        "Failed to read {} config: {}",
                        MAX_SAVE_PER_GAME_INFO.id,
                        err
                    );
                    None
                }
            },
            Ok(None) => {
                tracing::error!("No {} in the database", MAX_SAVE_PER_GAME_INFO.id);
                None
            }
            Err(err) => {
                tracing::error!(
                    "Failed to read {} config: {}",
                    MAX_SAVE_PER_GAME_INFO.id,
                    err
                );
                None
            }
        };
        if let Some(max_saves) = maybe_max_saves {
            while saves_ref.len() > max_saves {
                let old = saves_ref.remove(0);
                let old_path = format!("{}/{}.sav", SAVE_DIR, old.uuid);
                let _ = std::fs::remove_file(&old_path);
            }
        };
    } else {
        tracing::error!(
            "Failed to fetch database for save reference for path id: {}",
            path_id
        );
        return Err(format!(
            "Failed to fetch database for save reference for path id: {}",
            path_id
        )
        .into());
    }

    Ok(())
}

#[utoipa::path(
    get,
    path = concatcp!(ROOT_API_PATH, "/paths/{Id}/saves"),
    params(
        ("Id" = String, Path, description = "Id of the path")
    ),
    responses(
        (status = StatusCode::OK, description = "game saves returned", body = [SaveReference]),
    )
)]
pub async fn get_game_saves_reference_by_path_id(
    Path((path_id,)): Path<(i32,)>,
) -> Result<Json<Vec<SaveReference>>, StatusCode> {
    match DATABASE.get_reference_to_save_by_path_id(path_id) {
        Ok(Some(data)) => Ok(Json(data)),
        Ok(None) => Ok(Json(Vec::new())),
        Err(e) => {
            tracing::error!("Error getting game saves reference: {}", e);
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
        description = "chunked save upload"
    ),
    responses(
        (status = StatusCode::CREATED, description = "game save created", body = UploadedSaveResponse),
        (status = StatusCode::OK, description = "game save chunk accepted", body = UploadedSaveResponse)
    )
)]
pub async fn post_game_save_by_path_id(
    Path((path_id,)): Path<(i32,)>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<UploadedSaveResponse>), StatusCode> {
    let mut uuid = Uuid::new_v4();
    let mut chunk_number = 0;
    let mut total_chunks = 0;
    let mut chunk_data: Vec<u8> = Vec::new();
    let mut file_hash: Vec<FileHash> = Vec::new();

    async {
        while let Some(mut field) = multipart.next_field().await? {
            let field_name = field.name().unwrap_or_default();
            match field_name {
                "uuid" => {
                    uuid = Uuid::try_parse(field.text().await?.as_str())?;
                }
                "chunkNumber" => {
                    chunk_number = field.text().await?.parse()?;
                }
                "totalChunks" => {
                    total_chunks = field.text().await?.parse()?;
                }
                "chunk" => {
                    while let Some(chunk) = field.chunk().await? {
                        chunk_data.extend_from_slice(&chunk);
                    }
                }
                "file_hash" => {
                    let bytes = field.bytes().await?;
                    let json_str = String::from_utf8(bytes.to_vec())?;
                    file_hash = serde_json::from_str(&json_str)?;
                }
                _ => {}
            }
        }
        Ok::<(), Box<dyn Error + Send + Sync>>(())
    }
    .await
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    if chunk_data.is_empty() || total_chunks == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let temp_dir = format!("{}/{}", TMP_DIR, uuid);
    fs::create_dir_all(&temp_dir)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let chunk_path = format!("{}/chunk_{}", temp_dir, chunk_number);

    write_bytes_to_file(&chunk_path, &chunk_data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut count = 0;
    let mut dir = match fs::read_dir(&temp_dir).await {
        Ok(dir) => dir,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    while let Some(_entry) = dir
        .next_entry()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        count += 1;
    }
    let is_complete = count == total_chunks;

    if is_complete {
        let final_path = format!("{}/{}.sav", SAVE_DIR, uuid);

        let mut file = create_empty_file(&final_path)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        for i in 0..total_chunks {
            let part_path = format!("{}/chunk_{}", temp_dir, i);
            let data = read_file(part_path)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            append_file(&mut file, &data)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        fs::remove_dir_all(&temp_dir)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        DATABASE
            .add_reference_to_save(uuid, path_id, file_hash)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        cleanup_old_saves(path_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok((
            StatusCode::CREATED,
            Json(UploadedSaveResponse {
                uuid: uuid.to_string(),
            }),
        ))
    } else {
        Ok((
            StatusCode::OK,
            Json(UploadedSaveResponse {
                uuid: uuid.to_string(),
            }),
        ))
    }
}

#[utoipa::path(
    get,
    path = concatcp!(ROOT_API_PATH, "/saves/{uuid}"),
    params(
        ("uuid" = String, Path, description = "UUID of the game save")
    ),
    responses(
        (status = StatusCode::OK, description = "game save file returned", content_type = "application/octet-stream"),
        (status = StatusCode::NOT_FOUND, description = "save not found")
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
