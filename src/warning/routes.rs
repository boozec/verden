use crate::{
    auth::models::Claims,
    errors::AppError,
    model::models::Model,
    pagination::{Pagination, WarningPagination},
    routes::JsonCreate,
    user::models::User,
    warning::models::*,
};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

/// Create routes for `/v1/warnings/` namespace
pub fn create_route() -> Router {
    Router::new()
        .route("/", get(list_warnings).post(create_warning))
        .route(
            "/:id",
            get(get_warning).put(edit_warning).delete(delete_warning),
        )
        .route("/filter", post(filter_warnings))
}

/// List warnings. A staffer can see everything.
async fn list_warnings(
    pagination: Query<Pagination>,
    claims: Claims,
) -> Result<Json<WarningPagination>, AppError> {
    let page = pagination.0.page.unwrap_or_default();

    let user = User::find_by_id(claims.user_id).await?;

    let (results, count) = match user.is_staff.unwrap() {
        true => (
            Warning::list(page, None).await?,
            Warning::count(None).await?,
        ),
        false => (
            Warning::list(page, Some(user.id)).await?,
            Warning::count(Some(user.id)).await?,
        ),
    };

    Ok(Json(WarningPagination { count, results }))
}

/// Get a warning with id = `model_id`
async fn get_warning(
    Path(warning_id): Path<i32>,
    claims: Claims,
) -> Result<Json<Warning>, AppError> {
    let user = User::find_by_id(claims.user_id).await?;

    if !(user.is_staff.unwrap()) {
        return Err(AppError::Unauthorized);
    }

    match Warning::find_by_id(warning_id).await {
        Ok(warning) => Ok(Json(warning.into())),
        Err(_) => Err(AppError::NotFound("Warning not found".to_string())),
    }
}

/// Create a warning. Checks Authorization token
async fn create_warning(
    Json(payload): Json<WarningCreate>,
    claims: Claims,
) -> Result<JsonCreate<Warning>, AppError> {
    let model = match Model::find_by_id(payload.model_id).await {
        Ok(model) => model,
        Err(_) => return Err(AppError::NotFound("Report not found".to_string())),
    };

    let warning = Warning::new(claims.user_id, model.id, payload.note);

    let warning_new = Warning::create(warning).await?;

    Ok(JsonCreate(warning_new))
}

/// Staffers can edit a warning
async fn edit_warning(
    Json(payload): Json<WarningEdit>,
    claims: Claims,
    Path(warning_id): Path<i32>,
) -> Result<Json<Warning>, AppError> {
    let mut warning: Warning = match Warning::find_by_id(warning_id).await {
        Ok(warning) => warning.into(),
        Err(_) => {
            return Err(AppError::NotFound("Report not found".to_string()));
        }
    };

    let user = User::find_by_id(claims.user_id).await?;

    if !(user.is_staff.unwrap()) {
        return Err(AppError::Unauthorized);
    }

    if payload.resolved_by.is_none() || payload.resolved_by.unwrap() > 0 {
        warning.edit(Some(user.id), payload).await?;
    } else {
        warning.edit(None, payload).await?;
    }

    Ok(Json(warning))
}

/// A staffer can delete a warning
async fn delete_warning(
    claims: Claims,
    Path(warning_id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let user = User::find_by_id(claims.user_id).await?;

    if !user.is_staff.unwrap() {
        return Err(AppError::Unauthorized);
    }

    if Warning::delete(warning_id).await.is_ok() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::BAD_REQUEST)
    }
}

/// Apply a filter to warnings list
async fn filter_warnings(
    Json(payload): Json<WarningFilterPayload>,
    pagination: Query<Pagination>,
    claims: Claims,
) -> Result<Json<WarningPagination>, AppError> {
    let page = pagination.0.page.unwrap_or_default();

    let user = User::find_by_id(claims.user_id).await?;

    let (results, count) = match user.is_staff.unwrap() {
        true => (
            Warning::filter(
                page,
                WarningFilter {
                    model_id: payload.model_id,
                    resolved_by: payload.resolved_by,
                    user_id: None,
                },
            )
            .await?,
            Warning::count_by_model_id(WarningFilter {
                model_id: payload.model_id,
                resolved_by: payload.resolved_by,
                user_id: None,
            })
            .await?,
        ),
        false => (
            Warning::filter(
                page,
                WarningFilter {
                    model_id: payload.model_id,
                    resolved_by: payload.resolved_by,
                    user_id: Some(user.id),
                },
            )
            .await?,
            Warning::count_by_model_id(WarningFilter {
                model_id: payload.model_id,
                resolved_by: payload.resolved_by,
                user_id: Some(user.id),
            })
            .await?,
        ),
    };

    Ok(Json(WarningPagination { count, results }))
}
