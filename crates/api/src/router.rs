use crate::config::ApiConfig;
use crate::handlers::auth::{ApiAppState, api_login, api_register, validate_token};
use crate::response::ApiResponse;
use axum::{
    Json, Router,
    http::StatusCode,
    middleware,
    routing::{get, post},
};

/// API Router structure
pub struct ApiRouter {
    config: ApiConfig,
}

impl ApiRouter {
    /// Creates a new API Router with the provided configuration
    pub fn new(config: ApiConfig) -> Self {
        Self { config }
    }

    /// Creates the Axum router for the API with the provided app state
    pub fn create_router_with_state(&self, state: ApiAppState) -> Router {
        // Create auth routes
        let auth_routes = Router::new()
            .route("/login", post(api_login))
            .route("/register", post(api_register))
            .route("/validate-token", post(validate_token));

        // Create base router
        let router = Router::new()
            // Health check
            .route("/health", get(|| async { "OK" }))
            // Example route demonstrating the API response
            .route("/example", get(example_handler))
            // Nest auth routes
            .nest("/auth", auth_routes)

            // Apply middleware chain (in reverse order of execution)
            .layer(middleware::from_fn(crate::middleware::logging::logging_middleware))
            .with_state(state);

        // Apply base URL path
        if self.config.base_path.is_empty() {
            router
        } else {
            Router::new().nest(&self.config.base_path, router)
        }
    }

    /// Creates the Axum router for the API (without state, for compatibility)
    /// Note: In a real application, this method would be removed and replaced
    /// with create_router_with_state.
    pub fn create_router(&self) -> Router {
        // Since we don't have a state, we create a simple router without auth routes
        let router = Router::new()
            // Health check
            .route("/health", get(|| async { "OK" }))
            // Example route demonstrating the API response
            .route("/example", get(example_handler))

            // Apply middleware chain (in reverse order of execution)
            .layer(middleware::from_fn(crate::middleware::logging::logging_middleware));

        // Apply base URL path
        if self.config.base_path.is_empty() {
            router
        } else {
            Router::new().nest(&self.config.base_path, router)
        }
    }
}

/// Example handler that returns a successful API response
async fn example_handler() -> (StatusCode, Json<ApiResponse<ExampleResponse>>) {
    let request_id = generate_request_id();
    let response = ApiResponse::success(
        ExampleResponse {
            message: "Example API response".to_string(),
            timestamp: current_timestamp(),
        },
        request_id,
    );

    (StatusCode::OK, Json(response))
}

/// Simple example response
#[derive(serde::Serialize)]
struct ExampleResponse {
    message: String,
    timestamp: String,
}

/// Generates a unique request ID based on the current time
/// This is used for tracking requests in logs and error responses
pub fn generate_request_id() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_nanos())
}

/// Creates a timestamp in ISO 8601 format
fn current_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Format as a simple timestamp
    format!("{} seconds since UNIX Epoch", now)
}
