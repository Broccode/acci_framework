use std::sync::Arc;
use uuid::Uuid;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    services::session::{SessionService, SessionServiceError},
    session::{SessionFilter, types::SessionInvalidationReason},
};

/// Session service state for dependency injection
pub struct SessionServiceState {
    pub service: Arc<SessionService>,
}

/// Request for terminating all user sessions
#[derive(Debug, Deserialize)]
pub struct TerminateUserSessionsRequest {
    pub reason: SessionInvalidationReason,
}

/// Request for terminating sessions by IP
#[derive(Debug, Deserialize)]
pub struct TerminateSessionsByIpRequest {
    pub ip_address: String,
    pub reason: SessionInvalidationReason,
}

/// Request for terminating sessions by filter
#[derive(Debug, Deserialize)]
pub struct TerminateSessionsByFilterRequest {
    pub filter: SessionFilter,
    pub reason: SessionInvalidationReason,
}

/// Response for session termination
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionTerminationResponse {
    pub terminated_count: u64,
    pub success: bool,
    pub message: String,
}

/// Handler error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

/// Map SessionServiceError to HTTP response
impl IntoResponse for SessionServiceError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            SessionServiceError::Repository(err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Repository error: {}", err))
            }
            SessionServiceError::TokenGeneration => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate token".to_string())
            }
            SessionServiceError::TokenHashing => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash token".to_string())
            }
        };

        let body = Json(ErrorResponse {
            code: status.as_u16().to_string(),
            message: error_message,
        });

        (status, body).into_response()
    }
}

/// Terminate all sessions for a user (Admin action)
///
/// This endpoint allows administrators to forcibly terminate all sessions
/// for a specific user, providing a security mechanism for various scenarios.
pub async fn terminate_user_sessions(
    State(state): State<SessionServiceState>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<TerminateUserSessionsRequest>,
) -> Result<impl IntoResponse, SessionServiceError> {
    let count = state
        .service
        .force_terminate_user_sessions(user_id, request.reason)
        .await?;

    let response = SessionTerminationResponse {
        terminated_count: count,
        success: true,
        message: format!("Successfully terminated {} sessions for user", count),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Terminate sessions from a specific IP address (Admin action)
///
/// This endpoint provides a mechanism to respond to suspicious activities
/// from specific IP addresses by terminating all associated sessions.
pub async fn terminate_sessions_by_ip(
    State(state): State<SessionServiceState>,
    Json(request): Json<TerminateSessionsByIpRequest>,
) -> Result<impl IntoResponse, SessionServiceError> {
    let count = state
        .service
        .force_terminate_sessions_by_ip(&request.ip_address, request.reason)
        .await?;

    let response = SessionTerminationResponse {
        terminated_count: count,
        success: true,
        message: format!("Successfully terminated {} sessions from IP address", count),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Terminate sessions by filter criteria (Admin action)
///
/// This endpoint allows administrators to terminate sessions based on
/// filter criteria, useful for maintenance or security operations.
pub async fn terminate_sessions_by_filter(
    State(state): State<SessionServiceState>,
    Json(request): Json<TerminateSessionsByFilterRequest>,
) -> Result<impl IntoResponse, SessionServiceError> {
    let count = state
        .service
        .force_terminate_sessions_by_filter(request.filter, request.reason)
        .await?;

    let response = SessionTerminationResponse {
        terminated_count: count,
        success: true,
        message: format!("Successfully terminated {} sessions matching filter", count),
    };

    Ok((StatusCode::OK, Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::AuthConfig,
        session::Session,
        services::session::SessionService,
    };
    use std::time::SystemTime;

    // Mock session repository for testing
    struct MockSessionRepository;

    #[async_trait::async_trait]
    impl crate::session::SessionRepository for MockSessionRepository {
        async fn create_session(
            &self,
            _user_id: Uuid,
            _token_hash: String,
            _expires_at: SystemTime,
            _device_id: Option<String>,
            _device_fingerprint: Option<crate::session::types::DeviceFingerprint>,
            _ip_address: Option<String>,
            _user_agent: Option<String>,
            _metadata: Option<serde_json::Value>,
        ) -> Result<Session, crate::session::SessionError> {
            unimplemented!()
        }

        async fn get_session(
            &self,
            _id: Uuid,
        ) -> Result<Option<Session>, crate::session::SessionError> {
            unimplemented!()
        }

        async fn get_session_by_token(
            &self,
            _token_hash: &str,
        ) -> Result<Option<Session>, crate::session::SessionError> {
            unimplemented!()
        }

        async fn get_user_sessions(
            &self,
            _user_id: Uuid,
            _filter: SessionFilter,
        ) -> Result<Vec<Session>, crate::session::SessionError> {
            unimplemented!()
        }

        async fn update_session_activity(
            &self,
            _id: Uuid,
        ) -> Result<(), crate::session::SessionError> {
            unimplemented!()
        }

        async fn invalidate_session(
            &self,
            _id: Uuid,
            _reason: SessionInvalidationReason,
        ) -> Result<(), crate::session::SessionError> {
            unimplemented!()
        }

        async fn invalidate_all_user_sessions(
            &self,
            _user_id: Uuid,
            _reason: SessionInvalidationReason,
        ) -> Result<u64, crate::session::SessionError> {
            // Simulate terminating 3 sessions
            Ok(3)
        }

        async fn invalidate_sessions_by_filter(
            &self,
            _filter: SessionFilter,
            _reason: SessionInvalidationReason,
        ) -> Result<u64, crate::session::SessionError> {
            // Simulate terminating 5 sessions
            Ok(5)
        }

        async fn invalidate_sessions_by_ip(
            &self,
            _ip_address: &str,
            _reason: SessionInvalidationReason,
        ) -> Result<u64, crate::session::SessionError> {
            // Simulate terminating 2 sessions
            Ok(2)
        }

        async fn rotate_session_token(
            &self,
            _id: Uuid,
            _new_token_hash: String,
        ) -> Result<(), crate::session::SessionError> {
            unimplemented!()
        }

        async fn cleanup_expired_sessions(&self) -> Result<u64, crate::session::SessionError> {
            unimplemented!()
        }

        async fn update_mfa_status(
            &self,
            _id: Uuid,
            _status: crate::session::types::MfaStatus,
        ) -> Result<(), crate::session::SessionError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_terminate_user_sessions() {
        // Setup
        let repo = Arc::new(MockSessionRepository);
        let config = Arc::new(AuthConfig::default());
        let service = Arc::new(SessionService::new(repo, config));
        let state = SessionServiceState { service };
        
        let user_id = Uuid::new_v4();
        let request = TerminateUserSessionsRequest {
            reason: SessionInvalidationReason::AdminAction,
        };
        
        // Execute
        let result = terminate_user_sessions(
            State(state),
            Path(user_id),
            Json(request),
        ).await.unwrap();
        
        // Assert
        let response: SessionTerminationResponse = serde_json::from_slice(
            &axum::body::to_bytes(result.into_response().into_body(), usize::MAX).await.unwrap()
        ).unwrap();
        
        assert_eq!(response.terminated_count, 3);
        assert!(response.success);
    }
} 