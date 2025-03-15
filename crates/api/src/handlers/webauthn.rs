use crate::response::{ApiError, ApiResponse};
use crate::validation::{ValidatedJson, generate_request_id};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;
use validator::Validate;

// Import auth services
use acci_auth::{
    models::webauthn::{PublicKeyCredential, RegisterCredential},
    services::{session::SessionService, user::UserService, webauthn::WebAuthnService},
};

/// API State for WebAuthn operations
#[derive(Clone)]
pub struct WebAuthnAppState {
    /// User service for authentication
    pub user_service: Arc<UserService>,
    /// WebAuthn service for WebAuthn operations
    pub webauthn_service: Arc<WebAuthnService>,
    /// Session service for session management
    pub session_service: Arc<SessionService>,
}

/// Request to start WebAuthn registration
#[derive(Debug, Deserialize, Validate)]
pub struct StartRegistrationRequest {
    /// The user ID to register a credential for
    pub user_id: Uuid,

    /// The tenant ID for multi-tenant context
    #[validate(length(min = 36, max = 36, message = "Tenant ID must be a valid UUID"))]
    pub tenant_id: String,
}

/// Response for WebAuthn registration challenge
#[derive(Debug, Serialize)]
pub struct RegistrationChallengeResponse {
    /// The challenge options for the WebAuthn API
    pub challenge: serde_json::Value,
}

/// Request to complete WebAuthn registration
#[derive(Debug, Deserialize, Validate)]
pub struct CompleteRegistrationRequest {
    /// The credential from the authenticator
    pub credential: RegisterCredential,

    /// The tenant ID for multi-tenant context
    #[validate(length(min = 36, max = 36, message = "Tenant ID must be a valid UUID"))]
    pub tenant_id: String,
}

/// Response for completed WebAuthn registration
#[derive(Debug, Serialize)]
pub struct RegistrationCompleteResponse {
    /// The unique ID of the registered credential
    pub credential_id: String,

    /// The name of the credential
    pub name: String,

    /// When the credential was created
    pub created_at: String,
}

/// Request to start WebAuthn authentication
#[derive(Debug, Deserialize, Validate)]
pub struct StartAuthenticationRequest {
    /// Optional user ID to authenticate (if known)
    pub user_id: Option<Uuid>,

    /// The tenant ID for multi-tenant context
    #[validate(length(min = 36, max = 36, message = "Tenant ID must be a valid UUID"))]
    pub tenant_id: String,
}

/// Response for WebAuthn authentication challenge
#[derive(Debug, Serialize)]
pub struct AuthenticationChallengeResponse {
    /// The challenge options for the WebAuthn API
    pub challenge: serde_json::Value,
}

/// Request to complete WebAuthn authentication
#[derive(Debug, Deserialize, Validate)]
pub struct CompleteAuthenticationRequest {
    /// The credential from the authenticator
    pub credential: PublicKeyCredential,

    /// The tenant ID for multi-tenant context
    #[validate(length(min = 36, max = 36, message = "Tenant ID must be a valid UUID"))]
    pub tenant_id: String,

    /// The session ID to associate with this authentication
    pub session_id: Uuid,
}

/// Response for completed WebAuthn authentication
#[derive(Debug, Serialize)]
pub struct AuthenticationCompleteResponse {
    /// The user ID that was authenticated
    pub user_id: Uuid,

    /// The session ID that was associated with this authentication
    pub session_id: Uuid,

    /// Whether the session has been marked as MFA verified
    pub mfa_verified: bool,
}

