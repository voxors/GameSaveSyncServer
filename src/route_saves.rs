use crate::DATABASE;
use crate::datatype_endpoint::{SaveReference, UploadedFile};
use crate::file_system::{SAVE_DIR, TMP_DIR, write_file_to_data};
use axum::extract::Multipart;
use axum::{Json, extract::Path, http::StatusCode};
use std::fs;
use uuid::Uuid;

#[utoipa::path(
    get,
    path ="/games/{Id}/paths/{Id}/saves",
    params(
        ("Id" = String, Path, description = "Id of the game"),
        ("Id" = String, Path, description = "Id of the path")
    ),
    responses(
        (status = 200, description = "game saves returned", body = [Vec<SaveReference>]),
        (status = 400, description = "invalid operating system"),
        (status = 404, description = "game not found")
    )
)]
pub async fn get_game_saves_reference_by_path_id(
    Path((_game_id, path_id)): Path<(i32, i32)>,
) -> Result<Json<Vec<SaveReference>>, StatusCode> {
    match DATABASE.get_reference_to_save_by_path_id(path_id) {
        Ok(Some(data)) => Ok(Json(data)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error getting game saves reference: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// #[utoipa::path(
//     post,
//     path = "/games/{Id}/paths/{Id}/saves",
//     params(
//         ("Id" = String, Path, description = "Id of the game"),
//         ("Id" = String, Path, description = "Id of the path")
//     ),
//     responses(
//         (status = 201, description = "game save reference returned", body = [Vec<SaveReference>]),
//         (status = 400, description = "invalid operating system"),
//         (status = 404, description = "path not found")
//     )
// )]
// pub async fn post_game_saves_reference_by_path_id(
//     Path((_game_id, path_id)): Path<(i32, i32)>,
// ) -> StatusCode {
//     match DATABASE.add_reference_to_save(Uuid::new_v4(), path_id) {
//         Ok(()) => StatusCode::CREATED,
//         Err(e) => {
//             eprintln!("Error adding game save reference: {}", e);
//             StatusCode::INTERNAL_SERVER_ERROR
//         }
//     }
// }

#[utoipa::path(
    post,
    path = "/games/{Id}/paths/{Id}/saves/upload",
    params(
        ("Id" = String, Path, description = "Id of the game"),
        ("Id" = String, Path, description = "Id of the path")
    ),
    request_body(
        content = UploadedFile,
        content_type = "multipart/form-data",
        description = "save to upload"
    ),
    responses(
        (status = 201, description = "game save created", body = String),
        (status = 404, description = "path not found")
    )
)]
pub async fn post_game_save_by_path_id(
    Path((_game_id, path_id)): Path<(i32, i32)>,
    multipart: Multipart,
) -> StatusCode {
    let uuid = Uuid::new_v4();
    let tmp_path = format!("{}/{}.sav", TMP_DIR, uuid);
    let save_path = format!("{}/{}.sav", SAVE_DIR, uuid);
    let result = async {
        write_file_to_data(&tmp_path, &save_path, multipart).await?;
        DATABASE.add_reference_to_save(uuid, path_id)?;
        Ok::<(), Box<dyn std::error::Error>>(())
    }
    .await;

    if let Err(e) = result {
        eprintln!("Error uploading game save: {}", e);
        //Try to clean up
        let _ = fs::remove_file(&tmp_path);
        let _ = fs::remove_file(&save_path);
        StatusCode::INTERNAL_SERVER_ERROR
    } else {
        StatusCode::CREATED
    }
}
