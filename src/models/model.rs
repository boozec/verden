use crate::{config::CONFIG, db::get_client, errors::AppError, json::number_from_string};
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
    duration: f64,
    height: f64,
    weight: f64,
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
    #[serde(deserialize_with = "number_from_string")]
    pub duration: f64,
    #[serde(deserialize_with = "number_from_string")]
    pub height: f64,
    #[serde(deserialize_with = "number_from_string")]
    pub weight: f64,
    pub printer: Option<String>,
    pub material: Option<String>,
}

/// Payload used for model searching
#[derive(Deserialize)]
pub struct ModelFilter {
    /// Stands for "query"
    pub q: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct ModelUser {
    pub id: i32,
    name: String,
    description: Option<String>,
    duration: f64,
    height: f64,
    weight: f64,
    printer: Option<String>,
    material: Option<String>,
    author_id: i32,
    created: NaiveDateTime,
    updated: NaiveDateTime,
    author: Option<JsonValue>,
    uploads: Option<JsonValue>,
    likes: Option<JsonValue>,
}

#[derive(Deserialize, Serialize, sqlx::FromRow)]
pub struct ModelUpload {
    id: i32,
    pub model_id: i32,
    pub filepath: String,
    created: NaiveDateTime,
}

impl Model {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        description: Option<String>,
        duration: f64,
        height: f64,
        weight: f64,
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

