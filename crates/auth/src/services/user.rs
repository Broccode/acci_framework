use crate::{
    models::user::{CreateUser, LoginCredentials, User, UserError, UserRepository},
    utils::{
        jwt::{JwtError, JwtUtils},
        password::{PasswordError, check_password_strength, hash_password, verify_password},
    },
};
use regex::Regex;
use std::sync::Arc;
use uuid::Uuid;

lazy_static::lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
    ).unwrap();
}

#[derive(Debug, thiserror::Error)]
pub enum UserServiceError {
    #[error(transparent)]
    User(#[from] UserError),
    #[error(transparent)]
    Password(#[from] PasswordError),
    #[error(transparent)]
    Jwt(#[from] JwtError),
}

pub struct UserService {
    repository: Arc<dyn UserRepository>,
    jwt_utils: Arc<JwtUtils>,
}

impl UserService {
    pub fn new(repository: Arc<dyn UserRepository>, jwt_utils: Arc<JwtUtils>) -> Self {
        Self {
            repository,
            jwt_utils,
        }
    }

    pub async fn register(&self, create_user: CreateUser) -> Result<User, UserServiceError> {
        // Validate email format
        if !EMAIL_REGEX.is_match(&create_user.email) {
            return Err(UserError::InvalidEmail.into());
        }

        // Check if user already exists
        if let Some(_) = self.repository.find_by_email(&create_user.email).await? {
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

    pub async fn login(&self, credentials: LoginCredentials) -> Result<String, UserServiceError> {
        // Find user by email
        let mut user = self
            .repository
            .find_by_email(&credentials.email)
            .await?
            .ok_or(UserError::InvalidCredentials)?;

        // Check if user is active
        if !user.is_active {
            return Err(UserError::InactiveUser.into());
        }

        // Verify password
        if !verify_password(&credentials.password, &user.password_hash)? {
            return Err(UserError::InvalidCredentials.into());
        }

        // Update last login
        user.update_last_login();
        self.repository.update(&user).await?;

        // Generate JWT token
        let token = self.jwt_utils.create_token(user.id, &user.email)?;

        Ok(token)
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
    use crate::models::user::mock::MockUserRepository;

    #[tokio::test]
    async fn test_user_registration() {
        let repository = Arc::new(MockUserRepository::new());
        let jwt_utils = Arc::new(JwtUtils::new(b"test-secret"));
        let service = UserService::new(repository.clone(), jwt_utils);

        let create_user = CreateUser {
            email: "test@example.com".to_string(),
            password: "StrongP@ssw0rd123!".to_string(),
        };

        let user = service.register(create_user.clone()).await.unwrap();
        assert_eq!(user.email, "test@example.com");
        assert!(user.is_active);
        assert!(!user.is_verified);

        // Test duplicate registration
        let result = service.register(create_user).await;
        assert!(matches!(
            result,
            Err(UserServiceError::User(UserError::AlreadyExists))
        ));
    }

    #[tokio::test]
    async fn test_user_login() {
        let repository = Arc::new(MockUserRepository::new());
        let jwt_utils = Arc::new(JwtUtils::new(b"test-secret"));
        let service = UserService::new(repository.clone(), jwt_utils);

        // Register user
        let create_user = CreateUser {
            email: "test@example.com".to_string(),
            password: "StrongP@ssw0rd123!".to_string(),
        };
        let user = service.register(create_user).await.unwrap();

        // Test successful login
        let credentials = LoginCredentials {
            email: "test@example.com".to_string(),
            password: "StrongP@ssw0rd123!".to_string(),
        };
        let token = service.login(credentials).await.unwrap();
        assert!(!token.is_empty());

        // Test invalid credentials
        let invalid_credentials = LoginCredentials {
            email: "test@example.com".to_string(),
            password: "WrongPassword".to_string(),
        };
        let result = service.login(invalid_credentials).await;
        assert!(matches!(
            result,
            Err(UserServiceError::User(UserError::InvalidCredentials))
        ));

        // Test inactive user
        service.deactivate_user(user.id).await.unwrap();
        let credentials = LoginCredentials {
            email: "test@example.com".to_string(),
            password: "StrongP@ssw0rd123!".to_string(),
        };
        let result = service.login(credentials).await;
        assert!(matches!(
            result,
            Err(UserServiceError::User(UserError::InactiveUser))
        ));
    }
}
