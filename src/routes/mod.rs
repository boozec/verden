pub mod auth;
pub mod model;
pub mod user;

use crate::errors::AppError;
use axum::response::IntoResponse;

pub async fn page_404() -> impl IntoResponse {
    AppError::NotFound("Route not found".to_string())
}
