use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

const JWT_EXPIRATION_HOURS: i64 = 24;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,               // Subject (User ID)
    pub exp: i64,                // Expiration Time
    pub iat: i64,                // Issued At
    pub email: String,           // User's email
    pub tenant_id: Option<Uuid>, // Current tenant context (if any)
}

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("Failed to create token: {0}")]
    TokenCreation(String),
    #[error("Failed to validate token: {0}")]
    TokenValidation(String),
    #[error("Token expired")]
    TokenExpired,
}

pub struct JwtUtils {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtUtils {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        }
    }

    pub fn create_token(
        &self,
        user_id: Uuid,
        email: &str,
        tenant_id: Option<Uuid>,
    ) -> Result<String, JwtError> {
        let now = OffsetDateTime::now_utc();
        let exp = now + Duration::hours(JWT_EXPIRATION_HOURS);

        let claims = Claims {
            sub: user_id,
            exp: exp.unix_timestamp(),
            iat: now.unix_timestamp(),
            email: email.to_string(),
            tenant_id,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| JwtError::TokenCreation(e.to_string()))
    }

    // For backwards compatibility
    pub fn create_token_without_tenant(
        &self,
        user_id: Uuid,
        email: &str,
    ) -> Result<String, JwtError> {
        self.create_token(user_id, email, None)
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        let validation = Validation::default();

        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => JwtError::TokenExpired,
                _ => JwtError::TokenValidation(e.to_string()),
            })
    }
}
