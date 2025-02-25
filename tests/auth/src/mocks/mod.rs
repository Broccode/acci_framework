use mockall::automock;
use serde_json::Value;
use std::time::SystemTime;
use uuid::Uuid;

use acci_auth::{
    models::user::{User, UserError, UserRepository},
    session::{
        Session, SessionError, SessionRepository,
        types::{DeviceFingerprint, SessionInvalidationReason},
    },
};

#[automock]
pub trait UserRepositoryMock: UserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, UserError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, UserError>;
    async fn create(&self, user: &User) -> Result<(), UserError>;
    async fn verify_email(&self, id: Uuid) -> Result<(), UserError>;
    async fn deactivate(&self, id: Uuid) -> Result<(), UserError>;
    async fn activate(&self, id: Uuid) -> Result<(), UserError>;
}

#[automock]
pub trait SessionRepositoryMock: SessionRepository {
    async fn create_session(
        &self,
        user_id: Uuid,
        token_hash: String,
        device_id: Option<String>,
        device_fingerprint: Option<DeviceFingerprint>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        metadata: Option<Value>,
        expires_at: SystemTime,
    ) -> Result<Session, SessionError>;

    async fn get_session_by_token(&self, token_hash: &str)
    -> Result<Option<Session>, SessionError>;

    async fn invalidate_session(
        &self,
        token_hash: &str,
        reason: SessionInvalidationReason,
    ) -> Result<(), SessionError>;

    async fn get_user_sessions(&self, user_id: Uuid) -> Result<Vec<Session>, SessionError>;
}