/// Handler to start WebAuthn registration
// Temporarily disabled for compilation purposes
// #[axum::debug_handler]
pub async fn start_registration(
    State(state): State<WebAuthnAppState>,
    ValidatedJson(request): ValidatedJson<StartRegistrationRequest>,
) -> Response {
    debug!(
        "Starting WebAuthn registration for user: {}",
        request.user_id
    );

    // Generate request ID for tracing
    let request_id = generate_request_id();

    // Parse the tenant ID
    let tenant_id = match Uuid::parse_str(&request.tenant_id) {
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

    // Get the user
    let user = match state.user_service.get_user(request.user_id).await {
        Ok(user) => user,
        Err(acci_auth::services::user::UserServiceError::UserNotFound) => {
            return ApiError::new(
                StatusCode::NOT_FOUND,
                "User not found",
                "USER_NOT_FOUND",
                request_id,
            )
            .into_response();
        },
        Err(err) => {
            warn!(
                request_id = %request_id,
                error = %err,
                user_id = %request.user_id,
                "Error getting user for WebAuthn registration"
            );

            return ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to get user",
                "USER_ERROR",
                request_id,
            )
            .into_response();
        },
    };

    // Create session data container for WebAuthn state
    let mut session_data = serde_json::json!({});

    // Start registration
    match state
        .webauthn_service
        .start_registration(&user, &tenant_id, &mut session_data)
        .await
    {
        Ok(challenge) => {
            info!(
                request_id = %request_id,
                user_id = %user.id,
                "WebAuthn registration challenge created"
            );

            let response = RegistrationChallengeResponse { challenge };

            // Return the challenge with session data in a cookie or header
            let api_response = ApiResponse::success(response, request_id);
            // TODO: Add session data to cookie or header
            (StatusCode::OK, Json(api_response)).into_response()
        },
        Err(err) => {
            warn!(
                request_id = %request_id,
                error = %err,
                user_id = %user.id,
                "Failed to create WebAuthn registration challenge"
            );

            let (status, message, code) = map_webauthn_error(&err);
            ApiError::new(status, message, code, request_id).into_response()
        },
    }
}

/// Handler to complete WebAuthn registration
// Temporarily disabled for compilation purposes
// #[axum::debug_handler]
pub async fn complete_registration(
    State(state): State<WebAuthnAppState>,
    Path(user_id): Path<Uuid>,
    ValidatedJson(request): ValidatedJson<CompleteRegistrationRequest>,
) -> Response {
    debug!("Completing WebAuthn registration for user: {}", user_id);

    // Generate request ID for tracing
    let request_id = generate_request_id();

    // Parse the tenant ID
    let tenant_id = match Uuid::parse_str(&request.tenant_id) {
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

    // Get the user
    let user = match state.user_service.get_user(user_id).await {
        Ok(user) => user,
        Err(acci_auth::services::user::UserServiceError::UserNotFound) => {
            return ApiError::new(
                StatusCode::NOT_FOUND,
                "User not found",
                "USER_NOT_FOUND",
                request_id,
            )
            .into_response();
        },
        Err(err) => {
            warn!(
                request_id = %request_id,
                error = %err,
                user_id = %user_id,
                "Error getting user for WebAuthn registration completion"
            );

            return ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to get user",
                "USER_ERROR",
                request_id,
            )
            .into_response();
        },
    };

    // Create session data container for WebAuthn state
    // TODO: Get this from a cookie or header
    let mut session_data = serde_json::json!({
        "webauthn_registration_state": user_id.to_string()
    });

    // Complete registration
    match state
        .webauthn_service
        .complete_registration(&user, &tenant_id, &mut session_data, request.credential)
        .await
    {
        Ok(credential) => {
            info!(
                request_id = %request_id,
                user_id = %user.id,
                credential_id = %credential.id,
                "WebAuthn registration completed successfully"
            );

            let response = RegistrationCompleteResponse {
                credential_id: credential.id.to_string(),
                name: credential.name,
                created_at: credential.created_at.to_string(),
            };

            let api_response = ApiResponse::success(response, request_id);
            (StatusCode::OK, Json(api_response)).into_response()
        },
        Err(err) => {
            warn!(
                request_id = %request_id,
                error = %err,
                user_id = %user.id,
                "Failed to complete WebAuthn registration"
            );

            let (status, message, code) = map_webauthn_error(&err);
            ApiError::new(status, message, code, request_id).into_response()
        },
    }
}

