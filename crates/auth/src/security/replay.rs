use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use futures::future::BoxFuture;
use hex;
use rand::Rng;
use redis::{self, AsyncCommands};
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::{debug, error, warn};

use super::config::ReplayProtectionConfig;
use super::types::create_tenant_redis_key;

/// Store for managing nonces to prevent replay attacks
pub struct NonceStore {
    redis_client: Arc<redis::Client>,
    config: ReplayProtectionConfig,
}

impl NonceStore {
    /// Create a new nonce store
    pub fn new(redis_client: Arc<redis::Client>, config: ReplayProtectionConfig) -> Self {
        Self {
            redis_client,
            config,
        }
    }

    /// Generate a new nonce with expiration
    pub async fn generate_nonce(
        &self,
        tenant_id: &str,
        context: &str,
    ) -> Result<String, anyhow::Error> {
        if !self.config.enabled {
            // Return a dummy nonce when protection is disabled
            return Ok("disabled".to_string());
        }

        // Generate random bytes for the nonce
        let mut rng = rand::rng();
        let mut nonce_bytes = [0u8; 16];
        rng.fill(&mut nonce_bytes);

        // Convert to hex string
        let nonce = hex::encode(nonce_bytes);

        // Store in Redis with expiration
        let mut conn = self.redis_client.get_async_connection().await?;
        let redis_key =
            create_tenant_redis_key(tenant_id, "nonce", &format!("{}:{}", context, nonce));

        // Store the current timestamp with the nonce
        let now = Utc::now().timestamp();
        let _: () = conn.set(&redis_key, now.to_string()).await?;
        let _: () = conn
            .expire(&redis_key, self.config.nonce_expiration_seconds as i64)
            .await?;

        debug!(
            "Generated nonce for tenant {}, context {}: {}",
            tenant_id, context, nonce
        );

        Ok(nonce)
    }

    /// Validate and consume a nonce
    pub async fn validate_nonce(
        &self,
        tenant_id: &str,
        context: &str,
        nonce: &str,
        timestamp: Option<i64>,
    ) -> Result<bool, anyhow::Error> {
        if !self.config.enabled {
            // Bypass validation when protection is disabled
            return Ok(true);
        }

        // Get from Redis
        let mut conn = self.redis_client.get_async_connection().await?;
        let redis_key =
            create_tenant_redis_key(tenant_id, "nonce", &format!("{}:{}", context, nonce));

        let stored_timestamp: Option<String> = conn.get(&redis_key).await?;

        if let Some(ts_str) = stored_timestamp {
            // Delete the nonce to prevent reuse
            let _: () = conn.del(&redis_key).await?;

            // If timestamp validation is enabled, check the timestamp
            if self.config.timestamp_validation {
                if let Some(request_ts) = timestamp {
                    let stored_ts = ts_str.parse::<i64>().unwrap_or(0);
                    let now = Utc::now().timestamp();

                    // Check if request timestamp is within acceptable range
                    let max_skew = self.config.max_timestamp_skew_seconds as i64;
                    let ts_diff = (request_ts - stored_ts).abs();

                    if ts_diff > max_skew {
                        warn!(
                            "Timestamp skew too large: {}s (max: {}s)",
                            ts_diff, max_skew
                        );
                        return Ok(false);
                    }

                    // Check if nonce is too old
                    let age = now - stored_ts;
                    if age > self.config.nonce_expiration_seconds as i64 {
                        warn!(
                            "Nonce expired: {}s old (max: {}s)",
                            age, self.config.nonce_expiration_seconds
                        );
                        return Ok(false);
                    }
                }
            }

            debug!(
                "Validated nonce for tenant {}, context {}: {}",
                tenant_id, context, nonce
            );
            return Ok(true);
        }

        warn!(
            "Invalid nonce for tenant {}, context {}: {}",
            tenant_id, context, nonce
        );
        Ok(false)
    }

    /// Generate a CSRF token for a form
    pub async fn generate_csrf_token(
        &self,
        tenant_id: &str,
        form_id: &str,
    ) -> Result<String, anyhow::Error> {
        // CSRF tokens are just nonces with a specific context
        self.generate_nonce(tenant_id, &format!("csrf:{}", form_id))
            .await
    }

    /// Validate a CSRF token
    pub async fn validate_csrf_token(
        &self,
        tenant_id: &str,
        form_id: &str,
        token: &str,
    ) -> Result<bool, anyhow::Error> {
        self.validate_nonce(tenant_id, &format!("csrf:{}", form_id), token, None)
            .await
    }
}

