use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;

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
}

impl ApiError {
    /// Creates a new API error
    pub fn new(
        status_code: StatusCode,
        message: impl Into<String>,
        code: impl Into<String>,
        request_id: impl Into<String>,
    ) -> Self {
        Self {
            status_code,
            message: message.into(),
            code: code.into(),
            request_id: request_id.into(),
        }
    }

    /// Creates an internal server error
    pub fn internal_server_error(request_id: impl Into<String>) -> Self {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "An internal server error occurred".into(),
            code: "INTERNAL_SERVER_ERROR".into(),
            request_id: request_id.into(),
        }
    }

    /// Creates a validation error
    pub fn validation_error(message: impl Into<String>, request_id: impl Into<String>) -> Self {
        Self {
            status_code: StatusCode::BAD_REQUEST,
            message: message.into(),
            code: "VALIDATION_ERROR".into(),
            request_id: request_id.into(),
        }
    }

    /// Creates an authentication error
    pub fn authentication_error(request_id: impl Into<String>) -> Self {
        Self {
            status_code: StatusCode::UNAUTHORIZED,
            message: "Authentication required".into(),
            code: "AUTHENTICATION_REQUIRED".into(),
            request_id: request_id.into(),
        }
    }

    /// Creates an authorization error
    pub fn authorization_error(request_id: impl Into<String>) -> Self {
        Self {
            status_code: StatusCode::FORBIDDEN,
            message: "Not authorized".into(),
            code: "AUTHORIZATION_ERROR".into(),
            request_id: request_id.into(),
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
        let error_response = ApiResponse::<()>::error(self.message, self.code, self.request_id);
        (self.status_code, Json(error_response)).into_response()
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
