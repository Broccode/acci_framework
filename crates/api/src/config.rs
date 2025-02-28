use std::time::Duration;

/// Configuration for the API infrastructure
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Base path for the API
    pub base_path: String,
    /// CORS configuration
    pub cors: CorsConfig,
    /// Rate limiting configuration
    pub rate_limit: RateLimitConfig,
    /// Request timeout configuration
    pub timeout: TimeoutConfig,
    /// Maximum request body size in bytes
    pub body_limit: usize,
    /// API documentation configuration
    pub documentation: DocumentationConfig,
}

/// Default configuration for the API
impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_path: "/api/v1".to_string(),
            cors: CorsConfig::default(),
            rate_limit: RateLimitConfig::default(),
            timeout: TimeoutConfig::default(),
            body_limit: 5 * 1024 * 1024, // 5MB
            documentation: DocumentationConfig::default(),
        }
    }
}

/// CORS configuration
#[derive(Debug, Clone)]
pub struct CorsConfig {
    /// Allowed origins, empty means all origins
    pub allowed_origins: Vec<String>,
    /// Allowed HTTP methods
    pub allowed_methods: Vec<String>,
    /// Allowed HTTP headers
    pub allowed_headers: Vec<String>,
    /// Headers exposed to the client
    pub expose_headers: Vec<String>,
    /// Maximum cache duration for preflight requests
    pub max_age: Duration,
    /// Whether credentials are allowed
    pub allow_credentials: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec![],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec![
                "Authorization".to_string(),
                "Content-Type".to_string(),
                "X-Requested-With".to_string(),
            ],
            expose_headers: vec!["Content-Length".to_string(), "X-Request-ID".to_string()],
            max_age: Duration::from_secs(86400), // 24 hours
            allow_credentials: true,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed per minute
    pub requests_per_minute: u32,
    /// Maximum burst size for the token bucket algorithm
    pub burst_size: u32,
    /// Whether rate limiting is enabled
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            burst_size: 5,
            enabled: true,
        }
    }
}

/// Request timeout configuration
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// Maximum duration of a request
    pub request_timeout: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(30),
        }
    }
}

/// API documentation configuration
#[derive(Debug, Clone)]
pub struct DocumentationConfig {
    /// Whether documentation is enabled
    pub enabled: bool,
    /// Path to the documentation UI
    pub path: String,
    /// Whether authentication is required to access the documentation
    pub require_authentication: bool,
}

impl Default for DocumentationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: "/swagger-ui".to_string(),
            require_authentication: false,
        }
    }
}
