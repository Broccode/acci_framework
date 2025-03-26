use crate::models::tenant::TenantError;
use futures::future::BoxFuture;
use sqlx::{Pool, Postgres, postgres::PgConnection};
use std::future::Future;
use tracing::{debug, error};
use uuid::Uuid;

/// Trait for implementing tenant-aware contexts
pub trait TenantAwareContext: Send + Sync {
    /// Set the tenant context for the current repository
    fn set_tenant_context(&self, tenant_id: &Uuid) -> Result<(), RepositoryError>;

    /// Get the current tenant ID if available
    fn tenant_id(&self) -> Option<Uuid> {
        None
    }
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

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Entity already exists: {0}")]
    Duplicate(String),

    #[error("Unique constraint violation: {0}")]
    UniqueViolation(String),

    #[error("Tenant ID is required for this operation")]
    TenantRequired,

    #[error("Tenant error: {0}")]
    Tenant(TenantError),

    #[error("Invalid input: {0}")]
    ValidationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}

/// Tenant-aware database context manager for multi-tenancy
#[derive(Clone)]
pub struct TenantAwareContextManager {
    pool: Pool<Postgres>,
}

impl TenantAwareContextManager {
    /// Creates a new tenant-aware database context
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Executes a function within a tenant context using the tenant schema
    pub async fn with_tenant<F, R>(&self, tenant_id: &Uuid, f: F) -> Result<R, RepositoryError>
    where
        F: for<'a> FnOnce(&'a mut PgConnection) -> BoxFuture<'a, Result<R, RepositoryError>>,
    {
        let tenant_id_str = tenant_id.to_string();
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // Set the tenant ID in the current PostgreSQL session
        sqlx::query("SET app.tenant_id = $1")
            .bind(&tenant_id_str)
            .execute(&mut *conn)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        debug!(tenant_id = %tenant_id_str, "Set tenant context");

        // Execute the function with the tenant context
        f(&mut conn).await
    }

    /// Gets a transaction with tenant isolation
    pub async fn with_tenant_tx<'c, F, R, Fut>(
        &'c self,
        tenant_id: &Uuid,
        f: F,
    ) -> Result<R, RepositoryError>
    where
        F: FnOnce(sqlx::Transaction<'c, sqlx::Postgres>) -> Fut,
        Fut: Future<Output = Result<(R, sqlx::Transaction<'c, sqlx::Postgres>), RepositoryError>>
            + Send,
    {
        let tenant_id_str = tenant_id.to_string();
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RepositoryError::TransactionError(e.to_string()))?;

        // Set the tenant ID in the transaction
        sqlx::query("SET app.tenant_id = $1")
            .bind(&tenant_id_str)
            .execute(&mut *tx)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        debug!(tenant_id = %tenant_id_str, "Set tenant context in transaction");

        // Execute the function and pass ownership of the transaction
        match f(tx).await {
            Ok((result, tx)) => {
                // Commit the transaction
                tx.commit()
                    .await
                    .map_err(|e| RepositoryError::TransactionError(e.to_string()))?;
                debug!("Successfully committed transaction");
                Ok(result)
            },
            Err(err) => {
                // Transaction is dropped which automatically rolls it back
                debug!("Transaction rolled back due to error");
                Err(err)
            },
        }
    }

    /// Get a clone of the database pool
    pub fn get_pool(&self) -> Pool<Postgres> {
        self.pool.clone()
    }
}

/// Trait for repositories that are tenant-aware
pub trait TenantAwareRepository {
    /// Set the current tenant context for all operations
    fn set_tenant_context(&mut self, tenant_id: Uuid) -> Result<(), RepositoryError>;

    /// Get the current tenant ID if set
    fn get_current_tenant(&self) -> Option<Uuid>;
}
