# Multi-Tenancy Architecture Implementation

## Overview

This document describes the implementation details for multi-tenancy in the ACCI Framework. Multi-tenancy allows our application to serve multiple organizations (tenants) while ensuring complete data isolation, customization options, and efficient resource sharing.

## Implementation Status

Initial implementation completed for Milestone 2, including:
- Tenant database schema with tables for tenant management, subscriptions, and user associations
- Core tenant models and repository interfaces
- PostgreSQL implementation of tenant repository with CRUD operations
- Tenant isolation patterns defined and implemented
- Audit logging for tenant operations

The implementation follows the architecture and approaches described in this document.

## Multi-Tenancy Model

We've selected a hybrid multi-tenancy approach combining elements of different models to achieve optimal security, performance, and customization:

### 1. Database Schema Isolation

- Each tenant will have a dedicated PostgreSQL schema
- All tenant tables will be prefixed with the schema name (e.g., `tenant_123.users`)
- Cross-tenant operations will use schema-qualified queries
- System-wide tables remain in the public schema

```sql
-- Example: Creating a new tenant schema
CREATE SCHEMA tenant_123;

-- Example: Creating tenant-specific tables
CREATE TABLE tenant_123.custom_entities (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tenant_id UUID NOT NULL REFERENCES public.tenants(id)
);

-- Example: Adding Row-Level Security for additional protection
ALTER TABLE tenant_123.custom_entities ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation_policy ON tenant_123.custom_entities
    USING (tenant_id = current_setting('app.current_tenant_id')::UUID);
```

### 2. Application-Level Tenant Context

- Every request will have a tenant context
- Middleware will resolve and validate tenant information
- Database connections will set session variables for the current tenant
- Repositories will apply tenant filtering automatically

```rust
/// Tenant information resolved for the current request
pub struct TenantContext {
    pub id: TenantId,
    pub name: String,
    pub features: Vec<FeatureFlag>,
    pub settings: TenantSettings,
    pub database_schema: String,
}

/// Middleware for resolving tenant from request
pub async fn tenant_resolution_middleware<B>(
    State(state): State<AppState>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // Resolve tenant from subdomain, header, or JWT
    let tenant_id = resolve_tenant_id(&request)?;
    
    // Load tenant information
    let tenant = state.tenant_service.get_tenant(&tenant_id).await?;
    
    // Store tenant context in request extensions
    let mut request = request;
    request.extensions_mut().insert(tenant);
    
    // Continue to the next middleware or handler
    Ok(next.run(request).await)
}
```

### 3. Tenant Resolution Strategies

We'll support multiple ways to identify the tenant:

1. **Subdomain-based resolution**: `tenant-name.example.com`
2. **Custom domain resolution**: `tenant-custom-domain.com` → mapped to tenant
3. **Header-based resolution**: `X-Tenant-ID: tenant_123`
4. **JWT claim resolution**: `tenant_id` claim in the authentication token
5. **Path-based resolution**: `/api/tenants/tenant_123/resources`

```rust
/// Resolves tenant ID from the request
fn resolve_tenant_id<B>(request: &Request<B>) -> Result<TenantId, StatusCode> {
    // Try subdomain first
    if let Some(tenant_id) = resolve_from_subdomain(request) {
        return Ok(tenant_id);
    }
    
    // Try custom domain
    if let Some(tenant_id) = resolve_from_custom_domain(request) {
        return Ok(tenant_id);
    }
    
    // Try header
    if let Some(tenant_id) = resolve_from_header(request) {
        return Ok(tenant_id);
    }
    
    // Try JWT claim
    if let Some(tenant_id) = resolve_from_jwt(request) {
        return Ok(tenant_id);
    }
    
    // Try path
    if let Some(tenant_id) = resolve_from_path(request) {
        return Ok(tenant_id);
    }
    
    // No tenant found
    Err(StatusCode::BAD_REQUEST)
}
```

## Tenant-Aware Components

### 1. Tenant-Aware Repository

All repositories will be tenant-aware, ensuring data isolation:

