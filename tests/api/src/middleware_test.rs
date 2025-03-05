use acci_api::middleware::{
    error_handling::error_handling_middleware, logging::logging_middleware,
};
use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    http::{Request, Response, StatusCode},
    middleware::from_fn,
    routing::get,
};
use http::HeaderMap;
use hyper::body::Bytes;
use serde_json::Value;
use std::time::Duration;
use tower::{Service, ServiceExt};

// Helper for building test apps with middleware
fn create_test_app_with_error_handling() -> Router {
    Router::new()
        .route("/success", get(success_handler))
        .route("/error/400", get(bad_request_handler))
        .route("/error/401", get(unauthorized_handler))
        .route("/error/403", get(forbidden_handler))
        .route("/error/404", get(not_found_handler))
        .route("/error/422", get(validation_error_handler))
        .route("/error/500", get(server_error_handler))
        .route("/error/custom", get(custom_error_handler))
        .layer(from_fn(error_handling_middleware))
}

fn create_test_app_with_logging() -> Router {
    Router::new()
        .route("/test", get(success_handler))
        .layer(from_fn(logging_middleware))
}

fn create_test_app_with_both_middleware() -> Router {
    Router::new()
        .route("/success", get(success_handler))
        .route("/error/500", get(server_error_handler))
        .layer(from_fn(error_handling_middleware))
        .layer(from_fn(logging_middleware))
}

// Test handlers
async fn success_handler() -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(r#"{"message":"Success"}"#))
        .unwrap()
}

async fn bad_request_handler() -> Response<Body> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Body::empty())
        .unwrap()
}

async fn unauthorized_handler() -> Response<Body> {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body(Body::empty())
        .unwrap()
}

async fn forbidden_handler() -> Response<Body> {
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .body(Body::empty())
        .unwrap()
}

async fn not_found_handler() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .unwrap()
}

async fn validation_error_handler() -> Response<Body> {
    Response::builder()
        .status(StatusCode::UNPROCESSABLE_ENTITY)
        .body(Body::empty())
        .unwrap()
}

async fn server_error_handler() -> Response<Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::empty())
        .unwrap()
}

async fn custom_error_handler() -> Response<Body> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"message":"Custom error message","code":"CUSTOM_ERROR"}"#,
        ))
        .unwrap()
}

// Helper to extract JSON from response body
async fn extract_json_body(response: Response<Body>) -> Value {
    let bytes = hyper::body::to_bytes(response.into_body())
        .await
        .expect("Failed to read response body");

    serde_json::from_slice(&bytes).expect("Failed to parse JSON response")
}

#[tokio::test]
async fn test_error_middleware_transforms_client_error() {
    let app = create_test_app_with_error_handling();

    let request = Request::builder()
        .uri("/error/400")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let json = extract_json_body(response).await;

    // Verify standardized error format
    assert_eq!(json["status"], "error");
    assert_eq!(json["code"], "BAD_REQUEST");
    assert!(json["message"].is_string());
    assert!(json["request_id"].is_string());
}

#[tokio::test]
async fn test_error_middleware_transforms_server_error() {
    let app = create_test_app_with_error_handling();

    let request = Request::builder()
        .uri("/error/500")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let json = extract_json_body(response).await;

    // Verify standardized error format
    assert_eq!(json["status"], "error");
    assert_eq!(json["code"], "INTERNAL_SERVER_ERROR");
    assert!(json["message"].is_string());
    assert!(json["request_id"].is_string());
}

#[tokio::test]
async fn test_error_middleware_handles_auth_errors() {
    let app = create_test_app_with_error_handling();

    // Test 401 Unauthorized
    let unauthorized_request = Request::builder()
        .uri("/error/401")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(unauthorized_request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let json = extract_json_body(response).await;
    assert_eq!(json["code"], "UNAUTHORIZED");

    // Test 403 Forbidden
    let forbidden_request = Request::builder()
        .uri("/error/403")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(forbidden_request).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let json = extract_json_body(response).await;
    assert_eq!(json["code"], "FORBIDDEN");
}

#[tokio::test]
async fn test_error_middleware_handles_validation_errors() {
    let app = create_test_app_with_error_handling();

    let request = Request::builder()
        .uri("/error/422")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let json = extract_json_body(response).await;
    assert_eq!(json["code"], "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_error_middleware_preserves_custom_error_details() {
    let app = create_test_app_with_error_handling();

    let request = Request::builder()
        .uri("/error/custom")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let json = extract_json_body(response).await;

    // Custom error code and message should be preserved
    assert_eq!(json["status"], "error");
    assert_eq!(json["code"], "CUSTOM_ERROR");
    assert_eq!(json["message"], "Custom error message");
    assert!(json["request_id"].is_string());
}

#[tokio::test]
async fn test_error_middleware_passthrough_for_success() {
    let app = create_test_app_with_error_handling();

    let request = Request::builder()
        .uri("/success")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_str = String::from_utf8(bytes.to_vec()).unwrap();

    // Original success response should be passed through untouched
    assert_eq!(body_str, r#"{"message":"Success"}"#);
}

#[tokio::test]
async fn test_logging_middleware_adds_headers() {
    let app = create_test_app_with_logging();

    let request = Request::builder()
        .uri("/test")
        .header("user-agent", "test-agent")
        .header("x-forwarded-for", "127.0.0.1")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // We can't easily test the logging itself, but we can verify that the response
    // is passed through correctly
    let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_str = String::from_utf8(bytes.to_vec()).unwrap();

    assert_eq!(body_str, r#"{"message":"Success"}"#);
}

#[tokio::test]
async fn test_middleware_chaining() {
    let app = create_test_app_with_both_middleware();

    // Test success path
    let success_req = Request::builder()
        .uri("/success")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(success_req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Test error path
    let error_req = Request::builder()
        .uri("/error/500")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(error_req).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let json = extract_json_body(response).await;
    assert_eq!(json["status"], "error");
    assert_eq!(json["code"], "INTERNAL_SERVER_ERROR");
}

#[tokio::test]
async fn test_middleware_handles_not_found() {
    let app = create_test_app_with_error_handling();

    let request = Request::builder()
        .uri("/error/404")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let json = extract_json_body(response).await;
    assert_eq!(json["code"], "NOT_FOUND");
}
