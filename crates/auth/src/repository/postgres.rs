use crate::models::{
    tenant::{
        CreateSubscriptionDto, CreateTenantDto, CreateTenantUserDto, Tenant, TenantRepository,
        TenantSubscription, TenantUser, UpdateSubscriptionDto, UpdateTenantDto,
        UpdateTenantUserDto,
    },
    user::{User, UserError, UserRepository},
};
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

use crate::models::tenant::TenantError;

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

#[derive(Debug, Clone, Serialize)]
pub struct TenantAuditEvent {
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

pub struct PostgresUserRepository {
    pool: PgPool,
    rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>>,
}

pub struct PostgresTenantRepository {
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

impl PostgresTenantRepository {
    #[instrument(skip(config))]
    pub async fn new(config: RepositoryConfig) -> Result<Self, TenantError> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(config.connect_timeout)
            .connect(&config.database_url)
            .await
            .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        // Initialize rate limiter
        let burst = NonZeroU32::new(config.rate_limit_burst)
            .ok_or_else(|| TenantError::ConfigError("Invalid rate limit burst value".into()))?;
        let quota = Quota::with_period(Duration::from_millis(config.rate_limit_replenish_ms))
            .ok_or_else(|| TenantError::ConfigError("Invalid rate limit period".into()))?
            .allow_burst(burst);

        let rate_limiter = Arc::new(RateLimiter::direct(quota));

        info!("PostgresTenantRepository initialized successfully");
        Ok(Self { pool, rate_limiter })
    }

    #[instrument(skip(self, event))]
    async fn log_tenant_audit(&self, event: TenantAuditEvent) -> Result<(), TenantError> {
        sqlx::query(
            r#"
            INSERT INTO tenant_audit_log (tenant_id, user_id, action, details, ip_address, user_agent)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(event.tenant_id)
        .bind(event.user_id)
        .bind(event.action)
        .bind(event.details)
        .bind(event.ip_address)
        .bind(event.user_agent)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to log tenant audit event: {}", e);
            TenantError::DatabaseError(e.to_string())
        })?;

