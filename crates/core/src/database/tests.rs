use super::*;
use rstest::*;
use sqlx::Executor;

#[fixture]
async fn db() -> Database {
    // Load test environment
    dotenvy::dotenv().ok();

    // Create database instance
    let database = Database::new_test()
        .await
        .expect("Failed to create test database");

    // Clean up database before each test
    database
        .pool()
        .execute("TRUNCATE TABLE users CASCADE")
        .await
        .expect("Failed to clean up database");

    database
}

#[rstest]
#[tokio::test]
async fn test_database_connection() {
    let db = Database::new_test()
        .await
        .expect("Failed to create test database");

    // Test that we can acquire a connection
    let conn = db
        .pool()
        .acquire()
        .await
        .expect("Failed to acquire connection");

    // Test that we can execute a simple query
    let result: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(db.pool())
        .await
        .expect("Failed to execute query");

    assert_eq!(result.0, 1);
}

#[rstest]
#[tokio::test]
async fn test_database_pool_limits() {
    let db = Database::new_test()
        .await
        .expect("Failed to create test database");

    // Try to acquire more connections than the pool limit (5)
    let mut connections = Vec::new();
    for _ in 0..5 {
        let conn = db
            .pool()
            .acquire()
            .await
            .expect("Failed to acquire connection");
        connections.push(conn);
    }

    // The 6th connection should timeout after 3 seconds
    let timeout_result =
        tokio::time::timeout(std::time::Duration::from_secs(4), db.pool().acquire()).await;

    assert!(
        timeout_result.is_err(),
        "Expected timeout error when exceeding pool limit"
    );
}

#[rstest]
#[tokio::test]
async fn test_database_concurrent_queries(#[future] db: Database) {
    let db = db.await;

    // Run multiple queries concurrently
    let futures = (0..10).map(|i| {
        let pool = db.pool();
        async move {
            let result: (i32,) = sqlx::query_as(&format!("SELECT {}", i))
                .fetch_one(pool)
                .await
                .expect("Failed to execute concurrent query");
            assert_eq!(result.0, i);
        }
    });

    // All queries should complete successfully
    futures::future::join_all(futures).await;
}

#[rstest]
#[tokio::test]
async fn test_database_transaction(#[future] db: Database) {
    let db = db.await;
    let pool = db.pool();

    // Test successful transaction
    let result = pool
        .begin()
        .await
        .expect("Failed to begin transaction")
        .execute("SELECT 1")
        .await
        .expect("Failed to execute query in transaction")
        .rows_affected();

    assert_eq!(result, 1);

    // Test transaction rollback
    let tx = pool.begin().await.expect("Failed to begin transaction");
    tx.execute("SELECT 1")
        .await
        .expect("Failed to execute query");
    tx.rollback().await.expect("Failed to rollback transaction");
}

#[rstest]
#[tokio::test]
async fn test_database_error_handling(#[future] db: Database) {
    let db = db.await;

    // Test invalid SQL
    let result = sqlx::query("INVALID SQL").execute(db.pool()).await;

    assert!(result.is_err());
    assert!(matches!(
        Database::new("invalid://connection-string").await,
        Err(Error::Database(_))
    ));
}
