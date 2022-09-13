use crate::errors::AppError;
use crate::models::{
    auth::Claims,
    user::{User, UserList},
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
        .route("/", get(list_users))
        .route("/me", get(get_me))
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

/// Get info about me
async fn get_me(claims: Claims) -> Result<Json<UserList>, AppError> {
    match User::find_by_id(claims.user_id).await {
        Ok(user) => Ok(Json(user)),
        Err(_) => Err(AppError::NotFound("User not found".to_string())),
    }
}

/// Get an user with id = `user_id`. Checks Authorization token
async fn get_user(Path(user_id): Path<i32>, _: Claims) -> Result<Json<UserList>, AppError> {
    match User::find_by_id(user_id).await {
        Ok(user) => Ok(Json(user)),
        Err(_) => Err(AppError::NotFound("User not found".to_string())),
    }
}
