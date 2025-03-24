pub mod enhanced_security;
pub mod types;

use async_trait::async_trait;
use serde_json::Value;
use sqlx::types::ipnetwork::IpNetwork;
use std::time::{Duration, SystemTime};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::session::types::{DeviceFingerprint, MfaStatus, SessionInvalidationReason};

const _METRIC_PREFIX: &str = "auth.session";
const METRIC_CREATE: &str = "create";
const METRIC_GET: &str = "get";
const METRIC_GET_BY_TOKEN: &str = "get_by_token";
const METRIC_GET_USER: &str = "get_user";
const METRIC_UPDATE_ACTIVITY: &str = "update_activity";
const METRIC_INVALIDATE: &str = "invalidate";
const METRIC_ROTATE_TOKEN: &str = "rotate_token";
const METRIC_CLEANUP: &str = "cleanup";

// Mock implementations when metrics feature is not enabled
#[cfg(not(feature = "metrics"))]
mod metrics_mock {
    #[macro_export]
    macro_rules! counter {
        ($name:expr) => {{
            struct Counter {}
            impl Counter {
                pub fn increment(&self, _value: u64) {}
            }
            Counter {}
        }};
    }

    #[macro_export]
    macro_rules! histogram {
        ($name:expr) => {{
            struct Histogram {}
            impl Histogram {
                pub fn record(&self, _value: f64) {}
            }
            Histogram {}
        }};
    }
}

// Explicitly import macros when metrics is not enabled

// Hilfsfunktion zur Konvertierung von SystemTime zu OffsetDateTime
fn system_time_to_offset_date_time(time: SystemTime) -> OffsetDateTime {
    let unix_time = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    OffsetDateTime::from_unix_timestamp(unix_time.as_secs() as i64)
        .unwrap_or(OffsetDateTime::UNIX_EPOCH)
        .saturating_add(time::Duration::nanoseconds(unix_time.subsec_nanos() as i64))
}

// Hilfsfunktion zur Konvertierung von String zu IpNetwork
fn string_to_ip_network(ip_str: Option<String>) -> Option<IpNetwork> {
    ip_str.and_then(|s| s.parse::<IpNetwork>().ok())
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub previous_token_hash: Option<String>,
    pub token_rotation_at: Option<SystemTime>,
    pub expires_at: SystemTime,
    pub created_at: SystemTime,
    pub last_activity_at: SystemTime,
    pub last_activity_update_at: Option<SystemTime>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_id: Option<String>,
    pub device_fingerprint: Option<DeviceFingerprint>,
    pub is_valid: bool,
    pub invalidated_reason: Option<SessionInvalidationReason>,
    pub metadata: Option<Value>,
    pub mfa_status: MfaStatus,
}

#[derive(Debug, Clone)]
pub enum SessionFilter {
    All,
    Active,
    Inactive,
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Session not found")]
    NotFound,
    #[error("Session expired")]
    Expired,
    #[error("Session invalid")]
    Invalid,
    #[error("Token mismatch")]
    TokenMismatch,
}

#[derive(Debug, Clone)]
pub struct SessionRepositoryConfig {
    /// Duration after which invalid sessions are deleted
    pub invalid_session_retention: Duration,
    /// Duration after which audit logs are deleted
    pub audit_log_retention: Duration,
    /// Duration after which session activity updates are allowed
    pub activity_update_interval: Duration,
}

impl Default for SessionRepositoryConfig {
    fn default() -> Self {
        Self {
            invalid_session_retention: Duration::from_secs(90 * 24 * 60 * 60), // 90 days
            audit_log_retention: Duration::from_secs(90 * 24 * 60 * 60),       // 90 days
            activity_update_interval: Duration::from_secs(5 * 60),             // 5 minutes
        }
    }
}

#[async_trait]
pub trait SessionRepository: Send + Sync + 'static {
    async fn create_session(
        &self,
        user_id: Uuid,
        token_hash: String,
        expires_at: SystemTime,
        device_id: Option<String>,
        device_fingerprint: Option<DeviceFingerprint>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        metadata: Option<Value>,
    ) -> Result<Session, SessionError>;

    async fn get_session(&self, id: Uuid) -> Result<Option<Session>, SessionError>;

    async fn get_session_by_token(&self, token_hash: &str)
    -> Result<Option<Session>, SessionError>;

    async fn get_user_sessions(
        &self,
        user_id: Uuid,
        filter: SessionFilter,
    ) -> Result<Vec<Session>, SessionError>;

    async fn update_session_activity(&self, id: Uuid) -> Result<(), SessionError>;

    async fn invalidate_session(
        &self,
        id: Uuid,
        reason: SessionInvalidationReason,
    ) -> Result<(), SessionError>;

    async fn rotate_session_token(
        &self,
        id: Uuid,
        new_token_hash: String,
    ) -> Result<(), SessionError>;

    async fn cleanup_expired_sessions(&self) -> Result<u64, SessionError>;
}

