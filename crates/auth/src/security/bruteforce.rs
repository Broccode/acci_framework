use chrono::{DateTime, Utc};
use redis::{self, AsyncCommands};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tracing::{debug, warn};

use super::config::BruteForceConfig;
use super::types::{BruteForceError, LoginAttempt, create_tenant_redis_key};

/// Implements brute force protection using Redis-backed storage
pub struct BruteForceProtection {
    redis_client: Arc<redis::Client>,
    config: BruteForceConfig,
}

impl BruteForceProtection {
    /// Create a new brute force protection instance
    pub fn new(redis_client: Arc<redis::Client>, config: BruteForceConfig) -> Self {
        Self {
            redis_client,
            config,
        }
    }

    /// Records a failed authentication attempt
    pub async fn record_attempt(&self, tenant_id: &str, key: &str) -> Result<(), BruteForceError> {
        if !self.config.enabled {
            debug!("Brute force protection disabled, skipping attempt recording");
            return Ok(());
        }

        let redis_key = create_tenant_redis_key(tenant_id, "bruteforce", key);
        let now = Utc::now();

        let mut conn = self
            .redis_client
            .get_async_connection()
            .await
            .map_err(BruteForceError::Redis)?;

        // Store attempt timestamp
        let _: () = conn
            .rpush(&redis_key, now.timestamp().to_string())
            .await
            .map_err(BruteForceError::Redis)?;

        // Set expiration if not already set (for automatic cleanup)
        let ttl: i64 = conn.ttl(&redis_key).await.map_err(BruteForceError::Redis)?;
        if ttl < 0 {
            let expiry = (self.config.window_seconds + 60) as i64; // Add a minute buffer
            let _: () = conn
                .expire(&redis_key, expiry)
                .await
                .map_err(BruteForceError::Redis)?;
        }

        let count: usize = conn
            .llen(&redis_key)
            .await
            .map_err(BruteForceError::Redis)?;
        debug!("Recorded failed attempt for {}: {} attempts", key, count);

        Ok(())
    }

    /// Calculates the delay that should be applied before processing the request
    pub async fn calculate_delay(
        &self,
        tenant_id: &str,
        key: &str,
    ) -> Result<StdDuration, BruteForceError> {
        if !self.config.enabled {
            return Ok(StdDuration::from_millis(0));
        }

        let redis_key = create_tenant_redis_key(tenant_id, "bruteforce", key);
        let mut conn = self
            .redis_client
            .get_async_connection()
            .await
            .map_err(BruteForceError::Redis)?;

        // Get count of attempts within the window
        let attempts: Vec<String> = conn
            .lrange(&redis_key, 0, -1)
            .await
            .map_err(BruteForceError::Redis)?;

        let now = Utc::now();
        let window_start = now - chrono::Duration::seconds(self.config.window_seconds as i64);

        // Filter attempts within window
        let recent_attempts = attempts
            .iter()
            .filter_map(|ts_str| ts_str.parse::<i64>().ok())
            .filter(|&ts| ts >= window_start.timestamp())
            .count();

        if recent_attempts == 0 {
            return Ok(StdDuration::from_millis(0));
        }

        // Calculate exponential delay with cap
        let base_delay = self.config.base_delay_ms;
        let max_delay = self.config.max_delay_ms;

        let exp = recent_attempts.saturating_sub(1) as u32; // First attempt has no delay
        let delay = base_delay * (2_u32.saturating_pow(exp.min(16))); // Prevent overflow with min
        let delay = delay.min(max_delay);

        debug!(
            "Calculated delay for {}: {}ms ({} attempts)",
            key, delay, recent_attempts
        );

        Ok(StdDuration::from_millis(delay as u64))
    }

    /// Check if account is locked due to too many failed attempts
    pub async fn is_account_locked(
        &self,
        tenant_id: &str,
        key: &str,
    ) -> Result<bool, BruteForceError> {
        if !self.config.enabled {
            return Ok(false);
        }

        let redis_key = create_tenant_redis_key(tenant_id, "bruteforce", key);
        let mut conn = self
            .redis_client
            .get_async_connection()
            .await
            .map_err(BruteForceError::Redis)?;

        // Get count of attempts within the window
        let attempts: Vec<String> = conn
            .lrange(&redis_key, 0, -1)
            .await
            .map_err(BruteForceError::Redis)?;

        let now = Utc::now();
        let window_start = now - chrono::Duration::seconds(self.config.window_seconds as i64);

        // Filter attempts within window
        let recent_attempts = attempts
            .iter()
            .filter_map(|ts_str| ts_str.parse::<i64>().ok())
            .filter(|&ts| ts >= window_start.timestamp())
            .count();

        let is_locked = recent_attempts >= self.config.max_attempts as usize;

        if is_locked {
            warn!("Account locked due to too many failed attempts: {}", key);
        }

        Ok(is_locked)
    }

