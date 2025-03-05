use acci_api::{config::ApiConfig, handlers::auth::ApiAppState, router::ApiRouter};
use acci_auth::{
    models::user::User,
    services::{
        session::SessionService,
        user::{LoginResult, UserService, UserServiceError},
    },
};
use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    response::Response,
};
use mockall::mock;
use mockall::predicate::*;
use serde_json::{Value, json};
use std::sync::Arc;
use time::OffsetDateTime;
use tower::ServiceExt;
use uuid::Uuid;

// Mock for UserService
mock! {
    pub UserService {}

    impl UserService {
        pub async fn login(
            &self,
            email: &str,
            password: &str,
            device_id: Option<String>,
            device_fingerprint: Option<acci_auth::session::types::DeviceFingerprint>,
            ip_address: Option<String>,
            user_agent: Option<String>
        ) -> Result<LoginResult, UserServiceError>;

        pub async fn register(
            &self,
            create_user: acci_auth::CreateUser,
        ) -> Result<User, UserServiceError>;

        pub async fn validate_session(
            &self,
            session_token: &str,
        ) -> Result<Option<User>, UserServiceError>;
    }
}

// Mock for SessionService
mock! {
    pub SessionService {}

    impl SessionService {
        // We don't need any specific methods for our tests here
    }
}

// Helper function to create test dependencies
fn create_test_dependencies() -> (ApiAppState, ApiConfig) {
    let mock_user_service = MockUserService::new();
    let mock_session_service = MockSessionService::new();

    let app_state = ApiAppState {
        user_service: Arc::new(mock_user_service),
        session_service: Arc::new(mock_session_service),
    };

    let config = ApiConfig {
        base_path: "/api/v1".to_string(),
        // Add other config options as needed
    };

    (app_state, config)
}

// Helper function to extract JSON from response
async fn extract_json(response: Response) -> Value {
    let bytes = hyper::body::to_bytes(response.into_body())
        .await
        .expect("Failed to extract body bytes");

    serde_json::from_slice(&bytes).expect("Failed to parse response as JSON")
}

#[tokio::test]
async fn test_router_health_endpoint() {
    // Arrange
    let (app_state, config) = create_test_dependencies();
    let router = ApiRouter::new(config.clone());
    let app = router.create_router_with_state(app_state);

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    assert_eq!(body_str, "OK");
}

#[tokio::test]
async fn test_router_example_endpoint() {
    // Arrange
    let (app_state, config) = create_test_dependencies();
    let router = ApiRouter::new(config.clone());
    let app = router.create_router_with_state(app_state);

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/example")
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let json = extract_json(response).await;

    // Check response structure
    assert_eq!(json["status"], "success");
    assert!(json["data"]["message"].is_string());
    assert!(json["data"]["timestamp"].is_string());
    assert!(json["request_id"].is_string());
}

#[tokio::test]
async fn test_router_auth_endpoints_exist() {
    // Arrange
    let (app_state, config) = create_test_dependencies();
    let router = ApiRouter::new(config.clone());
    let app = router.create_router_with_state(app_state);

    // Test login endpoint - expect method not allowed for GET
    let login_response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/login")
                .method(Method::GET) // Should be POST
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert method not allowed (since we used GET)
    assert_eq!(login_response.status(), StatusCode::METHOD_NOT_ALLOWED);

    // Test register endpoint - expect method not allowed for GET
    let register_response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/register")
                .method(Method::GET) // Should be POST
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert method not allowed (since we used GET)
    assert_eq!(register_response.status(), StatusCode::METHOD_NOT_ALLOWED);

    // Test validate-token endpoint - expect method not allowed for GET
    let validate_response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/validate-token")
                .method(Method::GET) // Should be POST
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert method not allowed (since we used GET)
    assert_eq!(validate_response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn test_router_nonexistent_endpoint() {
    // Arrange
    let (app_state, config) = create_test_dependencies();
    let router = ApiRouter::new(config.clone());
    let app = router.create_router_with_state(app_state);

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/nonexistent")
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_router_base_path_handling() {
    // Test with empty base path
    let (app_state, _) = create_test_dependencies();
    let empty_config = ApiConfig {
        base_path: "".to_string(),
    };

    let router = ApiRouter::new(empty_config);
    let app = router.create_router_with_state(app_state.clone());

    // With empty base path, health should be at /health
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Test with custom base path
    let custom_config = ApiConfig {
        base_path: "/custom/api".to_string(),
    };

    let router = ApiRouter::new(custom_config);
    let app = router.create_router_with_state(app_state);

    // With custom base path, health should be at /custom/api/health
    let response = app
        .oneshot(
            Request::builder()
                .uri("/custom/api/health")
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_router_without_state() {
    // Test router without state (fallback for compatibility)
    let config = ApiConfig {
        base_path: "/api/v1".to_string(),
    };

    let router = ApiRouter::new(config);
    let app = router.create_router();

    // Health endpoint should still work
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Example endpoint should still work
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/example")
                .method(Method::GET)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Auth endpoints should not be available
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/login")
                .method(Method::POST)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "email": "test@example.com",
                        "password": "password"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
