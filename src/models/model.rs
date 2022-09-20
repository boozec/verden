use crate::{config::CONFIG, db::get_client, errors::AppError};
use serde_json::json;
use sqlx::types::JsonValue;
use sqlx::Row;

use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Model for models.
#[derive(Deserialize, Serialize, Validate, sqlx::FromRow)]
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

#[derive(Serialize, sqlx::FromRow)]
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
    uploads: Option<JsonValue>,
}

#[derive(Deserialize, Serialize, sqlx::FromRow)]
pub struct ModelUpload {
    id: i32,
    model_id: i32,
    filepath: String,
    created: NaiveDateTime,
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

        let rec: Model = sqlx::query_as(
            r#"
                INSERT INTO models (name, description, duration, height, weight, printer, material, author_id, created, updated)
                VALUES ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                RETURNING *
            "#)
            .bind(model.name)
            .bind(model.description)
            .bind(model.duration)
            .bind(model.height)
            .bind(model.weight)
            .bind(model.printer)
            .bind(model.material)
            .bind(model.author_id)
            .bind(model.created)
            .bind(model.updated)
        .fetch_one(pool)
        .await?;

        Ok(rec)
    }

    /// Returns the model with id = `model_id`
    pub async fn find_by_id(model_id: i32) -> Result<ModelUser, AppError> {
        let pool = unsafe { get_client() };

        let rec: ModelUser = sqlx::query_as(
            r#"
                SELECT
                    models.*,
                    json_build_object('id', users.id, 'email', users.email, 'username', users.username, 'is_staff', users.is_staff, 'avatar', users.avatar) as author,
                    json_agg(uploads.*) filter (where uploads.* is not null) as uploads
                FROM models
                JOIN users ON users.id = models.author_id
                LEFT JOIN uploads ON uploads.model_id = models.id
                WHERE models.id = $1
                GROUP BY models.id, users.id
            "#)
        .bind(model_id)
        .fetch_one(pool)
        .await?;

        Ok(rec)
    }

    /// List all models
    pub async fn list(page: i64) -> Result<Vec<ModelUser>, AppError> {
        let pool = unsafe { get_client() };
        let rows: Vec<ModelUser> = sqlx::query_as(
            r#"
            SELECT
                models.*,
                json_build_object('id', users.id, 'email', users.email, 'username', users.username, 'is_staff', users.is_staff, 'avatar', users.avatar) as author,
                json_agg(uploads.*) filter (where uploads.* is not null) as uploads
            FROM models
            JOIN users ON users.id = models.author_id
            LEFT JOIN uploads ON uploads.model_id = models.id
            GROUP BY models.id, users.id
            ORDER BY id DESC
            LIMIT $1 OFFSET $2
            "#)
        .bind(CONFIG.page_limit)
        .bind(CONFIG.page_limit * page)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    /// Delete a model
    pub async fn delete(model_id: i32) -> Result<(), AppError> {
        let pool = unsafe { get_client() };

        sqlx::query(
            r#"
            DELETE FROM models WHERE id = $1
            "#,
        )
        .bind(model_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Return the number of models.
    pub async fn count() -> Result<i64, AppError> {
        let pool = unsafe { get_client() };
        let cursor = sqlx::query(r#"SELECT COUNT(id) as count FROM models"#)
            .fetch_one(pool)
            .await?;

        let count: i64 = cursor.try_get(0).unwrap();
        Ok(count)
    }
}

impl ModelUser {
    pub fn author_id(&self) -> JsonValue {
        match &self.author {
            Some(json) => json.get("id").unwrap().clone(),
            None => json!(0),
        }
    }

    pub async fn upload_paths(&self) -> Option<Vec<String>> {
        if self.uploads.is_none() {
            return None;
        }

        let uploads = ModelUpload::find_by_model(self.id)
            .await
            .unwrap_or_default();

        let paths = uploads
            .iter()
            .map(|x| x.filepath.clone())
            .collect::<Vec<String>>();

        return Some(paths);
    }
}

impl ModelUpload {
    pub fn new(filepath: String, model_id: i32) -> Self {
        let now = Local::now().naive_utc();
        Self {
            id: 0,
            filepath,
            model_id,
            created: now,
        }
    }

    /// Create a new upload for model
    pub async fn create(file: ModelUpload) -> Result<ModelUpload, AppError> {
        let pool = unsafe { get_client() };

        let rec: ModelUpload = sqlx::query_as(
            r#"
                INSERT INTO uploads (filepath, model_id, created)
                VALUES ( $1, $2, $3)
                RETURNING *
            "#,
        )
        .bind(file.filepath)
        .bind(file.model_id)
        .bind(file.created)
        .fetch_one(pool)
        .await?;

        Ok(rec)
    }

    /// Find all paths of a model
    pub async fn find_by_model(model_id: i32) -> Result<Vec<ModelUpload>, AppError> {
        let pool = unsafe { get_client() };

        let rec: Vec<ModelUpload> = sqlx::query_as(
            r#"
                SELECT * FROM uploads WHERE model_id = $1
            "#,
        )
        .bind(model_id)
        .fetch_all(pool)
        .await?;

        Ok(rec)
    }
}
