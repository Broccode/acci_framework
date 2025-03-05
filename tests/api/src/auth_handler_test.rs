// Auth handler tests
// Import directly from source modules
use acci_api::handlers::auth::{
    ApiAppState, LoginRequest, LoginResponse, RegistrationRequest, RegistrationResponse,
};
use acci_api::response::{ApiError, ApiResponse, ResponseStatus};
use acci_auth::{
    models::user::{User, UserError},
    services::{
        session::{SessionService, SessionServiceError},
        user::{LoginResult, UserService, UserServiceError},
    },
};
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use time::OffsetDateTime;
use tokio::runtime::Runtime;
use uuid::Uuid;

// Mock for the monitoring module - reduces dependencies on metrics in tests
mod monitoring {
    pub fn record_auth_operation(_operation: &str, _result: &str) {}
    pub fn record_request_duration(_duration_secs: f64, _method: &str, _path: &str) {}
}

// Test helper to create a mock user object
fn create_test_user() -> User {
    User {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        last_login: None,
        is_active: true,
        is_verified: false,
    }
}

// Helper function to create and run a tokio runtime for testing async code
fn with_runtime<F, R>(test: F) -> R
where
    F: FnOnce(&mut Runtime) -> R,
{
    let mut runtime = Runtime::new().expect("Failed to create tokio runtime");
    test(&mut runtime)
}

// Helper function to extract data from an axum response
async fn response_to_json(response: Response) -> (StatusCode, serde_json::Value) {
    let status = response.status();

    // Use http_body_util to extract the body
    let body = response.into_body();
    let bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .expect("Failed to extract body");

    // Parse the body as JSON
    let json = serde_json::from_slice(&bytes).expect("Failed to parse response as JSON");

    (status, json)
}

// ============================================================================
// Unit tests for auth handlers using test stubs
// ============================================================================

// Struct for testing the APIs directly without HTTP
struct TestUserService {
    // For login tests
    login_email: String,
    login_password: String,
    login_result: Result<LoginResult, UserServiceError>,

    // For registration tests
    register_email: String,
    register_password: String,
    register_result: Result<User, UserServiceError>,

    // For session validation tests
    validate_token: String,
    validate_result: Result<Option<User>, UserServiceError>,
}

impl TestUserService {
    fn new_login_test(
        email: &str,
        password: &str,
        result: Result<LoginResult, UserServiceError>,
    ) -> Self {
        Self {
            login_email: email.to_string(),
            login_password: password.to_string(),
            login_result: result,

            // Default values for other fields
            register_email: "".to_string(),
            register_password: "".to_string(),
            register_result: Err(UserServiceError::InvalidCredentials),
            validate_token: "".to_string(),
            validate_result: Err(UserServiceError::InvalidCredentials),
        }
    }

    fn new_register_test(
        email: &str,
        password: &str,
        result: Result<User, UserServiceError>,
    ) -> Self {
        Self {
            login_email: "".to_string(),
            login_password: "".to_string(),
            login_result: Err(UserServiceError::InvalidCredentials),

            register_email: email.to_string(),
            register_password: password.to_string(),
            register_result: result,

            validate_token: "".to_string(),
            validate_result: Err(UserServiceError::InvalidCredentials),
        }
    }

    fn new_validate_test(token: &str, result: Result<Option<User>, UserServiceError>) -> Self {
        Self {
            login_email: "".to_string(),
            login_password: "".to_string(),
            login_result: Err(UserServiceError::InvalidCredentials),

            register_email: "".to_string(),
            register_password: "".to_string(),
            register_result: Err(UserServiceError::InvalidCredentials),

            validate_token: token.to_string(),
            validate_result: result,
        }
    }

    // Test methods that match the UserService interface but return predetermined results
    async fn login(
        &self,
        email: &str,
        password: &str,
        _device_id: Option<String>,
        _device_fingerprint: Option<acci_auth::session::types::DeviceFingerprint>,
        _ip_address: Option<String>,
        _user_agent: Option<String>,
    ) -> Result<LoginResult, UserServiceError> {
        if email == self.login_email && password == self.login_password {
            // Create a new response instead of cloning to avoid the clone issue
            match &self.login_result {
                Ok(result) => {
                    // Create a new LoginResult with fresh data
                    let user = create_test_user();
                    Ok(LoginResult {
                        user,
                        session_token: result.session_token.clone(),
                    })
                },
                Err(_) => Err(UserServiceError::InvalidCredentials),
            }
        } else {
            Err(UserServiceError::InvalidCredentials)
        }
    }

    async fn register(&self, create_user: acci_auth::CreateUser) -> Result<User, UserServiceError> {
        if create_user.email == self.register_email
            && create_user.password == self.register_password
        {
            // Create a new user instead of cloning to avoid the clone issue
            match &self.register_result {
                Ok(_) => Ok(create_test_user()),
                Err(_) => Err(UserServiceError::User(UserError::AlreadyExists)),
            }
        } else {
            Err(UserServiceError::User(UserError::AlreadyExists))
        }
    }

