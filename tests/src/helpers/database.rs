use anyhow::Result;
use sqlx::PgPool;
use std::path::Path;
use testcontainers_modules::{
    postgres,
    testcontainers::{ImageExt, runners::AsyncRunner},
};

pub async fn setup_test_db() -> Result<(Box<dyn std::any::Any>, PgPool)> {
    // Start Postgres container
    let container = postgres::Postgres::default()
        .with_tag("16-alpine")
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .with_env_var("POSTGRES_DB", "postgres")
        .start()
        .await?;

    let port = container.get_host_port_ipv4(5432).await?;
    let connection_string = format!("postgres://postgres:postgres@localhost:{}/postgres", port);

    // Create connection pool
    let pool = PgPool::connect(&connection_string).await?;

    // Run migrations
    let migrations_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("migrations");

    sqlx::migrate::Migrator::new(migrations_path)
        .await?
        .run(&pool)
        .await?;

    Ok((Box::new(container), pool))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_setup() {
        let (_container, pool) = setup_test_db().await.unwrap();

        // Test that we can execute a query
        let result: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await
            .expect("Failed to execute test query");

        assert_eq!(result.0, 1);
    }
}
