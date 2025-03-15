pub mod tenant;
pub mod totp;
pub mod user;
pub mod verification;
#[cfg(feature = "enable_webauthn")]
pub mod webauthn;

// Re-export common model types
pub use tenant::TenantId;
pub use totp::{Algorithm, TotpConfig, TotpSecret, TotpSecretInfo};
pub use user::UserId;
pub use verification::{
    VerificationCode, VerificationConfig, VerificationStatus, VerificationType,
};
#[cfg(feature = "enable_webauthn")]
pub use webauthn::{
    Credential, CredentialID, PublicKeyCredential, RegisterCredential, WebAuthnError,
};
