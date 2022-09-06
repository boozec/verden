use crate::db::get_client;
use crate::errors::AppError;

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
    pub async fn list() -> Result<Vec<Model>, AppError> {
        let pool = unsafe { get_client() };
        let rows = sqlx::query_as!(Model, r#"SELECT * FROM models"#)
            .fetch_all(pool)
            .await?;

        Ok(rows)
    }
}
