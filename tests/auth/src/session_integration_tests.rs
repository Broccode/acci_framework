use std::time::Duration;
use testcontainers::clients::Cli;
use uuid::Uuid;

use acci_auth::session::types::SessionInvalidationReason;
use acci_auth::session::{PostgresSessionRepository, SessionFilter, SessionRepository};

use crate::helpers::session_test_helper::{future_timestamp, generate_test_session, setup_test_db};

#[tokio::test]
async fn test_session_lifecycle() {
    let docker = Cli::default();
    let pool = setup_test_db(&docker).await;
    let repo = PostgresSessionRepository::new(pool);

    // Create test data
    let user_id = Uuid::new_v4();
    let test_session = generate_test_session(user_id);
    let expires_at = future_timestamp(3600); // 1 hour from now

    // Test session creation
    let session = repo
        .create_session(
            test_session.user_id,
            test_session.token.clone(),
            expires_at,
            Some(test_session.device_id.clone()),
            None,
            Some("127.0.0.1".to_string()),
            Some("Test User Agent".to_string()),
            None,
        )
        .await
        .expect("Failed to create session");

    assert_eq!(session.user_id, test_session.user_id);
    assert!(session.is_valid);

    // Test session retrieval
    let retrieved = repo
        .get_session(session.id)
        .await
        .expect("Failed to get session")
        .expect("Session not found");

    assert_eq!(retrieved.id, session.id);
    assert_eq!(retrieved.user_id, test_session.user_id);

    // Test session activity update
    tokio::time::sleep(Duration::from_secs(1)).await;
    repo.update_session_activity(session.id)
        .await
        .expect("Failed to update session activity");

    let updated = repo
        .get_session(session.id)
        .await
        .expect("Failed to get session")
        .expect("Session not found");

    assert!(updated.last_activity_at > session.last_activity_at);

    // Test token rotation
    let new_token = format!("new_token_{}", Uuid::new_v4());
    repo.rotate_session_token(session.id, new_token.clone())
        .await
        .expect("Failed to rotate token");

    let rotated = repo
        .get_session_by_token(&new_token)
        .await
        .expect("Failed to get session by token")
        .expect("Session not found");

    assert_eq!(rotated.id, session.id);

    // Test session invalidation
    repo.invalidate_session(session.id, SessionInvalidationReason::UserLogout)
        .await
        .expect("Failed to invalidate session");

    let invalidated = repo
        .get_session(session.id)
        .await
        .expect("Failed to get session")
        .expect("Session not found");

    assert!(!invalidated.is_valid);
    assert_eq!(
        invalidated.invalidated_reason,
        Some(SessionInvalidationReason::UserLogout)
    );
}

#[tokio::test]
async fn test_concurrent_session_management() {
    let docker = Cli::default();
    let pool = setup_test_db(&docker).await;
    let repo = PostgresSessionRepository::new(pool);

    // Create multiple sessions for the same user
    let user_id = Uuid::new_v4();
    let mut session_ids = Vec::new();

    for _ in 0..5 {
        let test_session = generate_test_session(user_id);
        let session = repo
            .create_session(
                test_session.user_id,
                test_session.token,
                future_timestamp(3600),
                Some(test_session.device_id),
                None,
                None,
                None,
                None,
            )
            .await
            .expect("Failed to create session");
        session_ids.push(session.id);
    }

    // Test concurrent session retrieval
    let sessions = repo
        .get_user_sessions(user_id, SessionFilter::Active)
        .await
        .expect("Failed to get user sessions");

    assert_eq!(sessions.len(), 5);
    assert!(sessions.iter().all(|s| s.is_valid));

    // Test concurrent session invalidation
    let futures: Vec<_> = session_ids
        .iter()
        .map(|id| {
            let repo = repo.clone();
            tokio::spawn(async move {
                repo.invalidate_session(*id, SessionInvalidationReason::AdminAction)
                    .await
            })
        })
        .collect();

    for future in futures {
        future
            .await
            .expect("Task panicked")
            .expect("Failed to invalidate session");
    }

    // Verify all sessions are invalidated
    let active_sessions = repo
        .get_user_sessions(user_id, SessionFilter::Active)
        .await
        .expect("Failed to get user sessions");

    assert_eq!(active_sessions.len(), 0);
}

#[tokio::test]
async fn test_session_cleanup() {
    let docker = Cli::default();
    let pool = setup_test_db(&docker).await;
    let repo = PostgresSessionRepository::new(pool);

    // Create expired and valid sessions
    let user_id = Uuid::new_v4();

    // Create expired session
    let expired_session = generate_test_session(user_id);
    let expired = repo
        .create_session(
            expired_session.user_id,
            expired_session.token,
            future_timestamp(0), // Expires immediately
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("Failed to create expired session");

    // Create valid session
    let valid_session = generate_test_session(user_id);
    let valid = repo
        .create_session(
            valid_session.user_id,
            valid_session.token,
            future_timestamp(3600),
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("Failed to create valid session");

    // Wait for expired session to expire
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Run cleanup
    let cleaned = repo
        .cleanup_expired_sessions()
        .await
        .expect("Failed to cleanup sessions");

    assert_eq!(cleaned, 1);

    // Verify expired session is invalidated
    let expired_check = repo
        .get_session(expired.id)
        .await
        .expect("Failed to get session")
        .expect("Session not found");

    assert!(!expired_check.is_valid);
    assert_eq!(
        expired_check.invalidated_reason,
        Some(SessionInvalidationReason::TokenExpired)
    );

    // Verify valid session is still valid
    let valid_check = repo
        .get_session(valid.id)
        .await
        .expect("Failed to get session")
        .expect("Session not found");

    assert!(valid_check.is_valid);
}
