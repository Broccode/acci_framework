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
use validator::Validate;

// Import auth services and models
use acci_auth::{
    CreateUser,
    models::user::UserError,
    services::{
        session::SessionService,
        user::{UserService, UserServiceError},
    },
};

/// API Application State
#[derive(Clone)]
pub struct ApiAppState {
    /// User service for authentication
    pub user_service: Arc<UserService>,
    /// Session service for session management
    pub session_service: Arc<SessionService>,
}

/// Login Request DTO
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,

    /// Optional tenant ID for multi-tenant context
    pub tenant_id: Option<String>,
}

/// Login Response DTO
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
    pub expires_at: i64,
    pub tenant_id: Option<String>,
}

/// Handler for API login request
#[axum::debug_handler]
pub async fn api_login(
    State(state): State<ApiAppState>,
    Json(request): Json<LoginRequest>,
) -> Response {
    debug!("Processing login request");
    let start = std::time::Instant::now();

    // Generate a unique request ID
    let request_id = generate_request_id();

    // Validate the request using our new validation function
    let validated = match validate_json_payload(Json(request)).await {
        Ok(data) => data,
        Err(validation_error) => {
            // Convert validation error to response
            return validation_error.into_response();
        },
    };

    // Record login attempt in metrics
    monitoring::record_auth_operation("login", "attempt");

    // Parse tenant ID if provided
    let tenant_id = if let Some(tenant_id_str) = &validated.tenant_id {
        match uuid::Uuid::parse_str(tenant_id_str) {
            Ok(id) => Some(id),
            Err(_) => {
                // Invalid UUID format for tenant ID
                return ApiError::new(
                    StatusCode::BAD_REQUEST,
                    "Invalid tenant ID format",
                    "INVALID_TENANT_ID",
                    request_id,
                )
                .into_response();
            },
        }
    } else {
        None
    };

    // Perform login process
    match state
        .user_service
        .login(
            &validated.email,
            &validated.password,
            None, // device_id
            None, // device_fingerprint
            None, // ip_address
            None, // user_agent
        )
        .await
    {
        Ok(login_result) => {
            // Verify user has access to the requested tenant if a tenant was specified
            let tenant_id_to_use = if let Some(tenant_id) = tenant_id {
                // Check if the tenant exists and user has access to it
                // For now, just use the tenant_id that was passed in
                // TODO: Add tenant access verification logic here
                Some(tenant_id)
            } else {
                None
            };

            // If we have a tenant ID, we need to create a token with that tenant ID
            if tenant_id_to_use.is_some() && login_result.session_token.starts_with("eyJ") {
                // This is a JWT token, we should create a new one with tenant context
                // TODO: Replace token with tenant-aware token
            }

            // Record successful login in metrics
            monitoring::record_auth_operation("login", "success");

            // Record login duration
            let duration = start.elapsed();
            monitoring::record_request_duration(duration.as_secs_f64(), "POST", "/auth/login");

            // Successful login
            let response = LoginResponse {
                token: login_result.session_token,
                user_id: login_result.user.id.to_string(),
                expires_at: 0, // We need to get this from somewhere else or compute it
                tenant_id: tenant_id_to_use.map(|id| id.to_string()),
            };

            info!(
                request_id = %request_id,
                user_id = %login_result.user.id,
                tenant_id = ?tenant_id_to_use,
                "Login successful"
            );

            let api_response = ApiResponse::success(response, request_id);
            (StatusCode::OK, Json(api_response)).into_response()
        },
        Err(err) => {
            // Record failed login in metrics
            monitoring::record_auth_operation("login", "failure");

            // Login error
            let (status, message, code) = match err {
                UserServiceError::InvalidCredentials => (
                    StatusCode::UNAUTHORIZED,
                    "Invalid email or password",
                    "INVALID_CREDENTIALS",
                ),
                UserServiceError::User(UserError::InactiveUser) => {
                    (StatusCode::FORBIDDEN, "Account is locked", "ACCOUNT_LOCKED")
                },
                UserServiceError::User(UserError::UnverifiedUser) => (
                    StatusCode::FORBIDDEN,
                    "Account is not verified",
                    "ACCOUNT_UNVERIFIED",
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An error occurred during login",
                    "LOGIN_ERROR",
                ),
            };

            warn!(
                request_id = %request_id,
                error = %err,
                email = %validated.email,
                "Login failed"
            );

            ApiError::new(status, message, code, request_id).into_response()
        },
    }
}

