use crate::repository::tenant_aware::{RepositoryError, TenantAwareContext};
use uuid::Uuid;

/// Mock implementation of TenantAwareContext for tests
pub struct MockTenantAwareContext;

impl TenantAwareContext for MockTenantAwareContext {
    fn set_tenant_context(&self, _tenant_id: &Uuid) -> Result<(), RepositoryError> {
        // Mock implementation that does nothing
        Ok(())
    }
}

impl MockTenantAwareContext {
    /// Creates a new mock tenant-aware context for testing
    pub fn new() -> impl TenantAwareContext {
        // Return a new instance of our mock implementation
        MockTenantAwareContext
    }
}

// Function that creates a PgPool that will only be used for testing
fn panic_on_use_pool() -> sqlx::PgPool {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(1) // Need at least one connection
        .connect_lazy_with(
            sqlx::postgres::PgConnectOptions::new()
                .host("localhost")
                .port(5432)
                .database("test_db")
                .username("test")
                .password("test")
        )
}
