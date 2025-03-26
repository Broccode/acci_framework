use chrono::{Duration, Utc};
use redis::{self, AsyncCommands};
use std::sync::Arc;
use tracing::{error, info};

use super::config::CredentialStuffingConfig;
use super::types::{
    CaptchaChallenge, CaptchaType, Challenge, LoginAttempt, RiskLevel, create_tenant_redis_key,
};

/// Detects and mitigates credential stuffing attacks
pub struct CredentialStuffingProtection {
    pattern_detector: Arc<PatternDetector>,
    challenge_provider: Arc<ChallengeProvider>,
    config: CredentialStuffingConfig,
}

impl CredentialStuffingProtection {
    /// Create a new credential stuffing protection system
    pub fn new(
        pattern_detector: Arc<PatternDetector>,
        challenge_provider: Arc<ChallengeProvider>,
        config: CredentialStuffingConfig,
    ) -> Self {
        Self {
            pattern_detector,
            challenge_provider,
            config,
        }
    }

    /// Analyze login attempt and determine risk level
    pub async fn analyze_login_attempt(&self, attempt: &LoginAttempt) -> RiskLevel {
        if !self.config.enabled {
            return RiskLevel::Low;
        }

        // Check IP velocity
        let ip_velocity = self
            .pattern_detector
            .check_ip_velocity(
                &attempt.tenant_id,
                &attempt.ip_address,
                self.config.velocity_window_seconds,
            )
            .await;

        // Check username patterns if configured
        let suspicious_username_pattern = if self.config.check_username_patterns {
            self.pattern_detector
                .check_username_pattern(&attempt.tenant_id, &attempt.username)
                .await
        } else {
            false
        };

        // Get recent attempts for pattern analysis
        let recent_attempts = self
            .pattern_detector
            .get_recent_attempts(
                &attempt.tenant_id,
                &attempt.ip_address,
                self.config.velocity_window_seconds,
            )
            .await;

        // Calculate overall risk level
        let mut risk_level = RiskLevel::Low;

        // Velocity-based risk assessment
        if ip_velocity > self.config.max_velocity * 2 {
            risk_level = risk_level.max(RiskLevel::Critical);
        } else if ip_velocity > self.config.max_velocity {
            risk_level = risk_level.max(RiskLevel::High);
        } else if ip_velocity > self.config.max_velocity / 2 {
            risk_level = risk_level.max(RiskLevel::Medium);
        }

        // Pattern-based risk assessment
        if suspicious_username_pattern {
            risk_level = risk_level.max(RiskLevel::High);
        }

        // Check for automation signs in user agent
        if attempt.user_agent.contains("bot")
            || attempt.user_agent.contains("curl")
            || attempt.user_agent.contains("python")
            || attempt.user_agent.len() < 20
        {
            risk_level = risk_level.max(RiskLevel::Medium);
        }

        // Log suspicious activity
        if risk_level > RiskLevel::Low {
            info!(
                "Suspicious login attempt detected: IP: {}, Risk: {:?}, Velocity: {}",
                attempt.ip_address, risk_level, ip_velocity
            );
        }

        risk_level
    }

    /// Get appropriate challenge based on risk level
    pub async fn get_challenge(&self, attempt: &LoginAttempt, risk_level: RiskLevel) -> Challenge {
        match risk_level {
            RiskLevel::Low => Challenge::None,
            RiskLevel::Medium => {
                if self.config.enable_captcha {
                    self.challenge_provider.get_captcha_challenge().await
                } else {
                    Challenge::Delay(500) // 500ms delay
                }
            },
            RiskLevel::High => {
                if self.config.enable_captcha {
                    self.challenge_provider.get_captcha_challenge().await
                } else {
                    Challenge::MfaRequired
                }
            },
            RiskLevel::Critical => {
                if self.config.enable_ip_blocking {
                    Challenge::IpBlock(Duration::minutes(self.config.ip_block_minutes as i64))
                } else {
                    Challenge::MfaRequired
                }
            },
        }
    }

