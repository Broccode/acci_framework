use crate::monitoring;
use crate::response::{ApiError, ApiResponse};
use crate::validation::{generate_request_id, validate_json_payload};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;
use validator::Validate;

// Import auth services and models
use acci_auth::{
    models::VerificationType,
    repository::TenantAwareContext,
    services::{session::SessionService, verification::VerificationService},
};

/// Verification Application State
#[derive(Clone)]
pub struct VerificationAppState {
    /// Verification service for handling verification codes
    pub verification_service: Arc<VerificationService>,
    /// Session service for session management
    pub session_service: Arc<SessionService>,
    /// Default tenant-aware context for operations
    pub tenant_context: Arc<dyn TenantAwareContext>,
}

/// Send Verification Request DTO
#[derive(Debug, Deserialize, Validate)]
pub struct SendVerificationRequest {
    /// User ID to send verification code to
    #[validate(length(min = 36, max = 36, message = "Invalid UUID format"))]
    pub user_id: String,

    /// Type of verification (email or sms)
    #[validate(custom(function = "validate_verification_type"))]
    pub verification_type: String,

    /// Recipient (email address or phone number)
    #[validate(length(min = 1, message = "Recipient is required"))]
    pub recipient: String,

    /// Tenant ID for multi-tenant context
    #[validate(length(min = 36, max = 36, message = "Invalid UUID format"))]
    pub tenant_id: String,

    /// Session token (optional)
    pub session_token: Option<String>,
}

/// Helper function to validate verification type
fn validate_verification_type(verification_type: &str) -> Result<(), validator::ValidationError> {
    match verification_type.to_lowercase().as_str() {
        "email" | "sms" => Ok(()),
        _ => {
            let mut error = validator::ValidationError::new("verification_type");
            error.message = Some("Verification type must be 'email' or 'sms'".into());
            Err(error)
        },
    }
}

/// Send Verification Response DTO
#[derive(Debug, Serialize)]
pub struct SendVerificationResponse {
    /// Success status
    pub success: bool,

    /// User ID
    pub user_id: String,

    /// Verification type
    pub verification_type: String,
}

/// Verify Code Request DTO
#[derive(Debug, Deserialize, Validate)]
pub struct VerifyCodeRequest {
    /// User ID for verification
    #[validate(length(min = 36, max = 36, message = "Invalid UUID format"))]
    pub user_id: String,

    /// Verification code
    #[validate(length(min = 6, message = "Verification code is required"))]
    pub code: String,

    /// Type of verification (email or sms)
    #[validate(custom(function = "validate_verification_type"))]
    pub verification_type: String,

    /// Tenant ID for multi-tenant context
    #[validate(length(min = 36, max = 36, message = "Invalid UUID format"))]
    pub tenant_id: String,

    /// Session token (optional)
    pub session_token: Option<String>,
}

/// Verify Code Response DTO
#[derive(Debug, Serialize)]
pub struct VerifyCodeResponse {
    /// Success status
    pub success: bool,

    /// User ID
    pub user_id: String,

    /// Verification type
    pub verification_type: String,
}

