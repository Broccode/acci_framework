use crate::models::{TenantId, TotpConfig, TotpSecret, TotpSecretInfo, UserId};
use crate::repository::TotpSecretRepository;
use crate::utils::password::generate_salt;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier},
};
use base32;
use rand::Rng;
use rand::rngs::ThreadRng;
use std::sync::Arc;
use thiserror::Error;
use time::OffsetDateTime;
use totp_rs::{Secret, TOTP};
use tracing::{debug, error, info, instrument, warn};
use urlencoding;

/// Errors that can occur during TOTP operations
#[derive(Debug, Error)]
pub enum TotpError {
    #[error("MFA is not enabled for this user")]
    MfaNotEnabled,

    #[error("Invalid MFA code")]
    InvalidMfaCode,

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Security validation failed")]
    SecurityValidationFailed,
}

/// Service for managing TOTP (Time-based One-Time Password) authentication
pub struct TotpService {
    secret_repository: Arc<dyn TotpSecretRepository>,
    config: TotpConfig,
}

impl TotpService {
    /// Create a new TotpService
    pub fn new(secret_repository: Arc<dyn TotpSecretRepository>, config: TotpConfig) -> Self {
        Self {
            secret_repository,
            config,
        }
    }

    /// Generate a new TOTP secret for a user
    #[instrument(skip(self), err)]
    pub async fn generate_totp_secret(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> Result<TotpSecretInfo, TotpError> {
        // Generate cryptographically secure random bytes for the secret
        let mut rng = ThreadRng::default();
        let secret_bytes: Vec<u8> = (0..32).map(|_| rng.random()).collect();

        // Encode as base32 for user entry into authenticator apps
        let secret = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &secret_bytes);

        // Generate a provisioning URI for QR code generation
        let uri = format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm={}&digits={}&period={}",
            self.config.issuer,
            user_id,
            secret,
            urlencoding::encode(&self.config.issuer),
            self.config.algorithm,
            self.config.digits,
            self.config.period
        );

        // Create recovery codes (one-time use backup codes)
        let recovery_codes = self.generate_recovery_codes()?;

        // Hash recovery codes before storing
        let argon2 = Argon2::default();
        let hashed_recovery_codes = recovery_codes
            .iter()
            .map(|code| {
                let salt = generate_salt();
                argon2
                    .hash_password(code.as_bytes(), &salt)
                    .map(|hash| hash.to_string())
                    .map_err(|e| TotpError::InternalError(e.to_string()))
            })
            .collect::<Result<Vec<String>, _>>()?;

        // Create TOTP secret object
        let totp_secret = TotpSecret::new(
            *user_id,
            *tenant_id,
            secret.clone(),
            self.config.algorithm.to_string(),
            self.config.digits,
            self.config.period,
            hashed_recovery_codes,
        );

        // Save to repository
        self.secret_repository
            .save(&totp_secret)
            .await
            .map_err(|e| TotpError::RepositoryError(e.to_string()))?;

        info!("Generated new TOTP secret for user {}", user_id);

