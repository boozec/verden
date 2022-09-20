use crate::{
    errors::AppError,
    models::{
        auth::Claims,
        model::Model,
        user::User,
        warning::{Warning, WarningCreate},
    },
    pagination::Pagination,
    routes::JsonCreate,
};
use axum::{extract::Query, routing::get, Json, Router};
use serde::Serialize;

/// Create routes for `/v1/warnings/` namespace
pub fn create_route() -> Router {
    Router::new().route("/", get(list_warnings).post(create_warning))
}

#[derive(Serialize)]
struct WarningPagination {
    count: i64,
    results: Vec<Warning>,
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

/// Create a warning. Checks Authorization token
async fn create_warning(
    Json(payload): Json<WarningCreate>,
    claims: Claims,
) -> Result<JsonCreate<Warning>, AppError> {
    let model = match Model::find_by_id(payload.model_id).await {
        Ok(model) => model,
        Err(_) => return Err(AppError::NotFound("Model not found".to_string())),
    };

    let warning = Warning::new(claims.user_id, model.id, payload.note);

    let warning_new = Warning::create(warning).await?;

    Ok(JsonCreate(warning_new))
}