pub struct PostgresSessionRepository {
    pool: sqlx::PgPool,
    config: SessionRepositoryConfig,
}

impl PostgresSessionRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool,
            config: SessionRepositoryConfig::default(),
        }
    }

    pub fn with_config(pool: sqlx::PgPool, config: SessionRepositoryConfig) -> Self {
        Self { pool, config }
    }

    fn record_metrics(_operation: &str, start_time: SystemTime) {
        // Temporarily disabled for compilation
        let _ = (_operation, start_time);
    }

    fn record_error_metrics(_operation: &str, _error: &SessionError) {
        // Temporarily disabled for compilation
        let _ = (_operation, _error);
    }
}

impl SessionError {
    #[allow(dead_code)]
    fn metric_name(&self) -> &'static str {
        match self {
            Self::Database(_) => "database_error",
            Self::NotFound => "not_found",
            Self::Expired => "expired",
            Self::Invalid => "invalid",
            Self::TokenMismatch => "token_mismatch",
        }
    }
}

#[async_trait]
impl SessionRepository for PostgresSessionRepository {
    async fn create_session(
        &self,
        user_id: Uuid,
        token_hash: String,
        expires_at: SystemTime,
        device_id: Option<String>,
        device_fingerprint: Option<DeviceFingerprint>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        metadata: Option<Value>,
    ) -> Result<Session, SessionError> {
        let start = SystemTime::now();
        tracing::debug!(
            user_id = %user_id,
            device_id = ?device_id,
            "Creating new session"
        );

        let result: Result<Session, SessionError> = async {
            let device_fingerprint_json =
                device_fingerprint.map(|fp| serde_json::to_value(fp).unwrap());
            let now = SystemTime::now();
            let now_offset = system_time_to_offset_date_time(now);
            let expires_at_offset = system_time_to_offset_date_time(expires_at);
            let ip_network = string_to_ip_network(ip_address.clone());

            let row = sqlx::query!(
                r#"
                INSERT INTO sessions (
                    id, user_id, token_hash, expires_at, created_at, last_activity_at,
                    ip_address, user_agent, device_id, device_fingerprint, is_valid, metadata,
                    mfa_status
                )
                VALUES (
                    gen_random_uuid(), $1, $2, $3, $4, $5,
                    $6, $7, $8, $9, true, $10, $11
                )
                RETURNING
                    id, user_id, token_hash, previous_token_hash, token_rotation_at,
                    expires_at, created_at, last_activity_at, last_activity_update_at,
                    ip_address, user_agent, device_id, device_fingerprint,
                    is_valid, invalidated_reason::text, metadata, mfa_status
                "#,
                user_id,
                token_hash,
                expires_at_offset,
                now_offset,
                now_offset,
                ip_network,
                user_agent,
                device_id,
                device_fingerprint_json,
                metadata,
                "NONE", // Default MFA status for new sessions
            )
            .fetch_one(&self.pool)
            .await
            .map_err(SessionError::Database)?;

            let mfa_status = match row.mfa_status.as_str() {
                "NONE" => MfaStatus::None,
                "PENDING" => MfaStatus::Pending,
                "VERIFIED" => MfaStatus::Verified,
                "FAILED" => MfaStatus::Failed,
                _ => MfaStatus::None, // Default if not specified
            };

            Ok(Session {
                id: row.id,
                user_id: row.user_id,
                token_hash: row.token_hash,
                previous_token_hash: row.previous_token_hash,
                token_rotation_at: row.token_rotation_at.map(|t| t.into()),
                expires_at: row.expires_at.into(),
                created_at: row.created_at.into(),
                last_activity_at: row.last_activity_at.into(),
                last_activity_update_at: row.last_activity_update_at.map(|t| t.into()),
                ip_address: row.ip_address.map(|ip| ip.to_string()),
                user_agent: row.user_agent,
                device_id: row.device_id,
                device_fingerprint: row
                    .device_fingerprint
                    .map(|v| serde_json::from_value(v).unwrap()),
                is_valid: row.is_valid,
                invalidated_reason: row
                    .invalidated_reason
                    .map(|r| serde_json::from_str(&r.to_string()).unwrap()),
                metadata: row.metadata,
                mfa_status,
            })
        }
        .await;

        match &result {
            Ok(session) => {
                tracing::info!(
                    session_id = %session.id,
                    user_id = %session.user_id,
                    device_id = ?session.device_id,
                    "Session created successfully"
                );
                Self::record_metrics(METRIC_CREATE, start);
            },
            Err(error) => {
                tracing::error!(
                    user_id = %user_id,
                    error = %error,
                    "Failed to create session"
                );
                Self::record_error_metrics(METRIC_CREATE, error);
            },
        }

        result
    }

