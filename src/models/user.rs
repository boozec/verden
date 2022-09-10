use crate::config::PAGE_LIMIT;
use crate::db::get_client;
use crate::errors::AppError;

use serde::{Deserialize, Serialize};
use validator::Validate;

/// User model
#[derive(Deserialize, Serialize, Validate)]
pub struct User {
    id: i32,
    #[validate(length(min = 4, message = "Can not be empty"))]
    email: String,
    #[validate(length(min = 2, message = "Can not be empty"))]
    username: String,
    #[validate(length(min = 8, message = "Must be min 8 chars length"))]
    password: String,
    is_staff: Option<bool>,
}

/// Response used to print a user (or a users list)
#[derive(Deserialize, Serialize)]
pub struct UserList {
    // It is public because it used by `Claims` creation
    pub id: i32,
    email: String,
    username: String,
    is_staff: Option<bool>,
}

/// Payload used for user creation
#[derive(Deserialize)]
pub struct UserCreate {
    pub email: String,
    pub username: String,
    pub password: String,
}

impl User {
    /// By default an user has id = 0. It is not created yet
    pub fn new(email: String, username: String, password: String) -> Self {
        Self {
            id: 0,
            email,
            username,
            password,
            is_staff: Some(false),
        }
    }

    /// Create a new user from the model using a SHA256 crypted password
    pub async fn create(user: User) -> Result<UserList, AppError> {
        let pool = unsafe { get_client() };

        user.validate()
            .map_err(|error| AppError::BadRequest(error.to_string()))?;

        let crypted_password = sha256::digest(user.password);

        let rec = sqlx::query_as!(
            UserList,
            r#"
                INSERT INTO users (email, username, password)
                VALUES ( $1, $2, $3)
                RETURNING id, email, username, is_staff
            "#,
            user.email,
            user.username,
            crypted_password
        )
        .fetch_one(pool)
        .await?;

        Ok(rec)
    }

    /// Find a user using the model. It used for login
    pub async fn find(user: User) -> Result<UserList, AppError> {
        let pool = unsafe { get_client() };

        let crypted_password = sha256::digest(user.password);

        let rec = sqlx::query_as!(
            UserList,
            r#"
                SELECT id, email, username, is_staff FROM "users"
                WHERE username = $1 AND password = $2
            "#,
            user.username,
            crypted_password
        )
        .fetch_one(pool)
        .await?;

        Ok(rec)
    }

    /// Returns the user with id = `user_id`
    pub async fn find_by_id(user_id: i32) -> Result<UserList, AppError> {
        let pool = unsafe { get_client() };

        let rec = sqlx::query_as!(
            UserList,
            r#"
                SELECT id, email, username, is_staff FROM "users"
                WHERE id = $1
            "#,
            user_id
        )
        .fetch_one(pool)
        .await?;

        Ok(rec)
    }

    /// List all users
    pub async fn list(page: i64) -> Result<Vec<UserList>, AppError> {
        let pool = unsafe { get_client() };
        let rows = sqlx::query_as!(
            UserList,
            r#"SELECT id, email, username, is_staff FROM users
            LIMIT $1 OFFSET $2
            "#,
            PAGE_LIMIT,
            PAGE_LIMIT * page
        )
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    /// Return the number of users.
    pub async fn count() -> Result<i64, AppError> {
        let pool = unsafe { get_client() };
        let row = sqlx::query!(r#"SELECT COUNT(id) as count FROM users"#)
            .fetch_one(pool)
            .await?;

        Ok(row.count.unwrap())
    }
}
