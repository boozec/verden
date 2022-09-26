use crate::{config::CONFIG, db::get_client, errors::AppError};
use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;
use sqlx::Row;
use std::convert::From;

/// Model for warnings.
#[derive(Deserialize, Serialize, sqlx::FromRow)]
pub struct Warning {
    pub id: i32,
    pub user_id: Option<i32>,
    pub model_id: Option<i32>,
    pub resolved_by: Option<i32>,
    pub note: String,
    pub admin_note: String,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct WarningUser {
    pub id: i32,
    pub user_id: Option<i32>,
    pub model_id: Option<i32>,
    pub resolved_by: Option<i32>,
    pub note: String,
    pub admin_note: String,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
    user: Option<JsonValue>,
    resolved: Option<JsonValue>,
}

/// Impl conversion from `WarningUser` to `Warning`
impl From<WarningUser> for Warning {
    fn from(item: WarningUser) -> Self {
        Self {
            id: item.id,
            user_id: item.user_id,
            model_id: item.model_id,
            resolved_by: item.resolved_by,
            note: item.note,
            admin_note: item.admin_note,
            created: item.created,
            updated: item.created,
        }
    }
}

/// Payload used to create a new warning
#[derive(Deserialize)]
pub struct WarningCreate {
    pub model_id: i32,
    pub note: String,
}

/// Payload used to edit a warning
#[derive(Deserialize)]
pub struct WarningEdit {
    pub admin_note: String,
}

/// Payload used for warning filtering
#[derive(Deserialize)]
pub struct WarningFilterPayload {
    pub model_id: i32,
}

/// Struct used as argument for filtering by the backend
pub struct WarningFilter {
    pub model_id: i32,
    pub user_id: Option<i32>,
}

impl Warning {
    /// Create a warning means create an object which has an `user_id` (creator of the warning), a
    /// `model_id` (suspect model) and a `note`
    pub fn new(user_id: i32, model_id: i32, note: String) -> Self {
        let now = Local::now().naive_utc();
        Self {
            id: 0,
            user_id: Some(user_id),
            model_id: Some(model_id),
            resolved_by: None,
            note,
            admin_note: String::new(),
            created: now,
            updated: now,
        }
    }

