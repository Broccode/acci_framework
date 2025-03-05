//! ACCI Framework - API Module
//!
//! Implements the API infrastructure and routing

pub mod config;
pub mod documentation;
pub mod handlers;
pub mod middleware;
pub mod monitoring;
pub mod response;
pub mod router;
pub mod validation;

// Re-exports
pub use config::ApiConfig;
pub use documentation::ApiDocumentation;
pub use handlers::auth::{ApiAppState, api_login, api_register, validate_token};
pub use monitoring::init_metrics;
pub use response::{ApiError, ApiResponse, ResponseStatus, ResultExt};
pub use router::ApiRouter;

// Customized public API for validation
pub use validation::{
    ValidatedData, ValidationErrorResponse, generate_request_id, handle_json_extraction_error,
    rate_limiter, validate_json_payload,
};

/// Initializes the API with the provided configuration
///
/// # Example
///
/// ```rust,no_run
/// use acci_api::{init_api, ApiConfig};
///
/// let config = ApiConfig::default();
/// let router = init_api(config);
/// ```
pub fn init_api(config: ApiConfig) -> axum::Router {
    // Initialize metrics first
    match monitoring::init_metrics() {
        Ok(_) => {},
        Err(e) => {
            tracing::error!("Failed to initialize metrics: {}", e);
        },
    }

    let api_router = ApiRouter::new(config.clone());
    let documentation = ApiDocumentation::new(config.clone());

    let router = api_router.create_router();
    documentation.register_routes(router)
}
