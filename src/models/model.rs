use crate::config::PAGE_LIMIT;
use crate::db::get_client;

use crate::errors::AppError;
use sqlx::types::JsonValue;

use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Model for models.
#[derive(Deserialize, Serialize, Validate)]
pub struct Model {
    id: i32,
    #[validate(length(min = 2, message = "Can not be empty"))]
    name: String,
    description: Option<String>,
    duration: i32,
    height: i32,
    weight: i32,
    printer: Option<String>,
    material: Option<String>,
    author_id: i32,
    created: NaiveDateTime,
    updated: NaiveDateTime,
}

/// Payload used for model creation
#[derive(Deserialize)]
pub struct ModelCreate {
    pub name: String,
    pub description: Option<String>,
    pub duration: i32,
    pub height: i32,
    pub weight: i32,
    pub printer: Option<String>,
    pub material: Option<String>,
}

#[derive(Serialize)]
pub struct ModelUser {
    id: i32,
    name: String,
    description: Option<String>,
    duration: i32,
    height: i32,
    weight: i32,
    printer: Option<String>,
    material: Option<String>,
    author_id: i32,
    created: NaiveDateTime,
    updated: NaiveDateTime,
    author: Option<JsonValue>,
}

impl Model {
    pub fn new(
        name: String,
        description: Option<String>,
        duration: i32,
        height: i32,
        weight: i32,
        printer: Option<String>,
        material: Option<String>,
        author_id: i32,
    ) -> Self {
        let now = Local::now().naive_utc();
        Self {
            id: 0,
            name,
            description,
            duration,
            height,
            weight,
            printer,
            material,
            author_id,
            created: now,
            updated: now,
        }
    }

    /// Create a new model
    pub async fn create(model: Model) -> Result<Model, AppError> {
        let pool = unsafe { get_client() };

        model
            .validate()
            .map_err(|error| AppError::BadRequest(error.to_string()))?;

        let rec = sqlx::query_as!(
            Model,
            r#"
                INSERT INTO models (name, description, duration, height, weight, printer, material, author_id, created, updated)
                VALUES ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                RETURNING *
            "#,
            model.name,
            model.description,
            model.duration,
            model.height,
            model.weight,
            model.printer,
            model.material,
            model.author_id,
            model.created,
            model.updated,
        )
        .fetch_one(pool)
        .await?;

        Ok(rec)
    }

    /// List all models
    pub async fn list(page: i64) -> Result<Vec<ModelUser>, AppError> {
        let pool = unsafe { get_client() };
        let rows = sqlx::query_as!(
            ModelUser,
            r#"SELECT
            models.*,
            json_build_object('id', users.id, 'email', users.email, 'username', users.username, 'is_staff', users.is_staff) as author
            FROM models JOIN users ON users.id = models.author_id
            LIMIT $1 OFFSET $2
            "#,
            PAGE_LIMIT,
            PAGE_LIMIT * page
        )
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    /// Return the number of models.
    pub async fn count() -> Result<i64, AppError> {
        let pool = unsafe { get_client() };
        let row = sqlx::query!(r#"SELECT COUNT(id) as count FROM models"#)
            .fetch_one(pool)
            .await?;

        Ok(row.count.unwrap())
    }
}
