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
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/acci_test".to_string()
        });
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