```rust
pub struct PostgresTenantAwareRepository<T> {
    pool: PgPool,
    phantom: PhantomData<T>,
}

impl<T> PostgresTenantAwareRepository<T> {
    /// Sets the tenant context for the database session
    async fn set_tenant_context(&self, conn: &mut PgConnection, tenant_id: &TenantId) -> Result<(), RepositoryError> {
        // Set current tenant ID as a session variable for RLS policies
        sqlx::query("SET LOCAL app.current_tenant_id = $1")
            .bind(tenant_id.to_string())
            .execute(conn)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
            
        // Set search_path to include tenant schema
        let schema = format!("tenant_{}", tenant_id);
        sqlx::query(&format!("SET LOCAL search_path = {}, public", schema))
            .execute(conn)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
            
        Ok(())
    }
    
    /// Executes a function within a tenant context
    async fn with_tenant<F, R>(&self, tenant_id: &TenantId, f: F) -> Result<R, RepositoryError>
    where
        F: FnOnce(&mut PgConnection) -> Pin<Box<dyn Future<Output = Result<R, RepositoryError>> + Send>>,
    {
        let mut conn = self.pool.acquire().await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            
        // Begin transaction
        let mut tx = conn.begin().await
            .map_err(|e| RepositoryError::TransactionError(e.to_string()))?;
            
        // Set tenant context
        self.set_tenant_context(&mut tx, tenant_id).await?;
        
        // Execute the function
        let result = f(&mut tx).await;
        
        // Commit or rollback
        match result {
            Ok(value) => {
                tx.commit().await
                    .map_err(|e| RepositoryError::TransactionError(e.to_string()))?;
                Ok(value)
            }
            Err(e) => {
                tx.rollback().await
                    .map_err(|_| RepositoryError::TransactionError("Rollback failed".to_string()))?;
                Err(e)
            }
        }
    }
}
```

### 2. Tenant-Aware Service

Services will operate within the tenant context:

```rust
pub struct TenantAwareUserService {
    user_repository: Arc<dyn UserRepository>,
    tenant_repository: Arc<dyn TenantRepository>,
}

impl TenantAwareUserService {
    pub async fn create_user(
        &self,
        tenant_context: &TenantContext,
        create_user: CreateUser,
    ) -> Result<User, ServiceError> {
        // Validate tenant-specific policies
        self.validate_tenant_policies(tenant_context, &create_user).await?;
        
        // Create user within tenant context
        let user = self.user_repository
            .create_user_for_tenant(&tenant_context.id, create_user)
            .await
            .map_err(ServiceError::from)?;
            
        // Log audit event with tenant context
        audit_log::log_event(
            AuditEvent::UserCreated {
                user_id: user.id.clone(),
                tenant_id: tenant_context.id.clone(),
            },
            Some(tenant_context.id.clone()),
        ).await;
        
        Ok(user)
    }
    
    async fn validate_tenant_policies(
        &self,
        tenant_context: &TenantContext,
        create_user: &CreateUser,
    ) -> Result<(), ServiceError> {
        // Check tenant-specific user limits
        let user_count = self.user_repository
            .count_users_for_tenant(&tenant_context.id)
            .await
            .map_err(ServiceError::from)?;
            
        let tenant = self.tenant_repository
            .get_tenant(&tenant_context.id)
            .await
            .map_err(ServiceError::from)?;
            
        if let Some(max_users) = tenant.limits.max_users {
            if user_count >= max_users {
                return Err(ServiceError::TenantLimitExceeded("User limit exceeded".to_string()));
            }
        }
        
        // Check tenant-specific email domain restrictions
        if let Some(allowed_domains) = &tenant.settings.allowed_email_domains {
            let email_domain = create_user.email.split('@').nth(1)
                .ok_or_else(|| ServiceError::ValidationError("Invalid email format".to_string()))?;
                
            if !allowed_domains.contains(&email_domain.to_string()) {
                return Err(ServiceError::ValidationError(format!(
                    "Email domain '{}' is not allowed for this tenant", 
                    email_domain
                )));
            }
        }
        
        Ok(())
    }
}
```

### 3. Tenant Configuration System

Each tenant will have its own configuration:

