use crate::{
    errors::AppError,
    auth::models::{AuthBody, Claims, LoginCredentials, SignUpForm},
    user::models::User,
    routes::JsonCreate,
};
use axum::{routing::post, Json, Router};

/// Create routes for `/v1/auth/` namespace
pub fn create_route() -> Router {
    Router::new()
        .route("/login", post(make_login))
        .route("/signup", post(signup))
}

/// Make login. Check if a user with the email and password passed in request body exists into the
/// database
async fn make_login(Json(payload): Json<LoginCredentials>) -> Result<Json<AuthBody>, AppError> {
    let user = User::new(
        String::new(),
        String::new(),
        payload.username,
        payload.password,
    );
    match User::find(user).await {
        Ok(user) => {
            let claims = Claims::new(user.id);
            let token = claims.get_token()?;
            Ok(Json(AuthBody::new(token)))
        }
        Err(_) => Err(AppError::NotFound("User not found".to_string())),
    }
}

/// Create a new user
async fn signup(Json(payload): Json<SignUpForm>) -> Result<JsonCreate<AuthBody>, AppError> {
    if payload.password1 != payload.password2 {
        return Err(AppError::BadRequest(
            "The inserted passwords do not match".to_string(),
        ));
    }

    if User::email_has_taken(&payload.email).await? {
        return Err(AppError::BadRequest(
            "An user with this email already exists".to_string(),
        ));
    }

    if User::username_has_taken(&payload.username).await? {
        return Err(AppError::BadRequest(
            "An user with this username already exists".to_string(),
        ));
    }

    let user = User::new(
        payload.name,
        payload.email,
        payload.username,
        payload.password1,
    );
    let user = User::create(user).await?;

    let claims = Claims::new(user.id);
    let token = claims.get_token()?;
    Ok(JsonCreate(AuthBody::new(token)))
}
