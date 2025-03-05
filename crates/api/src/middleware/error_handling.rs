use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use metrics::counter;
use serde_json::{Value, json};
use tracing::{error, warn};

use crate::monitoring;
use crate::response::ApiError;
use crate::validation::generate_request_id;

/// Error handling middleware that catches errors and converts them to standardized API responses
///
/// This middleware intercepts responses from downstream handlers and checks for error status codes.
/// When errors are detected, it:
/// 1. Generates a unique request ID for tracking
/// 2. Records appropriate metrics based on error type (client vs server)
/// 3. Logs the error with contextual information
/// 4. Extracts error details from response body when available
/// 5. Transforms the error into a standardized API error format
///
/// # Examples
///
/// ```
/// use axum::{Router, routing::get};
/// use acci_api::middleware::error_handling::error_handling_middleware;
///
/// async fn handler() -> &'static str {
///     "Hello, World!"
/// }
///
/// let app = Router::new()
///     .route("/", get(handler))
///     .layer(axum::middleware::from_fn(error_handling_middleware));
/// ```
///
/// # Parameters
///
/// * `req` - The incoming HTTP request
/// * `next` - The next middleware or handler in the chain
///
/// # Returns
///
/// Returns a standardized API response, either passing through the original response
/// for success cases or a structured error response for error cases.
pub async fn error_handling_middleware(req: Request, next: Next) -> Response {
    // Extract path and method for error metrics before consuming the request
    let path = req.uri().path().to_string();
    let method = req.method().as_str().to_string();

    // Pass the request to the next handler
    let response = next.run(req).await;

    // If the response is an error (4xx or 5xx), log it and format consistently
    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
        // Generate a request ID for tracking
        let request_id = generate_request_id();

        // Increment error counters by status code
        let status_code = status.as_u16();
        if status.is_client_error() {
            counter!("api.errors.client", "status" => status_code.to_string(), "path" => path.clone(), "method" => method.clone()).increment(1);
            warn!(
                request_id = %request_id,
                status = %status_code,
                path = %path,
                method = %method,
                "Client error response"
            );
        } else {
            counter!("api.errors.server", "status" => status_code.to_string(), "path" => path.clone(), "method" => method.clone()).increment(1);
            error!(
                request_id = %request_id,
                status = %status_code,
                path = %path,
                method = %method,
                "Server error response"
            );
        }

        // Extract error details from the response body if possible
        let (_parts, body) = response.into_parts();
        let error_details = extract_error_details(body, &request_id).await;

        // Create a standardized error response using extracted details when available
        let error_response = create_error_response(status, error_details, request_id);

        return error_response.into_response();
    }

    // If not an error, return the original response
    response
}

/// Attempts to extract error details from a response body
async fn extract_error_details(body: Body, request_id: &str) -> Option<Value> {
    // Try to read the body bytes without consuming the body
    match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => {
            // Try to parse as JSON
            match serde_json::from_slice::<Value>(&bytes) {
                Ok(json) => {
                    // Log the extracted error details for debugging
                    if let Some(error_msg) = json.get("message").and_then(|m| m.as_str()) {
                        warn!(request_id = %request_id, error_message = %error_msg, "Extracted error message");
                    }
                    Some(json)
                },
                Err(_) => {
                    // Try to interpret as plain text
                    if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                        if !text.is_empty() {
                            return Some(json!({ "message": text }));
                        }
                    }
                    None
                },
            }
        },
        Err(e) => {
            error!(request_id = %request_id, error = %format!("{}", e), "Failed to extract response body");
            None
        },
    }
}