```rust
pub struct TenantSettings {
    // Branding
    pub company_name: String,
    pub logo_url: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    
    // Features
    pub enabled_features: Vec<String>,
    pub feature_settings: HashMap<String, Value>,
    
    // Security
    pub password_policy: PasswordPolicy,
    pub mfa_required: bool,
    pub session_timeout_minutes: u32,
    pub allowed_email_domains: Option<Vec<String>>,
    
    // Limits
    pub max_users: Option<u32>,
    pub max_storage_mb: Option<u32>,
    
    // Customization
    pub custom_fields: HashMap<String, Value>,
    pub default_language: String,
}

pub struct PasswordPolicy {
    pub min_length: u8,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_symbols: bool,
    pub password_history_count: u8,
    pub max_age_days: Option<u32>,
    pub min_strength_score: u8,
}
```

## Tenant Provisioning

The tenant provisioning process will include:

1. **Tenant registration**: Basic information collection
2. **Schema creation**: Setting up the database schema
3. **Initial user creation**: Admin user setup
4. **Default configuration**: Setting default tenant settings
5. **Feature activation**: Enabling appropriate features

```rust
pub struct TenantProvisioningService {
    db_pool: PgPool,
    config: TenantProvisioningConfig,
}

impl TenantProvisioningService {
    pub async fn provision_tenant(
        &self,
        request: ProvisionTenantRequest,
    ) -> Result<ProvisionedTenant, ProvisioningError> {
        // Begin transaction
        let mut tx = self.db_pool.begin().await
            .map_err(|e| ProvisioningError::DatabaseError(e.to_string()))?;
        
        // 1. Create tenant record
        let tenant_id = self.create_tenant_record(&mut tx, &request).await?;
        
        // 2. Create tenant schema
        self.create_tenant_schema(&mut tx, &tenant_id).await?;
        
        // 3. Create initial admin user
        let admin_user = self.create_admin_user(&mut tx, &tenant_id, &request.admin_user).await?;
        
        // 4. Set default configurations
        self.set_default_configurations(&mut tx, &tenant_id, &request.settings).await?;
        
        // 5. Enable features
        self.enable_features(&mut tx, &tenant_id, &request.features).await?;
        
        // Commit transaction
        tx.commit().await
            .map_err(|e| ProvisioningError::DatabaseError(e.to_string()))?;
            
        // Return provisioned tenant information
        Ok(ProvisionedTenant {
            tenant_id,
            admin_user,
            dashboard_url: self.generate_dashboard_url(&tenant_id),
            api_key: self.generate_initial_api_key(&tenant_id),
        })
    }
    
    async fn create_tenant_schema(&self, tx: &mut Transaction<'_, Postgres>, tenant_id: &TenantId) -> Result<(), ProvisioningError> {
        // Create schema
        let schema_name = format!("tenant_{}", tenant_id);
        sqlx::query(&format!("CREATE SCHEMA IF NOT EXISTS {}", schema_name))
            .execute(&mut **tx)
            .await
            .map_err(|e| ProvisioningError::SchemaCreationError(e.to_string()))?;
            
        // Apply migrations to the new schema
        self.apply_migrations_to_schema(tx, &schema_name).await?;
            
        Ok(())
    }
    
    async fn apply_migrations_to_schema(&self, tx: &mut Transaction<'_, Postgres>, schema_name: &str) -> Result<(), ProvisioningError> {
        // Set search path to the new schema
        sqlx::query(&format!("SET search_path TO {}", schema_name))
            .execute(&mut **tx)
            .await
            .map_err(|e| ProvisioningError::MigrationError(e.to_string()))?;
            
        // Apply tenant-specific migrations
        // This would use a migration manager to apply all tenant-specific migrations
        // Here we're showing a simplified example
        
        // Create users table in tenant schema
        sqlx::query(
            "CREATE TABLE users (
                id UUID PRIMARY KEY,
                email TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                tenant_id UUID NOT NULL
            )"
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| ProvisioningError::MigrationError(e.to_string()))?;
        
        // Add more tables...
        
        Ok(())
    }
}
```

## Cross-Tenant Operations

Some operations may need to span multiple tenants:

```rust
pub struct CrossTenantOperations {
    db_pool: PgPool,
}

impl CrossTenantOperations {
    /// Gets user counts across all tenants (admin operation)
    pub async fn get_tenant_user_counts(&self) -> Result<Vec<TenantUserCount>, RepositoryError> {
        // This query joins the public tenants table with each tenant's users table
        // It requires admin privileges
        let counts = sqlx::query_as::<_, TenantUserCount>(
            "SELECT
                t.id as tenant_id,
                t.name as tenant_name,
                COUNT(u.id) as user_count
             FROM
                public.tenants t
             LEFT JOIN LATERAL (
                SELECT COUNT(*) as user_count
                FROM (
                    SELECT 1
                    FROM information_schema.schemata
                    WHERE schema_name = 'tenant_' || t.id::text
                ) s
                CROSS JOIN LATERAL (
                    SELECT COUNT(*) as count
                    FROM (
                        SELECT 1
                        FROM information_schema.tables
                        WHERE table_schema = 'tenant_' || t.id::text
                        AND table_name = 'users'
                    ) tables
                    CROSS JOIN LATERAL (
                        EXECUTE 'SELECT COUNT(*) FROM tenant_' || t.id::text || '.users'
                    ) users(count)
                ) counts(count)
             ) counts(user_count) ON true
             GROUP BY t.id, t.name"
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        
        Ok(counts)
    }
    
    /// Copy a template entity to a specific tenant
    pub async fn copy_template_to_tenant(
        &self,
        template_id: &TemplateId,
        tenant_id: &TenantId,
    ) -> Result<(), RepositoryError> {
        let mut tx = self.db_pool.begin().await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            
        // Get template from public schema
        let template = sqlx::query_as::<_, Template>(
            "SELECT * FROM public.templates WHERE id = $1"
        )
        .bind(template_id)
        .fetch_one(&mut tx)
        .await
        .map_err(|e| RepositoryError::NotFound(e.to_string()))?;
        
        // Set search path to tenant schema
        let schema_name = format!("tenant_{}", tenant_id);
        sqlx::query(&format!("SET search_path TO {}", schema_name))
            .execute(&mut tx)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
            
        // Copy template to tenant's schema
        sqlx::query(
            "INSERT INTO templates (id, name, content, created_at, tenant_id)
             VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(Uuid::new_v4())
        .bind(&template.name)
        .bind(&template.content)
        .bind(Utc::now())
        .bind(tenant_id)
        .execute(&mut tx)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        
        tx.commit().await
            .map_err(|e| RepositoryError::TransactionError(e.to_string()))?;
            
        Ok(())
    }
}
```

## Tenant Isolation Testing

We'll implement comprehensive tests to ensure tenant isolation:

```rust
#[tokio::test]
async fn test_tenant_data_isolation() {
    // Setup test database
    let pool = setup_test_database().await;
    
    // Create two test tenants
    let tenant1_id = create_test_tenant(&pool, "tenant1").await;
    let tenant2_id = create_test_tenant(&pool, "tenant2").await;
    
    // Create a user in tenant1
    let user1 = create_test_user(&pool, &tenant1_id, "user1@tenant1.com").await;
    
    // Create a user in tenant2 with the same email
    let user2 = create_test_user(&pool, &tenant2_id, "user1@tenant1.com").await;
    
    // Both users should exist (email uniqueness is per-tenant)
    assert_ne!(user1.id, user2.id);
    
    // Test tenant1 context
    let tenant1_context = TenantContext {
        id: tenant1_id.clone(),
        schema: format!("tenant_{}", tenant1_id),
    };
    
    let user_repo = PostgresTenantAwareUserRepository::new(pool.clone());
    
    // Query from tenant1 perspective should only return tenant1's user
    let users1 = user_repo.find_by_email(&tenant1_context, "user1@tenant1.com").await.unwrap();
    assert_eq!(users1.len(), 1);
    assert_eq!(users1[0].id, user1.id);
    
    // Test tenant2 context
    let tenant2_context = TenantContext {
        id: tenant2_id.clone(),
        schema: format!("tenant_{}", tenant2_id),
    };
    
    // Query from tenant2 perspective should only return tenant2's user
    let users2 = user_repo.find_by_email(&tenant2_context, "user1@tenant1.com").await.unwrap();
    assert_eq!(users2.len(), 1);
    assert_eq!(users2[0].id, user2.id);
}

#[tokio::test]
async fn test_row_level_security() {
    // Setup test database
    let pool = setup_test_database().await;
    
    // Create test tenant
    let tenant_id = create_test_tenant(&pool, "test_tenant").await;
    
    // Create a user in the tenant
    let user = create_test_user(&pool, &tenant_id, "user@test.com").await;
    
    // Attempt to bypass tenant isolation with a direct query
    // This should fail because of RLS policies
    let result = sqlx::query("SELECT * FROM tenant_123.users WHERE email = $1")
        .bind("user@test.com")
        .execute(&pool)
        .await;
        
    assert!(result.is_err());
    
    // Now set the tenant context and try again
    let mut conn = pool.acquire().await.unwrap();
    
    sqlx::query("SET LOCAL app.current_tenant_id = $1")
        .bind(&tenant_id.to_string())
        .execute(&mut conn)
        .await
        .unwrap();
        
    // Now the query should work
    let user_result = sqlx::query_as::<_, User>(
        "SELECT * FROM tenant_123.users WHERE email = $1"
    )
    .bind("user@test.com")
    .fetch_one(&mut conn)
    .await
    .unwrap();
    
    assert_eq!(user_result.id, user.id);
}
```

