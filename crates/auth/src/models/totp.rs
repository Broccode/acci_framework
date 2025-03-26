use crate::models::{TenantId, UserId};
use serde::{Deserialize, Serialize};
use std::fmt;
use time::OffsetDateTime;
use uuid::Uuid;

/// A Time-based One-Time Password (TOTP) secret for a user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSecret {
    /// The unique identifier for this TOTP secret
    pub id: Uuid,

    /// The user this TOTP secret belongs to
    pub user_id: UserId,

    /// The tenant this TOTP secret belongs to
    pub tenant_id: TenantId,

    /// The Base32-encoded secret used for generating TOTP codes
    pub secret: String,

    /// The algorithm used for TOTP generation (SHA1, SHA256, SHA512)
    pub algorithm: String,

    /// The number of digits in the generated TOTP code (usually 6)
    pub digits: u32,

    /// The period in seconds for which a TOTP code is valid (usually 30)
    pub period: u64,

    /// Hashed recovery codes for account access if TOTP is unavailable
    pub recovery_codes: Vec<String>,

    /// Whether TOTP is enabled for this user
    pub enabled: bool,

    /// When the TOTP secret was created
    pub created_at: OffsetDateTime,

    /// When the TOTP secret was last used for authentication
    pub last_used_at: Option<OffsetDateTime>,
}

impl TotpSecret {
    /// Create a new TOTP secret
    pub fn new(
        user_id: UserId,
        tenant_id: TenantId,
        secret: String,
        algorithm: String,
        digits: u32,
        period: u64,
        recovery_codes: Vec<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            tenant_id,
            secret,
            algorithm,
            digits,
            period,
            recovery_codes,
            enabled: false,
            created_at: OffsetDateTime::now_utc(),
            last_used_at: None,
        }
    }

    /// Determine if the TOTP setup is complete and ready to use
    pub fn is_setup_complete(&self) -> bool {
        self.enabled
    }
}

/// Information needed for setting up TOTP authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpSecretInfo {
    /// The Base32-encoded secret
    pub secret: String,

    /// The URI for QR code generation
    pub uri: String,

    /// Recovery codes for backup access
    pub recovery_codes: Vec<String>,
}

/// Algorithm used for TOTP generation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Algorithm {
    SHA1,
    SHA256,
    SHA512,
}

impl fmt::Display for Algorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Algorithm::SHA1 => write!(f, "SHA1"),
            Algorithm::SHA256 => write!(f, "SHA256"),
            Algorithm::SHA512 => write!(f, "SHA512"),
        }
    }
}

/// Configuration for TOTP generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpConfig {
    /// Application name shown in authenticator apps
    pub issuer: String,

    /// Algorithm used for TOTP generation
    pub algorithm: Algorithm,

    /// Number of digits in the generated code (usually 6)
    pub digits: u32,

    /// Time step in seconds (usually 30)
    pub period: u64,

    /// Number of time periods to check before/after current time
    pub window_size: u64,
}

impl Default for TotpConfig {
    fn default() -> Self {
        Self {
            issuer: "ACCI Framework".to_string(),
            algorithm: Algorithm::SHA1,
            digits: 6,
            period: 30,
            window_size: 1,
        }
    }
}