/// Creates a standardized error response using extracted details when available
fn create_error_response(
    status: StatusCode,
    error_details: Option<Value>,
    request_id: String,
) -> ApiError {
    // Default messages based on status code
    let (default_message, default_code) = match status.as_u16() {
        400 => ("Bad request", "BAD_REQUEST"),
        401 => ("Authentication required", "UNAUTHORIZED"),
        403 => ("Permission denied", "FORBIDDEN"),
        404 => ("Resource not found", "NOT_FOUND"),
        422 => ("Validation error", "VALIDATION_ERROR"),
        _ if status.is_server_error() => ("Internal server error", "INTERNAL_SERVER_ERROR"),
        _ => {
            // For other status codes, use the status code as the error code
            // We need to create a String and store it in the match arm scope, then use a string slice
            // Use a string literal for the error code
            // This is a workaround for lifetimes - the original code would try to return a reference
            // to a local variable which would go out of scope
            ("Error processing request", "UNKNOWN_ERROR")
        },
    };

    // Record error in metrics
    let error_type = if status.is_server_error() {
        "server"
    } else {
        "client"
    };
    monitoring::record_api_error(error_type, default_code, status.as_u16());

    // Extract error message and code from details if available
    if let Some(details) = error_details {
        let message = details
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or(default_message);

        let code = details
            .get("code")
            .and_then(|c| c.as_str())
            .unwrap_or(default_code);

        // Record more detailed error metrics if validation error
        if code == "VALIDATION_ERROR" {
            if let Some(validation_details) = details.get("details") {
                if let Some(validation_obj) = validation_details.as_object() {
                    for (field, _) in validation_obj {
                        monitoring::record_validation_error(field, "field_error");
                    }
                }
            }
        }

        let additional_info = details.get("details").cloned();

        // Check if ApiError has the new_with_details method, otherwise fall back to new
        #[cfg(feature = "extended_errors")]
        return ApiError::new_with_details(status, message, code, request_id, additional_info);

        // Fall back to the standard method if extended_errors feature is not enabled
        #[cfg(not(feature = "extended_errors"))]
        {
            if additional_info.is_some() {
                debug!(
                    request_id = %request_id,
                    "Additional error details available but extended_errors feature not enabled"
                );
            }
            return ApiError::new(status, message, code, request_id);
        }
    } else {
        // Use default error response based on status code
        ApiError::new(status, default_message, default_code, request_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, Response as AxumResponse, StatusCode},
        middleware::from_fn,
        routing::get,
    };
    use serde_json::json;

    use tower::Service;

    // Helper function to create a test app with error_handling_middleware
    fn setup_test_app() -> Router {
        Router::new()
            .route("/success", get(success_handler))
            .route("/error/400", get(bad_request_handler))
            .route("/error/404", get(not_found_handler))
            .route("/error/500", get(server_error_handler))
            .route("/error/custom", get(custom_error_handler))
            .layer(from_fn(error_handling_middleware))
    }

    // Test handlers
    async fn success_handler() -> AxumResponse<Body> {
        AxumResponse::builder()
            .status(StatusCode::OK)
            .body(Body::from(r#"{"message":"Success"}"#))
            .unwrap()
    }

    async fn bad_request_handler() -> AxumResponse<Body> {
        AxumResponse::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::empty())
            .unwrap()
    }

    async fn not_found_handler() -> AxumResponse<Body> {
        AxumResponse::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    }

    async fn server_error_handler() -> AxumResponse<Body> {
        AxumResponse::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap()
    }

    async fn custom_error_handler() -> AxumResponse<Body> {
        AxumResponse::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"message":"Custom error message","code":"CUSTOM_ERROR"}"#,
            ))
            .unwrap()
    }

    #[tokio::test]
    async fn test_middleware_passthrough_success() {
        let app = setup_test_app();

        let request = Request::builder()
            .uri("/success")
            .body(Body::empty())
            .unwrap();

        let mut svc = app.into_service();
        let response = svc.call(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert_eq!(body_str, r#"{"message":"Success"}"#);
    }

    #[tokio::test]
    async fn test_middleware_transforms_client_error() {
        let app = setup_test_app();

        let request = Request::builder()
            .uri("/error/400")
            .body(Body::empty())
            .unwrap();

        let mut svc = app.into_service();
        let response = svc.call(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Parse response as JSON and check structure
        let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(json["status"], "error");
        assert_eq!(json["code"], "BAD_REQUEST");
        assert!(json["request_id"].is_string());
    }

    #[tokio::test]
    async fn test_middleware_transforms_server_error() {
        let app = setup_test_app();

        let request = Request::builder()
            .uri("/error/500")
            .body(Body::empty())
            .unwrap();

        let mut svc = app.into_service();
        let response = svc.call(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Parse response as JSON and check structure
        let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(json["status"], "error");
        assert_eq!(json["code"], "INTERNAL_SERVER_ERROR");
        assert!(json["message"].is_string());
        assert!(json["request_id"].is_string());
    }

    #[tokio::test]
    async fn test_middleware_preserves_custom_error_details() {
        let app = setup_test_app();

        let request = Request::builder()
            .uri("/error/custom")
            .body(Body::empty())
            .unwrap();

        let mut svc = app.into_service();
        let response = svc.call(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Parse response as JSON and check custom values were preserved
        let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(json["status"], "error");
        assert_eq!(json["code"], "CUSTOM_ERROR");
        assert_eq!(json["message"], "Custom error message");
        assert!(json["request_id"].is_string());
    }

    #[tokio::test]
    async fn test_extract_error_details_empty_body() {
        let empty_body = Body::empty();
        let request_id = "test-req-id";

        let result = extract_error_details(empty_body, request_id).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_extract_error_details_json_body() {
        let json_body = Body::from(r#"{"message":"Test error","code":"TEST_ERROR"}"#);
        let request_id = "test-req-id";

        let result = extract_error_details(json_body, request_id).await;
        assert!(result.is_some());

        let details = result.unwrap();
        assert_eq!(details["message"], "Test error");
        assert_eq!(details["code"], "TEST_ERROR");
    }

    #[tokio::test]
    async fn test_extract_error_details_text_body() {
        let text_body = Body::from("Plain text error message");
        let request_id = "test-req-id";

        let result = extract_error_details(text_body, request_id).await;
        assert!(result.is_some());

        let details = result.unwrap();
        assert_eq!(details["message"], "Plain text error message");
    }

    #[tokio::test]
    async fn test_create_error_response_default_messages() {
        // Test several status codes with default messages
        let test_cases = [
            (StatusCode::BAD_REQUEST, "BAD_REQUEST"),
            (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            (StatusCode::FORBIDDEN, "FORBIDDEN"),
            (StatusCode::NOT_FOUND, "NOT_FOUND"),
            (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_SERVER_ERROR"),
            (StatusCode::BAD_GATEWAY, "INTERNAL_SERVER_ERROR"), // Test unknown status code
        ];

        let request_id = "test-req-id".to_string();

        for (status, expected_code) in test_cases.iter() {
            let error = create_error_response(*status, None, request_id.clone());

            // Da die internen Felder private sind, testen wir das Verhalten stattdessen
            let response = error.into_response();
            assert_eq!(response.status(), *status);

            // Extract the body to check the code
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let body_str = String::from_utf8(body.to_vec()).unwrap();
            let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

            assert_eq!(json["code"].as_str().unwrap(), *expected_code);
        }
    }

    #[tokio::test]
    async fn test_create_error_response_with_details() {
        let status = StatusCode::BAD_REQUEST;
        let details = Some(json!({
            "message": "Custom message",
            "code": "CUSTOM_CODE"
        }));
        let request_id = "test-req-id".to_string();

        let error = create_error_response(status, details, request_id);

        // Da die internen Felder private sind, testen wir das Verhalten stattdessen
        let response = error.into_response();
        assert_eq!(response.status(), status);

        // Extract the body to check message and code
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

        assert_eq!(json["message"], "Custom message");
        assert_eq!(json["code"], "CUSTOM_CODE");
    }
}
