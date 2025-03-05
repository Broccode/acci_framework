use crate::helpers::setup_test_db;

mod migration_test;
mod session_repository_test;

#[tokio::test]
async fn test_database_connection() {
    let (_container, pool) = setup_test_db().await.unwrap();

    // Test that we can execute a query
    let result: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(&pool)
        .await
        .expect("Failed to execute test query");

    assert_eq!(result.0, 1);
}

#[tokio::test]
async fn test_migrations() {
    let (_container, pool) = setup_test_db().await.unwrap();

    // Test that users table exists
    let result = sqlx::query!("SELECT COUNT(*) as count FROM users")
        .fetch_one(&pool)
        .await
        .expect("Failed to query users table");

    // Table should be empty
    assert_eq!(result.count.unwrap(), 0);
}

#[tokio::test]
async fn test_user_audit_log() {
    let (_container, pool) = setup_test_db().await.unwrap();

    // Test that user_audit_log table exists
    let result = sqlx::query!("SELECT COUNT(*) as count FROM user_audit_log")
        .fetch_one(&pool)
        .await
        .expect("Failed to query user_audit_log table");

    // Table should be empty
    assert_eq!(result.count.unwrap(), 0);
}
