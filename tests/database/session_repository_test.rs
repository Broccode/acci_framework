use crate::helpers::setup_test_db;
use acci_auth::{
    models::user::User,
    session::{
        PostgresSessionRepository, SessionRepository, SessionRepositoryConfig,
        types::{Session, SessionError},
    },
};
use acci_core::error::Error;
use sqlx::{Pool, Postgres};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

async fn create_test_user(pool: &Pool<Postgres>) -> User {
    let user_id = Uuid::new_v4();
    let now = OffsetDateTime::now_utc();

    // Create a test user directly in the database
    sqlx::query!(
        r#"
        INSERT INTO users (id, email, password_hash, created_at, updated_at, is_active, is_verified)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        user_id,
        format!("test-{}@example.com", user_id),
        "hashed_password",
        now,
        now,
        true,
        false
    )
    .execute(pool)
    .await
    .expect("Failed to create test user");

    User {
        id: user_id,
        email: format!("test-{}@example.com", user_id),
        password_hash: "hashed_password".to_string(),
        created_at: now,
        updated_at: now,
        last_login: None,
        is_active: true,
        is_verified: false,
    }
}

#[tokio::test]
async fn test_session_crud_operations() -> Result<(), SessionError> {
    // Setup test database
    let (_container, pool) = setup_test_db().await.unwrap();

    // Create test user
    let user = create_test_user(&pool).await;

    // Create session repository
    let config = SessionRepositoryConfig {
        database_url: format!(
            "postgres://postgres:postgres@localhost:{}/postgres",
            pool.connect_options().get_port()
        ),
        session_timeout: 3600,
        ..Default::default()
    };
    let repo = PostgresSessionRepository::new(config).await?;

    // Create a test session
    let session_id = Uuid::new_v4();
    let token = "test_session_token";
    let device = "test_device";
    let ip = "127.0.0.1";
    let expires_at = OffsetDateTime::now_utc() + Duration::hours(1);

    let session = Session {
        id: session_id,
        user_id: user.id,
        token: token.to_string(),
        created_at: OffsetDateTime::now_utc(),
        expires_at,
        last_accessed_at: OffsetDateTime::now_utc(),
        device: Some(device.to_string()),
        ip: Some(ip.to_string()),
    };

    // Test create session
    repo.create_session(&session).await?;

    // Test find by token
    let found = repo.find_by_token(token).await?;
    assert!(found.is_some());
    let found_session = found.unwrap();
    assert_eq!(found_session.id, session_id);
    assert_eq!(found_session.user_id, user.id);

    // Test find by user
    let user_sessions = repo.find_by_user(user.id).await?;
    assert_eq!(user_sessions.len(), 1);
    assert_eq!(user_sessions[0].id, session_id);

    // Test update last accessed
    let new_time = OffsetDateTime::now_utc() + Duration::minutes(5);
    repo.update_last_accessed(session_id, new_time).await?;

    let updated = repo.find_by_token(token).await?.unwrap();
    assert!(updated.last_accessed_at > session.last_accessed_at);

    // Test delete session
    repo.delete_session(session_id).await?;
    let found = repo.find_by_token(token).await?;
    assert!(found.is_none());

    Ok(())
}

#[tokio::test]
async fn test_session_expiration() -> Result<(), SessionError> {
    // Setup test database
    let (_container, pool) = setup_test_db().await.unwrap();

    // Create test user
    let user = create_test_user(&pool).await;

    // Create session repository
    let config = SessionRepositoryConfig {
        database_url: format!(
            "postgres://postgres:postgres@localhost:{}/postgres",
            pool.connect_options().get_port()
        ),
        session_timeout: 3600,
        ..Default::default()
    };
    let repo = PostgresSessionRepository::new(config).await?;

    // Create an expired session
    let session_id = Uuid::new_v4();
    let token = "expired_session_token";
    let expires_at = OffsetDateTime::now_utc() - Duration::hours(1); // expired 1 hour ago

    let session = Session {
        id: session_id,
        user_id: user.id,
        token: token.to_string(),
        created_at: OffsetDateTime::now_utc() - Duration::hours(2),
        expires_at,
        last_accessed_at: OffsetDateTime::now_utc() - Duration::hours(1),
        device: None,
        ip: None,
    };

    // Insert the expired session
    repo.create_session(&session).await?;

    // Test that expired session is not found
    let found = repo.find_valid_session(token).await?;
    assert!(found.is_none());

    // Test clean expired sessions
    let deleted = repo.clean_expired_sessions().await?;
    assert!(deleted >= 1);

    // Verify session is gone
    let found = repo.find_by_token(token).await?;
    assert!(found.is_none());

    Ok(())
}

