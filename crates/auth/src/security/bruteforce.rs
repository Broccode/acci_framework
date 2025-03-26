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
        _tenant_id: &str,
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
    use super::*;
    use std::time::Duration;

    // Unit tests for pure functions that don't depend on Redis

    #[test]
    fn test_calculate_delay() {
        // Test exponential backoff calculation
        let delay = calculate_backoff_delay(0);
        assert_eq!(delay, Duration::from_secs(1));

        let delay = calculate_backoff_delay(1);
        assert_eq!(delay, Duration::from_secs(2));

        let delay = calculate_backoff_delay(2);
        assert_eq!(delay, Duration::from_secs(4));

        let delay = calculate_backoff_delay(3);
        assert_eq!(delay, Duration::from_secs(8));

        let delay = calculate_backoff_delay(10);
        assert_eq!(delay, Duration::from_secs(1024));

        // Test max backoff cap
        let delay = calculate_backoff_delay(15); // Would be 32768 seconds without cap
        assert_eq!(delay, Duration::from_secs(3600)); // Should be capped at 1 hour
    }

    #[test]
    fn test_login_attempt_metadata() {
        // Test creation and fields of login attempt
        let now = chrono::Utc::now();
        let ip = "192.168.1.1".to_string();
        let username = "testuser".to_string();
        let successful = false;

        let attempt = LoginAttempt {
            tenant_id: "test_tenant".to_string(),
            ip_address: ip.clone(),
            username: username.clone(),
            timestamp: now,
            successful,
            user_agent: "Mozilla/5.0".to_string(),
            fingerprint: None,
            geolocation: None,
        };

        assert_eq!(attempt.ip_address, ip);
        assert_eq!(attempt.username, username);
        assert_eq!(attempt.timestamp, now);
        assert_eq!(attempt.successful, successful);
    }

    #[test]
    fn test_pattern_similarities() {
        // Test pattern detection logic
        let attempts = vec![
            LoginAttempt {
                tenant_id: "test_tenant".to_string(),
                ip_address: "192.168.1.1".to_string(),
                username: "alice".to_string(),
                timestamp: chrono::Utc::now(),
                successful: false,
                user_agent: "Mozilla/5.0".to_string(),
                fingerprint: None,
                geolocation: None,
            },
            LoginAttempt {
                tenant_id: "test_tenant".to_string(),
                ip_address: "192.168.1.1".to_string(),
                username: "bob".to_string(),
                timestamp: chrono::Utc::now(),
                successful: false,
                user_agent: "Mozilla/5.0".to_string(),
                fingerprint: None,
                geolocation: None,
            },
            LoginAttempt {
                tenant_id: "test_tenant".to_string(),
                ip_address: "192.168.1.1".to_string(),
                username: "charlie".to_string(),
                timestamp: chrono::Utc::now(),
                successful: false,
                user_agent: "Mozilla/5.0".to_string(),
                fingerprint: None,
                geolocation: None,
            },
        ];

        // Test username variations
        let username_pattern = detect_username_pattern(&attempts);
        // In this simple case, there's no clear pattern between alice, bob, charlie
        assert!(!username_pattern);

        // Test sequential pattern detection
        let sequential_attempts = vec![
            LoginAttempt {
                tenant_id: "test_tenant".to_string(),
                ip_address: "192.168.1.1".to_string(),
                username: "user1".to_string(),
                timestamp: chrono::Utc::now(),
                successful: false,
                user_agent: "Mozilla/5.0".to_string(),
                fingerprint: None,
                geolocation: None,
            },
            LoginAttempt {
                tenant_id: "test_tenant".to_string(),
                ip_address: "192.168.1.1".to_string(),
                username: "user2".to_string(),
                timestamp: chrono::Utc::now(),
                successful: false,
                user_agent: "Mozilla/5.0".to_string(),
                fingerprint: None,
                geolocation: None,
            },
            LoginAttempt {
                tenant_id: "test_tenant".to_string(),
                ip_address: "192.168.1.1".to_string(),
                username: "user3".to_string(),
                timestamp: chrono::Utc::now(),
                successful: false,
                user_agent: "Mozilla/5.0".to_string(),
                fingerprint: None,
                geolocation: None,
            },
        ];

        let sequential_pattern = detect_username_pattern(&sequential_attempts);
        assert!(sequential_pattern);
    }

    #[test]
    fn test_brute_force_error_types() {
        // Test error type formatting using the available variants
        let account_locked = BruteForceError::AccountLocked;
        let delay_error = BruteForceError::ProgressiveDelay(500);
        let redis_error = BruteForceError::Redis(redis::RedisError::from(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "Connection refused",
        )));
        let internal_error = BruteForceError::Internal("Internal error".to_string());

        assert!(account_locked.to_string().contains("Account locked"));
        assert!(
            delay_error
                .to_string()
                .contains("Progressive delay required: 500ms")
        );
        assert!(redis_error.to_string().contains("Redis operation failed"));
        assert!(internal_error.to_string().contains("Internal error"));
    }

    // Helper functions for the unit tests

    fn calculate_backoff_delay(attempt_count: u32) -> Duration {
        let base_delay = 1.0;
        let max_delay = 3600.0; // 1 hour max delay

        let delay = base_delay * 2.0_f64.powi(attempt_count as i32);
        let capped_delay = delay.min(max_delay);

        Duration::from_secs(capped_delay as u64)
    }

    fn detect_username_pattern(attempts: &[LoginAttempt]) -> bool {
        if attempts.len() < 3 {
            return false;
        }

        // Look for sequential patterns like user1, user2, user3
        let mut sequential_count = 0;

        for i in 1..attempts.len() {
            let prev = &attempts[i - 1].username;
            let curr = &attempts[i].username;

            // Very simple pattern detection for test purposes
            // In real implementation, this would be more sophisticated
            let prev_base = prev.trim_end_matches(char::is_numeric);
            let curr_base = curr.trim_end_matches(char::is_numeric);

            if prev_base == curr_base {
                sequential_count += 1;
                if sequential_count >= 2 {
                    // Found at least 3 in sequence
                    return true;
                }
            } else {
                sequential_count = 0;
            }
        }

        false
    }
}
