use async_trait::async_trait;
use mockall::mock;
use serde_json::Value;
use std::time::SystemTime;
use uuid::Uuid;

use crate::session::types::{DeviceFingerprint, SessionInvalidationReason};
use crate::session::{Session, SessionError, SessionFilter, SessionRepository};

mock! {
    pub SessionRepository {
        fn create_session(
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

        fn get_session(
            &self,
            id: Uuid,
        ) -> Result<Option<Session>, SessionError>;

        fn get_session_by_token(
            &self,
            token_hash: &str,
        ) -> Result<Option<Session>, SessionError>;

        fn get_user_sessions(
            &self,
            user_id: Uuid,
            filter: SessionFilter,
        ) -> Result<Vec<Session>, SessionError>;

        fn update_session_activity(
            &self,
            id: Uuid,
        ) -> Result<(), SessionError>;

        fn invalidate_session(
            &self,
            id: Uuid,
            reason: SessionInvalidationReason,
        ) -> Result<(), SessionError>;

        fn rotate_session_token(
            &self,
            id: Uuid,
            new_token_hash: String,
        ) -> Result<(), SessionError>;

        fn cleanup_expired_sessions(&self) -> Result<u64, SessionError>;
    }
}
