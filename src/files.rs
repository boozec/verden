use crate::config::{SAVE_FILE_BASE_PATH, UPLOADS_ENDPOINT};
use crate::errors::AppError;
use axum::{
    extract::{Multipart, Path},
    http::header::HeaderMap,
};
use std::fs::read;

use rand::random;

/// Upload a file. Returns an `AppError` or the path of the uploaded file
pub async fn upload(
    mut multipart: Multipart,
    allowed_extensions: Vec<&str>,
) -> Result<String, AppError> {
    let mut uploaded_file = String::new();

    if let Some(file) = multipart.next_field().await.unwrap() {
        let content_type = file.content_type().unwrap().to_string();

        let index = content_type.find('/').unwrap_or(usize::max_value());
        let mut ext_name = "xxx";
        if index != usize::max_value() {
            ext_name = &content_type[index + 1..];
        }

        if allowed_extensions
            .iter()
            .any(|&x| x.to_lowercase() == ext_name)
        {
            let rnd = (random::<f32>() * 1000000000 as f32) as i32;

            let save_filename = format!("{}/{}.{}", SAVE_FILE_BASE_PATH, rnd, ext_name);
            uploaded_file = format!("{}/{}.{}", UPLOADS_ENDPOINT, rnd, ext_name);

            let data = file.bytes().await.unwrap();

            tokio::fs::write(&save_filename, &data)
                .await
                .map_err(|err| err.to_string())?;
        }
    }

    if !uploaded_file.is_empty() {
        return Ok(uploaded_file);
    }

    Err(AppError::BadRequest(
        "File extension not supported".to_string(),
    ))
}

/// Axum endpoint which shows uploaded file
pub async fn show_uploads(Path(id): Path<String>) -> (HeaderMap, Vec<u8>) {
    // let index = id.find(".").map(|i| i).unwrap_or(usize::max_value());

    // let mut ext_name = "xxx";
    // if index != usize::max_value() {
    //     ext_name = &id[index + 1..];
    // }
    let headers = HeaderMap::new();
    let file_name = format!("{}/{}", SAVE_FILE_BASE_PATH, id);
    (headers, read(&file_name).unwrap())
}