/// Handler for sending a verification code
#[axum::debug_handler]
pub async fn send_verification(
    State(state): State<VerificationAppState>,
    Json(request): Json<SendVerificationRequest>,
) -> Response {
    debug!("Processing send verification request");
    let start = std::time::Instant::now();

    // Generate a unique request ID
    let request_id = generate_request_id();

    // Validate the request
    let validated = match validate_json_payload(Json(request)).await {
        Ok(data) => data,
        Err(validation_error) => {
            // Convert validation error to response
            return validation_error.into_response();
        },
    };

    // Record operation attempt in metrics
    monitoring::record_auth_operation("verification_send", "attempt");

    // Parse UUIDs
    let user_id = match Uuid::parse_str(&validated.user_id) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::new(
                StatusCode::BAD_REQUEST,
                "Invalid user ID format",
                "INVALID_USER_ID",
                request_id,
            )
            .into_response();
        },
    };

    let tenant_id = match Uuid::parse_str(&validated.tenant_id) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::new(
                StatusCode::BAD_REQUEST,
                "Invalid tenant ID format",
                "INVALID_TENANT_ID",
                request_id,
            )
            .into_response();
        },
    };

    // Convert verification type string to enum
    let verification_type = match validated.verification_type.to_lowercase().as_str() {
        "email" => VerificationType::Email,
        "sms" => VerificationType::Sms,
        _ => {
            return ApiError::new(
                StatusCode::BAD_REQUEST,
                "Invalid verification type",
                "INVALID_VERIFICATION_TYPE",
                request_id,
            )
            .into_response();
        },
    };

    // If session token is provided, validate it
    if let Some(session_token) = &validated.session_token {
        match state.session_service.validate_session(session_token).await {
            Ok(Some(session)) => {
                // Ensure the session belongs to the user
                if session.user_id != user_id {
                    return ApiError::new(
                        StatusCode::FORBIDDEN,
                        "Session does not belong to this user",
                        "UNAUTHORIZED_SESSION",
                        request_id,
                    )
                    .into_response();
                }
            },
            _ => {
                return ApiError::new(
                    StatusCode::UNAUTHORIZED,
                    "Invalid session token",
                    "INVALID_SESSION",
                    request_id,
                )
                .into_response();
            },
        }
    }

    // Send verification code
    match state
        .verification_service
        .send_verification(
            tenant_id,
            user_id,
            verification_type,
            validated.recipient,
            state.tenant_context.as_ref(),
        )
        .await
    {
        Ok(_) => {
            // Record successful operation in metrics
            monitoring::record_auth_operation("verification_send", "success");

            // Record duration
            let duration = start.elapsed();
            monitoring::record_request_duration(
                duration.as_secs_f64(),
                "POST",
                "/auth/verify/send",
            );

            // Create response
            let response = SendVerificationResponse {
                success: true,
                user_id: user_id.to_string(),
                verification_type: verified_type_to_string(verification_type),
            };

            info!(
                request_id = %request_id,
                user_id = %user_id,
                tenant_id = %tenant_id,
                verification_type = ?verification_type,
                "Verification code sent successfully"
            );

            let api_response = ApiResponse::success(response, request_id);
            (StatusCode::OK, Json(api_response)).into_response()
        },
        Err(err) => {
            // Record failed operation in metrics
            monitoring::record_auth_operation("verification_send", "failure");

            // Handle the error
            let (status, message, code) = match err {
                acci_core::error::Error::Validation(ref msg) => {
                    // Handle validation errors
                    if msg.contains("Rate limit") {
                        (
                            StatusCode::TOO_MANY_REQUESTS,
                            "Rate limit exceeded. Please try again later.",
                            "RATE_LIMIT_EXCEEDED",
                        )
                    } else {
                        (StatusCode::BAD_REQUEST, msg.as_str(), "VALIDATION_ERROR")
                    }
                },
                _ => {
                    // Handle other errors
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to send verification code",
                        "VERIFICATION_ERROR",
                    )
                },
            };

            warn!(
                request_id = %request_id,
                error = %err,
                user_id = %user_id,
                tenant_id = %tenant_id,
                verification_type = ?verification_type,
                "Failed to send verification code"
            );

            ApiError::new(status, message, code, request_id).into_response()
        },
    }
}