    async fn validate_session(
        &self,
        session_token: &str,
    ) -> Result<Option<User>, UserServiceError> {
        if session_token == self.validate_token {
            // Create a new response instead of cloning to avoid the clone issue
            match &self.validate_result {
                Ok(Some(_)) => Ok(Some(create_test_user())),
                Ok(None) => Ok(None),
                Err(_) => Err(UserServiceError::InvalidCredentials),
            }
        } else {
            Err(UserServiceError::InvalidCredentials)
        }
    }
}

// Test session service - just a stub for the state
struct TestSessionService {}

impl TestSessionService {
    fn new() -> Self {
        Self {}
    }
}

// Create a test-specific AppState that reuses the ApiAppState name for tests
#[derive(Clone)]
struct TestApiAppState {
    user_service: Arc<TestUserService>,
    session_service: Arc<TestSessionService>,
}

// Implement a conversion from TestApiAppState to the ApiAppState from the handlers module
impl From<TestApiAppState> for State<TestApiAppState> {
    fn from(state: TestApiAppState) -> Self {
        State(state)
    }
}

// Helper to run API handler functions directly
fn create_test_app_state(user_service: TestUserService) -> TestApiAppState {
    TestApiAppState {
        user_service: Arc::new(user_service),
        session_service: Arc::new(TestSessionService::new()),
    }
}

// This test is left as a stub as we can't directly test the handlers without
// properly mocking the services. A different approach using integration tests
// with the real services or more sophisticated mocking would be needed.
// Test-specific implementations of the API handlers
async fn test_api_login(state: &TestApiAppState, request: LoginRequest) -> Response {
    // Create a unique request ID for tests
    let request_id = "test-request-id".to_string();

    // Validate the request payload manually for tests
    if request.email.is_empty() || request.password.is_empty() {
        return ApiError::new(
            StatusCode::BAD_REQUEST,
            "Invalid request data",
            "VALIDATION_ERROR",
            request_id,
        )
        .into_response();
    }

    // Attempt login
    let login_result = state
        .user_service
        .login(
            &request.email,
            &request.password,
            None, // device_id
            None, // device_fingerprint
            None, // ip_address
            None, // user_agent
        )
        .await;

    match login_result {
        Ok(login_result) => {
            // Successful login
            let response = LoginResponse {
                token: login_result.session_token,
                user_id: login_result.user.id.to_string(),
                expires_at: 0, // Default for tests
            };

            let api_response = ApiResponse::success(response, request_id);
            (StatusCode::OK, Json(api_response)).into_response()
        },
        Err(err) => {
            // Login error
            let (status, message, code) = match err {
                UserServiceError::InvalidCredentials => (
                    StatusCode::UNAUTHORIZED,
                    "Invalid email or password",
                    "INVALID_CREDENTIALS",
                ),
                UserServiceError::User(UserError::InactiveUser) => {
                    (StatusCode::FORBIDDEN, "Account is locked", "ACCOUNT_LOCKED")
                },
                UserServiceError::User(UserError::UnverifiedUser) => (
                    StatusCode::FORBIDDEN,
                    "Account is not verified",
                    "ACCOUNT_UNVERIFIED",
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An error occurred during login",
                    "LOGIN_ERROR",
                ),
            };

            ApiError::new(status, message, code, request_id).into_response()
        },
    }
}

async fn test_api_register(state: &TestApiAppState, request: RegistrationRequest) -> Response {
    // Create a unique request ID for tests
    let request_id = "test-request-id".to_string();

    // Simple validation for tests
    if request.email.is_empty() || request.password.is_empty() {
        return ApiError::new(
            StatusCode::BAD_REQUEST,
            "Invalid request data",
            "VALIDATION_ERROR",
            request_id,
        )
        .into_response();
    }

    // Check password confirmation
    if request.password != request.password_confirmation {
        return ApiError::new(
            StatusCode::BAD_REQUEST,
            "Passwords do not match",
            "PASSWORD_MISMATCH",
            request_id,
        )
        .into_response();
    }

    // Create new user
    let create_user = acci_auth::CreateUser {
        email: request.email.clone(),
        password: request.password.clone(),
    };

    match state.user_service.register(create_user).await {
        Ok(user) => {
            let response = RegistrationResponse {
                user_id: user.id.to_string(),
                email: user.email,
            };

            let api_response = ApiResponse::success(response, request_id);
            (StatusCode::CREATED, Json(api_response)).into_response()
        },
        Err(err) => {
            // Registration error
            let (status, message, code) = match err {
                UserServiceError::User(UserError::AlreadyExists) => (
                    StatusCode::CONFLICT,
                    "User with this email already exists",
                    "USER_ALREADY_EXISTS",
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An error occurred during registration",
                    "REGISTRATION_ERROR",
                ),
            };

            ApiError::new(status, message, code, request_id).into_response()
        },
    }
}

