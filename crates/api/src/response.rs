use crate::monitoring;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;
use tracing::{error, info, warn};

/// Standardized API response format
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Response status (success or error)
    pub status: ResponseStatus,
    /// Response data (only for successful responses)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// Error message (only for error responses)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Error code (only for error responses)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Request ID for tracing
    pub request_id: String,
}

/// API response status
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResponseStatus {
    /// Successful response
    Success,
    /// Error response
    Error,
}

impl<T> ApiResponse<T> {
    /// Creates a successful response
    pub fn success(data: T, request_id: impl Into<String>) -> Self {
        Self {
            status: ResponseStatus::Success,
            data: Some(data),
            message: None,
            code: None,
            request_id: request_id.into(),
        }
    }

    /// Creates an error response
    pub fn error(
        message: impl Into<String>,
        code: impl Into<String>,
        request_id: impl Into<String>,
    ) -> Self {
        Self {
            status: ResponseStatus::Error,
            data: None,
            message: Some(message.into()),
            code: Some(code.into()),
            request_id: request_id.into(),
        }
    }
}

/// Transforms any error message into a standardized API error response
pub struct ApiError {
    status_code: StatusCode,
    message: String,
    code: String,
    request_id: String,
    details: Option<serde_json::Value>,
}

impl ApiError {
    /// Creates a new API error
    pub fn new(
        status_code: StatusCode,
        message: impl Into<String>,
        code: impl Into<String>,
        request_id: impl Into<String>,
    ) -> Self {
        let message = message.into();
        let code = code.into();
        let request_id = request_id.into();

        // Log the error
        if status_code.is_server_error() {
            error!(
                request_id = %request_id,
                status = %status_code.as_u16(),
                code = %code,
                message = %message,
                "Server error response generated"
            );
        } else {
            warn!(
                request_id = %request_id,
                status = %status_code.as_u16(),
                code = %code,
                message = %message,
                "Client error response generated"
            );
        }

        // Record error metrics
        let error_type = if status_code.is_server_error() {
            "server"
        } else {
            "client"
        };
        monitoring::record_api_error(error_type, &code, status_code.as_u16());

        Self {
            status_code,
            message,
            code,
            request_id,
            details: None,
        }
    }

    /// Creates a new API error with additional details
    #[cfg(feature = "extended_errors")]
    pub fn new_with_details(
        status_code: StatusCode,
        message: impl Into<String>,
        code: impl Into<String>,
        request_id: impl Into<String>,
        details: Option<serde_json::Value>,
    ) -> Self {
        let mut error = Self::new(status_code, message, code, request_id);
        error.details = details;
        error
    }

    /// Creates an internal server error
    pub fn internal_server_error(request_id: impl Into<String>) -> Self {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "An internal server error occurred".into(),
            code: "INTERNAL_SERVER_ERROR".into(),
            request_id: request_id.into(),
            details: None,
        }
    }

    /// Creates a validation error
    pub fn validation_error(message: impl Into<String>, request_id: impl Into<String>) -> Self {
        Self {
            status_code: StatusCode::BAD_REQUEST,
            message: message.into(),
            code: "VALIDATION_ERROR".into(),
            request_id: request_id.into(),
            details: None,
        }
    }

    /// Creates an authentication error
    pub fn authentication_error(request_id: impl Into<String>) -> Self {
        Self {
            status_code: StatusCode::UNAUTHORIZED,
            message: "Authentication required".into(),
            code: "AUTHENTICATION_REQUIRED".into(),
            request_id: request_id.into(),
            details: None,
        }
    }

    /// Creates an authorization error
    pub fn authorization_error(request_id: impl Into<String>) -> Self {
        Self {
            status_code: StatusCode::FORBIDDEN,
            message: "Not authorized".into(),
            code: "AUTHORIZATION_ERROR".into(),
            request_id: request_id.into(),
            details: None,
        }
    }

    /// Creates a resource not found error
    pub fn not_found_error(resource: impl Into<String>, request_id: impl Into<String>) -> Self {
        let resource = resource.into();
        Self {
            status_code: StatusCode::NOT_FOUND,
            message: format!("Resource not found: {}", resource),
            code: "RESOURCE_NOT_FOUND".into(),
            request_id: request_id.into(),
            details: None,
        }
    }
}

impl fmt::Debug for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiError")
            .field("status_code", &self.status_code)
            .field("message", &self.message)
            .field("code", &self.code)
            .field("request_id", &self.request_id)
            .finish()
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "API Error: {} ({})", self.message, self.status_code)
    }
}

impl StdError for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // Log the error response being sent
        info!(
            request_id = %self.request_id,
            status = %self.status_code.as_u16(),
            "Sending error response"
        );

        #[cfg(feature = "extended_errors")]
        {
            let error_response = if let Some(details) = self.details {
                let response = ApiResponse::<()>::error(self.message, self.code, self.request_id);
                // Wir erstellen ein zusätzliches Feld "details" in der API-Antwort
                let response_value = serde_json::to_value(&response).unwrap_or_default();
                if let serde_json::Value::Object(mut obj) = response_value {
                    obj.insert("details".to_string(), details);
                    return (self.status_code, Json(obj)).into_response();
                }
                response
            } else {
                ApiResponse::<()>::error(self.message, self.code, self.request_id)
            };

            (self.status_code, Json(error_response)).into_response()
        }

        #[cfg(not(feature = "extended_errors"))]
        {
            let error_response = ApiResponse::<()>::error(self.message, self.code, self.request_id);
            (self.status_code, Json(error_response)).into_response()
        }
    }
}