#[tokio::test]
async fn test_session_concurrent_access() -> Result<(), SessionError> {
    // Setup test database
    let (_container, pool) = setup_test_db().await.unwrap();

    // Create test user
    let user = create_test_user(&pool).await;

    // Create session repository
    let config = SessionRepositoryConfig {
        database_url: format!(
            "postgres://postgres:postgres@localhost:{}/postgres",
            pool.connect_options().get_port()
        ),
        session_timeout: 3600,
        ..Default::default()
    };
    let repo = PostgresSessionRepository::new(config).await?;

    // Create multiple sessions for the same user (simulating different devices)
    let tokens = vec!["token1", "token2", "token3"];
    let mut session_ids = Vec::new();

    for token in &tokens {
        let session_id = Uuid::new_v4();
        session_ids.push(session_id);

        let session = Session {
            id: session_id,
            user_id: user.id,
            token: token.to_string(),
            created_at: OffsetDateTime::now_utc(),
            expires_at: OffsetDateTime::now_utc() + Duration::hours(1),
            last_accessed_at: OffsetDateTime::now_utc(),
            device: Some(format!("device_{}", token)),
            ip: Some("127.0.0.1".to_string()),
        };

        repo.create_session(&session).await?;
    }

    // Test that all sessions are found for the user
    let user_sessions = repo.find_by_user(user.id).await?;
    assert_eq!(user_sessions.len(), 3);

    // Test that each token retrieves the correct session
    for (i, token) in tokens.iter().enumerate() {
        let found = repo.find_by_token(token).await?.unwrap();
        assert_eq!(found.id, session_ids[i]);
    }

    // Test deleting all sessions for user
    repo.delete_all_for_user(user.id).await?;

    // Verify all sessions are gone
    let user_sessions = repo.find_by_user(user.id).await?;
    assert_eq!(user_sessions.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_transaction_management() -> Result<(), Error> {
    // Setup test database
    let (_container, pool) = setup_test_db().await.unwrap();

    // Start a transaction
    let mut tx = pool.begin().await?;

    // Create a test user in the transaction
    let user_id = Uuid::new_v4();
    let now = OffsetDateTime::now_utc();

    sqlx::query!(
        r#"
        INSERT INTO users (id, email, password_hash, created_at, updated_at, is_active, is_verified)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        user_id,
        format!("tx-test-{}@example.com", user_id),
        "hashed_password",
        now,
        now,
        true,
        false
    )
    .execute(&mut *tx)
    .await?;

    // Create a session in the same transaction
    let session_id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO sessions (id, user_id, token, created_at, expires_at, last_accessed_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        session_id,
        user_id,
        "tx_test_token",
        now,
        now + Duration::hours(1),
        now
    )
    .execute(&mut *tx)
    .await?;

    // Before committing, the data should not be visible outside the transaction
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE token = 'tx_test_token'")
            .fetch_one(&pool)
            .await?;

    assert_eq!(count.0, 0);

    // Commit the transaction
    tx.commit().await?;

    // After committing, the data should be visible
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE token = 'tx_test_token'")
            .fetch_one(&pool)
            .await?;

    assert_eq!(count.0, 1);

    // Test transaction rollback
    let mut tx = pool.begin().await?;

    let another_session_id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO sessions (id, user_id, token, created_at, expires_at, last_accessed_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        another_session_id,
        user_id,
        "tx_rollback_token",
        now,
        now + Duration::hours(1),
        now
    )
    .execute(&mut *tx)
    .await?;

    // Rollback the transaction
    tx.rollback().await?;

    // The data should not be visible after rollback
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE token = 'tx_rollback_token'")
            .fetch_one(&pool)
            .await?;

    assert_eq!(count.0, 0);

    Ok(())
}