## Performance Considerations

To ensure the multi-tenancy implementation remains performant:

1. **Connection Pooling**: Separate connection pools per tenant for high-traffic tenants
2. **Schema Caching**: Cache schema metadata to reduce catalog lookups
3. **Query Optimization**: Optimize tenant-aware queries
4. **Index Strategy**: Ensure proper indexing including tenant_id columns
5. **Monitoring**: Tenant-aware performance metrics

```rust
pub struct TenantAwareConnectionManager {
    connection_pools: RwLock<HashMap<TenantId, PgPool>>,
    default_pool: PgPool,
    config: ConnectionConfig,
}

impl TenantAwareConnectionManager {
    pub async fn get_pool(&self, tenant_id: &TenantId) -> PgPool {
        // Check if tenant has a dedicated pool
        if let Some(pool) = self.connection_pools.read().await.get(tenant_id) {
            return pool.clone();
        }
        
        // Check if tenant should have a dedicated pool based on traffic/size
        if self.should_have_dedicated_pool(tenant_id).await {
            // Create a dedicated pool for this tenant
            let pool = self.create_tenant_pool(tenant_id).await;
            
            // Store in cache
            self.connection_pools.write().await.insert(tenant_id.clone(), pool.clone());
            
            return pool;
        }
        
        // Use default pool
        self.default_pool.clone()
    }
    
    async fn create_tenant_pool(&self, tenant_id: &TenantId) -> PgPool {
        // Create connection options with tenant-specific settings
        let mut options = PgConnectOptions::new()
            .host(&self.config.host)
            .port(self.config.port)
            .username(&self.config.username)
            .password(&self.config.password)
            .database(&self.config.database);
            
        // Add tenant-specific connection parameters
        options = options
            .application_name(&format!("app-tenant-{}", tenant_id))
            .options([
                ("search_path", format!("tenant_{},public", tenant_id)),
                ("app.current_tenant_id", tenant_id.to_string()),
            ]);
            
        // Create the pool with tenant-specific settings
        let pool = PgPoolOptions::new()
            .max_connections(self.config.max_connections_per_tenant)
            .connect_with(options)
            .await
            .expect("Failed to create tenant database pool");
            
        pool
    }
    
    async fn should_have_dedicated_pool(&self, tenant_id: &TenantId) -> bool {
        // Check tenant activity metrics
        let metrics = self.metrics_service.get_tenant_metrics(tenant_id).await;
        
        // If tenant has high activity, give it a dedicated pool
        metrics.avg_requests_per_minute > self.config.dedicated_pool_threshold
    }
}
```

## Migration Strategy

For existing systems transitioning to multi-tenancy, we'll implement a phased migration:

1. **Schema Creation**: Create tenant schemas for each organization
2. **Data Migration**: Copy data from shared tables to tenant-specific schemas
3. **Schema Validation**: Verify data integrity after migration
4. **Dual-Write Period**: Write to both old and new schemas during transition
5. **Cutover**: Switch to reading from tenant schemas only

