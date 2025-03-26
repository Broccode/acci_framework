use std::sync::Arc;
use acci_auth::{
    config::AuthConfig,
    services::session::SessionService,
    session::{
        Session, SessionError, SessionFilter, SessionInvalidationReason, SessionRepository,
        types::{DeviceFingerprint, MfaStatus},
    },
};
use async_trait::async_trait;
use serde_json::Value;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

// Test-Repository mit simulierten Daten
struct TestSessionRepository {
    user_sessions: Vec<(Uuid, u64)>, // (user_id, session_count)
    ip_sessions: Vec<(String, u64)>, // (ip_address, session_count)
}

impl TestSessionRepository {
    fn new() -> Self {
        Self {
            user_sessions: vec![
                (Uuid::new_v4(), 3),  // User 1 hat 3 Sessions
                (Uuid::new_v4(), 5),  // User 2 hat 5 Sessions
                (Uuid::new_v4(), 0),  // User 3 hat keine Sessions
            ],
            ip_sessions: vec![
                ("192.168.1.1".to_string(), 2),  // IP 1 hat 2 Sessions
                ("10.0.0.1".to_string(), 4),     // IP 2 hat 4 Sessions
            ],
        }
    }
}

#[async_trait]
impl SessionRepository for TestSessionRepository {
    async fn create_session(
        &self,
        _user_id: Uuid,
        _token_hash: String,
        _expires_at: SystemTime,
        _device_id: Option<String>,
        _device_fingerprint: Option<DeviceFingerprint>,
        _ip_address: Option<String>,
        _user_agent: Option<String>,
        _metadata: Option<Value>,
    ) -> Result<Session, SessionError> {
        unimplemented!("Not needed for this test")
    }

    async fn get_session(&self, _id: Uuid) -> Result<Option<Session>, SessionError> {
        unimplemented!("Not needed for this test")
    }

    async fn get_session_by_token(&self, _token_hash: &str) -> Result<Option<Session>, SessionError> {
        unimplemented!("Not needed for this test")
    }

    async fn get_user_sessions(
        &self,
        _user_id: Uuid,
        _filter: SessionFilter,
    ) -> Result<Vec<Session>, SessionError> {
        unimplemented!("Not needed for this test")
    }

    async fn update_session_activity(&self, _id: Uuid) -> Result<(), SessionError> {
        unimplemented!("Not needed for this test")
    }

    async fn invalidate_session(
        &self,
        _id: Uuid,
        _reason: SessionInvalidationReason,
    ) -> Result<(), SessionError> {
        unimplemented!("Not needed for this test")
    }

    async fn invalidate_all_user_sessions(
        &self,
        user_id: Uuid,
        _reason: SessionInvalidationReason,
    ) -> Result<u64, SessionError> {
        // Finde den Benutzer und gib die Anzahl der beendeten Sessions zurück
        for (uid, count) in &self.user_sessions {
            if *uid == user_id {
                return Ok(*count);
            }
        }
        Ok(0) // Benutzer nicht gefunden, keine Sessions beendet
    }

    async fn invalidate_sessions_by_filter(
        &self,
        filter: SessionFilter,
        _reason: SessionInvalidationReason,
    ) -> Result<u64, SessionError> {
        // Simuliere Filterergebnisse basierend auf dem Filter
        match filter {
            SessionFilter::All => Ok(10), // Alle Sessions
            SessionFilter::Active => Ok(8), // Nur aktive Sessions
            SessionFilter::Inactive => Ok(2), // Nur inaktive Sessions
        }
    }

    async fn invalidate_sessions_by_ip(
        &self,
        ip_address: &str,
        _reason: SessionInvalidationReason,
    ) -> Result<u64, SessionError> {
        // Finde die IP und gib die Anzahl der beendeten Sessions zurück
        for (ip, count) in &self.ip_sessions {
            if ip == ip_address {
                return Ok(*count);
            }
        }
        Ok(0) // IP nicht gefunden, keine Sessions beendet
    }

