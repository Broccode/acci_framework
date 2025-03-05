use acci_auth::{
    AuthConfig,
    models::user::{CreateUser, MockUserRepository, User, UserRepository},
    services::user::{LoginResult, UserService, UserServiceError},
    session::{SessionService, mock::MockSessionRepository, types::DeviceFingerprint},
    utils::jwt::JwtUtils,
};
use std::sync::Arc;
use uuid::Uuid;

fn create_test_config() -> Arc<AuthConfig> {
    Arc::new(AuthConfig {
        session_lifetime_secs: 3600,
        ..Default::default()
    })
}

#[tokio::test]
async fn test_register_user() {
    // Setup
    let user_repo = Arc::new(MockUserRepository::new());
    let session_repo = Arc::new(MockSessionRepository::new());
    let config = create_test_config();
    let jwt_utils = Arc::new(JwtUtils::new(&config));
    let session_service = Arc::new(SessionService::new(session_repo, config.clone()));
    let user_service = UserService::new(user_repo.clone(), jwt_utils, session_service, config);

    // Test data
    let create_user = CreateUser {
        email: "test@example.com".to_string(),
        password: "StrongPassword123!".to_string(),
    };

    // Execute
    let result = user_service.register(create_user).await;

    // Verify
    assert!(result.is_ok(), "User registration should succeed");
    let user = result.unwrap();
    assert_eq!(user.email, "test@example.com");
    assert!(user.is_active);
    assert!(!user.is_verified);
}

#[tokio::test]
async fn test_register_existing_user() {
    // Setup
    let user_repo = Arc::new(MockUserRepository::new());
    let session_repo = Arc::new(MockSessionRepository::new());
    let config = create_test_config();
    let jwt_utils = Arc::new(JwtUtils::new(&config));
    let session_service = Arc::new(SessionService::new(session_repo, config.clone()));
    let user_service = UserService::new(user_repo.clone(), jwt_utils, session_service, config);

    // Register user first time
    let create_user = CreateUser {
        email: "existing@example.com".to_string(),
        password: "StrongPassword123!".to_string(),
    };
    let _ = user_service.register(create_user.clone()).await;

    // Try to register again with same email
    let result = user_service.register(create_user).await;

    // Verify
    assert!(result.is_err(), "Registering existing user should fail");
    match result {
        Err(UserServiceError::User(err)) => {
            assert!(format!("{}", err).contains("already exists"));
        },
        _ => panic!("Expected UserError::AlreadyExists"),
    }
}

#[tokio::test]
async fn test_register_with_weak_password() {
    // Setup
    let user_repo = Arc::new(MockUserRepository::new());
    let session_repo = Arc::new(MockSessionRepository::new());
    let config = create_test_config();
    let jwt_utils = Arc::new(JwtUtils::new(&config));
    let session_service = Arc::new(SessionService::new(session_repo, config.clone()));
    let user_service = UserService::new(user_repo.clone(), jwt_utils, session_service, config);

    // Test data with weak password
    let create_user = CreateUser {
        email: "test@example.com".to_string(),
        password: "123456".to_string(), // Weak password
    };

    // Execute
    let result = user_service.register(create_user).await;

    // Verify
    assert!(
        result.is_err(),
        "Registration with weak password should fail"
    );
    match result {
        Err(UserServiceError::Password(err)) => {
            // Password error should contain information about weak password
            assert!(format!("{}", err).contains("password"));
        },
        _ => panic!("Expected PasswordError for weak password"),
    }
}

