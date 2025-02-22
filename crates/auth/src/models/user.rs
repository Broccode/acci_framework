use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub last_login: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}

#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User not found")]
    NotFound,
    #[error("User already exists")]
    AlreadyExists,
    #[error("Password too weak: {0}")]
    WeakPassword(String),
    #[error("Invalid email format")]
    InvalidEmail,
}