/// Registration Request DTO
#[derive(Debug, Deserialize, Validate)]
pub struct RegistrationRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,

    #[validate(must_match(other = "password", message = "Passwords do not match"))]
    pub password_confirmation: String,
}

/// Registration Response DTO
#[derive(Debug, Serialize)]
pub struct RegistrationResponse {
    pub user_id: String,
    pub email: String,
}

/// Handler for API registration request
#[axum::debug_handler]
pub async fn api_register(
    State(state): State<ApiAppState>,
    Json(request): Json<RegistrationRequest>,
) -> Response {
    debug!("Processing registration request");
    let start = std::time::Instant::now();

    // Generate a unique request ID
    let request_id = generate_request_id();

    // Validate the request using our new validation function
    let validated = match validate_json_payload(Json(request)).await {
        Ok(data) => data,
        Err(validation_error) => {
            // Convert validation error to response
            return validation_error.into_response();
        },
    };

    // Record registration attempt
    monitoring::record_auth_operation("register", "attempt");

    // Create new user
    let create_user = CreateUser {
        email: validated.email.clone(),
        password: validated.password.clone(),
    };

    match state.user_service.register(create_user).await {
        Ok(user) => {
            // Record successful registration
            monitoring::record_auth_operation("register", "success");

            // Record registration duration
            let duration = start.elapsed();
            monitoring::record_request_duration(duration.as_secs_f64(), "POST", "/auth/register");

            let response = RegistrationResponse {
                user_id: user.id.to_string(),
                email: user.email,
            };

            info!(
                request_id = %request_id,
                user_id = %user.id,
                "User registration successful"
            );

            let api_response = ApiResponse::success(response, request_id);
            (StatusCode::CREATED, Json(api_response)).into_response()
        },
        Err(err) => {
            // Record failed registration
            monitoring::record_auth_operation("register", "failure");

            // Registration error
            let (status, message, code) = match err {
                UserServiceError::User(UserError::AlreadyExists) => (
                    StatusCode::CONFLICT,
                    "User with this email already exists",
                    "USER_ALREADY_EXISTS",
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An error occurred during registration",
                    "REGISTRATION_ERROR",
                ),
            };

            warn!(
                request_id = %request_id,
                error = %err,
                email = %validated.email,
                "Registration failed"
            );

            ApiError::new(status, message, code, request_id).into_response()
        },
    }
}

/// Handler for token validation
#[axum::debug_handler]
pub async fn validate_token(
    State(state): State<ApiAppState>,
    Json(token): Json<String>,
) -> Response {
    debug!("Processing token validation request");

    // Generate a unique request ID
    let request_id = generate_request_id();

    // No validation needed for token as it's just a string
    // Record validation attempt
    monitoring::record_auth_operation("validate_token", "attempt");

    match state.session_service.validate_session(&token).await {
        Ok(Some(session)) => {
            // Record successful validation
            monitoring::record_auth_operation("validate_token", "success");

            info!(
                request_id = %request_id,
                user_id = %session.user_id,
                "Token validation successful"
            );

            let api_response = ApiResponse::success(true, request_id);
            (StatusCode::OK, Json(api_response)).into_response()
        },
        _ => {
            // Record failed validation
            monitoring::record_auth_operation("validate_token", "failure");

            warn!(
                request_id = %request_id,
                "Token validation failed"
            );

            ApiError::authentication_error(request_id).into_response()
        },
    }
}