#[tokio::test]
async fn test_login() {
    // Setup
    let user_repo = Arc::new(MockUserRepository::new());
    let session_repo = Arc::new(MockSessionRepository::new());
    let config = create_test_config();
    let jwt_utils = Arc::new(JwtUtils::new(&config));
    let session_service = Arc::new(SessionService::new(session_repo, config.clone()));
    let user_service = UserService::new(
        user_repo.clone(),
        jwt_utils,
        session_service.clone(),
        config,
    );

    // Create a user first
    let create_user = CreateUser {
        email: "login@example.com".to_string(),
        password: "StrongPassword123!".to_string(),
    };
    let user = user_service.register(create_user).await.unwrap();

    // Login
    let result = user_service
        .login(
            "login@example.com",
            "StrongPassword123!",
            None,
            None,
            None,
            None,
        )
        .await;

    // Verify
    assert!(result.is_ok(), "Login should succeed");
    let login_result = result.unwrap();
    assert_eq!(login_result.user.email, "login@example.com");
    assert!(
        !login_result.session_token.is_empty(),
        "Session token should be generated"
    );
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    // Setup
    let user_repo = Arc::new(MockUserRepository::new());
    let session_repo = Arc::new(MockSessionRepository::new());
    let config = create_test_config();
    let jwt_utils = Arc::new(JwtUtils::new(&config));
    let session_service = Arc::new(SessionService::new(session_repo, config.clone()));
    let user_service = UserService::new(user_repo.clone(), jwt_utils, session_service, config);

    // Create a user first
    let create_user = CreateUser {
        email: "invalid@example.com".to_string(),
        password: "StrongPassword123!".to_string(),
    };
    let _ = user_service.register(create_user).await.unwrap();

    // Login with wrong password
    let result = user_service
        .login(
            "invalid@example.com",
            "WrongPassword123!",
            None,
            None,
            None,
            None,
        )
        .await;

    // Verify
    assert!(result.is_err(), "Login with wrong password should fail");
    match result {
        Err(UserServiceError::InvalidCredentials) => {},
        _ => panic!("Expected InvalidCredentials error"),
    }
}

#[tokio::test]
async fn test_logout() {
    // Setup
    let user_repo = Arc::new(MockUserRepository::new());
    let session_repo = Arc::new(MockSessionRepository::new());
    let config = create_test_config();
    let jwt_utils = Arc::new(JwtUtils::new(&config));
    let session_service = Arc::new(SessionService::new(session_repo, config.clone()));
    let user_service = UserService::new(
        user_repo.clone(),
        jwt_utils,
        session_service.clone(),
        config,
    );

    // Create a user and login
    let create_user = CreateUser {
        email: "logout@example.com".to_string(),
        password: "StrongPassword123!".to_string(),
    };
    let _ = user_service.register(create_user).await.unwrap();
    let login_result = user_service
        .login(
            "logout@example.com",
            "StrongPassword123!",
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

    // Logout
    let result = user_service.logout(&login_result.session_token).await;

    // Verify
    assert!(result.is_ok(), "Logout should succeed");

    // Validate the session should now be invalid
    let validation = user_service
        .validate_session(&login_result.session_token)
        .await;
    assert!(validation.is_ok());
    assert!(
        validation.unwrap().is_none(),
        "Session should be invalidated after logout"
    );
}

#[tokio::test]
async fn test_validate_session() {
    // Setup
    let user_repo = Arc::new(MockUserRepository::new());
    let session_repo = Arc::new(MockSessionRepository::new());
    let config = create_test_config();
    let jwt_utils = Arc::new(JwtUtils::new(&config));
    let session_service = Arc::new(SessionService::new(session_repo, config.clone()));
    let user_service = UserService::new(
        user_repo.clone(),
        jwt_utils,
        session_service.clone(),
        config,
    );

    // Create a user and login
    let create_user = CreateUser {
        email: "validate@example.com".to_string(),
        password: "StrongPassword123!".to_string(),
    };
    let user = user_service.register(create_user).await.unwrap();
    let login_result = user_service
        .login(
            "validate@example.com",
            "StrongPassword123!",
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

    // Validate session
    let result = user_service
        .validate_session(&login_result.session_token)
        .await;

    // Verify
    assert!(result.is_ok(), "Session validation should succeed");
    let validated_user = result.unwrap();
    assert!(
        validated_user.is_some(),
        "User should be returned from validation"
    );
    assert_eq!(validated_user.unwrap().id, user.id);
}
