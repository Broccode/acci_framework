use axum::body::Body;
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use futures::future::BoxFuture;
use redis::{self, AsyncCommands};
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::{debug, error};

use super::config::{RateLimit, RateLimitingConfig};
use super::types::{RateLimitError, create_tenant_redis_key};

/// Rate limiter implementation with Redis backend
pub struct RateStore {
    redis_client: Arc<redis::Client>,
}

impl RateStore {
    /// Create a new rate store with Redis client
    pub fn new(redis_client: Arc<redis::Client>) -> Self {
        Self { redis_client }
    }

    /// Check if the request should be rate limited
    pub async fn check_rate_limit(
        &self,
        tenant_id: &str,
        key: &str,
        rate_limit: &RateLimit,
    ) -> Result<RateLimitInfo, RateLimitError> {
        let redis_key = create_tenant_redis_key(
            tenant_id,
            &format!("ratelimit:{}s", rate_limit.window_seconds),
            key,
        );

        let now = Utc::now().timestamp() as usize;
        let window_start = now - rate_limit.window_seconds as usize;

        let mut conn = self
            .redis_client
            .get_async_connection()
            .await
            .map_err(RateLimitError::Redis)?;

        // Add the current timestamp to the list of requests
        let _: () = conn
            .zadd(&redis_key, now.to_string(), now)
            .await
            .map_err(RateLimitError::Redis)?;

        // Try to clean up old entries - handle both new and old Redis versions
        // New Redis versions use zrem_range_by_score, older use zremrangebyscore
        // Use the raw Redis command directly
        let _: Result<(), redis::RedisError> = redis::cmd("ZREMRANGEBYSCORE")
            .arg(&redis_key)
            .arg(0)
            .arg(window_start)
            .query_async(&mut conn)
            .await;

        // Count current requests in window
        let count: usize = conn
            .zcount(&redis_key, window_start, "+inf")
            .await
            .map_err(RateLimitError::Redis)?;

        // Set expiration if not already set
        let ttl: i64 = conn.ttl(&redis_key).await.map_err(RateLimitError::Redis)?;
        if ttl < 0 {
            let _: () = conn
                .expire(&redis_key, (rate_limit.window_seconds + 60) as i64)
                .await
                .map_err(RateLimitError::Redis)?;
        }

        // Get multiplier from Redis (for backoff)
        let multiplier_key = format!("{}:multiplier", redis_key);
        let multiplier: f32 = match conn.get(&multiplier_key).await {
            Ok(val) => val,
            Err(_) => 1.0,
        };

        // Calculate remaining and reset time
        let effective_limit = (rate_limit.max_requests as f32 / multiplier) as u32;
        let remaining = effective_limit.saturating_sub(count as u32);
        let limit_exceeded = count as u32 > effective_limit;

        // If limit is exceeded, increase backoff multiplier
        if limit_exceeded {
            let new_multiplier = multiplier * rate_limit.backoff_multiplier;
            let capped_multiplier = new_multiplier.min(32.0); // Cap at 32x

            let _: () = conn
                .set(&multiplier_key, capped_multiplier)
                .await
                .map_err(RateLimitError::Redis)?;

            let _: () = conn
                .expire(&multiplier_key, (rate_limit.window_seconds * 5) as i64)
                .await
                .map_err(RateLimitError::Redis)?;

            debug!(
                "Rate limit exceeded for {}, increasing backoff to {}",
                key, capped_multiplier
            );
        }

        Ok(RateLimitInfo {
            limit: effective_limit,
            remaining,
            reset: now + rate_limit.window_seconds as usize,
            window_seconds: rate_limit.window_seconds,
            limit_exceeded,
        })
    }

    /// Reset the backoff multiplier
    pub async fn reset_backoff(
        &self,
        tenant_id: &str,
        key: &str,
        window_seconds: u32,
    ) -> Result<(), RateLimitError> {
        let redis_key =
            create_tenant_redis_key(tenant_id, &format!("ratelimit:{}s", window_seconds), key);

        let multiplier_key = format!("{}:multiplier", redis_key);

        let mut conn = self
            .redis_client
            .get_async_connection()
            .await
            .map_err(RateLimitError::Redis)?;

        let _: () = conn
            .del(&multiplier_key)
            .await
            .map_err(RateLimitError::Redis)?;

        debug!("Reset backoff multiplier for {}", key);

        Ok(())
    }
}

use std::sync::Arc;

/// Information about a rate limit check
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    /// Current rate limit
    pub limit: u32,
    /// Remaining requests in the window
    pub remaining: u32,
    /// When the rate limit resets (Unix timestamp)
    pub reset: usize,
    /// Window size in seconds
    pub window_seconds: u32,
    /// Whether the limit has been exceeded
    pub limit_exceeded: bool,
}

/// Rate limiting middleware for HTTP services
pub struct RateLimitMiddleware<S> {
    inner: S,
    store: Arc<RateStore>,
    config: RateLimitingConfig,
}

impl<S> RateLimitMiddleware<S> {
    /// Create a new rate limiting middleware
    pub fn new(inner: S, store: Arc<RateStore>, config: RateLimitingConfig) -> Self {
        Self {
            inner,
            store,
            config,
        }
    }

