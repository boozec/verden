use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// All errors raised by the web app
pub enum AppError {
    /// Database error
    Database,
    /// Generic bad request. It is handled with a message value
    BadRequest(String),
    /// Not found error
    NotFound,
    /// Raised when a token is not good created
    TokenCreation,
    /// Raised when a passed token is not valid
    InvalidToken,
}

/// Use `AppError` as response for an endpoint
impl IntoResponse for AppError {
    /// Matches `AppError` into a tuple of status and error message.
    /// The response will be a JSON in the format of:
    /// ```json
    /// { "error": "<message>" }
    /// ```
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error with database connection".to_string(),
            ),
            AppError::BadRequest(value) => (StatusCode::BAD_REQUEST, value),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Element not found".to_string()),
            AppError::TokenCreation => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Token creation error".to_string(),
            ),
            AppError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token".to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

/// Transforms a `sqlx::Error` into a `AppError::Databse` error
impl From<sqlx::Error> for AppError {
    fn from(_error: sqlx::Error) -> AppError {
        AppError::Database
    }
}
