use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    /// JWT secret key for token signing
    pub jwt_secret: String,
    /// JWT token lifetime in seconds
    pub jwt_lifetime_secs: u64,
    /// Session lifetime in seconds
    pub session_lifetime_secs: u64,
    /// Session activity update interval in seconds
    pub session_activity_update_interval_secs: u64,
    /// Session cleanup interval in seconds
    pub session_cleanup_interval_secs: u64,
    /// Invalid session retention period in seconds
    pub invalid_session_retention_secs: u64,
    /// Session audit log retention period in seconds
    pub audit_log_retention_secs: u64,
    /// Maximum number of active sessions per user
    pub max_sessions_per_user: u32,
    /// Whether to enable device fingerprinting
    pub enable_device_fingerprinting: bool,
    /// Whether to enable session token rotation
    pub enable_session_token_rotation: bool,
    /// Session token rotation interval in seconds
    pub session_token_rotation_interval_secs: u64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "default-secret-please-change".to_string(),
            jwt_lifetime_secs: 3600,                    // 1 hour
            session_lifetime_secs: 86400,               // 24 hours
            session_activity_update_interval_secs: 300, // 5 minutes
            session_cleanup_interval_secs: 3600,        // 1 hour
            invalid_session_retention_secs: 7776000,    // 90 days
            audit_log_retention_secs: 7776000,          // 90 days
            max_sessions_per_user: 5,
            enable_device_fingerprinting: true,
            enable_session_token_rotation: true,
            session_token_rotation_interval_secs: 43200, // 12 hours
        }
    }
}

impl AuthConfig {
    pub fn session_lifetime(&self) -> Duration {
        Duration::from_secs(self.session_lifetime_secs)
    }

    pub fn session_activity_update_interval(&self) -> Duration {
        Duration::from_secs(self.session_activity_update_interval_secs)
    }

    pub fn session_cleanup_interval(&self) -> Duration {
        Duration::from_secs(self.session_cleanup_interval_secs)
    }

    pub fn invalid_session_retention(&self) -> Duration {
        Duration::from_secs(self.invalid_session_retention_secs)
    }

    pub fn audit_log_retention(&self) -> Duration {
        Duration::from_secs(self.audit_log_retention_secs)
    }

    pub fn session_token_rotation_interval(&self) -> Duration {
        Duration::from_secs(self.session_token_rotation_interval_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AuthConfig::default();
        assert_eq!(config.jwt_lifetime_secs, 3600);
        assert_eq!(config.session_lifetime_secs, 86400);
        assert_eq!(config.session_activity_update_interval_secs, 300);
        assert_eq!(config.session_cleanup_interval_secs, 3600);
        assert_eq!(config.invalid_session_retention_secs, 7776000);
        assert_eq!(config.audit_log_retention_secs, 7776000);
        assert_eq!(config.max_sessions_per_user, 5);
        assert!(config.enable_device_fingerprinting);
        assert!(config.enable_session_token_rotation);
        assert_eq!(config.session_token_rotation_interval_secs, 43200);
    }

    #[test]
    fn test_duration_conversions() {
        let config = AuthConfig::default();
        assert_eq!(config.session_lifetime(), Duration::from_secs(86400));
        assert_eq!(
            config.session_activity_update_interval(),
            Duration::from_secs(300)
        );
        assert_eq!(config.session_cleanup_interval(), Duration::from_secs(3600));
        assert_eq!(
            config.invalid_session_retention(),
            Duration::from_secs(7776000)
        );
        assert_eq!(config.audit_log_retention(), Duration::from_secs(7776000));
        assert_eq!(
            config.session_token_rotation_interval(),
            Duration::from_secs(43200)
        );
    }
}