    /// Get rate limits for a path
    fn get_limits_for_path(&self, path: &str) -> Vec<RateLimit> {
        if let Some(limits) = self.config.path_limits.get(path) {
            return limits.clone();
        }

        self.config.default_limits.clone()
    }

    /// Extract tenant ID from request
    fn extract_tenant_id<B>(&self, request: &Request<B>) -> String {
        // Extract tenant ID from headers, path, or other sources
        request
            .headers()
            .get("X-Tenant-ID")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("default")
            .to_string()
    }

    /// Extract client ID from request (IP address or API key)
    fn extract_client_id<B>(&self, request: &Request<B>) -> String {
        // Try to get from API key header
        if let Some(api_key) = request
            .headers()
            .get("X-API-Key")
            .and_then(|v| v.to_str().ok())
        {
            return format!("api:{}", api_key);
        }

        // Try to get from Authorization header
        if let Some(auth) = request
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
        {
            if auth.starts_with("Bearer ") {
                let token = &auth[7..];
                return format!("token:{}", token);
            }
        }

        // Fallback to forwarded IP or direct IP
        let ip = request
            .headers()
            .get("X-Forwarded-For")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim())
            .unwrap_or("unknown");

        format!("ip:{}", ip)
    }
}

impl<S, B> Service<Request<B>> for RateLimitMiddleware<S>
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
        if !self.config.enabled {
            let _inner = self.inner.clone();
            let mut inner_service = self.inner.clone();
            return Box::pin(async move { inner_service.call(request).await });
        }

        let path = request.uri().path().to_string();
        let tenant_id = self.extract_tenant_id(&request);
        let client_id = self.extract_client_id(&request);

        // Get limits for this request (checking tenant overrides)
        let mut limits = self.get_limits_for_path(&path);

        // Check for tenant-specific overrides
        if let Some(tenant_overrides) = self.config.tenant_overrides.get(&tenant_id) {
            if let Some(path_overrides) = tenant_overrides.get(&path) {
                limits = path_overrides.clone();
            }
        }

        let store = self.store.clone();
        let mut inner_service = self.inner.clone();

        Box::pin(async move {
            // Check each rate limit
            let mut headers = HeaderMap::new();
            let mut rate_limit_exceeded = false;
            let mut rate_limit_info: Option<RateLimitInfo> = None;

            for limit in &limits {
                match store.check_rate_limit(&tenant_id, &client_id, &limit).await {
                    Ok(info) => {
                        // Add rate limit headers for the most restrictive limit
                        if let Some(current_info) = &rate_limit_info {
                            if info.remaining < current_info.remaining {
                                rate_limit_info = Some(info.clone());
                            }
                        } else {
                            rate_limit_info = Some(info.clone());
                        }

                        if info.limit_exceeded {
                            rate_limit_exceeded = true;
                            debug!(
                                "Rate limit exceeded for tenant: {}, client: {}, path: {}, limit: {}/{} requests per {}s",
                                tenant_id,
                                client_id,
                                path,
                                info.limit,
                                info.remaining,
                                info.window_seconds
                            );
                        }
                    },
                    Err(e) => {
                        error!("Error checking rate limit: {}", e);
                        // Don't rate limit on errors
                    },
                }
            }

            // Add rate limit headers if we have info
            if let Some(ref info) = rate_limit_info {
                if let Ok(val) = HeaderValue::from_str(&info.limit.to_string()) {
                    headers.insert("X-RateLimit-Limit", val);
                }

                if let Ok(val) = HeaderValue::from_str(&info.remaining.to_string()) {
                    headers.insert("X-RateLimit-Remaining", val);
                }

                if let Ok(val) = HeaderValue::from_str(&info.reset.to_string()) {
                    headers.insert("X-RateLimit-Reset", val);
                }
            }

            // If rate limited, return 429 response
            if rate_limit_exceeded {
                let mut response = StatusCode::TOO_MANY_REQUESTS.into_response();
                let headers_mut = response.headers_mut();
                for (key, value) in headers.iter() {
                    headers_mut.insert(key, value.clone());
                }

                // Add Retry-After header
                if let Some(ref info) = rate_limit_info {
                    let now = Utc::now().timestamp() as usize;
                    let retry_after = info.reset.saturating_sub(now);
                    if let Ok(val) = HeaderValue::from_str(&retry_after.to_string()) {
                        headers_mut.insert("Retry-After", val);
                    }
                }

                return Ok(response);
            }

            // Continue to the service
            let mut response = inner_service.call(request).await?;

            // Add rate limit headers to the response
            let headers_mut = response.headers_mut();
            for (key, value) in headers.iter() {
                headers_mut.insert(key, value.clone());
            }

            Ok(response)
        })
    }
}

/// Layer that applies the rate limiting middleware
pub struct RateLimitLayer {
    store: Arc<RateStore>,
    config: RateLimitingConfig,
}

impl RateLimitLayer {
    /// Create a new rate limiting layer
    pub fn new(store: Arc<RateStore>, config: RateLimitingConfig) -> Self {
        Self { store, config }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        RateLimitMiddleware::new(service, self.store.clone(), self.config.clone())
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_check_rate_limit() {
        // This would use a mock Redis client
        // For now, these are placeholders
    }

    #[tokio::test]
    async fn test_reset_backoff() {
        // Test with mock Redis client
    }

    #[tokio::test]
    async fn test_middleware() {
        // Test middleware behavior
    }
}
