use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use uuid::Uuid;

use acci_auth::{
    models::{TenantId, UserId, VerificationCode, VerificationType},
    repository::TenantAwareContext,
    services::message_provider::{Message, MessageProvider},
    session::{
        Session, SessionError, SessionFilter, SessionRepository,
        types::{DeviceFingerprint, MfaStatus, SessionInvalidationReason},
    },
};
use acci_core::error::Result;

/// Mock session repository for testing
pub struct MockSessionRepository {
    sessions: Mutex<HashMap<Uuid, Session>>,
    token_map: Mutex<HashMap<String, Uuid>>,
}

impl MockSessionRepository {
    /// Create a new mock session repository
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            token_map: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl SessionRepository for MockSessionRepository {
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
        let session_id = Uuid::new_v4();
        let session = Session {
            id: session_id,
            user_id,
            token_hash: token_hash.clone(),
            expires_at,
            last_active_at: SystemTime::now(),
            created_at: SystemTime::now(),
            device_id,
            device_fingerprint,
            ip_address,
            user_agent,
            metadata,
            is_valid: true,
            invalidated_reason: None,
            mfa_status: MfaStatus::None,
        };

        let mut sessions = self.sessions.lock().await;
        sessions.insert(session_id, session.clone());

        let mut token_map = self.token_map.lock().await;
        token_map.insert(token_hash, session_id);

        Ok(session)
    }

    async fn get_session(&self, id: Uuid) -> Result<Option<Session>, SessionError> {
        let sessions = self.sessions.lock().await;
        Ok(sessions.get(&id).cloned())
    }

    async fn get_session_by_token(
        &self,
        token_hash: &str,
    ) -> Result<Option<Session>, SessionError> {
        let token_map = self.token_map.lock().await;
        if let Some(session_id) = token_map.get(token_hash) {
            let sessions = self.sessions.lock().await;
            return Ok(sessions.get(session_id).cloned());
        }
        Ok(None)
    }

    async fn get_user_sessions(
        &self,
        user_id: Uuid,
        filter: SessionFilter,
    ) -> Result<Vec<Session>, SessionError> {
        let sessions = self.sessions.lock().await;
        let mut result = vec![];

        for session in sessions.values() {
            if session.user_id != user_id {
                continue;
            }

            match filter {
                SessionFilter::All => result.push(session.clone()),
                SessionFilter::Active => {
                    if session.is_valid && session.expires_at > SystemTime::now() {
                        result.push(session.clone());
                    }
                },
                SessionFilter::Invalidated => {
                    if !session.is_valid {
                        result.push(session.clone());
                    }
                },
                SessionFilter::Expired => {
                    if session.expires_at <= SystemTime::now() {
                        result.push(session.clone());
                    }
                },
            }
        }

        Ok(result)
    }

    async fn update_session_activity(&self, id: Uuid) -> Result<(), SessionError> {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get_mut(&id) {
            session.last_active_at = SystemTime::now();
            Ok(())
        } else {
            Err(SessionError::NotFound)
        }
    }

    async fn invalidate_session(
        &self,
        id: Uuid,
        reason: SessionInvalidationReason,
    ) -> Result<(), SessionError> {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get_mut(&id) {
            session.is_valid = false;
            session.invalidated_reason = Some(reason);
            Ok(())
        } else {
            Err(SessionError::NotFound)
        }
    }

    async fn rotate_session_token(
        &self,
        id: Uuid,
        new_token_hash: String,
    ) -> Result<(), SessionError> {
        let mut sessions = self.sessions.lock().await;
        let mut token_map = self.token_map.lock().await;

        if let Some(session) = sessions.get_mut(&id) {
            // Remove old token mapping
            token_map.remove(&session.token_hash);

            // Update token hash
            session.token_hash = new_token_hash.clone();

            // Add new token mapping
            token_map.insert(new_token_hash, id);

            Ok(())
        } else {
            Err(SessionError::NotFound)
        }
    }

    async fn cleanup_expired_sessions(&self) -> Result<u64, SessionError> {
        let mut sessions = self.sessions.lock().await;
        let mut token_map = self.token_map.lock().await;
        let now = SystemTime::now();

        let mut count = 0;
        let mut to_remove = vec![];

        for (id, session) in sessions.iter() {
            if session.expires_at <= now {
                to_remove.push(*id);
                token_map.remove(&session.token_hash);
                count += 1;
            }
        }

        for id in to_remove {
            sessions.remove(&id);
        }

        Ok(count)
    }
}

/// SessionRepository extension for testing MFA functionality
#[async_trait]
pub trait SessionRepositoryExt: SessionRepository {
    /// Update the MFA status of a session
    async fn update_mfa_status(&self, id: Uuid, status: MfaStatus) -> Result<(), SessionError>;
}

#[async_trait]
impl SessionRepositoryExt for MockSessionRepository {
    async fn update_mfa_status(&self, id: Uuid, status: MfaStatus) -> Result<(), SessionError> {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get_mut(&id) {
            session.mfa_status = status;
            Ok(())
        } else {
            Err(SessionError::NotFound)
        }
    }
}

/// Mock message provider for testing
pub struct MockMessageProvider {
    messages: Mutex<Vec<Message>>,
}

