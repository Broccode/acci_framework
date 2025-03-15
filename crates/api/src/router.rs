use crate::config::ApiConfig;
use crate::handlers::auth::{ApiAppState, api_login, api_register, validate_token};
use crate::handlers::tenant::{
    TenantAppState, create_tenant, create_tenant_with_admin, delete_tenant, get_tenant,
    get_tenant_by_id, update_tenant,
};
use crate::handlers::verification::{VerificationAppState, send_verification, verify_code};
#[cfg(feature = "enable_webauthn")]
use crate::handlers::webauthn::WebAuthnAppState;
use crate::response::ApiResponse;
use axum::{
    Json, Router,
    http::StatusCode,
    middleware,
    routing::{delete, get, post, put},
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

    /// Creates the Axum router for the API with the provided app states
    pub fn create_router_with_state(
        &self,
        auth_state: ApiAppState,
        tenant_state: Option<TenantAppState>,
        verification_state: Option<VerificationAppState>,
        #[cfg(feature = "enable_webauthn")] webauthn_state: Option<WebAuthnAppState>,
        #[cfg(not(feature = "enable_webauthn"))] webauthn_state: Option<()>,
    ) -> Router {
        // Create auth routes
        let auth_routes = Router::new()
            .route("/login", post(api_login))
            .route("/register", post(api_register))
            .route("/validate-token", post(validate_token))
            .with_state(auth_state.clone());

        // Create verification routes if verification state is provided
        let verification_routes = if let Some(verification_state) = verification_state {
            Router::new()
                .route("/send", post(send_verification))
                .route("/code", post(verify_code))
                .with_state(verification_state)
        } else {
            Router::new()
        };

        // Create tenant routes if tenant state is provided
        let tenant_routes = if let Some(tenant_state) = tenant_state {
            Router::new()
                .route("/", get(get_tenant))
                .route("/", post(create_tenant))
                .route("/", put(update_tenant))
                .route("/with-admin", post(create_tenant_with_admin))
                .route("/:id", get(get_tenant_by_id))
                .route("/:id", delete(delete_tenant))
                .with_state(tenant_state)
        } else {
            Router::new()
        };

        // Create WebAuthn routes if webauthn state is provided
        #[cfg(feature = "enable_webauthn")]
        let webauthn_routes = if let Some(webauthn_state) = webauthn_state {
            // Define the routes without complex handler functions for now
            // We'll use lambda functions directly in the routes

            // For now, create empty routes that return a 404 just to make it compile
            // This is a placeholder - real implementation will be needed
            Router::new()
                .route("/register/start", get(|| async { "WebAuthn disabled" }))
                .route(
                    "/register/complete/:id",
                    get(|_: axum::extract::Path<String>| async { "WebAuthn disabled" }),
                )
                .route("/authenticate/start", get(|| async { "WebAuthn disabled" }))
                .route(
                    "/authenticate/complete",
                    get(|| async { "WebAuthn disabled" }),
                )
                .with_state(webauthn_state)
        } else {
            Router::new()
        };

        #[cfg(not(feature = "enable_webauthn"))]
        let webauthn_routes = Router::new();

        // Create auth router with nested verification routes
        let auth_router = Router::new()
            .merge(auth_routes)
            .nest("/verify", verification_routes);

        // Create base router
        let router = Router::new()
            // Health check
            .route("/health", get(|| async { "OK" }))
            // Example route demonstrating the API response
            .route("/example", get(example_handler))
            // Nest auth routes (including verification routes)
            .nest("/auth", auth_router)
            // Nest tenant routes if applicable
            .nest("/tenants", tenant_routes)
            // Nest WebAuthn routes if applicable
            .nest("/webauthn", webauthn_routes)

            // Apply middleware chain (in reverse order of execution)
            .layer(middleware::from_fn(crate::middleware::logging::logging_middleware))
            .with_state(auth_state);

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

    /// Wrapper method for backward compatibility that doesn't require verification_state
    pub fn create_router_with_auth_and_tenant(
        &self,
        auth_state: ApiAppState,
        tenant_state: Option<TenantAppState>,
    ) -> Router {
        self.create_router_with_state(auth_state, tenant_state, None, None)
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
