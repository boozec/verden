use crate::errors::AppError;
use crate::models::{
    auth::{AuthBody, Claims},
    user::{User, UserCreate},
};
use axum::{routing::post, Json, Router};

/// Create routes for `/v1/auth/` namespace
pub fn create_route() -> Router {
    Router::new().route("/login", post(make_login))
}

/// Make login. Check if a user with the email and password passed in request body exists into the
/// database
async fn make_login(Json(payload): Json<UserCreate>) -> Result<Json<AuthBody>, AppError> {
    let user = User::new(payload.email, payload.password);
    match User::find(user).await {
        Ok(user) => {
            let claims = Claims::new(user.id);
            let token = claims.get_token()?;
            Ok(Json(AuthBody::new(token)))
        }
        Err(_) => Err(AppError::NotFound),
    }
}
