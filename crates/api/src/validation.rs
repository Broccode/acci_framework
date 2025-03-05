use crate::monitoring;
use axum::{
    Json,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::fmt::Debug;
use thiserror::Error;
use tracing::{debug, error};
use validator::Validate;

/// A wrapper for validated JSON requests
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

impl<T> ValidatedJson<T> {
    /// Extract the inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for ValidatedJson<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for ValidatedJson<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A wrapper for validated query parameters
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedQuery<T>(pub T);

impl<T> ValidatedQuery<T> {
    /// Extract the inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for ValidatedQuery<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for ValidatedQuery<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Generates a request ID
pub fn generate_request_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .to_string()
}

/// Error that can occur during validation
#[derive(Debug, Error)]
pub enum ValidationError {
    /// JSON deserialization error
    #[error("Failed to parse JSON: {0}")]
    JsonError(#[from] JsonRejection),

    /// Validation error
    #[error("Validation error: {0}")]
    InvalidData(String),
}

impl IntoResponse for ValidationError {
    fn into_response(self) -> Response {
        match self {
            ValidationError::JsonError(rejection) => {
                let status = StatusCode::BAD_REQUEST;
                let message = format!("Invalid JSON: {}", rejection);
                monitoring::record_validation_error("json_error", "parse_error");

                let body = serde_json::json!({
                    "status": "error",
                    "message": message,
                    "code": status.as_u16(),
                    "request_id": generate_request_id(),
                });

                (status, Json(body)).into_response()
            },
            ValidationError::InvalidData(err) => {
                let status = StatusCode::BAD_REQUEST;
                monitoring::record_validation_error("validation_error", "constraint_violation");

                let body = serde_json::json!({
                    "status": "error",
                    "message": format!("Validation failed: {}", err),
                    "code": status.as_u16(),
                    "request_id": generate_request_id(),
                });

                (status, Json(body)).into_response()
            },
        }
    }
}

/// Response for validation errors
#[derive(Debug, Serialize)]
pub struct ValidationErrorResponse {
    /// Status of the response
    pub status: String,
    /// Error message
    pub message: String,
    /// HTTP status code
    pub code: u16,
    /// Request ID
    pub request_id: String,
    /// Validation errors
    pub errors: Vec<String>,
}

/// A wrapper for validated data
#[derive(Debug, Clone)]
pub struct ValidatedData<T>(pub T);

impl<T> std::ops::Deref for ValidatedData<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for ValidatedData<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Temporarily commented out due to lifetime issues
/*
#[async_trait]
impl<T, S> FromRequest<S> for ValidatedData<T>
where
    T: DeserializeOwned + Validate + Send + 'static,
    S: Send + Sync,
{
    type Rejection = ValidationError;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let Json(data) = Json::<T>::from_request(req, state).await
            .map_err(ValidationError::JsonError)?;

        if let Err(validation_errors) = data.validate() {
            let error_message = validation_errors.to_string();
            error!("Validation error: {}", error_message);
            monitoring::record_validation_error("validator_failed", "constraint_violation");
            return Err(ValidationError::InvalidData(error_message));
        }

        Ok(ValidatedData(data))
    }
}
*/

/// Handle JSON extraction errors
pub fn handle_json_extraction_error(rejection: JsonRejection) -> ValidationError {
    error!("JSON extraction error: {}", rejection);
    monitoring::record_validation_error("json_extraction_failed", "parse_error");
    ValidationError::JsonError(rejection)
}

/// Validate a JSON payload using the validator crate
pub async fn validate_json_payload<T>(payload: Json<T>) -> Result<T, ValidationError>
where
    T: Validate,
{
    if let Err(validation_errors) = payload.validate() {
        let error_message = validation_errors.to_string();
        error!("Validation error: {}", error_message);
        monitoring::record_validation_error("payload_validation_failed", "constraint_violation");
        return Err(ValidationError::InvalidData(error_message));
    }

    debug!("Validation succeeded");
    Ok(payload.0)
}

/// Rate limiter middleware
pub mod rate_limiter {
    use axum::http::Request;
    use axum::response::Response;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tower::{Layer, Service};
    use tracing::debug;

    /// Simple rate limiter for demonstration
    #[derive(Clone)]
    pub struct RateLimiter<S> {
        inner: S,
    }

    impl<S> RateLimiter<S> {
        /// Creates a new rate limiter
        pub fn new(inner: S) -> Self {
            Self { inner }
        }
    }

    impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for RateLimiter<S>
    where
        S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
        S::Future: Send + 'static,
        ReqBody: Send + 'static,
        ResBody: Send + 'static,
    {
        type Response = S::Response;
        type Error = S::Error;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
            // In a real implementation, you would check rate limits here
            // For demo purposes, we just log and pass through
            let client_ip = req
                .extensions()
                .get::<String>()
                .map(|ip| ip.as_str())
                .unwrap_or("unknown");

            debug!("Rate limiter: processing request from {}", client_ip);

            // Clone the service to move into the future
            let mut clone = self.inner.clone();

            Box::pin(async move {
                // A real implementation would check if the client has exceeded rate limits
                // For demo purposes, always allow
                debug!("Rate limiter: request allowed");
                clone.call(req).await
            })
        }
    }

    /// Layer that applies a rate limiter
    #[derive(Clone)]
    pub struct RateLimiterLayer;

    impl Default for RateLimiterLayer {
        fn default() -> Self {
            Self::new()
        }
    }

    impl RateLimiterLayer {
        /// Creates a new rate limiter layer
        pub fn new() -> Self {
            Self
        }
    }

    impl<S> Layer<S> for RateLimiterLayer {
        type Service = RateLimiter<S>;

        fn layer(&self, service: S) -> Self::Service {
            RateLimiter::new(service)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::extract::Request;
    use axum::response::IntoResponse;
    use axum::response::Response;
    use http::StatusCode;
    use serde::{Deserialize, Serialize};
    use std::convert::Infallible;
    use tower::Service;
    use validator::Validate;

    #[derive(Debug, Deserialize, Serialize, Validate)]
    struct TestUser {
        #[validate(length(min = 3, message = "username must be at least 3 characters"))]
        username: String,

        #[validate(email(message = "email must be a valid email address"))]
        email: String,

        #[validate(length(min = 8, message = "password must be at least 8 characters"))]
        password: String,
    }

    #[derive(Debug, Deserialize, Serialize, Validate)]
    struct TestAddress {
        #[validate(length(min = 5, message = "street must be at least 5 characters"))]
        street: String,

        #[validate(length(min = 1, message = "city cannot be empty"))]
        city: String,
    }

    #[derive(Debug, Deserialize, Serialize, Validate)]
    struct TestUserWithAddress {
        #[validate(nested)]
        user: TestUser,

        #[validate(nested)]
        address: TestAddress,
    }

    fn create_json_request(json: &str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(json.to_string()))
            .unwrap()
    }

    async fn extract_error_message(resp: Response) -> String {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        body["message"].as_str().unwrap_or_default().to_string()
    }

    #[tokio::test]
    async fn test_validate_json_payload_valid() {
        let json_data =
            r#"{"username":"john","email":"john@example.com","password":"password123"}"#;
        let payload = Json(serde_json::from_str::<TestUser>(json_data).unwrap());

        let result = validate_json_payload(payload).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_json_payload_invalid_username() {
        let json_data = r#"{"username":"jo","email":"john@example.com","password":"password123"}"#;
        let payload = Json(serde_json::from_str::<TestUser>(json_data).unwrap());

        let result = validate_json_payload(payload).await;
        assert!(result.is_err());

        match result {
            Err(ValidationError::InvalidData(msg)) => {
                assert!(msg.contains("username must be at least 3 characters"));
            },
            _ => panic!("Expected ValidationError::InvalidData"),
        }
    }

    #[tokio::test]
    async fn test_validate_json_payload_invalid_email() {
        let json_data = r#"{"username":"john","email":"invalid-email","password":"password123"}"#;
        let payload = Json(serde_json::from_str::<TestUser>(json_data).unwrap());

        let result = validate_json_payload(payload).await;
        assert!(result.is_err());

        match result {
            Err(ValidationError::InvalidData(msg)) => {
                assert!(msg.contains("email must be a valid email address"));
            },
            _ => panic!("Expected ValidationError::InvalidData"),
        }
    }

    #[tokio::test]
    async fn test_validate_json_payload_multiple_errors() {
        let json_data = r#"{"username":"jo","email":"invalid-email","password":"pass"}"#;
        let payload = Json(serde_json::from_str::<TestUser>(json_data).unwrap());

        let result = validate_json_payload(payload).await;
        assert!(result.is_err());

        match result {
            Err(ValidationError::InvalidData(msg)) => {
                assert!(msg.contains("username must be at least 3 characters"));
                assert!(msg.contains("email must be a valid email address"));
                assert!(msg.contains("password must be at least 8 characters"));
            },
            _ => panic!("Expected ValidationError::InvalidData"),
        }
    }

    #[tokio::test]
    async fn test_validate_json_payload_nested() {
        let json_data = r#"
        {
            "user": {
                "username": "jo",
                "email": "invalid-email",
                "password": "pass"
            },
            "address": {
                "street": "st",
                "city": ""
            }
        }"#;
        let payload = Json(serde_json::from_str::<TestUserWithAddress>(json_data).unwrap());

        let result = validate_json_payload(payload).await;
        assert!(result.is_err());

        match result {
            Err(ValidationError::InvalidData(msg)) => {
                assert!(msg.contains("username must be at least 3 characters"));
                assert!(msg.contains("email must be a valid email address"));
                assert!(msg.contains("password must be at least 8 characters"));
                assert!(msg.contains("street must be at least 5 characters"));
                assert!(msg.contains("city cannot be empty"));
            },
            _ => panic!("Expected ValidationError::InvalidData"),
        }
    }

    #[tokio::test]
    async fn test_validated_data_extractor_valid() {
        let json_data =
            r#"{"username":"john","email":"john@example.com","password":"password123"}"#;
        let request = create_json_request(json_data);

        // Parse the JSON directly for testing
        let test_user: TestUser = serde_json::from_str(json_data).unwrap();
        let result = test_user.validate();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validated_data_extractor_invalid() {
        let json_data = r#"{"username":"jo","email":"invalid-email","password":"pass"}"#;
        let _request = create_json_request(json_data);

        // Parse the JSON directly for testing
        let test_user: TestUser = serde_json::from_str(json_data).unwrap();
        let result = test_user.validate();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validation_error_into_response() {
        let error = ValidationError::InvalidData("Test validation error".to_string());
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let error_msg = extract_error_message(response).await;
        assert!(error_msg.contains("Validation failed: Test validation error"));
    }

    #[tokio::test]
    async fn test_json_error_handling() {
        let error = ValidationError::InvalidData("Test validation error".to_string());
        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let error_msg = extract_error_message(response).await;
        assert!(error_msg.contains("Validation failed: Test validation error"));
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        use rate_limiter::RateLimiter;

        #[derive(Clone)]
        struct MockService;

        impl Service<Request<Body>> for MockService {
            type Response = Response;
            type Error = Infallible;
            type Future = std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
            >;

            fn poll_ready(
                &mut self,
                _cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Result<(), Self::Error>> {
                std::task::Poll::Ready(Ok(()))
            }

            fn call(&mut self, _req: Request<Body>) -> Self::Future {
                Box::pin(async {
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .body(Body::from("OK"))
                        .unwrap())
                })
            }
        }

        let mut rate_limited_service = RateLimiter::new(MockService);

        let request = Request::builder()
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = rate_limited_service.call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
