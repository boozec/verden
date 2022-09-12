use crate::config::MAX_UPLOAD_FILE_SIZE;
use crate::errors::AppError;
use crate::files::upload;
use crate::models::{
    auth::Claims,
    model::{Model, ModelCreate, ModelUpload, ModelUser},
};
use crate::pagination::Pagination;
use axum::{
    extract::{ContentLengthLimit, Multipart, Path, Query},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;

/// Create routes for `/v1/models/` namespace
pub fn create_route() -> Router {
    Router::new()
        .route("/", get(list_models).post(create_model))
        .route("/:id/upload", post(upload_model_file))
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

/// Upload a file for a model
async fn upload_model_file(
    claims: Claims,
    Path(model_id): Path<i32>,
    ContentLengthLimit(multipart): ContentLengthLimit<Multipart, { MAX_UPLOAD_FILE_SIZE }>,
) -> Result<Json<ModelUpload>, AppError> {
    let model = match Model::find_by_id(model_id).await {
        Ok(model) => model,
        Err(_) => {
            return Err(AppError::NotFound("Model not found".to_string()));
        }
    };

    if model.author_id() != claims.user_id {
        return Err(AppError::Unauthorized);
    }

    match upload(multipart, vec!["stl"]).await {
        Ok(saved_file) => {
            let model_file = ModelUpload::create(ModelUpload::new(saved_file, model_id)).await?;

            return Ok(Json(model_file));
        }
        Err(e) => Err(e),
    }
}