    /// Handle a login attempt
    pub async fn handle_login_attempt(&self, attempt: &LoginAttempt) -> Challenge {
        // Store attempt for future analysis
        self.pattern_detector.record_login_attempt(attempt).await;

        // Analyze attempt and determine risk level
        let risk_level = self.analyze_login_attempt(attempt).await;

        // Get appropriate challenge based on risk level
        self.get_challenge(attempt, risk_level).await
    }
}

/// Detects patterns indicative of credential stuffing
pub struct PatternDetector {
    redis_client: Arc<redis::Client>,
}

impl PatternDetector {
    /// Create a new pattern detector
    pub fn new(redis_client: Arc<redis::Client>) -> Self {
        Self { redis_client }
    }

    /// Record a login attempt for future analysis
    pub async fn record_login_attempt(&self, attempt: &LoginAttempt) {
        let mut conn = match self.redis_client.get_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                return;
            },
        };

        // Store IP attempts
        let ip_key =
            create_tenant_redis_key(&attempt.tenant_id, "credstuffing:ip", &attempt.ip_address);

        // Store serialized attempt
        if let Ok(json) = serde_json::to_string(attempt) {
            let _: Result<(), _> = conn.lpush(&ip_key, json).await;
            let _: Result<(), _> = conn.ltrim(&ip_key, 0, 99).await; // Keep last 100 attempts
            let _: Result<(), _> = conn.expire(&ip_key, 86400).await; // Expire after 1 day
        }

        // Store username attempt count
        let username_key = create_tenant_redis_key(
            &attempt.tenant_id,
            "credstuffing:username",
            &attempt.username,
        );
        let _: Result<(), _> = conn.incr(&username_key, 1).await;
        let _: Result<(), _> = conn.expire(&username_key, 86400).await; // Expire after 1 day

        // Store timestamp for velocity checking
        let now = Utc::now().timestamp() as usize;
        let velocity_key = create_tenant_redis_key(
            &attempt.tenant_id,
            "credstuffing:velocity",
            &attempt.ip_address,
        );
        let _: Result<(), _> = conn.zadd(&velocity_key, now.to_string(), now).await;
        let _: Result<(), _> = conn.expire(&velocity_key, 3600).await; // Expire after 1 hour
    }

    /// Check IP velocity (number of attempts per time window)
    pub async fn check_ip_velocity(
        &self,
        tenant_id: &str,
        ip_address: &str,
        window_seconds: u32,
    ) -> u32 {
        let mut conn = match self.redis_client.get_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                return 0;
            },
        };

        let velocity_key = create_tenant_redis_key(tenant_id, "credstuffing:velocity", ip_address);
        let now = Utc::now().timestamp() as usize;
        let window_start = now - window_seconds as usize;

        // Clean up old entries using raw Redis command for compatibility
        let _: Result<(), _> = redis::cmd("ZREMRANGEBYSCORE")
            .arg(&velocity_key)
            .arg(0)
            .arg(window_start)
            .query_async(&mut conn)
            .await;

        // Count current entries in window
        match conn
            .zcount::<_, _, _, usize>(&velocity_key, window_start, "+inf")
            .await
        {
            Ok(count) => count as u32,
            Err(e) => {
                error!("Failed to count IP velocity: {}", e);
                0
            },
        }
    }

    /// Check for suspicious username patterns
    pub async fn check_username_pattern(&self, tenant_id: &str, username: &str) -> bool {
        let mut conn = match self.redis_client.get_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                return false;
            },
        };

        // Get all usernames attempted from this tenant in the last day
        let pattern_key = create_tenant_redis_key(tenant_id, "credstuffing:usernames", "all");

        // Add this username to the set
        let _: Result<bool, _> = conn.sadd(&pattern_key, username).await;
        let _: Result<(), _> = conn.expire(&pattern_key, 86400).await; // Expire after 1 day

        // Get all usernames
        let usernames: Vec<String> = match conn.smembers(&pattern_key).await {
            Ok(members) => members,
            Err(_) => return false,
        };

        // Check for sequential patterns (e.g., user1, user2, user3)
        if usernames.len() > 5 {
            // Simple check for username with same prefix but different suffix numbers
            let username_base = username.trim_end_matches(|c: char| c.is_numeric());
            let mut sequential_count = 0;

            for other in &usernames {
                let other_base = other.trim_end_matches(|c: char| c.is_numeric());
                if other_base == username_base {
                    sequential_count += 1;
                }
            }

            if sequential_count >= 3 {
                return true;
            }
        }

        // Check for similar usernames (edit distance)
        if usernames.len() > 10 {
            let mut similar_count = 0;

            for other in &usernames {
                if other != username && calculate_similarity(username, other) > 0.8 {
                    similar_count += 1;
                }
            }

            if similar_count >= 3 {
                return true;
            }
        }

        false
    }

    /// Get recent login attempts for pattern analysis
    pub async fn get_recent_attempts(
        &self,
        tenant_id: &str,
        ip_address: &str,
        window_seconds: u32,
    ) -> Vec<LoginAttempt> {
        let mut conn = match self.redis_client.get_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                return Vec::new();
            },
        };

        let ip_key = create_tenant_redis_key(tenant_id, "credstuffing:ip", ip_address);
        let raw_attempts: Vec<String> = match conn.lrange(&ip_key, 0, 20).await {
            Ok(attempts) => attempts,
            Err(_) => return Vec::new(),
        };

        let mut attempts = Vec::new();
        let now = Utc::now();
        let window_start = now - chrono::Duration::seconds(window_seconds as i64);

        for raw in raw_attempts {
            if let Ok(attempt) = serde_json::from_str::<LoginAttempt>(&raw) {
                if attempt.timestamp >= window_start {
                    attempts.push(attempt);
                }
            }
        }

        attempts
    }
}