    /// List all warnings. A staffer can see all the warnings, a user cannot
    pub async fn list(page: i64, user_id: Option<i32>) -> Result<Vec<WarningUser>, AppError> {
        let pool = unsafe { get_client() };
        let query = r#"
                    SELECT
                        warnings.*,
                        json_build_object('id', users.id, 'name', users.name, 'email', users.email, 'username', users.username, 'is_staff', users.is_staff, 'avatar', users.avatar) as user,
                        coalesce(r.data, '{}'::json) as resolved
                    FROM warnings
                    JOIN users ON users.id = warnings.user_id
                    LEFT JOIN (
                        SELECT id, json_build_object('id', r.id, 'name', r.name, 'email', r.email, 'username', r.username, 'is_staff', r.is_staff, 'avatar', r.avatar) as data
                        FROM users r
                    ) r ON r.id = warnings.resolved_by
                    "#;

        let rows: Vec<WarningUser> = match user_id {
            Some(id) => {
                sqlx::query_as(&format!(
                    r#"{} WHERE user_id = $1 LIMIT $2 OFFSET $3"#,
                    query
                ))
                .bind(id)
                .bind(CONFIG.page_limit)
                .bind(CONFIG.page_limit * page)
                .fetch_all(pool)
                .await?
            }
            None => {
                sqlx::query_as(&format!(r#"{} LIMIT $1 OFFSET $2"#, query))
                    .bind(CONFIG.page_limit)
                    .bind(CONFIG.page_limit * page)
                    .fetch_all(pool)
                    .await?
            }
        };

        Ok(rows)
    }

    /// Returns the warning with id = `warning_id`
    pub async fn find_by_id(warning_id: i32) -> Result<WarningUser, AppError> {
        let pool = unsafe { get_client() };

        let rec: WarningUser = sqlx::query_as(
            r#"
                SELECT
                    warnings.*,
                    json_build_object('id', users.id, 'name', users.name, 'email', users.email, 'username', users.username, 'is_staff', users.is_staff, 'avatar', users.avatar) as user,
                    coalesce(r.data, '{}'::json) as resolved
                FROM warnings
                JOIN users ON users.id = warnings.user_id
                LEFT JOIN (
                    SELECT id, json_build_object('id', r.id, 'name', r.name, 'email', r.email, 'username', r.username, 'is_staff', r.is_staff, 'avatar', r.avatar) as data
                    FROM users r
                ) r ON r.id = warnings.resolved_by
                WHERE warnings.id = $1
            "#)
        .bind(warning_id)
        .fetch_one(pool)
        .await?;

        Ok(rec)
    }

    /// Return the number of warnings.
    pub async fn count(user_id: Option<i32>) -> Result<i64, AppError> {
        let pool = unsafe { get_client() };

        let cursor = match user_id {
            Some(id) => {
                sqlx::query(r#"SELECT COUNT(id) as count FROM warnings WHERE user_id = $1"#)
                    .bind(id)
                    .fetch_one(pool)
                    .await?
            }
            None => {
                sqlx::query(r#"SELECT COUNT(id) as count FROM warnings"#)
                    .fetch_one(pool)
                    .await?
            }
        };

        let count: i64 = cursor.try_get(0).unwrap();
        Ok(count)
    }

    /// Create a new upload for model
    pub async fn create(warning: Warning) -> Result<Warning, AppError> {
        let pool = unsafe { get_client() };

        let rec: Warning = sqlx::query_as(
            r#"
                INSERT INTO warnings (user_id, model_id, resolved_by, note, admin_note, created, updated)
                VALUES ( $1, $2, $3, $4, $5, $6, $7)
                RETURNING *
            "#,
        )
        .bind(warning.user_id)
        .bind(warning.model_id)
        .bind(warning.resolved_by)
        .bind(warning.note)
        .bind(warning.admin_note)
        .bind(warning.created)
        .bind(warning.updated)
        .fetch_one(pool)
        .await?;

        Ok(rec)
    }

    /// Filter warnings. Pass a `WarningFilter` argument
    pub async fn filter(page: i64, args: WarningFilter) -> Result<Vec<WarningUser>, AppError> {
        let pool = unsafe { get_client() };

        let query = r#"
                    SELECT
                        warnings.*,
                        json_build_object('id', users.id, 'name', users.name, 'email', users.email, 'username', users.username, 'is_staff', users.is_staff, 'avatar', users.avatar) as user,
                        coalesce(r.data, '{}'::json) as resolved
                    FROM warnings
                    JOIN users ON users.id = warnings.user_id
                    LEFT JOIN (
                        SELECT id, json_build_object('id', r.id, 'name', r.name, 'email', r.email, 'username', r.username, 'is_staff', r.is_staff, 'avatar', r.avatar) as data
                        FROM users r
                    ) r ON r.id = warnings.resolved_by
                    WHERE model_id = $1
                    "#;

        let rows: Vec<WarningUser> = match args.user_id {
            Some(id) => {
                sqlx::query_as(&format!(r#"{} AND user_id = $2 LIMIT $3 OFFSET $4"#, query))
                    .bind(args.model_id)
                    .bind(id)
                    .bind(CONFIG.page_limit)
                    .bind(CONFIG.page_limit * page)
                    .fetch_all(pool)
                    .await?
            }
            None => {
                sqlx::query_as(&format!(r#"{} LIMIT $2 OFFSET $3"#, query))
                    .bind(args.model_id)
                    .bind(CONFIG.page_limit)
                    .bind(CONFIG.page_limit * page)
                    .fetch_all(pool)
                    .await?
            }
        };

        Ok(rows)
    }

    /// Return the number of filtered warnings.
    pub async fn count_by_model_id(args: WarningFilter) -> Result<i64, AppError> {
        let pool = unsafe { get_client() };

        let query = r#"
                    SELECT COUNT(id) as count FROM warnings WHERE model_id = $1
                    "#;

        let cursor = match args.user_id {
            Some(id) => {
                sqlx::query(&format!(r#"{} AND user_id = $2"#, query))
                    .bind(args.model_id)
                    .bind(id)
                    .fetch_one(pool)
                    .await?
            }
            None => {
                sqlx::query(&format!(r#"{}"#, query))
                    .bind(args.model_id)
                    .fetch_one(pool)
                    .await?
            }
        };

        let count: i64 = cursor.try_get(0).unwrap();
        Ok(count)
    }

    /// Edit a warning
    pub async fn edit(&mut self, resolver: i32, payload: WarningEdit) -> Result<(), AppError> {
        let pool = unsafe { get_client() };

        let now = Local::now().naive_utc();

        sqlx::query(
            r#"
            UPDATE warnings SET admin_note = $1, resolved_by = $2, updated = $3 WHERE id = $4
            "#,
        )
        .bind(&payload.admin_note)
        .bind(resolver)
        .bind(now)
        .bind(self.id)
        .execute(pool)
        .await?;

        self.admin_note = payload.admin_note;
        self.resolved_by = Some(resolver);
        self.updated = now;

        Ok(())
    }
}
