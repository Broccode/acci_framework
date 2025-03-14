use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::models::{TenantId, UserId};

/// Types of verification methods available
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationType {
    /// Email-based verification
    Email,
    /// SMS-based verification
    Sms,
}

/// Status of a verification code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// Code is pending verification
    Pending,
    /// Code has been verified successfully
    Verified,
    /// Code has expired without being used
    Expired,
    /// Code has been invalidated (e.g., due to too many failed attempts)
    Invalidated,
}

/// Configuration for verification codes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    /// Length of the verification code
    pub code_length: usize,
    /// How long the code is valid for (in seconds)
    pub expiration_seconds: i64,
    /// Maximum number of attempts allowed
    pub max_attempts: usize,
    /// Minimum time between code generation requests (in seconds)
    pub throttle_seconds: i64,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            code_length: 6,
            expiration_seconds: 600, // 10 minutes
            max_attempts: 5,
            throttle_seconds: 60, // 1 minute
        }
    }
}

/// Represents a verification code for second-factor authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCode {
    /// Unique identifier for this verification code
    pub id: Uuid,
    /// The tenant this verification belongs to
    pub tenant_id: TenantId,
    /// User this verification code belongs to
    pub user_id: UserId,
    /// The verification code itself
    pub code: String,
    /// Type of verification (Email/SMS)
    pub verification_type: VerificationType,
    /// When this code was created
    pub created_at: OffsetDateTime,
    /// When this code expires
    pub expires_at: OffsetDateTime,
    /// Current status of this code
    pub status: VerificationStatus,
    /// Number of verification attempts made
    pub attempts: usize,
}

impl VerificationCode {
    /// Create a new verification code
    pub fn new(
        tenant_id: TenantId,
        user_id: UserId,
        code: String,
        verification_type: VerificationType,
        config: &VerificationConfig,
    ) -> Self {
        let now = OffsetDateTime::now_utc();
        let expires_at = now + Duration::seconds(config.expiration_seconds);

        Self {
            id: Uuid::new_v4(),
            tenant_id,
            user_id,
            code,
            verification_type,
            created_at: now,
            expires_at,
            status: VerificationStatus::Pending,
            attempts: 0,
        }
    }

    /// Check if this verification code is expired
    pub fn is_expired(&self) -> bool {
        self.expires_at < OffsetDateTime::now_utc() || self.status == VerificationStatus::Expired
    }

    /// Check if this code has too many attempts
    pub fn has_max_attempts(&self, config: &VerificationConfig) -> bool {
        self.attempts >= config.max_attempts
    }

    /// Increment the attempt counter
    pub fn increment_attempts(&mut self) {
        self.attempts += 1;
    }

    /// Mark this code as verified
    pub fn mark_verified(&mut self) {
        self.status = VerificationStatus::Verified;
    }

    /// Mark this code as expired
    pub fn mark_expired(&mut self) {
        self.status = VerificationStatus::Expired;
    }

    /// Mark this code as invalidated
    pub fn mark_invalidated(&mut self) {
        self.status = VerificationStatus::Invalidated;
    }
}
