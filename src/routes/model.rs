use crate::{
    errors::AppError,
    files::{delete_upload, upload},
    models::{
        auth::Claims,
        model::{Model, ModelCreate, ModelUpload, ModelUser},
        user::User,
    },
    pagination::Pagination,
    routes::JsonCreate,
};
use axum::{
    extract::{ContentLengthLimit, Multipart, Path, Query},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Serialize;

/// Create routes for `/v1/models/` namespace
pub fn create_route() -> Router {
    Router::new()
        .route("/", get(list_models).post(create_model))
        .route("/:id", get(get_model).delete(delete_model).put(edit_model))
        .route("/:id/upload", post(upload_model_file))
        .route("/:id/upload/:uid", delete(delete_model_file))
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
) -> Result<JsonCreate<Model>, AppError> {
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

    Ok(JsonCreate(model_new))
}

/// Get a model with id = `model_id`
async fn get_model(Path(model_id): Path<i32>) -> Result<Json<ModelUser>, AppError> {
    match Model::find_by_id(model_id).await {
        Ok(model) => Ok(Json(model)),
        Err(_) => Err(AppError::NotFound("Model not found".to_string())),
    }
}

/// The owner or a staffer can delete a model
async fn delete_model(claims: Claims, Path(model_id): Path<i32>) -> Result<StatusCode, AppError> {
    let model = match Model::find_by_id(model_id).await {
        Ok(model) => model,
        Err(_) => {
            return Err(AppError::NotFound("Model not found".to_string()));
        }
    };

    let user = User::find_by_id(claims.user_id).await?;

    let uploads: Vec<String> = model.list_upload_filepaths().await.unwrap();

    if !(model.author_id() == user.id || user.is_staff.unwrap()) {
        return Err(AppError::Unauthorized);
    }

    // If the model has been deleted, remove all old uploads from the file system
    if Model::delete(model_id).await.is_ok() {
        uploads
            .iter()
            .for_each(|path: &String| delete_upload(path).unwrap_or_default());
    }

    Ok(StatusCode::NO_CONTENT)
}

/// The owner or a staffer can edit a model
async fn edit_model(
    Json(payload): Json<ModelCreate>,
    claims: Claims,
    Path(model_id): Path<i32>,
) -> Result<Json<ModelUser>, AppError> {
    let model = match Model::find_by_id(model_id).await {
        Ok(model) => model,
        Err(_) => {
            return Err(AppError::NotFound("Model not found".to_string()));
        }
    };

    let user = User::find_by_id(claims.user_id).await?;

    if !(model.author_id() == user.id || user.is_staff.unwrap()) {
        return Err(AppError::Unauthorized);
    }

    let model_body = Model::new(
        payload.name,
        payload.description,
        payload.duration,
        payload.height,
        payload.weight,
        payload.printer,
        payload.material,
        claims.user_id,
    );
    Model::edit(model.id, model_body).await?;
    Ok(Json(model))
}

/// Upload a file for a model
async fn upload_model_file(
    claims: Claims,
    Path(model_id): Path<i32>,
    ContentLengthLimit(multipart): ContentLengthLimit<Multipart, { 1024 * 1024 }>,
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

    let allowed_extensions = vec!["stl", "obj", "png", "jpg", "jpeg", "gif", "webp", "blend"];

    match upload(multipart, allowed_extensions, None).await {
        Ok(saved_file) => {
            let model_file = ModelUpload::create(ModelUpload::new(saved_file, model_id)).await?;

            Ok(Json(model_file))
        }
        Err(e) => Err(e),
    }
}

/// The owner or a staffer can delete a model upload
async fn delete_model_file(
    claims: Claims,
    Path((model_id, upload_id)): Path<(i32, i32)>,
) -> Result<StatusCode, AppError> {
    let model = match Model::find_by_id(model_id).await {
        Ok(model) => model,
        Err(_) => {
            return Err(AppError::NotFound("Model not found".to_string()));
        }
    };

    let user = User::find_by_id(claims.user_id).await?;

    if !(model.author_id() == user.id || user.is_staff.unwrap()) {
        return Err(AppError::Unauthorized);
    }

    let upload = match ModelUpload::find_by_id(upload_id).await {
        Ok(upload) => upload,
        Err(_) => {
            return Err(AppError::NotFound("Upload not found".to_string()));
        }
    };

    if upload.model_id != model.id {
        return Err(AppError::NotFound("Upload not found".to_string()));
    }

    let filepath = upload.filepath.clone();

    match ModelUpload::delete(upload_id).await {
        Ok(_) => {
            delete_upload(&filepath)?;

            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => Err(e),
    }
}
