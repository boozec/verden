use crate::config::SAVE_FILE_BASE_PATH;
use crate::errors::AppError;
use axum::extract::Multipart;

use rand::random;

/// Upload a file. Returns an `AppError` or the path of the uploaded file
pub async fn upload(
    mut multipart: Multipart,
    allowed_extensions: Vec<&str>,
) -> Result<String, AppError> {
    let mut save_filename = String::new();

    if let Some(file) = multipart.next_field().await.unwrap() {
        let content_type = file.content_type().unwrap().to_string();

        let index = content_type
            .find("/")
            .map(|i| i)
            .unwrap_or(usize::max_value());
        let mut ext_name = "xxx";
        if index != usize::max_value() {
            ext_name = &content_type[index + 1..];
        }

        if allowed_extensions
            .iter()
            .position(|&x| x.to_lowercase() == ext_name)
            .is_some()
        {
            let rnd = (random::<f32>() * 1000000000 as f32) as i32;

            save_filename = format!("{}/{}.{}", SAVE_FILE_BASE_PATH, rnd, ext_name);

            let data = file.bytes().await.unwrap();

            tokio::fs::write(&save_filename, &data)
                .await
                .map_err(|err| err.to_string())?;
        }
    }

    Ok(save_filename)
}
