use crate::errors::AppError;
use crate::models::{
    auth::Claims,
    user::{User, UserCreate, UserList},
};
use crate::pagination::Pagination;
use axum::{
    extract::{Path, Query},
    routing::get,
    Json, Router,
};
use serde::Serialize;

/// Create routes for `/v1/users/` namespace
pub fn create_route() -> Router {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/:id", get(get_user))
}

#[derive(Serialize)]
struct UserPagination {
    count: i64,
    results: Vec<UserList>,
}

/// List users. Checks Authorization token
async fn list_users(
    _: Claims,
    pagination: Query<Pagination>,
) -> Result<Json<UserPagination>, AppError> {
    let page = pagination.0.page.unwrap_or_default();
    let results = User::list(page).await?;
    let count = User::count().await?;

    Ok(Json(UserPagination { count, results }))
}

/// Create an user. Checks Authorization token
async fn create_user(
    Json(payload): Json<UserCreate>,
    _: Claims,
) -> Result<Json<UserList>, AppError> {
    let user = User::new(payload.email, payload.username, payload.password);
    let user_new = User::create(user).await?;

    Ok(Json(user_new))
}

/// Get an user with id = `user_id`. Checks Authorization token
async fn get_user(Path(user_id): Path<i32>, _: Claims) -> Result<Json<UserList>, AppError> {
    match User::find_by_id(user_id).await {
        Ok(user) => Ok(Json(user)),
        Err(_) => Err(AppError::NotFound("User not found".to_string())),
    }
}