impl MockMessageProvider {
    /// Create a new mock message provider
    pub fn new() -> Self {
        Self {
            messages: Mutex::new(Vec::new()),
        }
    }

    /// Get all messages sent through this provider
    pub async fn get_messages(&self) -> Vec<Message> {
        self.messages.lock().await.clone()
    }
}

#[async_trait]
impl MessageProvider for MockMessageProvider {
    async fn send_message(&self, message: Message) -> Result<()> {
        let mut messages = self.messages.lock().await;
        messages.push(message);
        Ok(())
    }
}

/// Mock verification code repository for testing
pub struct MockVerificationCodeRepository {
    codes: Mutex<Vec<VerificationCode>>,
}

impl MockVerificationCodeRepository {
    /// Create a new mock verification code repository
    pub fn new() -> Self {
        Self {
            codes: Mutex::new(Vec::new()),
        }
    }

    /// Get all codes in this repository
    pub fn get_codes(&self) -> Vec<VerificationCode> {
        let codes =
            tokio::runtime::Handle::current().block_on(async { self.codes.lock().await.clone() });
        codes
    }
}

#[async_trait]
impl acci_auth::repository::VerificationCodeRepository for MockVerificationCodeRepository {
    async fn save(&self, code: &VerificationCode, _context: &TenantAwareContext) -> Result<()> {
        let mut codes = self.codes.lock().await;
        codes.push(code.clone());
        Ok(())
    }

    async fn get_by_id(
        &self,
        id: Uuid,
        tenant_id: TenantId,
        _context: &TenantAwareContext,
    ) -> Result<Option<VerificationCode>> {
        let codes = self.codes.lock().await;
        let code = codes
            .iter()
            .find(|c| c.id == id && c.tenant_id == tenant_id)
            .cloned();
        Ok(code)
    }

    async fn get_by_code(
        &self,
        code: &str,
        user_id: UserId,
        verification_type: VerificationType,
        tenant_id: TenantId,
        _context: &TenantAwareContext,
    ) -> Result<Option<VerificationCode>> {
        let codes = self.codes.lock().await;
        let code = codes
            .iter()
            .find(|c| {
                c.code == code
                    && c.user_id == user_id
                    && c.verification_type == verification_type
                    && c.tenant_id == tenant_id
            })
            .cloned();
        Ok(code)
    }

    async fn get_pending_by_user(
        &self,
        user_id: UserId,
        verification_type: VerificationType,
        tenant_id: TenantId,
        _context: &TenantAwareContext,
    ) -> Result<Vec<VerificationCode>> {
        let codes = self.codes.lock().await;
        let codes = codes
            .iter()
            .filter(|c| {
                c.user_id == user_id
                    && c.verification_type == verification_type
                    && c.tenant_id == tenant_id
                    && c.status == acci_auth::models::VerificationStatus::Pending
            })
            .cloned()
            .collect();
        Ok(codes)
    }

    async fn update(&self, code: &VerificationCode, _context: &TenantAwareContext) -> Result<()> {
        let mut codes = self.codes.lock().await;
        if let Some(index) = codes.iter().position(|c| c.id == code.id) {
            codes[index] = code.clone();
        }
        Ok(())
    }

    async fn delete(
        &self,
        id: Uuid,
        tenant_id: TenantId,
        _context: &TenantAwareContext,
    ) -> Result<()> {
        let mut codes = self.codes.lock().await;
        if let Some(index) = codes
            .iter()
            .position(|c| c.id == id && c.tenant_id == tenant_id)
        {
            codes.remove(index);
        }
        Ok(())
    }

    async fn delete_expired(
        &self,
        before: time::OffsetDateTime,
        _tenant_id: TenantId,
        _context: &TenantAwareContext,
    ) -> Result<u64> {
        let mut codes = self.codes.lock().await;
        let count = codes.iter().filter(|c| c.expires_at < before).count() as u64;
        codes.retain(|c| c.expires_at >= before);
        Ok(count)
    }

    async fn invalidate_pending(
        &self,
        user_id: UserId,
        verification_type: VerificationType,
        tenant_id: TenantId,
        _context: &TenantAwareContext,
    ) -> Result<u64> {
        let mut codes = self.codes.lock().await;
        let mut count = 0;
        for code in codes.iter_mut() {
            if code.user_id == user_id
                && code.verification_type == verification_type
                && code.tenant_id == tenant_id
                && code.status == acci_auth::models::VerificationStatus::Pending
            {
                code.status = acci_auth::models::VerificationStatus::Invalidated;
                count += 1;
            }
        }
        Ok(count)
    }

    async fn count_recent_attempts(
        &self,
        user_id: UserId,
        verification_type: VerificationType,
        since: time::OffsetDateTime,
        tenant_id: TenantId,
        _context: &TenantAwareContext,
    ) -> Result<u64> {
        let codes = self.codes.lock().await;
        let count = codes
            .iter()
            .filter(|c| {
                c.user_id == user_id
                    && c.verification_type == verification_type
                    && c.tenant_id == tenant_id
                    && c.created_at > since
            })
            .count() as u64;
        Ok(count)
    }
}
