use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::path::Path;
use std::time::Duration;

use crate::error::Result;

/// Represents the database connection pool and related functionality
#[derive(Clone)]
pub struct Database {
    pool: Pool<Postgres>,
}

impl Database {
    /// Creates a new database instance with the given connection string
    pub async fn new(database_url: &str) -> Result<Self> {
        Self::with_options(database_url, 5, Duration::from_secs(3)).await
    }

    /// Creates a new database instance with custom pool options
    pub async fn with_options(
        database_url: &str,
        max_connections: u32,
        acquire_timeout: Duration,
    ) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(acquire_timeout)
            .connect(database_url)
            .await?;

        // Verify connection
        pool.acquire().await?;

        Ok(Self { pool })
    }

    /// Creates a new database instance for testing
    #[cfg(test)]
    pub async fn new_test() -> Result<Self> {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/acci_test".to_string());
        Self::new(&database_url).await
    }

    /// Returns a reference to the connection pool
    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    /// Runs all database migrations
    pub async fn run_migrations(&self) -> Result<()> {
        let migrations_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("migrations");

        sqlx::migrate::Migrator::new(migrations_path)
            .await?
            .run(self.pool())
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[tokio::test]
    async fn test_database_invalid_url() {
        // Test with an invalid database URL
        let invalid_url = "not-a-valid-postgres-url";
        let result = Database::new(invalid_url).await;

        // Should return an error
        assert!(result.is_err());

        // Verify it's a database error
        match result {
            Err(Error::Database(_)) => {}, // Expected error type
            _ => panic!("Expected Database error"),
        }
    }

    #[tokio::test]
    async fn test_database_with_options() {
        // Test with custom options but invalid URL (to avoid actual connection)
        let invalid_url = "postgres://invalid:invalid@localhost:5432/nonexistent";
        let max_connections = 10;
        let timeout = Duration::from_secs(5);

        let result = Database::with_options(invalid_url, max_connections, timeout).await;

        // Should return an error since we can't connect
        assert!(result.is_err());
    }

    // For database pool, we simply test it exists
    #[test]
    fn test_pool_accessor_exists() {
        // Simple test to ensure the pool method exists
        // This test is mainly to verify the method doesn't change signature

        // Static type checking - the pool() method must exist and return &Pool<Postgres>
        let db_pool_fn: fn(&Database) -> &Pool<Postgres> = Database::pool;

        // No runtime assertions needed - this is a compile-time test
        assert!(std::mem::size_of::<fn(&Database) -> &Pool<Postgres>>() > 0);
        assert!(db_pool_fn as usize > 0); // Ensure function pointer is valid
    }

    #[test]
    fn test_migrations_path_construction() {
        // Test that the migrations path is constructed correctly
        // This is a unit test for the path construction logic
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let expected_path = Path::new(manifest_dir)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("migrations");

        assert!(expected_path.exists(), "Migrations path should exist");
    }
}