async fn test_validate_token(state: &TestApiAppState, token: String) -> Response {
    // Create a unique request ID for tests
    let request_id = "test-request-id".to_string();

    // In our test implementation, we forward directly to the validate_session method
    match state.user_service.validate_session(&token).await {
        Ok(Some(_)) => {
            let api_response = ApiResponse::success(true, request_id);
            (StatusCode::OK, Json(api_response)).into_response()
        },
        _ => ApiError::new(
            StatusCode::UNAUTHORIZED,
            "Invalid or expired token",
            "INVALID_TOKEN",
            request_id,
        )
        .into_response(),
    }
}

// Now the real tests
#[test]
fn test_successful_login() {
    with_runtime(|rt| {
        // Arrange
        let user = create_test_user();
        let login_result = LoginResult {
            user: user.clone(),
            session_token: "test-session-token".to_string(),
        };

        let test_user_service =
            TestUserService::new_login_test("test@example.com", "password123", Ok(login_result));

        let state = create_test_app_state(test_user_service);

        let login_request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        // Act
        let response = rt.block_on(async { test_api_login(&state, login_request).await });

        // Assert
        let status = response.status();
        assert_eq!(status, StatusCode::OK);

        // Extract and verify response body
        let (_, json) = rt.block_on(async { response_to_json(response).await });

        // Check that we got a success response with the token
        assert_eq!(json["status"], "success");
        assert_eq!(json["data"]["token"], "test-session-token");
    });
}

#[test]
fn test_login_invalid_credentials() {
    with_runtime(|rt| {
        // Arrange
        let test_user_service = TestUserService::new_login_test(
            "test@example.com",
            "password123",
            Err(UserServiceError::InvalidCredentials),
        );

        let state = create_test_app_state(test_user_service);

        let login_request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        // Act
        let response = rt.block_on(async { test_api_login(&state, login_request).await });

        // Assert
        let status = response.status();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_successful_registration() {
    with_runtime(|rt| {
        // Arrange
        let test_user_service = TestUserService::new_register_test(
            "new@example.com",
            "password123",
            Ok(create_test_user()),
        );

        let state = create_test_app_state(test_user_service);

        let registration_request = RegistrationRequest {
            email: "new@example.com".to_string(),
            password: "password123".to_string(),
            password_confirmation: "password123".to_string(),
        };

        // Act
        let response = rt.block_on(async { test_api_register(&state, registration_request).await });

        // Assert
        let status = response.status();
        assert_eq!(status, StatusCode::CREATED);
    });
}

#[test]
fn test_registration_password_mismatch() {
    with_runtime(|rt| {
        // Arrange
        // We don't need a proper user service for this test since it fails before calling it
        let test_user_service = TestUserService::new_register_test(
            "new@example.com",
            "password123",
            Ok(create_test_user()),
        );

        let state = create_test_app_state(test_user_service);

        let registration_request = RegistrationRequest {
            email: "new@example.com".to_string(),
            password: "password123".to_string(),
            password_confirmation: "different_password".to_string(), // Different password
        };

        // Act
        let response = rt.block_on(async { test_api_register(&state, registration_request).await });

        // Assert
        let status = response.status();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    });
}

#[test]
fn test_registration_user_already_exists() {
    with_runtime(|rt| {
        // Arrange
        let test_user_service = TestUserService::new_register_test(
            "existing@example.com",
            "password123",
            Err(UserServiceError::User(UserError::AlreadyExists)),
        );

        let state = create_test_app_state(test_user_service);

        let registration_request = RegistrationRequest {
            email: "existing@example.com".to_string(),
            password: "password123".to_string(),
            password_confirmation: "password123".to_string(),
        };

        // Act
        let response = rt.block_on(async { test_api_register(&state, registration_request).await });

        // Assert
        let status = response.status();
        assert_eq!(status, StatusCode::CONFLICT);
    });
}

#[test]
fn test_token_validation_success() {
    with_runtime(|rt| {
        // Arrange
        let test_user_service =
            TestUserService::new_validate_test("valid-token", Ok(Some(create_test_user())));

        let state = create_test_app_state(test_user_service);

        // Act
        let response =
            rt.block_on(async { test_validate_token(&state, "valid-token".to_string()).await });

        // Assert
        let status = response.status();
        assert_eq!(status, StatusCode::OK);
    });
}

#[test]
fn test_token_validation_invalid_token() {
    with_runtime(|rt| {
        // Arrange
        let test_user_service = TestUserService::new_validate_test(
            "valid-token",
            Err(UserServiceError::InvalidCredentials),
        );

        let state = create_test_app_state(test_user_service);

        // Act
        let response =
            rt.block_on(async { test_validate_token(&state, "invalid-token".to_string()).await });

        // Assert
        let status = response.status();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    });
}

// Note: We would need a more sophisticated approach like testcontainers
// or a full integration test suite to properly test these handlers with
// the database and actual services. The direct testing of handlers
// with mocks is challenging given the use of concrete service types
// rather than traits in the app state.
