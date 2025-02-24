use crate::models::user::{User, UserError, UserRepository};
use async_trait::async_trait;
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{num::NonZeroU32, sync::Arc, time::Duration};
use time::OffsetDateTime;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub connect_timeout: Duration,
    pub rate_limit_burst: u32,
    pub rate_limit_replenish_ms: u64,
}

impl Default for RepositoryConfig {
    fn default() -> Self {
        Self {
            database_url: "postgres://localhost/auth".to_string(),
            max_connections: 5,
            connect_timeout: Duration::from_secs(3),
            rate_limit_burst: 50,
            rate_limit_replenish_ms: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AuditEvent {
    pub user_id: Uuid,
    pub action: String,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

pub struct PostgresUserRepository {
    pool: PgPool,
    rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>>,
}

impl PostgresUserRepository {
    #[instrument(skip(config))]
    pub async fn new(config: RepositoryConfig) -> Result<Self, UserError> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(config.connect_timeout)
            .connect(&config.database_url)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        // Initialize rate limiter
        let burst = NonZeroU32::new(config.rate_limit_burst)
            .ok_or_else(|| UserError::ConfigError("Invalid rate limit burst value".into()))?;
        let quota = Quota::with_period(Duration::from_millis(config.rate_limit_replenish_ms))
            .ok_or_else(|| UserError::ConfigError("Invalid rate limit period".into()))?
            .allow_burst(burst);

        let rate_limiter = Arc::new(RateLimiter::direct(quota));

        info!("PostgresUserRepository initialized successfully");
        Ok(Self { pool, rate_limiter })
    }

    #[instrument(skip(self, event))]
    async fn log_audit(&self, event: AuditEvent) -> Result<(), UserError> {
        sqlx::query(
            r#"
            INSERT INTO user_audit_log (user_id, action, details, ip_address, user_agent)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(event.user_id)
        .bind(event.action)
        .bind(event.details)
        .bind(event.ip_address)
        .bind(event.user_agent)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to log audit event: {}", e);
            UserError::DatabaseError(e.to_string())
        })?;

        debug!("Audit event logged successfully");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn check_rate_limit(&self) -> Result<(), UserError> {
        if self.rate_limiter.check().is_err() {
            warn!("Rate limit exceeded");
            return Err(UserError::RateLimitExceeded);
        }
        Ok(())
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    #[instrument(skip(self, user))]
    async fn create(&self, user: &User) -> Result<(), UserError> {
        self.check_rate_limit().await?;

        // Check if email already exists
        if (self.find_by_email(&user.email).await?).is_some() {
            return Err(UserError::AlreadyExists);
        }

        // Create user
        sqlx::query(
            r#"
            INSERT INTO users (
                id, email, password_hash, created_at, updated_at,
                last_login, is_active, is_verified
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(user.id)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(user.created_at)
        .bind(user.updated_at)
        .bind(user.last_login)
        .bind(user.is_active)
        .bind(user.is_verified)
        .execute(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        // Log audit event
        self.log_audit(AuditEvent {
            user_id: user.id,
            action: "REGISTRATION".to_string(),
            details: {
                let mut map = serde_json::Map::new();
                map.insert(
                    "email".to_string(),
                    serde_json::Value::String(user.email.clone()),
                );
                map.insert(
                    "is_verified".to_string(),
                    serde_json::Value::Bool(user.is_verified),
                );
                serde_json::Value::Object(map)
            },
            ip_address: None,
            user_agent: None,
        })
        .await?;

        info!("User created successfully: {}", user.id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, UserError> {
        self.check_rate_limit().await?;

        let user = sqlx::query_as!(
            User,
            r#"
            SELECT
                id, email, password_hash, created_at, updated_at,
                last_login, is_active, is_verified
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        debug!("User lookup by ID complete: {}", id);
        Ok(user)
    }

    #[instrument(skip(self))]
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, UserError> {
        self.check_rate_limit().await?;

        let user = sqlx::query_as!(
            User,
            r#"
            SELECT
                id, email, password_hash, created_at, updated_at,
                last_login, is_active, is_verified
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        debug!("User lookup by email complete: {}", email);
        Ok(user)
    }

    #[instrument(skip(self, user))]
    async fn update(&self, user: &User) -> Result<(), UserError> {
        self.check_rate_limit().await?;

        let result = sqlx::query(
            r#"
            UPDATE users
            SET
                email = $1,
                password_hash = $2,
                updated_at = $3,
                last_login = $4,
                is_active = $5,
                is_verified = $6
            WHERE id = $7
            "#,
        )
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(user.updated_at)
        .bind(user.last_login)
        .bind(user.is_active)
        .bind(user.is_verified)
        .bind(user.id)
        .execute(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(UserError::NotFound);
        }

        info!("User updated successfully: {}", user.id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: Uuid) -> Result<(), UserError> {
        self.check_rate_limit().await?;

        // First delete audit logs
        sqlx::query!("DELETE FROM user_audit_log WHERE user_id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        // Then delete user
        let result = sqlx::query!("DELETE FROM users WHERE id = $1", id)
            .execute(&self.pool)
            .await
            .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(UserError::NotFound);
        }

        info!("User deleted successfully: {}", id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn verify_email(&self, id: Uuid) -> Result<(), UserError> {
        self.check_rate_limit().await?;

        let now = OffsetDateTime::now_utc();
        let result = sqlx::query(
            r#"
            UPDATE users
            SET
                is_verified = true,
                updated_at = $1,
                verification_token = NULL,
                verification_token_expires_at = NULL
            WHERE id = $2
            "#,
        )
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(UserError::NotFound);
        }

        // Log audit event
        self.log_audit(AuditEvent {
            user_id: id,
            action: "EMAIL_VERIFICATION_SUCCESS".to_string(),
            details: {
                let mut map = serde_json::Map::new();
                map.insert(
                    "verified_at".to_string(),
                    serde_json::Value::String(now.to_string()),
                );
                serde_json::Value::Object(map)
            },
            ip_address: None,
            user_agent: None,
        })
        .await?;

        info!("User email verified successfully: {}", id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn deactivate(&self, id: Uuid) -> Result<(), UserError> {
        self.check_rate_limit().await?;

        let now = OffsetDateTime::now_utc();
        let result = sqlx::query(
            r#"
            UPDATE users
            SET
                is_active = false,
                updated_at = $1
            WHERE id = $2
            "#,
        )
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(UserError::NotFound);
        }

        // Log audit event
        self.log_audit(AuditEvent {
            user_id: id,
            action: "USER_DEACTIVATED".to_string(),
            details: {
                let mut map = serde_json::Map::new();
                map.insert(
                    "deactivated_at".to_string(),
                    serde_json::Value::String(now.to_string()),
                );
                serde_json::Value::Object(map)
            },
            ip_address: None,
            user_agent: None,
        })
        .await?;

        info!("User deactivated successfully: {}", id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn activate(&self, id: Uuid) -> Result<(), UserError> {
        self.check_rate_limit().await?;

        let now = OffsetDateTime::now_utc();
        let result = sqlx::query(
            r#"
            UPDATE users
            SET
                is_active = true,
                updated_at = $1
            WHERE id = $2
            "#,
        )
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| UserError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(UserError::NotFound);
        }

        // Log audit event
        self.log_audit(AuditEvent {
            user_id: id,
            action: "USER_ACTIVATED".to_string(),
            details: {
                let mut map = serde_json::Map::new();
                map.insert(
                    "activated_at".to_string(),
                    serde_json::Value::String(now.to_string()),
                );
                serde_json::Value::Object(map)
            },
            ip_address: None,
            user_agent: None,
        })
        .await?;

        info!("User activated successfully: {}", id);
        Ok(())
    }
}
