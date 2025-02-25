use sqlx::PgPool;
use std::time::{Duration, SystemTime};
use testcontainers::clients::Cli;
use testcontainers::images::postgres::Postgres;
use uuid::Uuid;

pub struct TestSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub device_id: String,
}

pub async fn setup_test_db(docker: &Cli) -> PgPool {
    let container = docker.run(Postgres::default());
    let port = container.get_host_port_ipv4(5432);
    let connection_string = format!("postgres://postgres:postgres@localhost:{}/postgres", port);

    let pool = PgPool::connect(&connection_string)
        .await
        .expect("Failed to create connection pool");

    // Run migrations
    sqlx::migrate!("../../crates/auth/migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

pub fn generate_test_session(user_id: Uuid) -> TestSession {
    TestSession {
        id: Uuid::new_v4(),
        user_id,
        token: format!("test_token_{}", Uuid::new_v4()),
        device_id: format!("test_device_{}", Uuid::new_v4()),
    }
}

pub fn future_timestamp(seconds: u64) -> SystemTime {
    SystemTime::now() + Duration::from_secs(seconds)
}

pub fn past_timestamp(seconds: u64) -> SystemTime {
    SystemTime::now() - Duration::from_secs(seconds)
}
