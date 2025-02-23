use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
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
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
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
}