    async fn rotate_session_token(
        &self,
        _id: Uuid,
        _new_token_hash: String,
    ) -> Result<(), SessionError> {
        unimplemented!("Not needed for this test")
    }

    async fn cleanup_expired_sessions(&self) -> Result<u64, SessionError> {
        unimplemented!("Not needed for this test")
    }

    async fn update_mfa_status(&self, _id: Uuid, _status: MfaStatus) -> Result<(), SessionError> {
        unimplemented!("Not needed for this test")
    }
}

#[tokio::test]
async fn test_force_terminate_user_sessions() {
    // Setup
    let repo = Arc::new(TestSessionRepository::new());
    let config = Arc::new(AuthConfig::default());
    let service = SessionService::new(repo.clone(), config);
    
    // Test: Beenden aller Sessions für Benutzer 1
    let user_id = repo.user_sessions[0].0;
    let expected_count = repo.user_sessions[0].1;
    
    let count = service
        .force_terminate_user_sessions(user_id, SessionInvalidationReason::SecurityBreach)
        .await
        .expect("Die Beendigung der Sessions sollte erfolgreich sein");
    
    assert_eq!(count, expected_count, "Es sollten genau {} Sessions beendet werden", expected_count);
    
    // Test: Benutzer ohne Sessions
    let user_id = repo.user_sessions[2].0;
    let count = service
        .force_terminate_user_sessions(user_id, SessionInvalidationReason::SecurityBreach)
        .await
        .expect("Die Beendigung der Sessions sollte erfolgreich sein");
    
    assert_eq!(count, 0, "Es sollten keine Sessions beendet werden");
}

#[tokio::test]
async fn test_force_terminate_sessions_by_ip() {
    // Setup
    let repo = Arc::new(TestSessionRepository::new());
    let config = Arc::new(AuthConfig::default());
    let service = SessionService::new(repo.clone(), config);
    
    // Test: Beenden aller Sessions für IP 1
    let ip_address = &repo.ip_sessions[0].0;
    let expected_count = repo.ip_sessions[0].1;
    
    let count = service
        .force_terminate_sessions_by_ip(ip_address, SessionInvalidationReason::SuspiciousLocation)
        .await
        .expect("Die Beendigung der Sessions sollte erfolgreich sein");
    
    assert_eq!(count, expected_count, "Es sollten genau {} Sessions beendet werden", expected_count);
    
    // Test: IP ohne Sessions
    let count = service
        .force_terminate_sessions_by_ip("127.0.0.1", SessionInvalidationReason::SuspiciousLocation)
        .await
        .expect("Die Beendigung der Sessions sollte erfolgreich sein");
    
    assert_eq!(count, 0, "Es sollten keine Sessions beendet werden");
}

#[tokio::test]
async fn test_force_terminate_sessions_by_filter() {
    // Setup
    let repo = Arc::new(TestSessionRepository::new());
    let config = Arc::new(AuthConfig::default());
    let service = SessionService::new(repo.clone(), config);
    
    // Test: Beenden aller aktiven Sessions
    let count = service
        .force_terminate_sessions_by_filter(SessionFilter::Active, SessionInvalidationReason::SecurityPolicyChange)
        .await
        .expect("Die Beendigung der Sessions sollte erfolgreich sein");
    
    assert_eq!(count, 8, "Es sollten genau 8 aktive Sessions beendet werden");
    
    // Test: Beenden aller inaktiven Sessions
    let count = service
        .force_terminate_sessions_by_filter(SessionFilter::Inactive, SessionInvalidationReason::SecurityPolicyChange)
        .await
        .expect("Die Beendigung der Sessions sollte erfolgreich sein");
    
    assert_eq!(count, 2, "Es sollten genau 2 inaktive Sessions beendet werden");
    
    // Test: Beenden aller Sessions
    let count = service
        .force_terminate_sessions_by_filter(SessionFilter::All, SessionInvalidationReason::EmergencyTermination)
        .await
        .expect("Die Beendigung der Sessions sollte erfolgreich sein");
    
    assert_eq!(count, 10, "Es sollten genau 10 Sessions beendet werden");
} 