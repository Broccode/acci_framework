pub mod tenant;
pub mod totp;
pub mod user;
pub mod verification;

// Re-export common model types
pub use tenant::TenantId;
pub use totp::{Algorithm, TotpConfig, TotpSecret, TotpSecretInfo};
pub use user::UserId;
pub use verification::{
    VerificationCode, VerificationConfig, VerificationStatus, VerificationType,
};
