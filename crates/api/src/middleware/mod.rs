//! API Middleware
//!
//! This module contains middleware components for the API infrastructure.
//! Middlewares can be used to intercept and modify requests and responses.

pub mod error_handling;
pub mod logging;

use crate::config::ApiConfig;
use axum::Router;

/// Middleware stack builder
pub struct MiddlewareStack {
    _config: ApiConfig,
}

impl MiddlewareStack {
    /// Creates a new middleware stack with the specified configuration
    pub fn new(config: ApiConfig) -> Self {
        Self { _config: config }
    }

    /// Applies all middlewares to the router
    pub fn apply(self, router: Router) -> Router {
        // Apply middlewares in the reverse order of execution
        router
            // Error handling middleware should be first in the chain (last to execute)
            .layer(axum::middleware::from_fn(error_handling::error_handling_middleware))
            // Logging middleware
            .layer(axum::middleware::from_fn(logging::logging_middleware))
    }
}
