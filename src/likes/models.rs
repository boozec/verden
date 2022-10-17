use crate::{db::get_client, errors::AppError};
use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::Row;

/// Likes model
#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Like {
    id: i32,
    user_id: i32,
    model_id: i32,
    created: NaiveDateTime,
}

impl Like {
    /// Create a new like
    pub fn new(user_id: i32, model_id: i32) -> Self {
        let now = Local::now().naive_utc();
        Self {
            id: 0,
            user_id,
            model_id,
            created: now,
        }
    }

    /// Returns `true` if an user has already assigned a like to a model
    pub async fn exists(&self) -> Result<bool, AppError> {
        let pool = unsafe { get_client() };
        let cursor = sqlx::query(
            r#"
                SELECT COUNT(id) as count FROM likes WHERE user_id = $1 AND model_id = $2
            "#,
        )
        .bind(self.user_id)
        .bind(self.model_id)
        .fetch_one(pool)
        .await?;

        let count: i64 = cursor.try_get(0).unwrap();

        Ok(count > 0)
    }

    /// Save new like into db
    pub async fn save(&self) -> Result<Like, AppError> {
        let pool = unsafe { get_client() };

        if self.exists().await? {
            return Err(AppError::BadRequest(
                "This user already likes this model".to_string(),
            ));
        }

        let rec: Like = sqlx::query_as(
            r#"
            INSERT INTO likes (user_id, model_id, created)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(self.user_id)
        .bind(self.model_id)
        .bind(self.created)
        .fetch_one(pool)
        .await?;

        Ok(rec)
    }

    /// Remove a like
    pub async fn remove(&self) -> Result<(), AppError> {
        let pool = unsafe { get_client() };

        if !self.exists().await? {
            return Err(AppError::NotFound("Like not found".to_string()));
        }

        sqlx::query(
            r#"
            DELETE FROM likes WHERE user_id = $1 AND model_id = $2
            "#,
        )
        .bind(self.user_id)
        .bind(self.model_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