/// Provides challenges for suspicious login attempts
pub struct ChallengeProvider {
    // Placeholder for integrations with CAPTCHA providers, etc.
}

impl ChallengeProvider {
    /// Create a new challenge provider
    pub fn new() -> Self {
        Self {}
    }

    /// Get a CAPTCHA challenge
    pub async fn get_captcha_challenge(&self) -> Challenge {
        // In a real implementation, this would integrate with a CAPTCHA provider
        let challenge_id = format!("chid_{}", chrono::Utc::now().timestamp());

        Challenge::Captcha(CaptchaChallenge {
            challenge_id,
            challenge_data: "What is 2+2?".to_string(),
            captcha_type: CaptchaType::Text,
        })
    }
}

/// Calculate similarity between two strings (0.0 to 1.0)
fn calculate_similarity(s1: &str, s2: &str) -> f64 {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    if len1 == 0 || len2 == 0 {
        return if len1 == len2 { 1.0 } else { 0.0 };
    }

    let distance = levenshtein_distance(s1, s2);
    let max_len = len1.max(len2) as f64;

    1.0 - (distance as f64 / max_len)
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let v1: Vec<char> = s1.chars().collect();
    let v2: Vec<char> = s2.chars().collect();

    let len1 = v1.len();
    let len2 = v2.len();

    // Early return for empty strings
    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    // Create distance matrix
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first row and column
    for i in 0..=len1 {
        matrix[i][0] = i;
    }

    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    // Fill rest of matrix
    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if v1[i - 1] == v2[j - 1] { 0 } else { 1 };

            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[len1][len2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity() {
        assert_eq!(calculate_similarity("hello", "hello"), 1.0);
        assert_eq!(calculate_similarity("hello", "hallo"), 0.8);
        assert!(calculate_similarity("user123", "user124") > 0.8);
        assert!(calculate_similarity("completely", "different") < 0.5);
    }

    #[tokio::test]
    async fn test_ip_velocity() {
        // This would use a mock Redis client in an actual test
    }

    #[tokio::test]
    async fn test_username_pattern() {
        // This would use a mock Redis client in an actual test
    }
}
