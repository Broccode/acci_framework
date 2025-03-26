use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main security configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Brute force protection configuration
    #[serde(default)]
    pub brute_force: BruteForceConfig,

    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limiting: RateLimitingConfig,

    /// Credential stuffing protection configuration
    #[serde(default)]
    pub credential_stuffing: CredentialStuffingConfig,

    /// Browser fingerprinting configuration
    #[serde(default)]
    pub fingerprinting: FingerprintingConfig,

    /// Replay protection configuration
    #[serde(default)]
    pub replay_protection: ReplayProtectionConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            brute_force: BruteForceConfig::default(),
            rate_limiting: RateLimitingConfig::default(),
            credential_stuffing: CredentialStuffingConfig::default(),
            fingerprinting: FingerprintingConfig::default(),
            replay_protection: ReplayProtectionConfig::default(),
        }
    }
}

/// Configuration for brute force protection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BruteForceConfig {
    /// Whether brute force protection is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Maximum number of failed attempts before lockout
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,

    /// Time window in seconds for counting failed attempts
    #[serde(default = "default_window_seconds")]
    pub window_seconds: u32,

    /// Base delay in milliseconds for progressive backoff
    #[serde(default = "default_base_delay_ms")]
    pub base_delay_ms: u32,

    /// Maximum delay in milliseconds
    #[serde(default = "default_max_delay_ms")]
    pub max_delay_ms: u32,

    /// Account lockout duration in minutes
    #[serde(default = "default_account_lockout_minutes")]
    pub account_lockout_minutes: u32,
}

impl Default for BruteForceConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            max_attempts: default_max_attempts(),
            window_seconds: default_window_seconds(),
            base_delay_ms: default_base_delay_ms(),
            max_delay_ms: default_max_delay_ms(),
            account_lockout_minutes: default_account_lockout_minutes(),
        }
    }
}

/// Configuration for rate limiting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    /// Whether rate limiting is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Default rate limits for all paths
    #[serde(default)]
    pub default_limits: Vec<RateLimit>,

    /// Path-specific rate limits
    #[serde(default)]
    pub path_limits: HashMap<String, Vec<RateLimit>>,

    /// Tenant-specific overrides
    #[serde(default)]
    pub tenant_overrides: HashMap<String, HashMap<String, Vec<RateLimit>>>,
}

impl Default for RateLimitingConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            default_limits: vec![
                RateLimit {
                    window_seconds: 1,
                    max_requests: 10,
                    backoff_multiplier: 2.0,
                },
                RateLimit {
                    window_seconds: 60,
                    max_requests: 100,
                    backoff_multiplier: 2.0,
                },
            ],
            path_limits: HashMap::new(),
            tenant_overrides: HashMap::new(),
        }
    }
}

/// Single rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Time window in seconds
    pub window_seconds: u32,

    /// Maximum requests in the window
    pub max_requests: u32,

    /// Backoff multiplier for repeated violations
    pub backoff_multiplier: f32,
}

/// Configuration for credential stuffing protection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialStuffingConfig {
    /// Whether credential stuffing protection is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Maximum velocity of login attempts per IP
    #[serde(default = "default_max_velocity")]
    pub max_velocity: u32,

    /// Time window in seconds for velocity calculation
    #[serde(default = "default_velocity_window_seconds")]
    pub velocity_window_seconds: u32,

    /// Whether to check username/email patterns
    #[serde(default = "default_true")]
    pub check_username_patterns: bool,

    /// Whether to use CAPTCHA for suspicious activity
    #[serde(default = "default_true")]
    pub enable_captcha: bool,

    /// Whether to use IP blocking for highly suspicious activity
    #[serde(default = "default_true")]
    pub enable_ip_blocking: bool,

    /// IP block duration in minutes
    #[serde(default = "default_ip_block_minutes")]
    pub ip_block_minutes: u32,
}

impl Default for CredentialStuffingConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            max_velocity: default_max_velocity(),
            velocity_window_seconds: default_velocity_window_seconds(),
            check_username_patterns: default_true(),
            enable_captcha: default_true(),
            enable_ip_blocking: default_true(),
            ip_block_minutes: default_ip_block_minutes(),
        }
    }
}

/// Configuration for browser fingerprinting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintingConfig {
    /// Whether fingerprinting is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Whether to collect canvas fingerprint
    #[serde(default = "default_true")]
    pub collect_canvas: bool,

    /// Whether to collect WebGL fingerprint
    #[serde(default = "default_true")]
    pub collect_webgl: bool,

    /// Whether to collect font list
    #[serde(default = "default_true")]
    pub collect_fonts: bool,

    /// Number of days to keep fingerprints
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,

    /// Similarity threshold (0.0-1.0) for matching fingerprints
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f32,
}

impl Default for FingerprintingConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            collect_canvas: default_true(),
            collect_webgl: default_true(),
            collect_fonts: default_true(),
            retention_days: default_retention_days(),
            similarity_threshold: default_similarity_threshold(),
        }
    }
}

/// Configuration for replay protection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayProtectionConfig {
    /// Whether replay protection is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Nonce expiration time in seconds
    #[serde(default = "default_nonce_expiration_seconds")]
    pub nonce_expiration_seconds: u32,

    /// Whether to include timestamp validation
    #[serde(default = "default_true")]
    pub timestamp_validation: bool,

    /// Maximum timestamp skew allowed in seconds
    #[serde(default = "default_max_timestamp_skew_seconds")]
    pub max_timestamp_skew_seconds: u32,
}

impl Default for ReplayProtectionConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            nonce_expiration_seconds: default_nonce_expiration_seconds(),
            timestamp_validation: default_true(),
            max_timestamp_skew_seconds: default_max_timestamp_skew_seconds(),
        }
    }
}

// Default value functions
fn default_true() -> bool {
    true
}

fn default_max_attempts() -> u32 {
    5
}

fn default_window_seconds() -> u32 {
    300 // 5 minutes
}

fn default_base_delay_ms() -> u32 {
    100
}

fn default_max_delay_ms() -> u32 {
    30000 // 30 seconds
}

fn default_account_lockout_minutes() -> u32 {
    30
}

fn default_max_velocity() -> u32 {
    10
}

fn default_velocity_window_seconds() -> u32 {
    60
}

fn default_ip_block_minutes() -> u32 {
    60
}

fn default_retention_days() -> u32 {
    30
}

fn default_similarity_threshold() -> f32 {
    0.8
}

fn default_nonce_expiration_seconds() -> u32 {
    300 // 5 minutes
}

fn default_max_timestamp_skew_seconds() -> u32 {
    60
}
