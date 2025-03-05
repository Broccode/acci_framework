use acci_auth::utils::{
    jwt::JwtUtils,
    password::{hash_password, verify_password},
};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use time::OffsetDateTime;
use tokio::runtime::Runtime;
use uuid::Uuid;

fn password_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("password_operations");

    // Reduce sample size for password hashing as it's CPU intensive
    group.sample_size(50);

    // Benchmark password hashing
    group.bench_function("hash", |b| {
        b.iter(|| {
            let password = black_box("P@ssw0rd123ComplexEnough!");
            let _ = hash_password(password).unwrap();
        })
    });

    // For verification, we need to prepare a hash first
    let password = "P@ssw0rd123ComplexEnough!";
    let hash = hash_password(password).unwrap();

    // Benchmark password verification
    group.bench_function("verify", |b| {
        b.iter(|| {
            let valid = verify_password(black_box(password), black_box(&hash)).unwrap();
            assert!(valid);
        })
    });

    group.finish();
}

fn jwt_token_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("jwt_operations");
    group.sample_size(100);

    // Create JWT utility for testing
    let jwt_secret = "test_secret_that_is_at_least_32_bytes_long";
    let jwt_utils = JwtUtils::new(jwt_secret.as_bytes());

    // Benchmark token generation
    group.bench_function("token_generation", |b| {
        b.iter(|| {
            let user_id = black_box(Uuid::new_v4());
            let email = black_box("test@example.com");
            let _ = jwt_utils.create_token(user_id, email).unwrap();
        })
    });

    // Create a token to validate
    let user_id = Uuid::new_v4();
    let email = "test@example.com";
    let token = jwt_utils.create_token(user_id, email).unwrap();

    // Benchmark token validation
    group.bench_function("token_validation", |b| {
        b.iter(|| {
            let claims = jwt_utils.validate_token(black_box(&token)).unwrap();
            assert_eq!(claims.sub, user_id);
        })
    });

    group.finish();
}

async fn setup_test_db() -> Pool<Sqlite> {
    // Create an in-memory SQLite database for testing
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite::memory:")
        .await
        .unwrap();

    // Create users table (simplified schema for benchmarking)
    sqlx::query(
        "CREATE TABLE users (
            id TEXT PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            last_login TEXT,
            is_active INTEGER NOT NULL,
            is_verified INTEGER NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create test user
    let user_id = Uuid::new_v4().to_string();
    let email = "test@example.com";
    let password_hash = hash_password("P@ssw0rd123ComplexEnough!").unwrap();
    let now = OffsetDateTime::now_utc().to_string();

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, created_at, updated_at, is_active, is_verified)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&user_id)
    .bind(email)
    .bind(&password_hash)
    .bind(&now)
    .bind(&now)
    .bind(1) // is_active
    .bind(1) // is_verified
    .execute(&pool)
    .await
    .unwrap();

    pool
}

fn login_simulation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("login_simulation");
    group.sample_size(20); // Lower sample size for complex operations

    let rt = Runtime::new().unwrap();

    // Set up the database
    let pool = rt.block_on(setup_test_db());

    // Benchmark the login process (simplified simulation)
    group.bench_function("basic_login", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Find user
                let email = black_box("test@example.com");
                let password = black_box("P@ssw0rd123ComplexEnough!");

                // Using query without macro to avoid SQLx offline mode issues
                let user = sqlx::query_as::<_, (String, String, String)>(
                    "SELECT id, email, password_hash FROM users WHERE email = ?",
                )
                .bind(email)
                .fetch_one(&pool)
                .await
                .unwrap();

                // Verify password
                let valid = verify_password(password, &user.2).unwrap();
                assert!(valid);

                // Create JWT token
                let jwt_utils =
                    JwtUtils::new("test_secret_that_is_at_least_32_bytes_long".as_bytes());
                let user_id = Uuid::parse_str(&user.0).unwrap();
                let token = jwt_utils.create_token(user_id, &user.1).unwrap();

                // Return simulated session
                (user_id, token)
            })
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    password_benchmarks,
    jwt_token_benchmarks,
    login_simulation_benchmark
);
criterion_main!(benches);