/// Middleware for preventing replay attacks
pub struct ReplayProtectionMiddleware<S> {
    inner: S,
    nonce_store: Arc<NonceStore>,
}

impl<S> ReplayProtectionMiddleware<S> {
    /// Create a new replay protection middleware
    pub fn new(inner: S, nonce_store: Arc<NonceStore>) -> Self {
        Self { inner, nonce_store }
    }

    /// Extract tenant ID from request
    fn extract_tenant_id<B>(&self, request: &Request<B>) -> String {
        // Extract tenant ID from headers or other sources
        request
            .headers()
            .get("X-Tenant-ID")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("default")
            .to_string()
    }

    /// Extract nonce from request
    fn extract_nonce<B>(&self, request: &Request<B>) -> Option<String> {
        // Try to get from header
        if let Some(nonce) = request
            .headers()
            .get("X-Nonce")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
        {
            return Some(nonce);
        }

        // Try to get from query params
        if let Some(uri) = request.uri().query() {
            for param in uri.split('&') {
                if let Some(nonce) = param.strip_prefix("nonce=") {
                    return Some(nonce.to_string());
                }
            }
        }

        None
    }

    /// Extract timestamp from request
    fn extract_timestamp<B>(&self, request: &Request<B>) -> Option<i64> {
        // Try to get from header
        if let Some(ts_str) = request
            .headers()
            .get("X-Timestamp")
            .and_then(|v| v.to_str().ok())
        {
            return ts_str.parse::<i64>().ok();
        }

        // Try to get from query params
        if let Some(uri) = request.uri().query() {
            for param in uri.split('&') {
                if let Some(ts_str) = param.strip_prefix("timestamp=") {
                    return ts_str.parse::<i64>().ok();
                }
            }
        }

        None
    }

    /// Extract context from request (e.g., endpoint path)
    fn extract_context<B>(&self, request: &Request<B>) -> String {
        let method = request.method().as_str();
        let path = request.uri().path();

        format!("{}:{}", method, path)
    }

    /// Check if the request requires replay protection
    fn requires_protection<B>(&self, request: &Request<B>) -> bool {
        // Only protect certain methods
        let method = request.method().as_str();
        match method {
            "GET" => false,
            "HEAD" => false,
            "OPTIONS" => false,
            _ => true,
        }
    }
}

impl<S, B> Service<Request<B>> for ReplayProtectionMiddleware<S>
where
    S: Service<Request<B>, Response = Response<Body>> + Clone + Send + Sync + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<B>) -> Self::Future {
        // Skip protection for safe methods
        if !self.requires_protection(&request) {
            let mut inner_service = self.inner.clone();
            return Box::pin(async move { inner_service.call(request).await });
        }

        let tenant_id = self.extract_tenant_id(&request);
        let context = self.extract_context(&request);
        let nonce = self.extract_nonce(&request);
        let timestamp = self.extract_timestamp(&request);

        let nonce_store = self.nonce_store.clone();
        let mut inner_service = self.inner.clone();

        Box::pin(async move {
            // Validate the nonce
            if let Some(nonce_val) = nonce {
                match nonce_store
                    .validate_nonce(&tenant_id, &context, &nonce_val, timestamp)
                    .await
                {
                    Ok(valid) => {
                        if !valid {
                            let response = StatusCode::BAD_REQUEST.into_response();
                            return Ok(response);
                        }
                    },
                    Err(e) => {
                        error!("Error validating nonce: {}", e);
                        let response = StatusCode::INTERNAL_SERVER_ERROR.into_response();
                        return Ok(response);
                    },
                }
            } else {
                let response = StatusCode::BAD_REQUEST.into_response();
                return Ok(response);
            }

            // Pass through to the inner service
            inner_service.call(request).await
        })
    }
}

/// Layer that applies the replay protection middleware
pub struct ReplayProtectionLayer {
    nonce_store: Arc<NonceStore>,
}

impl ReplayProtectionLayer {
    /// Create a new replay protection layer
    pub fn new(nonce_store: Arc<NonceStore>) -> Self {
        Self { nonce_store }
    }
}

impl<S> Layer<S> for ReplayProtectionLayer {
    type Service = ReplayProtectionMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        ReplayProtectionMiddleware::new(service, self.nonce_store.clone())
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_generate_and_validate_nonce() {
        // This would use a mock Redis client in an actual test
    }

    #[tokio::test]
    async fn test_csrf_token() {
        // This would use a mock Redis client in an actual test
    }
}
