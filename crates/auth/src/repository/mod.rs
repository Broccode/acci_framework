pub mod postgres;
pub mod postgres_totp;
pub mod postgres_verification;
#[cfg(feature = "enable_webauthn")]
pub mod postgres_webauthn;
pub mod tenant_aware;
pub mod totp_repository;
pub mod verification_repository;
#[cfg(feature = "enable_webauthn")]
pub mod webauthn_repository;

pub use postgres::{
    AuditEvent, PostgresTenantRepository, PostgresUserRepository, RepositoryConfig,
    TenantAuditEvent,
};
pub use postgres_totp::PostgresTotpRepository;
pub use postgres_verification::PostgresVerificationCodeRepository;
#[cfg(feature = "enable_webauthn")]
pub use postgres_webauthn::PostgresWebAuthnRepository;
pub use tenant_aware::{RepositoryError, TenantAwareContext, TenantAwareRepository};
pub use totp_repository::TotpSecretRepository;
pub use verification_repository::VerificationCodeRepository;
#[cfg(feature = "enable_webauthn")]
pub use webauthn_repository::WebAuthnRepository;
