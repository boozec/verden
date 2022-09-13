use crate::errors::AppError;
use crate::files::{delete_upload, upload};
use crate::models::{
    auth::Claims,
    user::{User, UserList},
};
use crate::pagination::Pagination;
use axum::{
    extract::{ContentLengthLimit, Multipart, Path, Query},
    routing::{get, put},
    Json, Router,
};
use serde::Serialize;

/// Create routes for `/v1/users/` namespace
pub fn create_route() -> Router {
    Router::new()
        .route("/", get(list_users))
        .route("/me", get(get_me))
        .route("/me/avatar", put(edit_my_avatar))
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

/// Edit the avatar of the user linked to the claims
async fn edit_my_avatar(
    claims: Claims,
    ContentLengthLimit(multipart): ContentLengthLimit<Multipart, { 1024 * 1024 }>,
) -> Result<Json<UserList>, AppError> {
    let mut user = match User::find_by_id(claims.user_id).await {
        Ok(user) => user,
        Err(_) => {
            return Err(AppError::NotFound("User not found".to_string()));
        }
    };

    if user.avatar.is_some() {
        let avatar_url = user.avatar.as_ref().unwrap();
        delete_upload(&avatar_url)?;
    }

    match upload(
        multipart,
        vec!["jpg", "jpeg", "png", "webp"],
        Some(format!("avatar-{}", user.id)),
    )
    .await
    {
        Ok(saved_file) => {
            user.edit_avatar(saved_file).await?;

            Ok(Json(user))
        }
        Err(e) => Err(e),
    }
}

/// Get an user with id = `user_id`. Checks Authorization token
async fn get_user(Path(user_id): Path<i32>, _: Claims) -> Result<Json<UserList>, AppError> {
    match User::find_by_id(user_id).await {
        Ok(user) => Ok(Json(user)),
        Err(_) => Err(AppError::NotFound("User not found".to_string())),
    }
}
