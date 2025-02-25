use acci_auth::{
    AuthConfig,
    services::session::SessionService,
    session::{
        PostgresSessionRepository, Session, SessionFilter, SessionInvalidationReason,
        SessionRepository,
    },
};
use sqlx::postgres::PgPoolOptions;
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};
use uuid::Uuid;

async fn create_test_db() -> sqlx::PgPool {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/acci_test".to_string());

    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

fn create_test_config() -> Arc<AuthConfig> {
    Arc::new(AuthConfig {
        session_lifetime_secs: 3600,
        ..Default::default()
    })
}

#[tokio::test]
async fn test_create_session() {
    let pool = create_test_db().await;
    let repo = Arc::new(PostgresSessionRepository::new(pool.clone()));
    let config = create_test_config();
    let service = SessionService::new(repo, config);

    // Run migrations
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let user_id = Uuid::new_v4();
    let device_id = Some("test_device".to_string());

    let result = service
        .create_session(user_id, device_id, None, None, None, None)
        .await;

    assert!(result.is_ok());
    let (session, token) = result.unwrap();
    assert_eq!(session.user_id, user_id);
    assert!(session.is_valid);
    assert!(!token.is_empty());

    // Cleanup
    sqlx::query!("DELETE FROM sessions WHERE id = $1", session.id)
        .execute(&pool)
        .await
        .expect("Failed to cleanup test data");
}

#[tokio::test]
async fn test_validate_session() {
    let pool = create_test_db().await;
    let repo = Arc::new(PostgresSessionRepository::new(pool.clone()));
    let config = create_test_config();
    let service = SessionService::new(repo, config);

    // Run migrations
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Create a session first
    let user_id = Uuid::new_v4();
    let (session, token) = service
        .create_session(user_id, None, None, None, None, None)
        .await
        .expect("Failed to create session");

    // Test validation
    let result = service.validate_session(&token).await;
    assert!(result.is_ok());
    let validated = result.unwrap();
    assert!(validated.is_some());
    let validated = validated.unwrap();
    assert_eq!(validated.id, session.id);
    assert_eq!(validated.user_id, user_id);
    assert!(validated.is_valid);

    // Cleanup
    sqlx::query!("DELETE FROM sessions WHERE id = $1", session.id)
        .execute(&pool)
        .await
        .expect("Failed to cleanup test data");
}

#[tokio::test]
async fn test_invalidate_session() {
    let pool = create_test_db().await;
    let repo = Arc::new(PostgresSessionRepository::new(pool.clone()));
    let config = create_test_config();
    let service = SessionService::new(repo, config);

    // Run migrations
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Create a session first
    let user_id = Uuid::new_v4();
    let (session, token) = service
        .create_session(user_id, None, None, None, None, None)
        .await
        .expect("Failed to create session");

    // Test invalidation
    let result = service
        .invalidate_session(&token, SessionInvalidationReason::UserLogout)
        .await;
    assert!(result.is_ok());

    // Verify session is invalid
    let invalid_session = sqlx::query_as!(
        Session,
        r#"
        SELECT
            id, user_id, token_hash, previous_token_hash, token_rotation_at,
            expires_at, created_at, last_activity_at, last_activity_update_at,
            ip_address, user_agent, device_id,
            device_fingerprint as "device_fingerprint: Json<DeviceFingerprint>",
            is_valid, invalidated_reason as "invalidated_reason: _",
            metadata as "metadata: Value"
        FROM sessions
        WHERE id = $1
        "#,
        session.id
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch session");

    assert!(!invalid_session.is_valid);
    assert_eq!(
        invalid_session.invalidated_reason,
        Some(SessionInvalidationReason::UserLogout)
    );

    // Cleanup
    sqlx::query!("DELETE FROM sessions WHERE id = $1", session.id)
        .execute(&pool)
        .await
        .expect("Failed to cleanup test data");
}

#[tokio::test]
async fn test_postgres_session_repository() {
    let pool = create_test_db().await;
    let repo = PostgresSessionRepository::new(pool.clone());

    // Run migrations
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Test session creation
    let user_id = Uuid::new_v4();
    let token_hash = "test_token_hash".to_string();
    let expires_at = SystemTime::now() + Duration::from_secs(3600);
    let device_id = Some("test_device".to_string());

    let session = repo
        .create_session(
            user_id,
            token_hash.clone(),
            expires_at,
            device_id.clone(),
            None,
            Some("127.0.0.1".to_string()),
            Some("Test Agent".to_string()),
            None,
        )
        .await
        .expect("Failed to create session");

    assert_eq!(session.user_id, user_id);
    assert_eq!(session.token_hash, token_hash);
    assert!(session.is_valid);

    // Test session retrieval
    let retrieved = repo
        .get_session(session.id)
        .await
        .expect("Failed to get session")
        .expect("Session not found");

    assert_eq!(retrieved.id, session.id);
    assert_eq!(retrieved.user_id, user_id);
    assert_eq!(retrieved.token_hash, token_hash);

    // Test session invalidation
    repo.invalidate_session(session.id, SessionInvalidationReason::UserLogout)
        .await
        .expect("Failed to invalidate session");

    let invalid_session = repo
        .get_session(session.id)
        .await
        .expect("Failed to get session")
        .expect("Session not found");

    assert!(!invalid_session.is_valid);
    assert_eq!(
        invalid_session.invalidated_reason,
        Some(SessionInvalidationReason::UserLogout)
    );

    // Cleanup test data
    sqlx::query!("DELETE FROM sessions WHERE id = $1", session.id)
        .execute(&pool)
        .await
        .expect("Failed to cleanup test data");
}
