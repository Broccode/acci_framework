use crate::response::{ApiError, ApiResponse};
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Import auth services and models
use acci_auth::{
    CreateUser,
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
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Login Response DTO
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
    pub expires_at: i64,
}

/// Handler for API login request
#[axum::debug_handler]
pub async fn api_login(
    State(state): State<ApiAppState>,
    Json(request): Json<LoginRequest>,
) -> impl IntoResponse {
    // Perform login process
    match state
        .user_service
        .login(
            &request.email,
            &request.password,
            None, // device_id
            None, // device_fingerprint
            None, // ip_address
            None, // user_agent
        )
        .await
    {
        Ok(login_result) => {
            // Successful login
            let response = LoginResponse {
                token: login_result.session_token,
                user_id: login_result.user.id.to_string(),
                expires_at: 0, // We need to get this from somewhere else or compute it
            };

            let api_response =
                ApiResponse::success(response, format!("login-{}", login_result.user.id));

            (StatusCode::OK, Json(api_response)).into_response()
        },
        Err(err) => {
            // Login error
            let status_code = match err {
                UserServiceError::InvalidCredentials => StatusCode::UNAUTHORIZED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            let error = ApiError::new(
                status_code,
                err.to_string(),
                status_code.as_u16().to_string(),
                "login-error".to_string(),
            );

            error.into_response()
        },
    }
}

/// Registration Request DTO
#[derive(Debug, Deserialize)]
pub struct RegistrationRequest {
    pub email: String,
    pub password: String,
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
) -> impl IntoResponse {
    // Validate passwords
    if request.password != request.password_confirmation {
        let error = ApiError::new(
            StatusCode::BAD_REQUEST,
            "Passwords do not match",
            "VALIDATION_ERROR".to_string(),
            "register-validation-error".to_string(),
        );

        return error.into_response();
    }

    // Create registration data
    let create_user = CreateUser {
        email: request.email,
        password: request.password,
    };

    // Perform registration
    match state.user_service.register(create_user).await {
        Ok(user) => {
            // Successful registration
            let response = RegistrationResponse {
                user_id: user.id.to_string(),
                email: user.email,
            };

            let api_response = ApiResponse::success(response, format!("register-{}", user.id));

            (StatusCode::CREATED, Json(api_response)).into_response()
        },
        Err(err) => {
            // Registration error
            let error_message = err.to_string();
            let (status_code, error_code) = match &err {
                UserServiceError::User(user_err) if error_message.contains("already exists") => {
                    (StatusCode::CONFLICT, "USER_ALREADY_EXISTS")
                },
                UserServiceError::Password(_) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR"),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_SERVER_ERROR"),
            };

            let error = ApiError::new(
                status_code,
                error_message,
                error_code.to_string(),
                "register-error".to_string(),
            );

            error.into_response()
        },
    }
}

/// Handler for token validation
#[axum::debug_handler]
pub async fn validate_token(
    State(state): State<ApiAppState>,
    Json(token): Json<String>,
) -> impl IntoResponse {
    match state.user_service.validate_session(&token).await {
        Ok(_) => {
            let api_response = ApiResponse::success(true, "token-valid".to_string());

            (StatusCode::OK, Json(api_response)).into_response()
        },
        Err(_) => {
            let error = ApiError::new(
                StatusCode::UNAUTHORIZED,
                "Invalid or expired token",
                "INVALID_TOKEN".to_string(),
                "token-invalid".to_string(),
            );

            error.into_response()
        },
    }
}