/// Handler for verifying a code
#[axum::debug_handler]
pub async fn verify_code(
    State(state): State<VerificationAppState>,
    Json(request): Json<VerifyCodeRequest>,
) -> Response {
    debug!("Processing verify code request");
    let start = std::time::Instant::now();

    // Generate a unique request ID
    let request_id = generate_request_id();

    // Validate the request
    let validated = match validate_json_payload(Json(request)).await {
        Ok(data) => data,
        Err(validation_error) => {
            // Convert validation error to response
            return validation_error.into_response();
        },
    };

    // Record operation attempt in metrics
    monitoring::record_auth_operation("verification_verify", "attempt");

    // Parse UUIDs
    let user_id = match Uuid::parse_str(&validated.user_id) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::new(
                StatusCode::BAD_REQUEST,
                "Invalid user ID format",
                "INVALID_USER_ID",
                request_id,
            )
            .into_response();
        },
    };

    let tenant_id = match Uuid::parse_str(&validated.tenant_id) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::new(
                StatusCode::BAD_REQUEST,
                "Invalid tenant ID format",
                "INVALID_TENANT_ID",
                request_id,
            )
            .into_response();
        },
    };

    // Convert verification type string to enum
    let verification_type = match validated.verification_type.to_lowercase().as_str() {
        "email" => VerificationType::Email,
        "sms" => VerificationType::Sms,
        _ => {
            return ApiError::new(
                StatusCode::BAD_REQUEST,
                "Invalid verification type",
                "INVALID_VERIFICATION_TYPE",
                request_id,
            )
            .into_response();
        },
    };

    // Verify the code
    match state
        .verification_service
        .verify_code(
            user_id,
            verification_type,
            &validated.code,
            tenant_id,
            state.tenant_context.as_ref(),
        )
        .await
    {
        Ok(_) => {
            // If session token is provided, update the session MFA status
            if let Some(session_token) = &validated.session_token {
                if let Err(err) = state
                    .session_service
                    .update_session_mfa_status(
                        session_token,
                        acci_auth::session::types::MfaStatus::Verified,
                    )
                    .await
                {
                    warn!(
                        request_id = %request_id,
                        error = %err,
                        user_id = %user_id,
                        tenant_id = %tenant_id,
                        "Failed to update session MFA status"
                    );
                    // Continue even if this fails, as the verification itself was successful
                }
            }

            // Record successful operation in metrics
            monitoring::record_auth_operation("verification_verify", "success");

            // Record duration
            let duration = start.elapsed();
            monitoring::record_request_duration(
                duration.as_secs_f64(),
                "POST",
                "/auth/verify/code",
            );

            // Create response
            let response = VerifyCodeResponse {
                success: true,
                user_id: user_id.to_string(),
                verification_type: verified_type_to_string(verification_type),
            };

            info!(
                request_id = %request_id,
                user_id = %user_id,
                tenant_id = %tenant_id,
                verification_type = ?verification_type,
                "Verification code verified successfully"
            );

            let api_response = ApiResponse::success(response, request_id);
            (StatusCode::OK, Json(api_response)).into_response()
        },
        Err(err) => {
            // Record failed operation in metrics
            monitoring::record_auth_operation("verification_verify", "failure");

            // Handle the error
            let (status, message, code) = match err {
                acci_core::error::Error::Validation(ref msg) => {
                    // Handle validation errors
                    match msg.as_str() {
                        "Invalid verification code" => (
                            StatusCode::BAD_REQUEST,
                            "Invalid verification code",
                            "INVALID_CODE",
                        ),
                        "Code has expired" => (
                            StatusCode::BAD_REQUEST,
                            "Verification code has expired",
                            "CODE_EXPIRED",
                        ),
                        "Too many verification attempts" => (
                            StatusCode::BAD_REQUEST,
                            "Too many verification attempts",
                            "TOO_MANY_ATTEMPTS",
                        ),
                        "Rate limit exceeded" => (
                            StatusCode::TOO_MANY_REQUESTS,
                            "Rate limit exceeded. Please try again later.",
                            "RATE_LIMIT_EXCEEDED",
                        ),
                        _ => (StatusCode::BAD_REQUEST, msg.as_str(), "VALIDATION_ERROR"),
                    }
                },
                _ => {
                    // Handle other errors
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to verify code",
                        "VERIFICATION_ERROR",
                    )
                },
            };

            // If session token is provided, update the session MFA status to Failed
            if let Some(session_token) = &validated.session_token {
                if let Err(update_err) = state
                    .session_service
                    .update_session_mfa_status(
                        session_token,
                        acci_auth::session::types::MfaStatus::None, // Failed verification, set back to None
                    )
                    .await
                {
                    warn!(
                        request_id = %request_id,
                        error = %update_err,
                        user_id = %user_id,
                        tenant_id = %tenant_id,
                        "Failed to update session MFA status"
                    );
                    // Continue even if this fails, as we need to return the original error
                }
            }

            warn!(
                request_id = %request_id,
                error = %err,
                user_id = %user_id,
                tenant_id = %tenant_id,
                verification_type = ?verification_type,
                "Failed to verify code"
            );

            ApiError::new(status, message, code, request_id).into_response()
        },
    }
}

/// Helper function to convert verification type to string
fn verified_type_to_string(verification_type: VerificationType) -> String {
    match verification_type {
        VerificationType::Email => "email".to_string(),
        VerificationType::Sms => "sms".to_string(),
    }
}