```rust
pub struct TenantMigrationService {
    source_pool: PgPool,
    target_pool: PgPool,
}

impl TenantMigrationService {
    pub async fn migrate_tenant(&self, tenant_id: &TenantId) -> Result<MigrationStats, MigrationError> {
        // Start migration
        info!("Starting migration for tenant {}", tenant_id);
        
        // Create tenant schema if it doesn't exist
        self.create_tenant_schema(tenant_id).await?;
        
        // Migrate each entity type
        let user_stats = self.migrate_users(tenant_id).await?;
        let resource_stats = self.migrate_resources(tenant_id).await?;
        
        // Verify migration
        self.verify_migration(tenant_id).await?;
        
        // Return statistics
        Ok(MigrationStats {
            users_migrated: user_stats.count,
            resources_migrated: resource_stats.count,
            errors: user_stats.errors + resource_stats.errors,
        })
    }
    
    async fn migrate_users(&self, tenant_id: &TenantId) -> Result<EntityMigrationStats, MigrationError> {
        let mut stats = EntityMigrationStats::default();
        
        // Get all users for this tenant from the source
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE tenant_id = $1"
        )
        .bind(tenant_id)
        .fetch_all(&self.source_pool)
        .await
        .map_err(|e| MigrationError::SourceError(e.to_string()))?;
        
        info!("Migrating {} users for tenant {}", users.len(), tenant_id);
        
        // Start a transaction in the target database
        let mut tx = self.target_pool.begin().await
            .map_err(|e| MigrationError::TargetError(e.to_string()))?;
            
        // Set search path to tenant schema
        let schema = format!("tenant_{}", tenant_id);
        sqlx::query(&format!("SET search_path TO {}", schema))
            .execute(&mut tx)
            .await
            .map_err(|e| MigrationError::TargetError(e.to_string()))?;
            
        // Migrate each user
        for user in users {
            match self.migrate_single_user(&mut tx, tenant_id, &user).await {
                Ok(_) => stats.count += 1,
                Err(e) => {
                    error!("Error migrating user {}: {}", user.id, e);
                    stats.errors += 1;
                    stats.error_details.push(format!("User {}: {}", user.id, e));
                }
            }
        }
        
        // Commit transaction
        tx.commit().await
            .map_err(|e| MigrationError::TargetError(e.to_string()))?;
            
        Ok(stats)
    }
    
    async fn migrate_single_user(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        tenant_id: &TenantId,
        user: &User,
    ) -> Result<(), MigrationError> {
        // Insert user into tenant schema
        sqlx::query(
            "INSERT INTO users (id, email, name, created_at, tenant_id)
             VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(&user.id)
        .bind(&user.email)
        .bind(&user.name)
        .bind(&user.created_at)
        .bind(tenant_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| MigrationError::DataError(e.to_string()))?;
        
        Ok(())
    }
    
    async fn verify_migration(&self, tenant_id: &TenantId) -> Result<(), MigrationError> {
        // Count users in source database
        let source_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE tenant_id = $1"
        )
        .bind(tenant_id)
        .fetch_one(&self.source_pool)
        .await
        .map_err(|e| MigrationError::SourceError(e.to_string()))?;
        
        // Count users in target database (tenant schema)
        let schema = format!("tenant_{}", tenant_id);
        let target_count = sqlx::query_scalar::<_, i64>(
            &format!("SELECT COUNT(*) FROM {}.users", schema)
        )
        .fetch_one(&self.target_pool)
        .await
        .map_err(|e| MigrationError::TargetError(e.to_string()))?;
        
        // Verify counts match
        if source_count != target_count {
            return Err(MigrationError::VerificationError(format!(
                "Count mismatch: source={}, target={}",
                source_count, target_count
            )));
        }
        
        Ok(())
    }
}
```

## Conclusion

This multi-tenancy implementation provides:

1. **Strong Isolation**: Complete tenant data separation
2. **Flexibility**: Support for various tenant identification methods
3. **Performance**: Optimized for multi-tenant operations
4. **Customization**: Tenant-specific configurations and features
5. **Security**: Multiple layers of tenant isolation

By implementing this architecture, we'll establish a solid foundation for the multi-tenant capabilities required in Milestone 2, enabling the platform to securely serve multiple organizations while maintaining strict data isolation.

## Next Steps

1. Implement the tenant management service
2. Create the database schema isolation layer
3. Build the tenant-aware repository pattern
4. Develop the tenant resolution middleware
5. Set up the tenant configuration system