    /// Get remaining attempts before lockout
    pub async fn remaining_attempts(
        &self,
        tenant_id: &str,
        key: &str,
    ) -> Result<u32, BruteForceError> {
        if !self.config.enabled {
            return Ok(self.config.max_attempts);
        }

        let redis_key = create_tenant_redis_key(tenant_id, "bruteforce", key);
        let mut conn = self
            .redis_client
            .get_async_connection()
            .await
            .map_err(BruteForceError::Redis)?;

        // Get count of attempts within the window
        let attempts: Vec<String> = conn
            .lrange(&redis_key, 0, -1)
            .await
            .map_err(BruteForceError::Redis)?;

        let now = Utc::now();
        let window_start = now - chrono::Duration::seconds(self.config.window_seconds as i64);

        // Filter attempts within window
        let recent_attempts = attempts
            .iter()
            .filter_map(|ts_str| ts_str.parse::<i64>().ok())
            .filter(|&ts| ts >= window_start.timestamp())
            .count();

        let remaining = self
            .config
            .max_attempts
            .saturating_sub(recent_attempts as u32);
        debug!("Remaining attempts for {}: {}", key, remaining);

        Ok(remaining)
    }

    /// Reset failed attempts after successful authentication
    pub async fn reset_attempts(&self, tenant_id: &str, key: &str) -> Result<(), BruteForceError> {
        if !self.config.enabled {
            return Ok(());
        }

        let redis_key = create_tenant_redis_key(tenant_id, "bruteforce", key);
        let mut conn = self
            .redis_client
            .get_async_connection()
            .await
            .map_err(BruteForceError::Redis)?;

        let _: () = conn.del(&redis_key).await.map_err(BruteForceError::Redis)?;
        debug!("Reset attempts for {}", key);

        Ok(())
    }

    /// Check authentication attempt for brute force protection
    pub async fn check_authentication_attempt(
        &self,
        tenant_id: &str,
        key: &str,
        successful: bool,
    ) -> Result<(), BruteForceError> {
        if !self.config.enabled {
            return Ok(());
        }

        if successful {
            // On successful login, reset counters
            self.reset_attempts(tenant_id, key).await?;
            return Ok(());
        }

        // Record the failed attempt
        self.record_attempt(tenant_id, key).await?;

        // Check if account is locked
        if self.is_account_locked(tenant_id, key).await? {
            return Err(BruteForceError::AccountLocked);
        }

        // Calculate the appropriate delay
        let delay = self.calculate_delay(tenant_id, key).await?;
        if !delay.is_zero() {
            // Asynchronously wait for the delay duration
            tokio::time::sleep(delay).await;
        }

        Ok(())
    }
}

/// Pattern detector for more sophisticated brute force detection
pub struct PatternDetector {
    redis_client: Arc<redis::Client>,
}

impl PatternDetector {
    /// Create a new pattern detector
    pub fn new(redis_client: Arc<redis::Client>) -> Self {
        Self { redis_client }
    }

    /// Analyze login attempts for suspicious patterns
    pub async fn analyze_login_attempts(
        &self,
        tenant_id: &str,
        attempts: &[LoginAttempt],
    ) -> Result<bool, BruteForceError> {
        // Advanced pattern detection would go here
        // This is a simplified placeholder implementation

        if attempts.len() < 3 {
            return Ok(false);
        }

        // Check timing patterns (too regular timing between attempts)
        let mut timestamps: Vec<DateTime<Utc>> = attempts.iter().map(|a| a.timestamp).collect();
        timestamps.sort();

        let mut suspicious = false;

        // Check for suspiciously regular intervals
        if timestamps.len() >= 3 {
            let time_diffs: Vec<i64> = timestamps
                .windows(2)
                .map(|w| (w[1] - w[0]).num_milliseconds())
                .collect();

            let avg_diff: i64 = time_diffs.iter().sum::<i64>() / time_diffs.len() as i64;
            let variance: i64 = time_diffs
                .iter()
                .map(|d| (d - avg_diff).pow(2))
                .sum::<i64>()
                / time_diffs.len() as i64;

            // If variance is very low, timing is suspiciously regular
            if variance < 500 * 500 {
                // 500ms variance threshold
                suspicious = true;
            }
        }

        // Check for username pattern (sequential attempts with similar usernames)
        if attempts.len() >= 5 {
            let usernames: Vec<&str> = attempts.iter().map(|a| a.username.as_str()).collect();

            // Check for sequential patterns in usernames
            // This is a simplified check
            let mut username_pattern = false;
            for usernames in usernames.windows(2) {
                if usernames[0].len() == usernames[1].len() {
                    let diff_count = usernames[0]
                        .chars()
                        .zip(usernames[1].chars())
                        .filter(|(a, b)| a != b)
                        .count();

                    if diff_count <= 2 {
                        username_pattern = true;
                        break;
                    }
                }
            }

            if username_pattern {
                suspicious = true;
            }
        }

        Ok(suspicious)
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_record_attempt() {
        // This test would use a mock Redis client in actual implementation
        // For now, this is a placeholder test structure
    }

    #[tokio::test]
    async fn test_calculate_delay() {
        // Test with mock Redis client
    }

    #[tokio::test]
    async fn test_is_account_locked() {
        // Test with mock Redis client
    }

    #[tokio::test]
    async fn test_pattern_detection() {
        // Test pattern detection
    }
}
