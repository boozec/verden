use crate::{config::CONFIG, errors::AppError};
use axum::{
    extract::{Multipart, Path},
    http::header::{HeaderMap, HeaderName, HeaderValue},
};
use std::{fs, path};

use rand::random;

/// Upload a file. Returns an `AppError` or the path of the uploaded file.
/// If `filename` param has a value choose it as filename
pub async fn upload(
    mut multipart: Multipart,
    allowed_extensions: Vec<&str>,
    filename: Option<String>,
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
            let mut name = match filename {
                Some(name) => name,
                None => (random::<f32>() * 1000000000 as f32).to_string(),
            };

            loop {
                let save_filename = format!("{}/{}.{}", CONFIG.save_file_base_path, name, ext_name);

                if path::Path::exists(&path::Path::new(&save_filename)) {
                    name = (random::<f32>() * 1000000000 as f32).to_string();
                    continue;
                }

                uploaded_file = format!("{}/{}.{}", CONFIG.uploads_endpoint, name, ext_name);

                let data = file.bytes().await.unwrap();

                tokio::fs::write(&save_filename, &data)
                    .await
                    .map_err(|err| err.to_string())?;
                break;
            }
        }
    }

    if !uploaded_file.is_empty() {
        return Ok(uploaded_file);
    }

    Err(AppError::BadRequest(
        "File extension not supported".to_string(),
    ))
}

/// Delete a file from the filesystem
pub fn delete_upload(filename: &str) -> Result<(), AppError> {
    let last_slash_index = filename.rfind('/').unwrap();
    let path = format!(
        "{}/{}",
        CONFIG.save_file_base_path,
        &filename[last_slash_index + 1..]
    );

    fs::remove_file(path)?;

    Ok(())
}

/// Axum endpoint which shows uploaded file
pub async fn show_uploads(Path(id): Path<String>) -> (HeaderMap, Vec<u8>) {
    let index = id.find('.').unwrap_or(usize::max_value());

    let mut ext_name = "xxx";
    if index != usize::max_value() {
        ext_name = &id[index + 1..];
    }
    let mut headers = HeaderMap::new();

    if vec!["jpg", "jpeg", "png", "gif", "webp"]
        .iter()
        .any(|&x| x == ext_name)
    {
        let content_type = format!("image/{}", ext_name);
        headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_str(&content_type).unwrap(),
        );
    }
    let file_name = format!("{}/{}", CONFIG.save_file_base_path, id);
    (headers, fs::read(&file_name).unwrap())
}