/// Handler to start WebAuthn authentication
// Temporarily disabled for compilation purposes
// #[axum::debug_handler]
pub async fn start_authentication(
    State(state): State<WebAuthnAppState>,
    ValidatedJson(request): ValidatedJson<StartAuthenticationRequest>,
) -> Response {
    debug!("Starting WebAuthn authentication");

    // Generate request ID for tracing
    let request_id = generate_request_id();

    // Parse the tenant ID
    let tenant_id = match Uuid::parse_str(&request.tenant_id) {
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

    // Create session data container for WebAuthn state
    let mut session_data = serde_json::json!({});

    // Start authentication
    match state
        .webauthn_service
        .start_authentication(request.user_id.as_ref(), &tenant_id, &mut session_data)
        .await
    {
        Ok(challenge) => {
            info!(
                request_id = %request_id,
                user_id = ?request.user_id,
                "WebAuthn authentication challenge created"
            );

            let response = AuthenticationChallengeResponse { challenge };

            // Return the challenge with session data in a cookie or header
            let api_response = ApiResponse::success(response, request_id);
            // TODO: Add session data to cookie or header
            (StatusCode::OK, Json(api_response)).into_response()
        },
        Err(err) => {
            warn!(
                request_id = %request_id,
                error = %err,
                user_id = ?request.user_id,
                "Failed to create WebAuthn authentication challenge"
            );

            let (status, message, code) = map_webauthn_error(&err);
            ApiError::new(status, message, code, request_id).into_response()
        },
    }
}

/// Handler to complete WebAuthn authentication
// Temporarily disabled for compilation purposes
// #[axum::debug_handler]
pub async fn complete_authentication(
    State(state): State<WebAuthnAppState>,
    ValidatedJson(request): ValidatedJson<CompleteAuthenticationRequest>,
) -> Response {
    debug!("Completing WebAuthn authentication");

    // Generate request ID for tracing
    let request_id = generate_request_id();

    // Parse the tenant ID
    let tenant_id = match Uuid::parse_str(&request.tenant_id) {
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

    // Create session data container for WebAuthn state
    // TODO: Get this from a cookie or header
    let mut session_data = serde_json::json!({
        "webauthn_authentication_state": Uuid::new_v4().to_string()
    });

    // Complete authentication
    match state
        .webauthn_service
        .complete_authentication(&tenant_id, &mut session_data, request.credential)
        .await
    {
        Ok((user, _credential)) => {
            // Update the session to mark it as verified with WebAuthn
            match state
                .session_service
                .update_session_mfa_status(
                    &request.session_id.to_string(),
                    acci_auth::session::types::MfaStatus::Verified,
                )
                .await
            {
                Ok(_updated_session) => {
                    info!(
                        request_id = %request_id,
                        user_id = %user.id,
                        session_id = %request.session_id,
                        "WebAuthn authentication completed successfully"
                    );

                    let response = AuthenticationCompleteResponse {
                        user_id: user.id,
                        session_id: request.session_id,
                        mfa_verified: true,
                    };

                    let api_response = ApiResponse::success(response, request_id);
                    (StatusCode::OK, Json(api_response)).into_response()
                },
                Err(err) => {
                    warn!(
                        request_id = %request_id,
                        error = %err,
                        user_id = %user.id,
                        session_id = %request.session_id,
                        "Failed to update session after WebAuthn authentication"
                    );

                    ApiError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to update session",
                        "SESSION_ERROR",
                        request_id,
                    )
                    .into_response()
                },
            }
        },
        Err(err) => {
            warn!(
                request_id = %request_id,
                error = %err,
                "Failed to complete WebAuthn authentication"
            );

            let (status, message, code) = map_webauthn_error(&err);
            ApiError::new(status, message, code, request_id).into_response()
        },
    }
}

/// Helper function to map WebAuthnError to API error information
fn map_webauthn_error(err: &acci_core::error::Error) -> (StatusCode, &'static str, &'static str) {
    match err {
        acci_core::error::Error::Validation(msg) if msg.contains("Credential not found") => (
            StatusCode::NOT_FOUND,
            "Credential not found",
            "CREDENTIAL_NOT_FOUND",
        ),
        acci_core::error::Error::Validation(msg) if msg.contains("User not found") => {
            (StatusCode::NOT_FOUND, "User not found", "USER_NOT_FOUND")
        },
        acci_core::error::Error::Validation(msg) if msg.contains("Invalid credential") => (
            StatusCode::BAD_REQUEST,
            "Invalid credential",
            "INVALID_CREDENTIAL",
        ),
        acci_core::error::Error::Validation(msg) if msg.contains("Attestation") => (
            StatusCode::BAD_REQUEST,
            "Invalid registration data",
            "INVALID_REGISTRATION",
        ),
        acci_core::error::Error::Validation(msg) if msg.contains("Authentication") => (
            StatusCode::UNAUTHORIZED,
            "Authentication failed",
            "AUTHENTICATION_FAILED",
        ),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "An error occurred during WebAuthn operation",
            "WEBAUTHN_ERROR",
        ),
    }
}
