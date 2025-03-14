use crate::repository::tenant_aware::TenantAwareContext;

/// Mock implementation of TenantAwareContext for tests
pub struct MockTenantAwareContext;

impl MockTenantAwareContext {
    /// Creates a new mock tenant-aware context for testing
    pub fn new() -> TenantAwareContext {
        // We need to instrument the TenantAwareContext to make it test-friendly
        // by extending it with a test-specific constructor
        // For our tests, we'll just reimplement the interface enough for verification

        // In a real project, we'd probably use a more sophisticated mocking approach
        // or adapt the TenantAwareContext to be more test-friendly

        // For now, we'll just return a dummy, non-functional context
        TenantAwareContext::new(panic_on_use_pool())
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
