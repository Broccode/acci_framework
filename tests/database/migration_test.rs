use anyhow::Result;
use sqlx::{Connection, Executor, PgConnection};
use std::path::Path;
use testcontainers_modules::{
    postgres,
    testcontainers::{ImageExt, runners::AsyncRunner},
};

#[tokio::test]
async fn test_migrations_apply_in_order() -> Result<()> {
    // Start a clean Postgres container
    let container = postgres::Postgres::default()
        .with_tag("16-alpine")
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .with_env_var("POSTGRES_DB", "postgres")
        .start()
        .await?;

    let port = container.get_host_port_ipv4(5432).await?;
    let connection_string = format!("postgres://postgres:postgres@localhost:{}/postgres", port);

    // Connect to database
    let mut conn = PgConnection::connect(&connection_string).await?;

    // Get migrations path
    let migrations_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("migrations");

    // Apply migrations
    let migrator = sqlx::migrate::Migrator::new(migrations_path).await?;
    migrator.run(&mut conn).await?;

    // Verify all expected tables exist
    let tables = sqlx::query!(
        r#"
        SELECT table_name 
        FROM information_schema.tables 
        WHERE table_schema = 'public'
        "#
    )
    .fetch_all(&mut conn)
    .await?;

    // Expected tables from migrations
    let expected_tables = vec!["users", "sessions", "user_audit_log"];

    for table in expected_tables {
        assert!(
            tables.iter().any(|t| t.table_name == table),
            "Table '{}' not found after migration",
            table
        );
    }

    // Verify users table schema
    let columns = sqlx::query!(
        r#"
        SELECT column_name, data_type 
        FROM information_schema.columns 
        WHERE table_name = 'users'
        "#
    )
    .fetch_all(&mut conn)
    .await?;

    // Check key columns
    assert!(
        columns
            .iter()
            .any(|c| c.column_name == "id" && c.data_type == "uuid")
    );
    assert!(
        columns
            .iter()
            .any(|c| c.column_name == "email" && c.data_type == "character varying")
    );
    assert!(
        columns
            .iter()
            .any(|c| c.column_name == "password_hash" && c.data_type == "character varying")
    );
    assert!(
        columns
            .iter()
            .any(|c| c.column_name == "created_at" && c.data_type.contains("timestamp"))
    );

    // Verify sessions table schema
    let columns = sqlx::query!(
        r#"
        SELECT column_name, data_type 
        FROM information_schema.columns 
        WHERE table_name = 'sessions'
        "#
    )
    .fetch_all(&mut conn)
    .await?;

    // Check key columns
    assert!(
        columns
            .iter()
            .any(|c| c.column_name == "id" && c.data_type == "uuid")
    );
    assert!(
        columns
            .iter()
            .any(|c| c.column_name == "user_id" && c.data_type == "uuid")
    );
    assert!(
        columns
            .iter()
            .any(|c| c.column_name == "token" && c.data_type == "character varying")
    );
    assert!(
        columns
            .iter()
            .any(|c| c.column_name == "expires_at" && c.data_type.contains("timestamp"))
    );

    // Verify foreign key constraints
    let foreign_keys = sqlx::query!(
        r#"
        SELECT
            tc.constraint_name,
            tc.table_name,
            kcu.column_name,
            ccu.table_name AS foreign_table_name,
            ccu.column_name AS foreign_column_name
        FROM
            information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu
              ON tc.constraint_name = kcu.constraint_name
            JOIN information_schema.constraint_column_usage AS ccu
              ON ccu.constraint_name = tc.constraint_name
        WHERE constraint_type = 'FOREIGN KEY'
        "#
    )
    .fetch_all(&mut conn)
    .await?;

    // Check for session -> user FK
    let has_session_user_fk = foreign_keys.iter().any(|fk| {
        fk.table_name == "sessions"
            && fk.column_name == "user_id"
            && fk.foreign_table_name == "users"
            && fk.foreign_column_name == "id"
    });

    assert!(
        has_session_user_fk,
        "Foreign key from sessions.user_id to users.id not found"
    );

    // Check for indexes on sessions.token
    let indexes = sqlx::query!(
        r#"
        SELECT indexname, indexdef
        FROM pg_indexes
        WHERE tablename = 'sessions'
        "#
    )
    .fetch_all(&mut conn)
    .await?;

    let has_token_index = indexes.iter().any(|idx| {
        idx.indexdef.to_lowercase().contains("sessions")
            && idx.indexdef.to_lowercase().contains("token")
    });

    assert!(has_token_index, "Index on sessions.token not found");

    Ok(())
}

#[tokio::test]
async fn test_migrations_idempotency() -> Result<()> {
    // Start a clean Postgres container
    let container = postgres::Postgres::default()
        .with_tag("16-alpine")
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .with_env_var("POSTGRES_DB", "postgres")
        .start()
        .await?;

    let port = container.get_host_port_ipv4(5432).await?;
    let connection_string = format!("postgres://postgres:postgres@localhost:{}/postgres", port);

    // Connect to database
    let mut conn = PgConnection::connect(&connection_string).await?;

    // Get migrations path
    let migrations_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("migrations");

    // Apply migrations
    let migrator = sqlx::migrate::Migrator::new(migrations_path).await?;
    migrator.run(&mut conn).await?;

    // Apply migrations again (should be idempotent)
    let result = migrator.run(&mut conn).await;

    // Should succeed (migrations are tracked and won't run twice)
    assert!(
        result.is_ok(),
        "Running migrations twice failed: {:?}",
        result
    );

    Ok(())
}

#[tokio::test]
async fn test_database_version_tracking() -> Result<()> {
    // Start a clean Postgres container
    let container = postgres::Postgres::default()
        .with_tag("16-alpine")
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .with_env_var("POSTGRES_DB", "postgres")
        .start()
        .await?;

    let port = container.get_host_port_ipv4(5432).await?;
    let connection_string = format!("postgres://postgres:postgres@localhost:{}/postgres", port);

    // Connect to database
    let mut conn = PgConnection::connect(&connection_string).await?;

    // Get migrations path
    let migrations_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("migrations");

    // Apply migrations
    let migrator = sqlx::migrate::Migrator::new(migrations_path).await?;
    migrator.run(&mut conn).await?;

    // Check that _sqlx_migrations table exists and has entries
    let versions = sqlx::query!(
        r#"
        SELECT version, description
        FROM _sqlx_migrations
        ORDER BY version
        "#
    )
    .fetch_all(&mut conn)
    .await?;

    // Should have at least 3 migrations applied
    assert!(
        versions.len() >= 3,
        "Expected at least 3 migrations, found {}",
        versions.len()
    );

    // Check that migrations are applied in version order
    for i in 1..versions.len() {
        let prev = &versions[i - 1];
        let curr = &versions[i];

        assert!(
            prev.version < curr.version,
            "Migrations not applied in order: {} with '{}' before {} with '{}'",
            prev.version,
            prev.description,
            curr.version,
            curr.description
        );
    }

    Ok(())
}
