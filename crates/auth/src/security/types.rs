use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Risk level enum for security assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk - normal operation
    Low,
    /// Medium risk - some suspicious signals
    Medium,
    /// High risk - strongly suspicious signals
    High,
    /// Critical risk - definitely malicious
    Critical,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "Low"),
            RiskLevel::Medium => write!(f, "Medium"),
            RiskLevel::High => write!(f, "High"),
            RiskLevel::Critical => write!(f, "Critical"),
        }
    }
}

impl Default for RiskLevel {
    fn default() -> Self {
        RiskLevel::Low
    }
}

/// Challenge type for suspicious activity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Challenge {
    /// No challenge required
    None,
    /// CAPTCHA challenge
    Captcha(CaptchaChallenge),
    /// Delay response by specified milliseconds
    Delay(u32),
    /// Require MFA authentication
    MfaRequired,
    /// Block IP address for specified duration
    IpBlock(Duration),
}

/// CAPTCHA challenge details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaChallenge {
    /// CAPTCHA challenge ID
    pub challenge_id: String,
    /// CAPTCHA image or challenge data
    pub challenge_data: String,
    /// CAPTCHA type
    pub captcha_type: CaptchaType,
}

/// Type of CAPTCHA challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaptchaType {
    /// Image-based CAPTCHA
    Image,
    /// Text-based CAPTCHA
    Text,
    /// Audio-based CAPTCHA
    Audio,
    /// reCAPTCHA
    ReCaptcha,
    /// hCaptcha
    HCaptcha,
}

/// Login attempt information for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginAttempt {
    /// Tenant ID
    pub tenant_id: String,
    /// Username or email
    pub username: String,
    /// IP address of the attempt
    pub ip_address: String,
    /// User agent string
    pub user_agent: String,
    /// Timestamp of the attempt
    pub timestamp: DateTime<Utc>,
    /// Optional browser fingerprint
    pub fingerprint: Option<String>,
    /// Optional geolocation information
    pub geolocation: Option<GeoLocation>,
    /// Whether the login was successful
    pub successful: bool,
}

/// Geolocation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    /// Country code (ISO 3166-1 alpha-2)
    pub country_code: String,
    /// City name
    pub city: Option<String>,
    /// Latitude
    pub latitude: Option<f64>,
    /// Longitude
    pub longitude: Option<f64>,
}

/// Base error type for security module
#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Brute force error: {0}")]
    BruteForce(#[from] BruteForceError),

    #[error("Rate limit error: {0}")]
    RateLimit(#[from] RateLimitError),

    #[error("Credential stuffing error: {0}")]
    CredentialStuffing(String),

    #[error("Fingerprint error: {0}")]
    Fingerprint(String),

    #[error("Replay protection error: {0}")]
    ReplayProtection(String),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Brute force specific errors
#[derive(Error, Debug)]
pub enum BruteForceError {
    #[error("Account locked: too many attempts")]
    AccountLocked,

    #[error("Progressive delay required: {0}ms")]
    ProgressiveDelay(u32),

    #[error("Redis operation failed: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Rate limit specific errors
#[derive(Error, Debug)]
pub enum RateLimitError {
    #[error("Rate limit exceeded: {0} requests in {1}s window")]
    RateLimitExceeded(u32, u32),

    #[error("Redis operation failed: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Creates a Redis key with tenant namespace
pub fn create_tenant_redis_key(tenant_id: &str, key_type: &str, key: &str) -> String {
    format!("security:{}:{}:{}", tenant_id, key_type, key)
}