    async fn get_session(&self, id: Uuid) -> Result<Option<Session>, SessionError> {
        let start = SystemTime::now();
        tracing::debug!(session_id = %id, "Getting session by ID");

        let result: Result<Option<Session>, SessionError> = async {
            let row = sqlx::query!(
                r#"
                SELECT
                    id, user_id, token_hash, previous_token_hash, token_rotation_at,
                    expires_at, created_at, last_activity_at, last_activity_update_at,
                    ip_address, user_agent, device_id, device_fingerprint,
                    is_valid, invalidated_reason::text as "invalidated_reason?", metadata,
                    mfa_status
                FROM sessions
                WHERE id = $1
                "#,
                id
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(SessionError::Database)?;

            Ok(row.map(|row| {
                let mfa_status = match row.mfa_status.as_str() {
                    "NONE" => MfaStatus::None,
                    "PENDING" => MfaStatus::Pending,
                    "VERIFIED" => MfaStatus::Verified,
                    "FAILED" => MfaStatus::Failed,
                    _ => MfaStatus::None, // Default if not specified
                };

                Session {
                    id: row.id,
                    user_id: row.user_id,
                    token_hash: row.token_hash,
                    previous_token_hash: row.previous_token_hash,
                    token_rotation_at: row.token_rotation_at.map(|t| t.into()),
                    expires_at: row.expires_at.into(),
                    created_at: row.created_at.into(),
                    last_activity_at: row.last_activity_at.into(),
                    last_activity_update_at: row.last_activity_update_at.map(|t| t.into()),
                    ip_address: row.ip_address.map(|ip| ip.to_string()),
                    user_agent: row.user_agent,
                    device_id: row.device_id,
                    device_fingerprint: row
                        .device_fingerprint
                        .map(|v| serde_json::from_value(v).unwrap()),
                    is_valid: row.is_valid,
                    invalidated_reason: row
                        .invalidated_reason
                        .map(|r| serde_json::from_str(&r.to_string()).unwrap()),
                    metadata: row.metadata,
                    mfa_status,
                }
            }))
        }
        .await;

        match &result {
            Ok(session) => {
                tracing::debug!(
                    session_id = %id,
                    found = session.is_some(),
                    "Session lookup completed"
                );
                Self::record_metrics(METRIC_GET, start);
            },
            Err(error) => {
                tracing::error!(
                    session_id = %id,
                    error = %error,
                    "Failed to get session"
                );
                Self::record_error_metrics(METRIC_GET, error);
            },
        }

        result
    }

    async fn get_session_by_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<Session>, SessionError> {
        let start = SystemTime::now();
        tracing::debug!("Getting session by token hash");

        let result: Result<Option<Session>, SessionError> = async {
            let row = sqlx::query!(
                r#"
                SELECT
                    id, user_id, token_hash, previous_token_hash, token_rotation_at,
                    expires_at, created_at, last_activity_at, last_activity_update_at,
                    ip_address, user_agent, device_id, device_fingerprint,
                    is_valid, invalidated_reason::text as "invalidated_reason?", metadata,
                    mfa_status
                FROM sessions
                WHERE token_hash = $1 OR previous_token_hash = $1
                "#,
                token_hash
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(SessionError::Database)?;

            Ok(row.map(|row| {
                let mfa_status = match row.mfa_status.as_str() {
                    "NONE" => MfaStatus::None,
                    "PENDING" => MfaStatus::Pending,
                    "VERIFIED" => MfaStatus::Verified,
                    "FAILED" => MfaStatus::Failed,
                    _ => MfaStatus::None, // Default if not specified
                };

                Session {
                    id: row.id,
                    user_id: row.user_id,
                    token_hash: row.token_hash,
                    previous_token_hash: row.previous_token_hash,
                    token_rotation_at: row.token_rotation_at.map(|t| t.into()),
                    expires_at: row.expires_at.into(),
                    created_at: row.created_at.into(),
                    last_activity_at: row.last_activity_at.into(),
                    last_activity_update_at: row.last_activity_update_at.map(|t| t.into()),
                    ip_address: row.ip_address.map(|ip| ip.to_string()),
                    user_agent: row.user_agent,
                    device_id: row.device_id,
                    device_fingerprint: row
                        .device_fingerprint
                        .map(|v| serde_json::from_value(v).unwrap()),
                    is_valid: row.is_valid,
                    invalidated_reason: row
                        .invalidated_reason
                        .map(|r| serde_json::from_str(&r.to_string()).unwrap()),
                    metadata: row.metadata,
                    mfa_status,
                }
            }))
        }
        .await;

        match &result {
            Ok(session) => {
                tracing::debug!(found = session.is_some(), "Session token lookup completed");
                Self::record_metrics(METRIC_GET_BY_TOKEN, start);
            },
            Err(error) => {
                tracing::error!(
                    error = %error,
                    "Failed to get session by token"
                );
                Self::record_error_metrics(METRIC_GET_BY_TOKEN, error);
            },
        }

        result
    }

    async fn get_user_sessions(
        &self,
        user_id: Uuid,
        filter: SessionFilter,
    ) -> Result<Vec<Session>, SessionError> {
        let start = SystemTime::now();
        tracing::debug!(
            user_id = %user_id,
            filter = ?filter,
            "Getting user sessions"
        );

        let result: Result<Vec<Session>, SessionError> = async {
            let (is_valid, include_filter) = match filter {
                SessionFilter::All => (true, false),
                SessionFilter::Active => (true, true),
                SessionFilter::Inactive => (false, true),
            };

            let rows = sqlx::query!(
                r#"
                SELECT
                    id, user_id, token_hash, previous_token_hash, token_rotation_at,
                    expires_at, created_at, last_activity_at, last_activity_update_at,
                    ip_address, user_agent, device_id, device_fingerprint,
                    is_valid, invalidated_reason::text as "invalidated_reason?", metadata,
                    mfa_status
                FROM sessions
                WHERE user_id = $1
                AND ($2 = false OR is_valid = $3)
                ORDER BY created_at DESC
                "#,
                user_id,
                include_filter,
                is_valid
            )
            .fetch_all(&self.pool)
            .await
            .map_err(SessionError::Database)?;

            Ok(rows
                .into_iter()
                .map(|row| {
                    let mfa_status = match row.mfa_status.as_str() {
                        "NONE" => MfaStatus::None,
                        "PENDING" => MfaStatus::Pending,
                        "VERIFIED" => MfaStatus::Verified,
                        "FAILED" => MfaStatus::Failed,
                        _ => MfaStatus::None, // Default if not specified
                    };

                    Session {
                        id: row.id,
                        user_id: row.user_id,
                        token_hash: row.token_hash,
                        previous_token_hash: row.previous_token_hash,
                        token_rotation_at: row.token_rotation_at.map(|t| t.into()),
                        expires_at: row.expires_at.into(),
                        created_at: row.created_at.into(),
                        last_activity_at: row.last_activity_at.into(),
                        last_activity_update_at: row.last_activity_update_at.map(|t| t.into()),
                        ip_address: row.ip_address.map(|ip| ip.to_string()),
                        user_agent: row.user_agent,
                        device_id: row.device_id,
                        device_fingerprint: row
                            .device_fingerprint
                            .map(|v| serde_json::from_value(v).unwrap()),
                        is_valid: row.is_valid,
                        invalidated_reason: row
                            .invalidated_reason
                            .map(|r| serde_json::from_str(&r.to_string()).unwrap()),
                        metadata: row.metadata,
                        mfa_status,
                    }
                })
                .collect())
        }
        .await;

        match &result {
            Ok(sessions) => {
                tracing::debug!(
                    user_id = %user_id,
                    count = sessions.len(),
                    "User sessions retrieved successfully"
                );
                Self::record_metrics(METRIC_GET_USER, start);
            },
            Err(error) => {
                tracing::error!(
                    user_id = %user_id,
                    error = %error,
                    "Failed to get user sessions"
                );
                Self::record_error_metrics(METRIC_GET_USER, error);
            },
        }

        result
    }

    async fn update_session_activity(&self, id: Uuid) -> Result<(), SessionError> {
        let start = SystemTime::now();
        tracing::debug!(session_id = %id, "Updating session activity");

        let result: Result<(), SessionError> = async {
            let result = sqlx::query!(
                r#"
                UPDATE sessions
                SET last_activity_at = CURRENT_TIMESTAMP
                WHERE id = $1 AND is_valid = true
                RETURNING id
                "#,
                id
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(SessionError::Database)?;

            match result {
                Some(_) => Ok(()),
                None => Err(SessionError::NotFound),
            }
        }
        .await;

        match &result {
            Ok(_) => {
                tracing::debug!(
                    session_id = %id,
                    "Session activity updated successfully"
                );
                Self::record_metrics(METRIC_UPDATE_ACTIVITY, start);
            },
            Err(error) => {
                tracing::error!(
                    session_id = %id,
                    error = %error,
                    "Failed to update session activity"
                );
                Self::record_error_metrics(METRIC_UPDATE_ACTIVITY, error);
            },
        }

        result
    }

    async fn invalidate_session(
        &self,
        id: Uuid,
        reason: SessionInvalidationReason,
    ) -> Result<(), SessionError> {
        let start = SystemTime::now();
        tracing::debug!(
            session_id = %id,
            reason = ?reason,
            "Invalidating session"
        );

        let result: Result<(), SessionError> = async {
            let result = sqlx::query!(
                r#"
                UPDATE sessions
                SET
                    is_valid = false,
                    invalidated_reason = $2::session_invalidation_reason
                WHERE id = $1 AND is_valid = true
                RETURNING id
                "#,
                id,
                reason as _
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(SessionError::Database)?;

            match result {
                Some(_) => Ok(()),
                None => Err(SessionError::NotFound),
            }
        }
        .await;

        match &result {
            Ok(_) => {
                tracing::info!(
                    session_id = %id,
                    reason = ?reason,
                    "Session invalidated successfully"
                );
                Self::record_metrics(METRIC_INVALIDATE, start);
            },
            Err(error) => {
                tracing::error!(
                    session_id = %id,
                    reason = ?reason,
                    error = %error,
                    "Failed to invalidate session"
                );
                Self::record_error_metrics(METRIC_INVALIDATE, error);
            },
        }

        result
    }

    async fn rotate_session_token(
        &self,
        id: Uuid,
        new_token_hash: String,
    ) -> Result<(), SessionError> {
        let start = SystemTime::now();
        tracing::debug!(session_id = %id, "Rotating session token");

        let result: Result<(), SessionError> = async {
            let result = sqlx::query!(
                r#"
                UPDATE sessions
                SET
                    token_hash = $2,
                    previous_token_hash = token_hash,
                    token_rotation_at = CURRENT_TIMESTAMP
                WHERE id = $1 AND is_valid = true
                RETURNING id
                "#,
                id,
                new_token_hash
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(SessionError::Database)?;

            match result {
                Some(_) => Ok(()),
                None => Err(SessionError::NotFound),
            }
        }
        .await;

        match &result {
            Ok(_) => {
                tracing::info!(
                    session_id = %id,
                    "Session token rotated successfully"
                );
                Self::record_metrics(METRIC_ROTATE_TOKEN, start);
            },
            Err(error) => {
                tracing::error!(
                    session_id = %id,
                    error = %error,
                    "Failed to rotate session token"
                );
                Self::record_error_metrics(METRIC_ROTATE_TOKEN, error);
            },
        }

        result
    }

    async fn cleanup_expired_sessions(&self) -> Result<u64, SessionError> {
        let start = SystemTime::now();
        tracing::debug!("Starting expired sessions cleanup");

        let result: Result<u64, SessionError> = async {
            // First, invalidate expired sessions
            let invalidated = sqlx::query!(
                r#"
                UPDATE sessions
                SET
                    is_valid = false,
                    invalidated_reason = 'TOKEN_EXPIRED'::session_invalidation_reason
                WHERE
                    is_valid = true
                    AND expires_at < CURRENT_TIMESTAMP
                "#
            )
            .execute(&self.pool)
            .await
            .map_err(SessionError::Database)?;

            // Then, delete old invalid sessions and their audit logs
            let deleted = sqlx::query!(
                r#"
                WITH deleted_sessions AS (
                    DELETE FROM sessions
                    WHERE
                        is_valid = false
                        AND last_activity_at < CURRENT_TIMESTAMP - make_interval(secs => $1)
                    RETURNING id
                )
                SELECT COUNT(*) as "count!"
                FROM deleted_sessions
                "#,
                self.config.invalid_session_retention.as_secs() as i64
            )
            .fetch_one(&self.pool)
            .await
            .map_err(SessionError::Database)?;

            // Also cleanup old audit logs
            sqlx::query!(
                r#"
                DELETE FROM session_audit_log
                WHERE created_at < CURRENT_TIMESTAMP - make_interval(secs => $1)
                "#,
                self.config.audit_log_retention.as_secs() as i64
            )
            .execute(&self.pool)
            .await
            .map_err(SessionError::Database)?;

            Ok(invalidated.rows_affected() + deleted.count as u64)
        }
        .await;

        match &result {
            Ok(count) => {
                tracing::info!(
                    cleaned_sessions = count,
                    duration = ?start.elapsed().unwrap_or_default(),
                    "Session cleanup completed successfully"
                );
                Self::record_metrics(METRIC_CLEANUP, start);
            },
            Err(error) => {
                tracing::error!(
                    error = %error,
                    "Failed to cleanup expired sessions"
                );
                Self::record_error_metrics(METRIC_CLEANUP, error);
            },
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_session_validity() {
        let user_id = Uuid::new_v4();
        let session = Session {
            id: Uuid::new_v4(),
            user_id,
            token_hash: "test_token_hash".to_string(),
            previous_token_hash: None,
            token_rotation_at: None,
            expires_at: SystemTime::now() + Duration::from_secs(3600),
            created_at: SystemTime::now(),
            last_activity_at: SystemTime::now(),
            last_activity_update_at: None,
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("Test Agent".to_string()),
            device_id: Some("test_device".to_string()),
            device_fingerprint: None,
            is_valid: true,
            invalidated_reason: None,
            metadata: None,
            mfa_status: MfaStatus::None,
        };

        assert!(session.is_valid);
        assert_eq!(session.user_id, user_id);
        assert!(session.invalidated_reason.is_none());
    }

    #[test]
    fn test_session_expiry() {
        let session = Session {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            token_hash: "test_token_hash".to_string(),
            previous_token_hash: None,
            token_rotation_at: None,
            expires_at: SystemTime::now(),
            created_at: SystemTime::now(),
            last_activity_at: SystemTime::now(),
            last_activity_update_at: None,
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("Test Agent".to_string()),
            device_id: Some("test_device".to_string()),
            device_fingerprint: None,
            is_valid: true,
            invalidated_reason: None,
            metadata: None,
            mfa_status: MfaStatus::None,
        };

        assert!(SystemTime::now() >= session.expires_at);
    }

    #[test]
    fn test_session_filter() {
        let filter = SessionFilter::Active;
        match filter {
            SessionFilter::Active => (),
            _ => panic!("Expected Active filter"),
        }
    }

    #[test]
    fn test_session_error() {
        let error = SessionError::NotFound;
        assert_eq!(error.to_string(), "Session not found");

        let error = SessionError::Expired;
        assert_eq!(error.to_string(), "Session expired");
    }

    #[test]
    fn test_session_repository_config() {
        let config = SessionRepositoryConfig::default();
        assert_eq!(
            config.invalid_session_retention,
            Duration::from_secs(90 * 24 * 60 * 60)
        );
        assert_eq!(
            config.audit_log_retention,
            Duration::from_secs(90 * 24 * 60 * 60)
        );
        assert_eq!(config.activity_update_interval, Duration::from_secs(5 * 60));

        let custom_config = SessionRepositoryConfig {
            invalid_session_retention: Duration::from_secs(30 * 24 * 60 * 60),
            audit_log_retention: Duration::from_secs(60 * 24 * 60 * 60),
            activity_update_interval: Duration::from_secs(10 * 60),
        };

        assert_eq!(
            custom_config.invalid_session_retention,
            Duration::from_secs(30 * 24 * 60 * 60)
        );
        assert_eq!(
            custom_config.audit_log_retention,
            Duration::from_secs(60 * 24 * 60 * 60)
        );
        assert_eq!(
            custom_config.activity_update_interval,
            Duration::from_secs(10 * 60)
        );
    }

    #[test]
    fn test_session_error_metric_name() {
        assert_eq!(
            SessionError::Database(sqlx::Error::RowNotFound).metric_name(),
            "database_error"
        );
        assert_eq!(SessionError::NotFound.metric_name(), "not_found");
        assert_eq!(SessionError::Expired.metric_name(), "expired");
        assert_eq!(SessionError::Invalid.metric_name(), "invalid");
        assert_eq!(SessionError::TokenMismatch.metric_name(), "token_mismatch");
    }
}
