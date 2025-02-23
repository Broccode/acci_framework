use super::*;
use rstest::*;
use sqlx::Executor;
use testcontainers::{Container, Docker, clients, images::postgres::Postgres};
use uuid::Uuid;

struct TestDb {
    _container: Container<'static, Postgres>,
    database: Database,
}

#[fixture]
async fn db() -> TestDb {
    // Start PostgreSQL container
    let docker = clients::Cli::default();
    let container = docker.run(Postgres::default());
    let port = container.get_host_port_ipv4(5432);

    // Generate unique database name for this test run
    let db_name = format!("test_{}", Uuid::new_v4().simple());

    // Connection string for initial connection to create test database
    let admin_url = format!("postgres://postgres:postgres@localhost:{}/postgres", port);

    // Create test database
    let admin_pool = sqlx::PgPool::connect(&admin_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    sqlx::query(&format!("CREATE DATABASE {}", db_name))
        .execute(&admin_pool)
        .await
        .expect("Failed to create test database");

    // Connection string for test database
    let database_url = format!(
        "postgres://postgres:postgres@localhost:{}/{}",
        port, db_name
    );

    // Create database instance
    let database = Database::new(&database_url)
        .await
        .expect("Failed to create database instance");

    // Run migrations
    sqlx::migrate!("../migrations")
        .run(database.pool())
        .await
        .expect("Failed to run migrations");

    TestDb {
        _container: container,
        database,
    }
}

#[rstest]
#[tokio::test]
async fn test_database_connection(#[future] db: TestDb) {
    let db = db.await;

    // Test that we can execute a simple query
    let result: (i32,) = sqlx::query_as("SELECT 1")
        .fetch_one(db.database.pool())
        .await
        .expect("Failed to execute query");

    assert_eq!(result.0, 1);
}

#[rstest]
#[tokio::test]
async fn test_database_pool_limits(#[future] db: TestDb) {
    let db = db.await;

    // Try to acquire more connections than the pool limit (5)
    let mut connections = Vec::new();
    for _ in 0..5 {
        let conn = db
            .database
            .pool()
            .acquire()
            .await
            .expect("Failed to acquire connection");
        connections.push(conn);
    }

    // The 6th connection should timeout after 3 seconds
    let timeout_result = tokio::time::timeout(
        std::time::Duration::from_secs(4),
        db.database.pool().acquire(),
    )
    .await;

    assert!(
        timeout_result.is_err(),
        "Expected timeout error when exceeding pool limit"
    );
}

#[rstest]
#[tokio::test]
async fn test_database_concurrent_queries(#[future] db: TestDb) {
    let db = db.await;

    // Run multiple queries concurrently
    let futures = (0..10).map(|i| {
        let pool = db.database.pool();
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
async fn test_database_transaction(#[future] db: TestDb) {
    let db = db.await;
    let pool = db.database.pool();

    // Test successful transaction
    let mut tx = pool.begin().await.expect("Failed to begin transaction");
    let result = tx
        .execute("SELECT 1")
        .await
        .expect("Failed to execute query in transaction")
        .rows_affected();

    assert_eq!(result, 1);
    tx.commit().await.expect("Failed to commit transaction");

    // Test transaction rollback
    let mut tx = pool.begin().await.expect("Failed to begin transaction");
    tx.execute("SELECT 1")
        .await
        .expect("Failed to execute query");
    tx.rollback().await.expect("Failed to rollback transaction");
}

#[rstest]
#[tokio::test]
async fn test_database_error_handling(#[future] db: TestDb) {
    let db = db.await;

    // Test invalid SQL
    let result = sqlx::query("INVALID SQL").execute(db.database.pool()).await;

    assert!(result.is_err());
    assert!(matches!(
        Database::new("invalid://connection-string").await,
        Err(crate::error::Error::Database(_))
    ));
}
