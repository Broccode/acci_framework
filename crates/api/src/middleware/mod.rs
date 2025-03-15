//! API Middleware
//!
//! This module contains middleware components for the API infrastructure.
//! Middlewares can be used to intercept and modify requests and responses.

pub mod error_handling;
pub mod logging;
pub mod tenant;

use crate::config::ApiConfig;
use acci_auth::models::tenant::TenantRepository;
use axum::Router;
use std::sync::Arc;
// All middleware imports are available through the unified axum import

/// Middleware stack builder for API
///
/// This struct builds and applies the middleware stack for the API router.
pub struct MiddlewareStack {
    #[allow(dead_code)]
    config: ApiConfig,
    tenant_repository: Option<Arc<dyn TenantRepository>>,
    tenant_config: Option<tenant::TenantResolutionConfig>,
}

impl MiddlewareStack {
    /// Creates a new middleware stack with the given configuration
    pub fn new(config: ApiConfig) -> Self {
        Self {
            config,
            tenant_repository: None,
            tenant_config: None,
        }
    }

    /// Adds tenant resolution middleware to the stack
    pub fn with_tenant_resolution(
        mut self,
        tenant_repository: Arc<dyn TenantRepository>,
        config: Option<tenant::TenantResolutionConfig>,
    ) -> Self {
        self.tenant_repository = Some(tenant_repository);
        self.tenant_config = config;
        self
    }

    /// Applies the middleware stack to the given router
    pub fn apply(self, router: Router) -> Router {
        let mut router = router;

        // Error handling middleware
        router = router.layer(axum::middleware::from_fn(
            error_handling::error_handling_middleware,
        ));

        // Tenant resolution middleware (if configured)
        if let Some(tenant_repository) = self.tenant_repository {
            let tenant_state = tenant::TenantState {
                tenant_repository,
                config: self.tenant_config.unwrap_or_default(),
            };

            // Apply middleware with the state
            router = router.layer(axum::middleware::from_fn_with_state(
                tenant_state,
                tenant::tenant_resolution_middleware,
            ));
        }

        // Logging middleware (first to execute)
        router = router.layer(axum::middleware::from_fn(logging::logging_middleware));

        router
    }
}
