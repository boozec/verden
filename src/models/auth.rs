use crate::errors::AppError;
use axum::{
    async_trait,
    extract::{FromRequest, RequestParts, TypedHeader},
    headers::{authorization::Bearer, Authorization},
};
use chrono::{Duration, Local};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

/// Claims struct
#[derive(Serialize, Deserialize)]
pub struct Claims {
    /// ID from the user model
    pub user_id: i32,
    /// Expiration timestamp
    exp: usize,
}

/// Body used as response to login
#[derive(Serialize)]
pub struct AuthBody {
    /// Access token string
    access_token: String,
    /// "Bearer" string
    token_type: String,
}

/// Payload used for user creation
#[derive(Deserialize)]
pub struct LoginCredentials {
    pub username: String,
    pub password: String,
}

static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

impl Claims {
    /// Create a new Claim using the `user_id` and the current timestamp + 2 days
    pub fn new(user_id: i32) -> Self {
        let expiration = Local::now() + Duration::days(1);

        Self {
            user_id,
            exp: expiration.timestamp() as usize,
        }
    }

    /// Returns the token as a string. If a token is not encoded, raises an
    /// `AppError::TokenCreation`
    pub fn get_token(&self) -> Result<String, AppError> {
        let token = encode(&Header::default(), &self, &KEYS.encoding)
            .map_err(|_| AppError::TokenCreation)?;

        Ok(token)
    }
}

impl AuthBody {
    pub fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

/// Parse a request to get the Authorization header and then decode it checking its validation
#[async_trait]
impl<B> FromRequest<B> for Claims
where
    B: Send,
{
    type Rejection = AppError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request(req)
                .await
                .map_err(|_| AppError::InvalidToken)?;
        // Decode the user data
        let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|_| AppError::InvalidToken)?;

        Ok(token_data.claims)
    }
}
