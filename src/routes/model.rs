use crate::errors::AppError;
use crate::models::{
    auth::Claims,
    model::{Model, ModelCreate},
};
use axum::{routing::get, Json, Router};

/// Create routes for `/v1/models/` namespace
pub fn create_route() -> Router {
    Router::new().route("/", get(list_models).post(create_model))
}

/// List models.
async fn list_models() -> Result<Json<Vec<Model>>, AppError> {
    let models = Model::list().await?;

    Ok(Json(models))
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
