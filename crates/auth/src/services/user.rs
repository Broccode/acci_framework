use regex::Regex;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    AuthConfig, SessionService, SessionServiceError,
    models::user::{CreateUser, User, UserError, UserRepository},
    session::{
        Session, SessionFilter,
        types::{DeviceFingerprint, SessionInvalidationReason},
    },
    utils::{
        jwt::{JwtError, JwtUtils},
        password::{PasswordError, check_password_strength, hash_password, verify_password},
    },
};

lazy_static::lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(concat!(
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@",
        r"[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?",
        r"(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)+$"
    )).expect("Failed to compile email regex pattern - this is a bug");
}

#[derive(Debug, thiserror::Error)]
pub enum UserServiceError {
    #[error(transparent)]
    User(#[from] UserError),
    #[error(transparent)]
    Password(#[from] PasswordError),
    #[error(transparent)]
    Jwt(#[from] JwtError),
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User not found")]
    UserNotFound,
    #[error("Repository error: {0}")]
    Repository(#[from] sqlx::Error),
    #[error("Session error: {0}")]
    Session(#[from] SessionServiceError),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

pub struct UserService {
    repository: Arc<dyn UserRepository>,
    _jwt_utils: Arc<JwtUtils>,
    session_service: Arc<SessionService>,
    _config: Arc<AuthConfig>,
}

pub struct LoginResult {
    pub user: User,
    pub session_token: String,
}

impl UserService {
    pub fn new(
        repository: Arc<dyn UserRepository>,
        jwt_utils: Arc<JwtUtils>,
        session_service: Arc<SessionService>,
        config: Arc<AuthConfig>,
    ) -> Self {
        Self {
            repository,
            _jwt_utils: jwt_utils,
            session_service,
            _config: config,
        }
    }

    pub async fn register(&self, create_user: CreateUser) -> Result<User, UserServiceError> {
        // Validate email format
        if !EMAIL_REGEX.is_match(&create_user.email) {
            return Err(UserError::InvalidEmail.into());
        }

        // Check if user already exists
        if (self.repository.find_by_email(&create_user.email).await?).is_some() {
            return Err(UserError::AlreadyExists.into());
        }

        // Validate password strength
        check_password_strength(&create_user.password, &[&create_user.email])?;

        // Hash password
        let password_hash = hash_password(&create_user.password)?;

        // Create user
        let user = User::new(create_user.email, password_hash);
        self.repository.create(&user).await?;

        Ok(user)
    }

    pub async fn login(
        &self,
        email: &str,
        password: &str,
        device_id: Option<String>,
        device_fingerprint: Option<DeviceFingerprint>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<LoginResult, UserServiceError> {
        // Get user by email
        let user = self
            .repository
            .find_by_email(email)
            .await?
            .ok_or(UserServiceError::InvalidCredentials)?;

        // Verify password
        if !verify_password(password, &user.password_hash)? {
            return Err(UserServiceError::InvalidCredentials);
        }

        // Create session with device information
        let metadata = json!({
            "login_type": "password",
            "email": email,
        });

        let (_, session_token) = self
            .session_service
            .create_session(
                user.id,
                device_id,
                device_fingerprint,
                ip_address,
                user_agent,
                Some(metadata),
            )
            .await?;

        Ok(LoginResult {
            user,
            session_token,
        })
    }

    pub async fn logout(&self, session_token: &str) -> Result<(), UserServiceError> {
        self.session_service
            .invalidate_session(session_token, SessionInvalidationReason::UserLogout)
            .await?;
        Ok(())
    }

    pub async fn validate_session(
        &self,
        session_token: &str,
    ) -> Result<Option<User>, UserServiceError> {
        let session = self.session_service.validate_session(session_token).await?;

        if let Some(session) = session {
            let user = self.repository.find_by_id(session.user_id).await?;

            Ok(user)
        } else {
            Ok(None)
        }
    }

    pub async fn get_active_sessions(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<Session>, UserServiceError> {
        let sessions = self
            .session_service
            .get_user_sessions(user_id, SessionFilter::Active)
            .await?;
        Ok(sessions)
    }

    pub async fn invalidate_all_sessions(
        &self,
        user_id: Uuid,
        reason: SessionInvalidationReason,
    ) -> Result<(), UserServiceError> {
        let sessions = self
            .session_service
            .get_user_sessions(user_id, SessionFilter::Active)
            .await?;

        for session in sessions {
            if let Err(err) = self
                .session_service
                .invalidate_session(session.token_hash.as_str(), reason.clone())
                .await
            {
                tracing::error!(
                    session_id = %session.id,
                    error = %err,
                    "Failed to invalidate session"
                );
            }
        }

        Ok(())
    }

    pub async fn get_user(&self, id: Uuid) -> Result<User, UserServiceError> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| UserError::NotFound.into())
    }

    pub async fn verify_email(&self, id: Uuid) -> Result<(), UserServiceError> {
        self.repository.verify_email(id).await?;
        Ok(())
    }

    pub async fn deactivate_user(&self, id: Uuid) -> Result<(), UserServiceError> {
        self.repository.deactivate(id).await?;
        Ok(())
    }

    pub async fn activate_user(&self, id: Uuid) -> Result<(), UserServiceError> {
        self.repository.activate(id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn test_email_regex() {
        assert!(EMAIL_REGEX.is_match("test@example.com"));
        assert!(EMAIL_REGEX.is_match("user.name+tag@example.co.uk"));
        assert!(!EMAIL_REGEX.is_match("invalid@email@example.com"));
        assert!(!EMAIL_REGEX.is_match("no@domain"));
    }

    #[test]
    fn test_user_creation() {
        let user = User::new(
            "test@example.com".to_string(),
            "hashed_password".to_string(),
        );

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.password_hash, "hashed_password");
        assert!(user.is_active);
        assert!(!user.is_verified);
        assert!(user.created_at <= SystemTime::now());
        assert!(user.updated_at <= SystemTime::now());
        assert!(user.last_login.is_none());
    }
}