        debug!("Tenant audit event logged successfully");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn check_rate_limit(&self) -> Result<(), TenantError> {
        if self.rate_limiter.check().is_err() {
            warn!("Rate limit exceeded");
            return Err(TenantError::RateLimitExceeded);
        }
        Ok(())
    }
}

#[async_trait]
impl TenantRepository for PostgresTenantRepository {
    #[instrument(skip(self, tenant))]
    async fn create_tenant(&self, tenant: CreateTenantDto) -> Result<Tenant, TenantError> {
        self.check_rate_limit().await?;

        // Check if subdomain already exists
        let existing = sqlx::query!(
            r#"SELECT id FROM tenants WHERE subdomain = $1"#,
            tenant.subdomain
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        if existing.is_some() {
            return Err(TenantError::AlreadyExists);
        }

        let now = OffsetDateTime::now_utc();
        let id = Uuid::new_v4();

        // Create tenant
        let tenant = sqlx::query_as!(
            Tenant,
            r#"
            INSERT INTO tenants (
                id, name, subdomain, is_active, created_at, updated_at, metadata
            )
            VALUES ($1, $2, $3, true, $4, $5, $6)
            RETURNING id, name, subdomain, is_active, created_at, updated_at, metadata
            "#,
            id,
            tenant.name,
            tenant.subdomain,
            now,
            now,
            tenant
                .metadata
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        // Log audit event
        self.log_tenant_audit(TenantAuditEvent {
            tenant_id: tenant.id,
            user_id: None,
            action: "TENANT_CREATION".to_string(),
            details: {
                let mut map = serde_json::Map::new();
                map.insert(
                    "name".to_string(),
                    serde_json::Value::String(tenant.name.clone()),
                );
                map.insert(
                    "subdomain".to_string(),
                    serde_json::Value::String(tenant.subdomain.clone()),
                );
                serde_json::Value::Object(map)
            },
            ip_address: None,
            user_agent: None,
        })
        .await?;

        info!("Tenant created successfully: {}", tenant.id);
        Ok(tenant)
    }

    #[instrument(skip(self))]
    async fn find_tenant_by_id(&self, id: Uuid) -> Result<Option<Tenant>, TenantError> {
        self.check_rate_limit().await?;

        let tenant = sqlx::query_as!(
            Tenant,
            r#"
            SELECT id, name, subdomain, is_active, created_at, updated_at, metadata
            FROM tenants
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        debug!("Tenant lookup by ID complete: {}", id);
        Ok(tenant)
    }

    #[instrument(skip(self))]
    async fn find_tenant_by_subdomain(
        &self,
        subdomain: &str,
    ) -> Result<Option<Tenant>, TenantError> {
        self.check_rate_limit().await?;

        let tenant = sqlx::query_as!(
            Tenant,
            r#"
            SELECT id, name, subdomain, is_active, created_at, updated_at, metadata
            FROM tenants
            WHERE subdomain = $1
            "#,
            subdomain
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        debug!("Tenant lookup by subdomain complete: {}", subdomain);
        Ok(tenant)
    }

    #[instrument(skip(self, tenant))]
    async fn update_tenant(
        &self,
        id: Uuid,
        tenant: UpdateTenantDto,
    ) -> Result<Tenant, TenantError> {
        self.check_rate_limit().await?;

        // Check if tenant exists
        let existing = self.find_tenant_by_id(id).await?;
        if existing.is_none() {
            return Err(TenantError::NotFound);
        }
        let existing = existing.unwrap();

        // Check if new subdomain is already taken (if changing)
        if let Some(subdomain) = &tenant.subdomain {
            if subdomain != &existing.subdomain {
                let subdomain_exists = sqlx::query!(
                    r#"SELECT id FROM tenants WHERE subdomain = $1 AND id != $2"#,
                    subdomain,
                    id
                )
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

                if subdomain_exists.is_some() {
                    return Err(TenantError::AlreadyExists);
                }
            }
        }

        let now = OffsetDateTime::now_utc();

        // Update tenant
        let updated_tenant = sqlx::query_as!(
            Tenant,
            r#"
            UPDATE tenants
            SET
                name = COALESCE($1, name),
                subdomain = COALESCE($2, subdomain),
                is_active = COALESCE($3, is_active),
                updated_at = $4,
                metadata = COALESCE($5, metadata)
            WHERE id = $6
            RETURNING id, name, subdomain, is_active, created_at, updated_at, metadata
            "#,
            tenant.name,
            tenant.subdomain,
            tenant.is_active,
            now,
            tenant.metadata,
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        // Log audit event
        self.log_tenant_audit(TenantAuditEvent {
            tenant_id: updated_tenant.id,
            user_id: None,
            action: "TENANT_UPDATE".to_string(),
            details: {
                let mut map = serde_json::Map::new();
                if tenant.name.is_some() {
                    map.insert(
                        "name".to_string(),
                        serde_json::Value::String(updated_tenant.name.clone()),
                    );
                }
                if tenant.subdomain.is_some() {
                    map.insert(
                        "subdomain".to_string(),
                        serde_json::Value::String(updated_tenant.subdomain.clone()),
                    );
                }
                if tenant.is_active.is_some() {
                    map.insert(
                        "is_active".to_string(),
                        serde_json::Value::Bool(updated_tenant.is_active),
                    );
                }
                serde_json::Value::Object(map)
            },
            ip_address: None,
            user_agent: None,
        })
        .await?;

        info!("Tenant updated successfully: {}", updated_tenant.id);
        Ok(updated_tenant)
    }

    #[instrument(skip(self))]
    async fn delete_tenant(&self, id: Uuid) -> Result<(), TenantError> {
        self.check_rate_limit().await?;

        // Start a transaction
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        // Delete tenant subscriptions
        sqlx::query!(
            r#"DELETE FROM tenant_subscriptions WHERE tenant_id = $1"#,
            id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        // Delete tenant users
        sqlx::query!(r#"DELETE FROM tenant_users WHERE tenant_id = $1"#, id)
            .execute(&mut *tx)
            .await
            .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        // Delete tenant audit logs
        sqlx::query!(r#"DELETE FROM tenant_audit_log WHERE tenant_id = $1"#, id)
            .execute(&mut *tx)
            .await
            .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        // Delete tenant
        let result = sqlx::query!(r#"DELETE FROM tenants WHERE id = $1"#, id)
            .execute(&mut *tx)
            .await
            .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(TenantError::NotFound);
        }

        // Commit transaction
        tx.commit()
            .await
            .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        info!("Tenant deleted successfully: {}", id);
        Ok(())
    }

    #[instrument(skip(self, subscription))]
    async fn create_subscription(
        &self,
        tenant_id: Uuid,
        subscription: CreateSubscriptionDto,
    ) -> Result<TenantSubscription, TenantError> {
        self.check_rate_limit().await?;

        // Check if tenant exists
        if (self.find_tenant_by_id(tenant_id).await?).is_none() {
            return Err(TenantError::NotFound);
        }

        let now = OffsetDateTime::now_utc();
        let id = Uuid::new_v4();

        // Handle is_active default
        let is_active = subscription.is_active.unwrap_or(true);

        // Convert plan_type to string for SQL
        let plan_type_str = subscription.plan_type.to_string().to_uppercase();

        let subscription = sqlx::query_as!(
            TenantSubscription,
            r#"
            INSERT INTO tenant_subscriptions (
                id, tenant_id, plan_type, starts_at, expires_at, is_active, 
                payment_status, max_users, features, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, tenant_id, plan_type as "plan_type: _", starts_at, expires_at, is_active, 
                     payment_status, max_users, features, created_at, updated_at
            "#,
            id,
            tenant_id,
            plan_type_str as _,
            subscription.starts_at,
            subscription.expires_at,
            is_active,
            subscription.payment_status,
            subscription.max_users,
            subscription
                .features
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
            now,
            now
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        // Log the tenant subscription creation
        self.log_tenant_audit(TenantAuditEvent {
            tenant_id,
            user_id: None,
            action: "SUBSCRIPTION_CREATED".to_string(),
            details: serde_json::json!({
                "subscription_id": subscription.id,
                "plan_type": subscription.plan_type.to_string(),
                "starts_at": subscription.starts_at,
                "expires_at": subscription.expires_at,
            }),
            ip_address: None,
            user_agent: None,
        })
        .await?;

        Ok(subscription)
    }

    #[instrument(skip(self))]
    async fn get_active_subscription(
        &self,
        tenant_id: Uuid,
    ) -> Result<Option<TenantSubscription>, TenantError> {
        self.check_rate_limit().await?;

        let subscription = sqlx::query_as!(
            TenantSubscription,
            r#"
            SELECT 
                id, tenant_id, plan_type as "plan_type: _", starts_at, expires_at, is_active, 
                payment_status, max_users, features, created_at, updated_at
            FROM tenant_subscriptions
            WHERE tenant_id = $1 AND is_active = true
            AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            tenant_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        Ok(subscription)
    }

    #[instrument(skip(self, subscription))]
    async fn update_subscription(
        &self,
        id: Uuid,
        subscription: UpdateSubscriptionDto,
    ) -> Result<TenantSubscription, TenantError> {
        self.check_rate_limit().await?;

        let now = OffsetDateTime::now_utc();

        // Prepare plan_type string if it exists
        let plan_type_str = if let Some(plan_type) = &subscription.plan_type {
            Some(plan_type.to_string().to_uppercase())
        } else {
            None
        };

        // Update subscription
        let updated = sqlx::query_as!(
            TenantSubscription,
            r#"
            UPDATE tenant_subscriptions
            SET
                plan_type = COALESCE($1, plan_type),
                expires_at = COALESCE($2, expires_at),
                is_active = COALESCE($3, is_active),
                payment_status = COALESCE($4, payment_status),
                max_users = COALESCE($5, max_users),
                features = COALESCE($6, features),
                updated_at = $7
            WHERE id = $8
            RETURNING id, tenant_id, plan_type as "plan_type: _", starts_at, expires_at, is_active, 
                     payment_status, max_users, features, created_at, updated_at
            "#,
            plan_type_str as Option<String>,
            subscription.expires_at,
            subscription.is_active,
            subscription.payment_status,
            subscription.max_users,
            subscription.features,
            now,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        if let Some(subscription) = updated {
            // Log the update
            self.log_tenant_audit(TenantAuditEvent {
                tenant_id: subscription.tenant_id,
                user_id: None,
                action: "SUBSCRIPTION_UPDATED".to_string(),
                details: serde_json::json!({
                    "subscription_id": subscription.id,
                    "plan_type": subscription.plan_type.to_string(),
                    "expires_at": subscription.expires_at,
                    "is_active": subscription.is_active
                }),
                ip_address: None,
                user_agent: None,
            })
            .await?;

            info!("Subscription updated successfully: {}", subscription.id);
            Ok(subscription)
        } else {
            Err(TenantError::NotFound)
        }
    }

    #[instrument(skip(self, user))]
    async fn add_user_to_tenant(
        &self,
        tenant_id: Uuid,
        user: CreateTenantUserDto,
    ) -> Result<TenantUser, TenantError> {
        self.check_rate_limit().await?;

        // Check if tenant exists
        if (self.find_tenant_by_id(tenant_id).await?).is_none() {
            return Err(TenantError::NotFound);
        }

        // Check if user exists
        let user_exists = sqlx::query!(r#"SELECT id FROM users WHERE id = $1"#, user.user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        if user_exists.is_none() {
            return Err(TenantError::ValidationError("User does not exist".into()));
        }

        // Check if association already exists
        let existing = sqlx::query!(
            r#"SELECT tenant_id, user_id FROM tenant_users WHERE tenant_id = $1 AND user_id = $2"#,
            tenant_id,
            user.user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        if existing.is_some() {
            return Err(TenantError::AlreadyExists);
        }

        let now = OffsetDateTime::now_utc();
        let is_active = user.is_active.unwrap_or(true);

        // Add user to tenant
        let tenant_user = sqlx::query_as!(
            TenantUser,
            r#"
            INSERT INTO tenant_users (
                tenant_id, user_id, tenant_role, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING tenant_id, user_id, tenant_role, is_active, created_at, updated_at
            "#,
            tenant_id,
            user.user_id,
            user.tenant_role,
            is_active,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        // Log audit event
        self.log_tenant_audit(TenantAuditEvent {
            tenant_id,
            user_id: Some(user.user_id),
            action: "USER_ADDED_TO_TENANT".to_string(),
            details: {
                let mut map = serde_json::Map::new();
                map.insert(
                    "role".to_string(),
                    serde_json::Value::String(tenant_user.tenant_role.clone()),
                );
                map.insert(
                    "is_active".to_string(),
                    serde_json::Value::Bool(tenant_user.is_active),
                );
                serde_json::Value::Object(map)
            },
            ip_address: None,
            user_agent: None,
        })
        .await?;

        info!("User added to tenant: {} -> {}", user.user_id, tenant_id);
        Ok(tenant_user)
    }

    #[instrument(skip(self))]
    async fn get_tenant_users(&self, tenant_id: Uuid) -> Result<Vec<TenantUser>, TenantError> {
        self.check_rate_limit().await?;

        let users = sqlx::query_as!(
            TenantUser,
            r#"
            SELECT tenant_id, user_id, tenant_role, is_active, created_at, updated_at
            FROM tenant_users
            WHERE tenant_id = $1
            "#,
            tenant_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        debug!("Retrieved {} users for tenant {}", users.len(), tenant_id);
        Ok(users)
    }

    #[instrument(skip(self))]
    async fn get_user_tenants(&self, user_id: Uuid) -> Result<Vec<TenantUser>, TenantError> {
        self.check_rate_limit().await?;

        let tenants = sqlx::query_as!(
            TenantUser,
            r#"
            SELECT tenant_id, user_id, tenant_role, is_active, created_at, updated_at
            FROM tenant_users
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        debug!("Retrieved {} tenants for user {}", tenants.len(), user_id);
        Ok(tenants)
    }

    #[instrument(skip(self, update))]
    async fn update_tenant_user(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        update: UpdateTenantUserDto,
    ) -> Result<TenantUser, TenantError> {
        self.check_rate_limit().await?;

        let now = OffsetDateTime::now_utc();

        // Update tenant user
        let updated = sqlx::query_as!(
            TenantUser,
            r#"
            UPDATE tenant_users
            SET
                tenant_role = COALESCE($1, tenant_role),
                is_active = COALESCE($2, is_active),
                updated_at = $3
            WHERE tenant_id = $4 AND user_id = $5
            RETURNING tenant_id, user_id, tenant_role, is_active, created_at, updated_at
            "#,
            update.tenant_role,
            update.is_active,
            now,
            tenant_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        if let Some(tenant_user) = updated {
            // Log audit event
            self.log_tenant_audit(TenantAuditEvent {
                tenant_id,
                user_id: Some(user_id),
                action: "TENANT_USER_UPDATED".to_string(),
                details: {
                    let mut map = serde_json::Map::new();
                    if update.tenant_role.is_some() {
                        map.insert(
                            "role".to_string(),
                            serde_json::Value::String(tenant_user.tenant_role.clone()),
                        );
                    }
                    if update.is_active.is_some() {
                        map.insert(
                            "is_active".to_string(),
                            serde_json::Value::Bool(tenant_user.is_active),
                        );
                    }
                    serde_json::Value::Object(map)
                },
                ip_address: None,
                user_agent: None,
            })
            .await?;

            info!("Tenant user updated: {} in {}", user_id, tenant_id);
            Ok(tenant_user)
        } else {
            Err(TenantError::NotFound)
        }
    }

    #[instrument(skip(self))]
    async fn remove_user_from_tenant(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), TenantError> {
        self.check_rate_limit().await?;

        // Remove user from tenant
        let result = sqlx::query!(
            r#"DELETE FROM tenant_users WHERE tenant_id = $1 AND user_id = $2"#,
            tenant_id,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| TenantError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(TenantError::NotFound);
        }

        // Log audit event
        self.log_tenant_audit(TenantAuditEvent {
            tenant_id,
            user_id: Some(user_id),
            action: "USER_REMOVED_FROM_TENANT".to_string(),
            details: serde_json::Value::Object(serde_json::Map::new()),
            ip_address: None,
            user_agent: None,
        })
        .await?;

        info!("User removed from tenant: {} from {}", user_id, tenant_id);
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