    /// Edit a model
    pub async fn edit(id: i32, model: Model) -> Result<Model, AppError> {
        let pool = unsafe { get_client() };

        model
            .validate()
            .map_err(|error| AppError::BadRequest(error.to_string()))?;

        let rec: Model = sqlx::query_as(
            r#"
                UPDATE models SET name = $1, description = $2, duration = $3, height = $4, weight = $5, printer = $6, material = $7, updated = $8
                WHERE id = $9
                RETURNING *
            "#)
            .bind(model.name)
            .bind(model.description)
            .bind(model.duration)
            .bind(model.height)
            .bind(model.weight)
            .bind(model.printer)
            .bind(model.material)
            .bind(model.updated)
            .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(rec)
    }

    /// Returns the model with id = `model_id`
    pub async fn find_by_id(model_id: i32) -> Result<ModelUser, AppError> {
        let pool = unsafe { get_client() };

        let rec: ModelUser = sqlx::query_as(
            r#"
                WITH model_uploads AS (
                    SELECT models.id, json_agg(uploads.*) filter(WHERE uploads.* IS NOT NULL) AS uploads
                    FROM models
                    LEFT JOIN uploads ON uploads.model_id = models.id
                    GROUP BY models.id
                ),
                model_likes AS (
                    SELECT models.id, json_agg(likes.*) filter(WHERE likes.* IS NOT NULL) AS likes
                    FROM models
                    LEFT JOIN likes ON likes.model_id = models.id
                    GROUP BY models.id
                ),
                model_author AS (
                    SELECT models.id, json_build_object('id', users.id, 'name', users.name, 'email', users.email, 'username', users.username, 'is_staff', users.is_staff, 'avatar', users.avatar) as author
                    FROM models
                    JOIN users ON users.id = models.author_id
                )
                SELECT models.*, author, uploads, likes
                FROM models
                INNER JOIN model_author using (id)
                INNER JOIN model_uploads using (id)
                INNER JOIN model_likes using (id)
                WHERE models.id = $1
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
            WITH model_uploads AS (
                SELECT models.id, json_agg(uploads.*) filter(WHERE uploads.* IS NOT NULL) AS uploads
                FROM models
                LEFT JOIN uploads ON uploads.model_id = models.id
                GROUP BY models.id
            ),
            model_likes AS (
                SELECT models.id, json_agg(likes.*) filter(WHERE likes.* IS NOT NULL) AS likes
                FROM models
                LEFT JOIN likes ON likes.model_id = models.id
                GROUP BY models.id
            ),
            model_author AS (
                SELECT models.id, json_build_object('id', users.id, 'name', users.name, 'email', users.email, 'username', users.username, 'is_staff', users.is_staff, 'avatar', users.avatar) as author
                FROM models
                JOIN users ON users.id = models.author_id
            )
            SELECT models.*, author, uploads, likes
            FROM models
            INNER JOIN model_author using (id)
            INNER JOIN model_uploads using (id)
            INNER JOIN model_likes using (id)
            ORDER BY id DESC
            LIMIT $1 OFFSET $2
            "#)
        .bind(CONFIG.page_limit)
        .bind(CONFIG.page_limit * page)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    /// Filter models by some cols
    pub async fn filter(page: i64, query: String) -> Result<Vec<ModelUser>, AppError> {
        let pool = unsafe { get_client() };
        let rows: Vec<ModelUser> = sqlx::query_as(
            r#"
            WITH model_uploads AS (
                SELECT models.id, json_agg(uploads.*) filter(WHERE uploads.* IS NOT NULL) AS uploads
                FROM models
                LEFT JOIN uploads ON uploads.model_id = models.id
                GROUP BY models.id
            ),
            model_likes AS (
                SELECT models.id, json_agg(likes.*) filter(WHERE likes.* IS NOT NULL) AS likes
                FROM models
                LEFT JOIN likes ON likes.model_id = models.id
                GROUP BY models.id
            ),
            model_author AS (
                SELECT models.id, json_build_object('id', users.id, 'name', users.name, 'email', users.email, 'username', users.username, 'is_staff', users.is_staff, 'avatar', users.avatar) as author
                FROM models
                JOIN users ON users.id = models.author_id
            )
            SELECT models.*, author, uploads, likes
            FROM models
            INNER JOIN model_author using (id)
            INNER JOIN model_uploads using (id)
            INNER JOIN model_likes using (id)
            WHERE models.name ILIKE $1 OR description ILIKE $1 OR printer ILIKE $1 OR material ILIKE $1
            ORDER BY id DESC
            LIMIT $2 OFFSET $3
            "#)
        .bind(format!("%{}%", query))
        .bind(CONFIG.page_limit)
        .bind(CONFIG.page_limit * page)
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    /// List author's models
    pub async fn list_from_author(page: i64, author: i32) -> Result<Vec<ModelUser>, AppError> {
        let pool = unsafe { get_client() };
        let rows: Vec<ModelUser> = sqlx::query_as(
            r#"
            WITH model_uploads AS (
                SELECT models.id, json_agg(uploads.*) filter(WHERE uploads.* IS NOT NULL) AS uploads
                FROM models
                LEFT JOIN uploads ON uploads.model_id = models.id
                GROUP BY models.id
            ),
            model_likes AS (
                SELECT models.id, json_agg(likes.*) filter(WHERE likes.* IS NOT NULL) AS likes
                FROM models
                LEFT JOIN likes ON likes.model_id = models.id
                GROUP BY models.id
            ),
            model_author AS (
                SELECT models.id, json_build_object('id', users.id, 'name', users.name, 'email', users.email, 'username', users.username, 'is_staff', users.is_staff, 'avatar', users.avatar) as author
                FROM models
                JOIN users ON users.id = models.author_id
            )
            SELECT models.*, author, uploads, likes
            FROM models
            INNER JOIN model_author using (id)
            INNER JOIN model_uploads using (id)
            INNER JOIN model_likes using (id)
            WHERE models.author_id = $1
            ORDER BY id DESC
            LIMIT $2 OFFSET $3
            "#)
        .bind(author)
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

    /// Return the number of author models
    pub async fn count_filter_by_author(author: i32) -> Result<i64, AppError> {
        let pool = unsafe { get_client() };
        let cursor = sqlx::query(r#"SELECT COUNT(id) as count FROM models WHERE author_id = $1"#)
            .bind(author)
            .fetch_one(pool)
            .await?;

        let count: i64 = cursor.try_get(0).unwrap();
        Ok(count)
    }

    /// Return the number of models filtered by query
    pub async fn count_filter(query: String) -> Result<i64, AppError> {
        let pool = unsafe { get_client() };
        let cursor = sqlx::query(
                r#"
                SELECT COUNT(id) as count FROM models
                WHERE models.name ILIKE $1 OR description ILIKE $1 OR printer ILIKE $1 OR material ILIKE $1
                "#
            )
            .bind(format!("%{}%", query))
            .fetch_one(pool)
            .await?;

        let count: i64 = cursor.try_get(0).unwrap();
        Ok(count)
    }
}

impl ModelUser {
    /// Returns the author id from the `JsonValue`
    pub fn author_id(&self) -> JsonValue {
        match &self.author {
            Some(json) => json.get("id").unwrap().clone(),
            None => json!(0),
        }
    }

    /// Returns a vec of string made by all the filepaths from the model
    pub async fn list_upload_filepaths(&self) -> Option<Vec<String>> {
        // Raise a `None` if `self.uploads` is `None`
        self.uploads.as_ref()?;

        let uploads = ModelUpload::find_by_model(self.id)
            .await
            .unwrap_or_default();

        let paths = uploads
            .iter()
            .map(|x| x.filepath.clone())
            .collect::<Vec<String>>();

        Some(paths)
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

    /// Returns the model upload with id = `upload_id`
    pub async fn find_by_id(id: i32) -> Result<ModelUpload, AppError> {
        let pool = unsafe { get_client() };

        let rec: ModelUpload = sqlx::query_as(
            r#"
                SELECT * FROM uploads WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(rec)
    }

    /// Delete a model upload
    pub async fn delete(upload_id: i32) -> Result<(), AppError> {
        let pool = unsafe { get_client() };

        sqlx::query(
            r#"
            DELETE FROM uploads WHERE id = $1
            "#,
        )
        .bind(upload_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
