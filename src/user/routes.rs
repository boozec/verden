use crate::{
    auth::models::Claims,
    errors::AppError,
    files::{delete_upload, upload},
    pagination::{ModelPagination, Pagination, UserPagination},
    user::models::{User, UserEdit, UserList},
};
use axum::{
    extract::{ContentLengthLimit, Multipart, Path, Query},
    routing::{delete, get, put},
    Json, Router,
};

/// Create routes for `/v1/users/` namespace
pub fn create_route() -> Router {
    Router::new()
        .route("/", get(list_users))
        .route("/me", get(get_me))
        .route("/me/avatar", put(edit_my_avatar).delete(delete_my_avatar))
        .route("/:id", get(get_user).put(edit_user))
        .route("/:id/avatar", delete(delete_avatar))
        .route("/:id/models", get(get_user_models))
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
    ContentLengthLimit(multipart): ContentLengthLimit<Multipart, { 1024 * 1024 * 5 }>,
) -> Result<Json<UserList>, AppError> {
    let mut user = match User::find_by_id(claims.user_id).await {
        Ok(user) => user,
        Err(_) => {
            return Err(AppError::NotFound("User not found".to_string()));
        }
    };

    if user.avatar.is_some() {
        let avatar_url = user.avatar.as_ref().unwrap();
        delete_upload(avatar_url)?;
    }

    match upload(
        multipart,
        vec!["jpg", "jpeg", "png", "webp"],
        Some(format!("avatar-{}", user.id)),
    )
    .await
    {
        Ok(saved_file) => {
            user.edit_avatar(Some(saved_file)).await?;

            Ok(Json(user))
        }
        Err(e) => Err(e),
    }
}

/// A staffer can delete an user `id`'s avatar
async fn delete_avatar(
    Path(user_id): Path<i32>,
    claims: Claims,
) -> Result<Json<UserList>, AppError> {
    let mut user = match User::find_by_id(user_id).await {
        Ok(user) => user,
        Err(_) => {
            return Err(AppError::NotFound("User not found".to_string()));
        }
    };

    // If the user of the access token is different than the user they want to edit, checks if the
    // first user is an admin
    if claims.user_id != user.id {
        match User::find_by_id(claims.user_id).await {
            Ok(user) => {
                if !(user.is_staff.unwrap()) {
                    return Err(AppError::Unauthorized);
                }
            }
            Err(_) => {
                return Err(AppError::NotFound("User not found".to_string()));
            }
        };
    }

    if user.avatar.is_some() {
        let avatar_url = user.avatar.as_ref().unwrap();
        delete_upload(avatar_url)?;
    }

    user.edit_avatar(None).await?;

    Ok(Json(user))
}

/// Delete the avatar of the user linked to the claims
async fn delete_my_avatar(claims: Claims) -> Result<Json<UserList>, AppError> {
    let mut user = match User::find_by_id(claims.user_id).await {
        Ok(user) => user,
        Err(_) => {
            return Err(AppError::NotFound("User not found".to_string()));
        }
    };

    if user.avatar.is_some() {
        let avatar_url = user.avatar.as_ref().unwrap();
        delete_upload(avatar_url)?;
    }

    user.edit_avatar(None).await?;

    Ok(Json(user))
}

/// Get an user with id = `user_id`
async fn get_user(Path(user_id): Path<i32>) -> Result<Json<UserList>, AppError> {
    match User::find_by_id(user_id).await {
        Ok(user) => Ok(Json(user)),
        Err(_) => Err(AppError::NotFound("User not found".to_string())),
    }
}

/// Edit an user with id = `user_id`. Only staffers and owner of that account can perform this
/// action.
/// Only staffers can update the user `is_staff` value
async fn edit_user(
    Path(user_id): Path<i32>,
    Json(mut payload): Json<UserEdit>,
    claims: Claims,
) -> Result<Json<UserList>, AppError> {
    let mut user = match User::find_by_id(user_id).await {
        Ok(user) => user,
        Err(_) => {
            return Err(AppError::NotFound("User not found".to_string()));
        }
    };

    let claimed = match User::find_by_id(claims.user_id).await {
        Ok(user) => user,
        Err(_) => {
            return Err(AppError::NotFound("User not found".to_string()));
        }
    };

    if user.id != claimed.id {
        if !(claimed.is_staff.unwrap()) {
            return Err(AppError::Unauthorized);
        }
    }

    if !claimed.is_staff.unwrap() && user.is_staff != payload.is_staff {
        payload.is_staff = user.is_staff;
    }

    if user.email != payload.email && User::email_has_taken(&payload.email).await? {
        return Err(AppError::BadRequest(
            "An user with this email already exists".to_string(),
        ));
    }

    if user.username != payload.username && User::username_has_taken(&payload.username).await? {
        return Err(AppError::BadRequest(
            "An user with this username already exists".to_string(),
        ));
    }

    user.edit(payload).await?;

    Ok(Json(user))
}

/// Get user models list
async fn get_user_models(
    Path(user_id): Path<i32>,
    pagination: Query<Pagination>,
) -> Result<Json<ModelPagination>, AppError> {
    let user = match User::find_by_id(user_id).await {
        Ok(user) => user,
        Err(_) => {
            return Err(AppError::NotFound("User not found".to_string()));
        }
    };

    let page = pagination.0.page.unwrap_or_default();
    let results = user.get_models(page).await?;
    let count = user.count_models().await?;

    Ok(Json(ModelPagination { count, results }))
}