/// Extension for Result to easily convert to ApiResponse
pub trait ResultExt<T, E> {
    /// Converts a Result into an API response
    fn into_api_response(
        self,
        status_code: StatusCode,
        request_id: impl Into<String>,
    ) -> Result<(StatusCode, Json<ApiResponse<T>>), ApiError>
    where
        E: fmt::Display;
}

impl<T, E> ResultExt<T, E> for Result<T, E>
where
    E: fmt::Display,
{
    fn into_api_response(
        self,
        status_code: StatusCode,
        request_id: impl Into<String>,
    ) -> Result<(StatusCode, Json<ApiResponse<T>>), ApiError> {
        let request_id = request_id.into();
        match self {
            Ok(data) => {
                let response = ApiResponse::success(data, request_id);
                Ok((status_code, Json(response)))
            },
            Err(err) => Err(ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                err.to_string(),
                "INTERNAL_SERVER_ERROR",
                request_id,
            )),
        }
    }
}

// Add the IntoResponse implementation for ApiResponse
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let body = match serde_json::to_string(&self) {
            Ok(json) => json,
            Err(err) => {
                // If serialization fails, return a 500 error
                let error_response = ApiResponse::<()>::error(
                    format!("Failed to serialize response: {}", err),
                    "SERIALIZATION_ERROR",
                    self.request_id,
                );
                
                match serde_json::to_string(&error_response) {
                    Ok(error_json) => error_json,
                    Err(_) => String::from(r#"{"status":"error","message":"Critical serialization error","code":"CRITICAL_ERROR"}"#),
                }
            }
        };
        
        // Default to 200 OK for success responses and 400 Bad Request for error responses
        let status = match self.status {
            ResponseStatus::Success => StatusCode::OK,
            ResponseStatus::Error => StatusCode::BAD_REQUEST,
        };
        
        // Create the response with the appropriate content type
        Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(body.into())
            .unwrap_or_else(|_| {
                // If response creation fails, return a plain 500 error
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("Internal Server Error".into())
                    .unwrap()
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_api_response_success() {
        let data = json!({"id": "1", "name": "Test"});
        let request_id = "req-123";

        let response = ApiResponse::success(data, request_id);

        assert_eq!(response.status, ResponseStatus::Success);
        assert_eq!(response.request_id, request_id);
        assert!(response.data.is_some());
        assert!(response.message.is_none());
        assert!(response.code.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let message = "An error occurred";
        let code = "TEST_ERROR";
        let request_id = "req-123";

        let response = ApiResponse::<()>::error(message, code, request_id);

        assert_eq!(response.status, ResponseStatus::Error);
        assert_eq!(response.request_id, request_id);
        assert_eq!(response.message.unwrap(), message);
        assert_eq!(response.code.unwrap(), code);
        assert!(response.data.is_none());
    }

    #[test]
    fn test_api_error_creation() {
        let status_code = StatusCode::BAD_REQUEST;
        let message = "Invalid input";
        let code = "INVALID_INPUT";
        let request_id = "req-123";

        let error = ApiError::new(status_code, message, code, request_id);

        // Test Debug implementation
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("ApiError"));
        assert!(debug_str.contains("status_code"));

        // Test Display implementation
        let display_str = format!("{}", error);
        assert!(display_str.contains("API Error"));
        assert!(display_str.contains(message));
    }

    #[test]
    fn test_helper_error_methods() {
        let request_id = "req-123";

        // Test internal_server_error
        let internal_error = ApiError::internal_server_error(request_id);
        assert_eq!(
            internal_error.status_code,
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(internal_error.code, "INTERNAL_SERVER_ERROR");

        // Test validation_error
        let validation_error = ApiError::validation_error("Field is required", request_id);
        assert_eq!(validation_error.status_code, StatusCode::BAD_REQUEST);
        assert_eq!(validation_error.code, "VALIDATION_ERROR");

        // Test authentication_error
        let auth_error = ApiError::authentication_error(request_id);
        assert_eq!(auth_error.status_code, StatusCode::UNAUTHORIZED);
        assert_eq!(auth_error.code, "AUTHENTICATION_REQUIRED");

        // Test authorization_error
        let authz_error = ApiError::authorization_error(request_id);
        assert_eq!(authz_error.status_code, StatusCode::FORBIDDEN);
        assert_eq!(authz_error.code, "AUTHORIZATION_ERROR");

        // Test not_found_error
        let not_found = ApiError::not_found_error("User", request_id);
        assert_eq!(not_found.status_code, StatusCode::NOT_FOUND);
        assert_eq!(not_found.code, "RESOURCE_NOT_FOUND");
        assert!(not_found.message.contains("User"));
    }

    #[test]
    fn test_api_error_into_response() {
        let request_id = "req-123";
        let error = ApiError::new(
            StatusCode::BAD_REQUEST,
            "Test error",
            "TEST_ERROR",
            request_id,
        );

        let response = error.into_response();
        let status = response.status();

        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_result_ext_ok() {
        let result: Result<i32, &str> = Ok(42);
        let request_id = "req-123";

        let api_result = result.into_api_response(StatusCode::OK, request_id);

        assert!(api_result.is_ok());
        let (status, json_response) = api_result.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json_response.0.status, ResponseStatus::Success);
        assert_eq!(json_response.0.data.unwrap(), 42);
    }

    #[test]
    fn test_result_ext_err() {
        let result: Result<i32, &str> = Err("Test error");
        let request_id = "req-123";

        let api_result = result.into_api_response(StatusCode::OK, request_id);

        assert!(api_result.is_err());
        let error = api_result.unwrap_err();
        assert_eq!(error.status_code, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error.code, "INTERNAL_SERVER_ERROR");
        assert!(error.message.contains("Test error"));
    }
}
