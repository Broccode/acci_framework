use crate::models::tenant::TenantError;
use futures::future::BoxFuture;
use sqlx::{Acquire, PgPool, postgres::PgConnection};
use std::future::Future;
use std::pin::Pin;
use tracing::{debug, error};
use uuid::Uuid;

/// Tenant-aware database context manager for multi-tenancy
#[derive(Clone)]
pub struct TenantAwareContext {
    pool: PgPool,
}

/// Repository error types specific to tenant operations
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database connection error: {0}")]
    ConnectionError(String),

    #[error("Database transaction error: {0}")]
    TransactionError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Entity not found")]
    NotFound,

    #[error("Tenant error: {0}")]
    TenantError(#[from] TenantError),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

impl TenantAwareContext {
    /// Creates a new tenant-aware database context
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Executes a function within a tenant context using the tenant schema
    pub async fn with_tenant<F, R>(&self, tenant_id: &Uuid, f: F) -> Result<R, RepositoryError>
    where
        F: for<'a> FnOnce(&'a mut PgConnection) -> BoxFuture<'a, Result<R, RepositoryError>>,
    {
        debug!("Executing operation in tenant context: {}", tenant_id);

        // Acquire a connection from the pool
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // Begin a transaction
        let mut tx = conn
            .begin()
            .await
            .map_err(|e| RepositoryError::TransactionError(e.to_string()))?;

        // Set up the tenant context
        self.set_tenant_context(&mut tx, tenant_id).await?;

        // Execute the function within the transaction
        let result = f(&mut tx).await;

        // Commit or rollback the transaction
        match result {
            Ok(value) => {
                tx.commit()
                    .await
                    .map_err(|e| RepositoryError::TransactionError(e.to_string()))?;
                debug!("Transaction committed for tenant: {}", tenant_id);
                Ok(value)
            },
            Err(e) => {
                let _ = tx.rollback().await;
                error!("Transaction rolled back for tenant {}: {}", tenant_id, e);
                Err(e)
            },
        }
    }

    /// Sets up the tenant context for a database connection
    async fn set_tenant_context(
        &self,
        conn: &mut PgConnection,
        tenant_id: &Uuid,
    ) -> Result<(), RepositoryError> {
        debug!("Setting tenant context: {}", tenant_id);

        // Set the tenant ID as a session variable for row-level security
        sqlx::query("SET LOCAL app.current_tenant_id = $1")
            .bind(tenant_id.to_string())
            .execute(&mut *conn)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        // Set the search path to include the tenant schema
        let schema = format!("tenant_{}", tenant_id);
        sqlx::query(&format!("SET LOCAL search_path = {}, public", schema))
            .execute(conn)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        debug!("Tenant context set: {}", tenant_id);
        Ok(())
    }

    /// Executes a query without a tenant context (in public schema)
    pub async fn without_tenant<F, R>(&self, f: F) -> Result<R, RepositoryError>
    where
        F: FnOnce(PgPool) -> Pin<Box<dyn Future<Output = Result<R, RepositoryError>> + Send>>,
    {
        // Execute directly on the pool without setting tenant context
        f(self.pool.clone()).await
    }

    /// Get a reference to the underlying connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

/// Tenant-aware repository base trait
pub trait TenantAwareRepository {
    /// Get the tenant-aware context
    fn tenant_context(&self) -> &TenantAwareContext;

    /// Execute a query in the context of a specific tenant
    #[allow(async_fn_in_trait)]
    async fn execute_for_tenant<F, R>(&self, tenant_id: &Uuid, f: F) -> Result<R, RepositoryError>
    where
        F: for<'a> FnOnce(&'a mut PgConnection) -> BoxFuture<'a, Result<R, RepositoryError>>,
    {
        self.tenant_context().with_tenant(tenant_id, f).await
    }

    /// Execute a query without a tenant context (in public schema)
    #[allow(async_fn_in_trait)]
    async fn execute_without_tenant<F, R>(&self, f: F) -> Result<R, RepositoryError>
    where
        F: FnOnce(PgPool) -> Pin<Box<dyn Future<Output = Result<R, RepositoryError>> + Send>>,
    {
        self.tenant_context().without_tenant(f).await
    }
}