        // Return information needed for setup
        Ok(TotpSecretInfo {
            secret: secret.clone(),
            uri,
            recovery_codes,
        })
    }

    /// Verify a TOTP code provided by the user
    #[instrument(skip(self), err)]
    pub async fn verify_totp(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
        code: &str,
    ) -> Result<bool, TotpError> {
        // Get the user's TOTP secret
        let mut totp_secret = self
            .secret_repository
            .get_by_user_id(user_id, tenant_id)
            .await
            .map_err(|e| TotpError::RepositoryError(e.to_string()))?
            .ok_or(TotpError::MfaNotEnabled)?;

        // Try to verify TOTP code
        let is_valid = match self.verify_code(&totp_secret, code).await {
            Ok(result) => result,
            Err(e) => {
                warn!("Error verifying TOTP code: {}", e);
                return Err(e);
            },
        };

        // If valid, update last used time and enable if not already
        if is_valid {
            debug!("Valid TOTP code for user {}", user_id);
            totp_secret.last_used_at = Some(OffsetDateTime::now_utc());
            totp_secret.enabled = true;

            // Save updated secret
            self.secret_repository
                .save(&totp_secret)
                .await
                .map_err(|e| TotpError::RepositoryError(e.to_string()))?;
        } else {
            debug!("Invalid TOTP code for user {}", user_id);
        }

        Ok(is_valid)
    }

    /// Verify a TOTP code against the user's secret
    async fn verify_code(&self, secret: &TotpSecret, code: &str) -> Result<bool, TotpError> {
        // Parse code as a number (removing spaces if present)
        let code = code.replace(" ", "");

        let digits = secret.digits;
        let algorithm = match secret.algorithm.as_str() {
            "SHA1" => totp_rs::Algorithm::SHA1,
            "SHA256" => totp_rs::Algorithm::SHA256,
            "SHA512" => totp_rs::Algorithm::SHA512,
            _ => {
                return Err(TotpError::InternalError(
                    "Unsupported algorithm".to_string(),
                ));
            },
        };

        // Create TOTP from secret
        let totp = TOTP::new(
            algorithm,
            digits as usize,
            1, // step size
            secret.period,
            Secret::Encoded(secret.secret.clone())
                .to_bytes()
                .map_err(|e| TotpError::InternalError(e.to_string()))?,
        )
        .map_err(|e| TotpError::InternalError(e.to_string()))?;

        // Check if the code is valid
        let now = OffsetDateTime::now_utc();
        let current_timestamp = now.unix_timestamp() as u64;

        // We need to check with a window of periods both before and after
        let window_size = self.config.window_size as i64;
        let mut is_valid = false;

        // Check current time step
        let result = totp.check(code.trim(), current_timestamp);
        if result {
            is_valid = true;
        }

        // If not valid at current time, check window before and after
        if !is_valid {
            for i in 1..=window_size as u64 {
                // Check before current time
                let before_time = current_timestamp.saturating_sub(i * secret.period);
                if totp.check(code.trim(), before_time) {
                    is_valid = true;
                    break;
                }

                // Check after current time
                let after_time = current_timestamp.saturating_add(i * secret.period);
                if totp.check(code.trim(), after_time) {
                    is_valid = true;
                    break;
                }
            }
        }

        if !is_valid {
            // If TOTP code is not valid, check recovery codes
            return self.verify_recovery_code(secret, &code).await;
        }

        Ok(true)
    }

    /// Generate recovery codes for backup access
    fn generate_recovery_codes(&self) -> Result<Vec<String>, TotpError> {
        let mut rng = ThreadRng::default();
        let mut codes = Vec::with_capacity(10);

        for _ in 0..10 {
            // Generate random bytes
            let random_bytes: Vec<u8> = (0..10).map(|_| rng.random()).collect();

            // Encode with a custom alphabet for readability
            let alphabet = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // Removed similar-looking characters

            // Format as XXXX-XXXX-XXXX
            let chunks: Vec<String> = random_bytes
                .chunks(4)
                .map(|chunk| {
                    chunk
                        .iter()
                        .map(|&b| {
                            alphabet
                                .chars()
                                .nth((b as usize) % alphabet.len())
                                .expect("Failed to get character from alphabet")
                        })
                        .collect()
                })
                .collect();

            let code = format!("{}-{}-{}", chunks[0], chunks[1], chunks[2]);
            codes.push(code);
        }

        Ok(codes)
    }

    /// Verify a recovery code
    async fn verify_recovery_code(
        &self,
        secret: &TotpSecret,
        code: &str,
    ) -> Result<bool, TotpError> {
        debug!("Checking recovery code for user {}", secret.user_id);

        // Check each recovery code
        for (i, hashed_code) in secret.recovery_codes.iter().enumerate() {
            let parsed_hash = PasswordHash::new(hashed_code)
                .map_err(|e| TotpError::InternalError(e.to_string()))?;

            if Argon2::default()
                .verify_password(code.as_bytes(), &parsed_hash)
                .is_ok()
            {
                debug!("Valid recovery code used for user {}", secret.user_id);

                // Recovery code is valid - now invalidate it
                let mut updated_secret = secret.clone();

                // Remove the used recovery code
                let mut new_codes = updated_secret.recovery_codes.clone();
                new_codes.remove(i);
                updated_secret.recovery_codes = new_codes;

                // Save updated recovery codes
                self.secret_repository
                    .save(&updated_secret)
                    .await
                    .map_err(|e| TotpError::RepositoryError(e.to_string()))?;

                return Ok(true);
            }
        }

        debug!("No matching recovery code found");
        Ok(false)
    }

    /// Check if TOTP is enabled for a user
    #[instrument(skip(self), err)]
    pub async fn is_totp_enabled(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> Result<bool, TotpError> {
        let totp_secret = self
            .secret_repository
            .get_by_user_id(user_id, tenant_id)
            .await
            .map_err(|e| TotpError::RepositoryError(e.to_string()))?;

        Ok(totp_secret.map(|s| s.enabled).unwrap_or(false))
    }

    /// Disable TOTP for a user
    #[instrument(skip(self), err)]
    pub async fn disable_totp(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> Result<(), TotpError> {
        // Check if TOTP is enabled
        let totp_secret = self
            .secret_repository
            .get_by_user_id(user_id, tenant_id)
            .await
            .map_err(|e| TotpError::RepositoryError(e.to_string()))?;

        if totp_secret.is_none() {
            return Err(TotpError::MfaNotEnabled);
        }

        // Delete TOTP secret
        self.secret_repository
            .delete(user_id, tenant_id)
            .await
            .map_err(|e| TotpError::RepositoryError(e.to_string()))?;

        info!("Disabled TOTP for user {}", user_id);
        Ok(())
    }

    /// Regenerate recovery codes for a user
    #[instrument(skip(self), err)]
    pub async fn regenerate_recovery_codes(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> Result<Vec<String>, TotpError> {
        // Check if TOTP is enabled
        let mut totp_secret = self
            .secret_repository
            .get_by_user_id(user_id, tenant_id)
            .await
            .map_err(|e| TotpError::RepositoryError(e.to_string()))?
            .ok_or(TotpError::MfaNotEnabled)?;

        // Generate new recovery codes
        let recovery_codes = self.generate_recovery_codes()?;

        // Hash recovery codes before storing
        let argon2 = Argon2::default();
        let hashed_recovery_codes = recovery_codes
            .iter()
            .map(|code| {
                let salt = generate_salt();
                argon2
                    .hash_password(code.as_bytes(), &salt)
                    .map(|hash| hash.to_string())
                    .map_err(|e| TotpError::InternalError(e.to_string()))
            })
            .collect::<Result<Vec<String>, _>>()?;

        // Update recovery codes
        totp_secret.recovery_codes = hashed_recovery_codes;

        // Save updated secret
        self.secret_repository
            .save(&totp_secret)
            .await
            .map_err(|e| TotpError::RepositoryError(e.to_string()))?;

        info!("Regenerated recovery codes for user {}", user_id);
        Ok(recovery_codes)
    }
}
