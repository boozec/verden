use crate::errors::AppError;
use crate::models::{
    auth::Claims,
    user::{User, UserCreate, UserList},
};
use axum::{extract::Path, routing::get, Json, Router};

/// Create routes for `/v1/users/` namespace
pub fn create_route() -> Router {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/:id", get(get_user))
}

/// List users. Checks Authorization token
async fn list_users(_: Claims) -> Result<Json<Vec<UserList>>, AppError> {
    let users = User::list().await?;

    Ok(Json(users))
}

/// Create an user. Checks Authorization token
async fn create_user(
    Json(payload): Json<UserCreate>,
    _: Claims,
) -> Result<Json<UserList>, AppError> {
    let user = User::new(payload.email, payload.password);
    let user_new = User::create(user).await?;

    Ok(Json(user_new))
}

/// Get an user with id = `user_id`. Checks Authorization token
async fn get_user(Path(user_id): Path<i32>, _: Claims) -> Result<Json<UserList>, AppError> {
    match User::find_by_id(user_id).await {
        Ok(user) => Ok(Json(user)),
        Err(_) => Err(AppError::NotFound),
    }
}
