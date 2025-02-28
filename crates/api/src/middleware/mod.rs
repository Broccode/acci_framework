//! API Middleware
//!
//! This module contains middleware components for the API infrastructure.
//! Middlewares can be used to intercept and modify requests and responses.

pub mod logging;

use crate::config::ApiConfig;
use axum::Router;

/// Middleware stack builder
pub struct MiddlewareStack {
    config: ApiConfig,
}

impl MiddlewareStack {
    /// Creates a new middleware stack with the specified configuration
    pub fn new(config: ApiConfig) -> Self {
        Self { config }
    }

    /// Applies all middlewares to the router
    pub fn apply(self, router: Router) -> Router {
        // Basic logging middleware
        router.layer(axum::middleware::from_fn(logging::logging_middleware))
    }
}
