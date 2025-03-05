use acci_api::validation::{ValidationError, generate_request_id, validate_json_payload};
use axum::{Json, body::Body, extract::Request, http::StatusCode, response::IntoResponse};
use hyper::body::to_bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
struct TestUser {
    #[validate(length(min = 3, message = "Username must be at least 3 characters"))]
    username: String,

    #[validate(email(message = "Email must be a valid email address"))]
    email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    password: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
struct TestProduct {
    #[validate(length(
        min = 3,
        max = 100,
        message = "Name must be between 3 and 100 characters"
    ))]
    name: String,

    #[validate(range(min = 0.01, message = "Price must be greater than 0"))]
    price: f64,
}

// Helper to extract error message and code from response
async fn extract_error_info(response: axum::response::Response) -> (String, String) {
    let bytes = to_bytes(response.into_body())
        .await
        .expect("Failed to extract body bytes");

    let json: Value = serde_json::from_slice(&bytes).expect("Failed to parse response as JSON");

    let message = json["message"].as_str().unwrap_or("").to_string();
    let code = json["code"].as_str().unwrap_or("").to_string();

    (message, code)
}

#[tokio::test]
async fn test_validate_json_payload_valid_user() {
    // Arrange
    let valid_user = TestUser {
        username: "johndoe".to_string(),
        email: "john@example.com".to_string(),
        password: "password123".to_string(),
    };

    let payload = Json(valid_user);

    // Act
    let result = validate_json_payload(payload).await;

    // Assert
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_json_payload_invalid_user() {
    // Arrange
    let invalid_user = TestUser {
        username: "jo".to_string(),         // Too short
        email: "invalid-email".to_string(), // Not a valid email
        password: "pass".to_string(),       // Too short
    };

    let payload = Json(invalid_user);

    // Act
    let result = validate_json_payload(payload).await;

    // Assert
    assert!(result.is_err());

    match result {
        Err(ValidationError::InvalidData(msg)) => {
            assert!(msg.contains("Username must be at least 3 characters"));
            assert!(msg.contains("Email must be a valid email address"));
            assert!(msg.contains("Password must be at least 8 characters"));
        },
        _ => panic!("Expected ValidationError::InvalidData"),
    }
}

#[tokio::test]
async fn test_validation_error_response_format() {
    // Arrange
    let error = ValidationError::InvalidData("Test validation error".to_string());

    // Act
    let response = error.into_response();

    // Assert
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let (message, code) = extract_error_info(response).await;

    assert!(message.contains("Validation failed: Test validation error"));
    assert_eq!(code, "400"); // Default code for validation errors
}

#[tokio::test]
async fn test_validate_json_payload_with_range_validation() {
    // Arrange
    let invalid_product = TestProduct {
        name: "Product".to_string(),
        price: -10.0, // Negative price, should fail validation
    };

    let payload = Json(invalid_product);

    // Act
    let result = validate_json_payload(payload).await;

    // Assert
    assert!(result.is_err());

    match result {
        Err(ValidationError::InvalidData(msg)) => {
            assert!(msg.contains("Price must be greater than 0"));
        },
        _ => panic!("Expected ValidationError::InvalidData"),
    }
}

#[tokio::test]
async fn test_validate_json_payload_with_valid_product() {
    // Arrange
    let valid_product = TestProduct {
        name: "Valid Product".to_string(),
        price: 19.99,
    };

    let payload = Json(valid_product);

    // Act
    let result = validate_json_payload(payload).await;

    // Assert
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_generate_request_id() {
    // Generate multiple request IDs and ensure they're unique
    let id1 = generate_request_id();

    // Small delay to ensure different timestamps
    std::thread::sleep(std::time::Duration::from_millis(5));

    let id2 = generate_request_id();

    // Ensure request IDs are not empty and different from each other
    assert!(!id1.is_empty());
    assert!(!id2.is_empty());
    assert_ne!(id1, id2);
}
