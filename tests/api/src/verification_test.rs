use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

use acci_api::{
    config::ApiConfig,
    handlers::verification::{VerificationAppState, send_verification, verify_code},
    router::ApiRouter,
};

use acci_auth::{
    models::{TenantId, UserId, VerificationCode, VerificationConfig, VerificationType},
    repository::TenantAwareContext,
    services::{session::SessionService, verification::VerificationService},
};

// Import mocks from our mocks module
use crate::mocks::{MockMessageProvider, MockSessionRepository, MockVerificationCodeRepository};

// Test the verification endpoints
#[tokio::test]
async fn test_verification_endpoints() {
    // Create a mock context for tests
    let tenant_context = TenantAwareContext::new_with_pool(None);

    // Create a mock verification code repository
    let verification_repo = Arc::new(MockVerificationCodeRepository::new());

    // Create a mock email provider
    let email_provider = Arc::new(MockMessageProvider::new());

    // Create a mock session repository
    let session_repo = Arc::new(MockSessionRepository::new());

    // Create a verification service with the mocks
    let verification_service = Arc::new(VerificationService::new(
        verification_repo.clone(),
        VerificationConfig::default(),
        None, // No SMS provider
        Some(email_provider.clone()),
    ));

    // Create a session service with the mock
    let session_service = Arc::new(SessionService::new(
        session_repo.clone(),
        Arc::new(acci_auth::config::AuthConfig::default()),
    ));

    // Create the verification state
    let verification_state = VerificationAppState {
        verification_service: verification_service.clone(),
        session_service: session_service.clone(),
        tenant_context: tenant_context.clone(),
    };

    // Create the API router
    let api_config = ApiConfig {
        base_path: "".to_string(),
    };
    let api_router = ApiRouter::new(api_config);

    // Create the router with verification routes
    let app = api_router.create_router_with_state(
        acci_api::handlers::auth::ApiAppState {
            user_service: Arc::new(acci_auth::services::user::UserService::new(
                Arc::new(acci_auth::repository::postgres::PostgresUserRepository::new(None)),
                Arc::new(acci_auth::config::AuthConfig::default()),
            )),
            session_service: session_service.clone(),
        },
        None, // No tenant state
        Some(verification_state.clone()),
    );

    // Test IDs
    let user_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();

    // Create a test request to send a verification code
    let request = Request::builder()
        .uri("/auth/verify/send")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "user_id": user_id.to_string(),
                "verification_type": "email",
                "recipient": "test@example.com",
                "tenant_id": tenant_id.to_string(),
            })
            .to_string(),
        ))
        .unwrap();

    // Make the request and get the response
    let response = app.clone().oneshot(request).await.unwrap();

    // Check the response status
    assert_eq!(response.status(), StatusCode::OK);

    // Get the response body and parse it
    let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();

    // Verify the response
    assert_eq!(body["success"], json!(true));
    assert_eq!(body["data"]["user_id"], json!(user_id.to_string()));
    assert_eq!(body["data"]["verification_type"], json!("email"));

    // Get the verification code from the mock repository
    let codes = verification_repo.get_codes();
    assert_eq!(codes.len(), 1);

    let code = &codes[0].code;

    // Create a test request to verify the code
    let request = Request::builder()
        .uri("/auth/verify/code")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "user_id": user_id.to_string(),
                "verification_type": "email",
                "code": code,
                "tenant_id": tenant_id.to_string(),
            })
            .to_string(),
        ))
        .unwrap();

    // Make the request and get the response
    let response = app.oneshot(request).await.unwrap();

    // Check the response status
    assert_eq!(response.status(), StatusCode::OK);

    // Get the response body and parse it
    let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body_bytes).unwrap();

    // Verify the response
    assert_eq!(body["success"], json!(true));
    assert_eq!(body["data"]["user_id"], json!(user_id.to_string()));
    assert_eq!(body["data"]["verification_type"], json!("email"));

    // Check the verification code status in the repository
    let codes = verification_repo.get_codes();
    assert_eq!(codes.len(), 1);
    assert_eq!(
        codes[0].status,
        acci_auth::models::VerificationStatus::Verified
    );
}

// The mock implementations are now in the mocks module
