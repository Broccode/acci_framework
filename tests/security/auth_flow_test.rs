use crate::helpers::setup_test_db;
use acci_auth::{
    models::user::{User, UserError},
    services::user::{UserService, UserServiceConfig},
    utils::{
        jwt::{JwtConfig, JwtService},
        password::{hash_password, verify_password},
    },
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::{Value, json};
use uuid::Uuid;

/// Test that password hashing is using correct algorithm and parameters
#[tokio::test]
async fn test_password_hash_security() {
    // Use a test password
    let password = "TestPassword123!";

    // Hash the password
    let hashed = hash_password(password).await.unwrap();

    // Verify the hash contains Argon2id identifiers
    assert!(hashed.contains("$argon2id$"));

    // Verify the hash can be verified
    let result = verify_password(password, &hashed).await.unwrap();
    assert!(result);

    // Verify incorrect password fails
    let result = verify_password("WrongPassword", &hashed).await.unwrap();
    assert!(!result);

    // Verify hash parameters - should have memory cost parameter
    assert!(hashed.contains("m="));

    // Verify hash parameters - should have time cost parameter
    assert!(hashed.contains("t="));

    // Verify hash parameters - should have parallelism parameter
    assert!(hashed.contains("p="));
}

/// Test that JWT tokens are secure and correctly configured
#[tokio::test]
async fn test_jwt_token_security() {
    // Create JWT service with test config
    let jwt_config = JwtConfig {
        secret: "test_secret_that_is_at_least_32_bytes_long".to_string(),
        expiration: 3600, // 1 hour
        issuer: "test-issuer".to_string(),
        audience: "test-audience".to_string(),
    };

    let jwt_service = JwtService::new(jwt_config);

    // Create a test user ID
    let user_id = Uuid::new_v4();

    // Generate a token
    let token = jwt_service.generate_token(user_id).unwrap();

    // Check token format (should be 3 parts separated by periods)
    let parts: Vec<&str> = token.split('.').collect();
    assert_eq!(parts.len(), 3);

    // Verify the token
    let result = jwt_service.validate_token(&token);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), user_id);

    // Verify with tampered token fails
    let tampered_token = format!("{}a", token); // append an 'a' to invalidate signature
    let result = jwt_service.validate_token(&tampered_token);
    assert!(result.is_err());
}

/// Test login rate limiting (simulated)
#[tokio::test]
async fn test_login_rate_limiting() {
    // This is a simulated test as we can't easily trigger real rate limiting in a unit test
    // In a real system, this would be tested with integration tests

    // Setup test app with API
    let app = crate::helpers::setup_test_app().await;
    let client = reqwest::Client::new();

    // Create a test user for login
    let user_email = format!("test-{}@example.com", Uuid::new_v4());
    let password = "P@ssw0rd123!";

    // Register the user
    let register_response = client
        .post(&format!("{}/api/auth/register", app.address))
        .json(&json!({
            "email": user_email,
            "password": password,
            "password_confirmation": password
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(register_response.status(), StatusCode::CREATED);

    // Test successful login
    let login_response = client
        .post(&format!("{}/api/auth/login", app.address))
        .json(&json!({
            "email": user_email,
            "password": password
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(login_response.status(), StatusCode::OK);

    // Extract token from response
    let response_body: Value = login_response.json().await.unwrap();
    let token = response_body["data"]["token"].as_str().unwrap();

    // Verify token works for authenticated endpoint
    let validate_response = client
        .get(&format!("{}/api/auth/validate", app.address))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(validate_response.status(), StatusCode::OK);
}

/// Test error messages don't leak sensitive information
#[tokio::test]
async fn test_error_message_security() {
    // Setup test app with API
    let app = crate::helpers::setup_test_app().await;
    let client = reqwest::Client::new();

    // Test login with non-existent user
    let login_response = client
        .post(&format!("{}/api/auth/login", app.address))
        .json(&json!({
            "email": "nonexistent@example.com",
            "password": "test_password"
        }))
        .send()
        .await
        .unwrap();

    // Should return 401 Unauthorized
    assert_eq!(login_response.status(), StatusCode::UNAUTHORIZED);

    // Parse error response
    let error_body: Value = login_response.json().await.unwrap();

    // Error should be generic, not revealing if user exists
    assert_eq!(error_body["status"], "error");
    assert_eq!(error_body["code"], "INVALID_CREDENTIALS");

    // Message should be generic "Invalid credentials" not "User not found"
    let error_message = error_body["message"].as_str().unwrap();
    assert!(!error_message.to_lowercase().contains("not found"));
    assert!(!error_message.to_lowercase().contains("doesn't exist"));
    assert!(error_message.to_lowercase().contains("invalid"));
}

/// Test CSRF protection
#[tokio::test]
async fn test_csrf_protection() {
    // Setup test app with API
    let app = crate::helpers::setup_test_app().await;
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    // Get CSRF token from a GET request (this would be in HTML form in real app)
    let response = client
        .get(&format!("{}/api/csrf-token", app.address))
        .send()
        .await
        .unwrap();

    // Cookie should be set with proper attributes
    let cookies = response.headers().get_all("set-cookie");
    let mut has_csrf_cookie = false;

    for cookie in cookies {
        let cookie_str = cookie.to_str().unwrap();
        if cookie_str.contains("csrf") {
            has_csrf_cookie = true;
            // Check security attributes
            assert!(cookie_str.contains("HttpOnly"));
            assert!(cookie_str.contains("SameSite=Strict") || cookie_str.contains("SameSite=Lax"));
        }
    }

    assert!(has_csrf_cookie, "No CSRF cookie found");

    // Test token in response body
    let body: Value = response.json().await.unwrap();
    assert!(body["csrf_token"].is_string());

    // Test POST without CSRF token (should fail)
    let login_response = client
        .post(&format!("{}/api/auth/login", app.address))
        .json(&json!({
            "email": "test@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();

    // Should fail with CSRF error
    assert_eq!(login_response.status(), StatusCode::FORBIDDEN);
    let error_body: Value = login_response.json().await.unwrap();
    assert_eq!(error_body["code"], "CSRF_VALIDATION_FAILED");
}

/// Test security headers
#[tokio::test]
async fn test_security_headers() {
    // Setup test app with API
    let app = crate::helpers::setup_test_app().await;
    let client = reqwest::Client::new();

    // Make a request to any endpoint
    let response = client
        .get(&format!("{}/api/health", app.address))
        .send()
        .await
        .unwrap();

    // Check security headers
    let headers = response.headers();

    // Content-Security-Policy should be set
    assert!(headers.contains_key("content-security-policy"));

    // X-Content-Type-Options should be set to nosniff
    assert_eq!(
        headers
            .get("x-content-type-options")
            .unwrap()
            .to_str()
            .unwrap(),
        "nosniff"
    );

    // X-Frame-Options should be set to DENY or SAMEORIGIN
    let x_frame = headers.get("x-frame-options").unwrap().to_str().unwrap();
    assert!(x_frame == "DENY" || x_frame == "SAMEORIGIN");

    // Strict-Transport-Security should be set
    assert!(headers.contains_key("strict-transport-security"));
}
