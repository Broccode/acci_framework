use std::sync::Arc;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

use acci_auth::{
    AuthConfig,
    models::user::{CreateUser, User},
    services::{
        session::SessionService,
        user::{UserService, UserServiceError},
    },
    session::{Session, types::SessionInvalidationReason},
    utils::jwt::JwtUtils,
};

use crate::mocks::{MockSessionRepository, MockUserRepository};

fn create_test_config() -> Arc<AuthConfig> {
    Arc::new(AuthConfig {
        session_lifetime_secs: 3600,
        ..Default::default()
    })
}

fn create_test_user() -> User {
    User::new(
        "test@example.com".to_string(),
        "$argon2id$v=19$m=4096,t=3,p=1$salt$hash".to_string(),
    )
}

#[tokio::test]
async fn test_user_registration() {
    let repository = Arc::new(MockUserRepository::new());
    let jwt_utils = Arc::new(JwtUtils::new(b"test-secret"));
    let service = UserService::new(
        repository.clone(),
        jwt_utils,
        Arc::new(SessionService::new(
            Arc::new(MockSessionRepository::new()),
            create_test_config(),
        )),
        create_test_config(),
    );

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
async fn test_login_success() {
    let mut mock_user_repo = MockUserRepository::new();
    let mut mock_session_repo = MockSessionRepository::new();
    let config = create_test_config();

    let test_user = create_test_user();
    let test_email = test_user.email.clone();
    let test_password = "correct_password";

    mock_user_repo
        .expect_get_by_email()
        .with(eq(test_email.clone()))
        .returning(move |_| Ok(Some(test_user.clone())));

    mock_session_repo
        .expect_create_session()
        .returning(|user_id, _, _, _, _, _, _, _| {
            Ok(Session {
                id: Uuid::new_v4(),
                user_id,
                token_hash: "test_hash".to_string(),
                previous_token_hash: None,
                token_rotation_at: None,
                expires_at: SystemTime::now() + Duration::from_secs(3600),
                created_at: SystemTime::now(),
                last_activity_at: SystemTime::now(),
                last_activity_update_at: None,
                ip_address: None,
                user_agent: None,
                device_id: None,
                device_fingerprint: None,
                is_valid: true,
                invalidated_reason: None,
                metadata: None,
            })
        });

    let session_service = Arc::new(SessionService::new(
        Arc::new(mock_session_repo),
        config.clone(),
    ));
    let service = UserService::new(
        Arc::new(mock_user_repo),
        Arc::new(JwtUtils::new(b"test-secret")),
        session_service,
        config,
    );

    let result = service
        .login(
            &test_email,
            test_password,
            None,
            None,
            Some("127.0.0.1".to_string()),
            Some("Test Agent".to_string()),
        )
        .await;

    assert!(result.is_ok());
    let login_result = result.unwrap();
    assert_eq!(login_result.user.email, test_email);
    assert!(!login_result.session_token.is_empty());
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let mut mock_user_repo = MockUserRepository::new();
    let mock_session_repo = MockSessionRepository::new();
    let config = create_test_config();

    let test_email = "test@example.com";
    let test_password = "wrong_password";

    mock_user_repo
        .expect_get_by_email()
        .with(eq(test_email))
        .returning(|_| Ok(None));

    let session_service = Arc::new(SessionService::new(
        Arc::new(mock_session_repo),
        config.clone(),
    ));
    let service = UserService::new(
        Arc::new(mock_user_repo),
        Arc::new(JwtUtils::new(b"test-secret")),
        session_service,
        config,
    );

    let result = service
        .login(test_email, test_password, None, None, None, None)
        .await;

    assert!(matches!(result, Err(UserServiceError::InvalidCredentials)));
}

#[tokio::test]
async fn test_logout() {
    let mock_user_repo = MockUserRepository::new();
    let mut mock_session_repo = MockSessionRepository::new();
    let config = create_test_config();

    mock_session_repo
        .expect_get_session_by_token()
        .returning(|_| {
            Ok(Some(Session {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                token_hash: "test_hash".to_string(),
                previous_token_hash: None,
                token_rotation_at: None,
                expires_at: SystemTime::now() + Duration::from_secs(3600),
                created_at: SystemTime::now(),
                last_activity_at: SystemTime::now(),
                last_activity_update_at: None,
                ip_address: None,
                user_agent: None,
                device_id: None,
                device_fingerprint: None,
                is_valid: true,
                invalidated_reason: None,
                metadata: None,
            }))
        });

    mock_session_repo
        .expect_invalidate_session()
        .returning(|_, _| Ok(()));

    let session_service = Arc::new(SessionService::new(
        Arc::new(mock_session_repo),
        config.clone(),
    ));
    let service = UserService::new(
        Arc::new(mock_user_repo),
        Arc::new(JwtUtils::new(b"test-secret")),
        session_service,
        config,
    );

    let result = service.logout("test_token").await;
    assert!(result.is_ok());
}
