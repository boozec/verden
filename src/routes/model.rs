use crate::errors::AppError;
use crate::models::{
    auth::Claims,
    model::{Model, ModelCreate, ModelUser},
};
use crate::pagination::Pagination;
use axum::{extract::Query, routing::get, Json, Router};
use serde::Serialize;

/// Create routes for `/v1/models/` namespace
pub fn create_route() -> Router {
    Router::new().route("/", get(list_models).post(create_model))
}

#[derive(Serialize)]
struct ModelPagination {
    count: i64,
    results: Vec<ModelUser>,
}

/// List models.
async fn list_models(pagination: Query<Pagination>) -> Result<Json<ModelPagination>, AppError> {
    let page = pagination.0.page.unwrap_or_default();
    let results = Model::list(page).await?;
    let count = Model::count().await?;

    Ok(Json(ModelPagination { count, results }))
}

/// Create a model. Checks Authorization token
async fn create_model(
    Json(payload): Json<ModelCreate>,
    claims: Claims,
) -> Result<Json<Model>, AppError> {
    let model = Model::new(
        payload.name,
        payload.description,
        payload.duration,
        payload.height,
        payload.weight,
        payload.printer,
        payload.material,
        claims.user_id,
    );

    let model_new = Model::create(model).await?;

    Ok(Json(model_new))
}
