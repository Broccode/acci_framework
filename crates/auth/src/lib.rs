pub mod config;
pub mod handlers;
pub mod models;
pub mod repository;
pub mod security;
pub mod services;
pub mod session;
pub mod utils;

pub use config::AuthConfig;
pub use handlers::session::{
    SessionServiceState, TerminateUserSessionsRequest, TerminateSessionsByIpRequest,
    TerminateSessionsByFilterRequest, SessionTerminationResponse, terminate_user_sessions,
    terminate_sessions_by_ip, terminate_sessions_by_filter,
};
pub use models::tenant::{
    CreateTenantDto, Tenant, TenantError, TenantPlanType, TenantRepository, TenantSubscription,
    TenantUser, UpdateTenantDto,
};
pub use models::totp::{Algorithm, TotpConfig, TotpSecret, TotpSecretInfo};
pub use models::user::{CreateUser, LoginCredentials, User, UserError, UserRepository};
pub use models::verification::{
    VerificationCode, VerificationConfig, VerificationStatus, VerificationType,
};
pub use repository::{
    PostgresTenantRepository, PostgresTotpRepository, PostgresUserRepository,
    PostgresVerificationCodeRepository, RepositoryConfig, RepositoryError, TenantAwareContext,
    TenantAwareRepository, TotpSecretRepository, VerificationCodeRepository,
};
pub use security::{
    BruteForceError, BruteForceProtection, Challenge, CredentialStuffingProtection, NonceStore,
    RateLimitConfig, RateLimitMiddleware, ReplayProtectionMiddleware, RiskLevel, SecurityConfig,
    SecurityProtection, create_security_protection,
};
pub use services::{
    email_provider::{SendGridEmailProvider, SmtpEmailProvider, create_email_provider},
    message_provider::{
        EmailProviderConfig, Message, MessageProvider, MessageProviderConfig, SmsProviderConfig,
        SmtpConfig,
    },
    session::{SessionService, SessionServiceError},
    sms_provider::{TwilioSmsProvider, VonageSmsProvider, create_sms_provider},
    tenant::{
        CreateTenantWithAdminDto, TenantService, TenantServiceError, TenantWithAdminResponse,
    },
    totp::{TotpError, TotpService},
    user::{UserService, UserServiceError},
    verification::{VerificationError, VerificationService},
};
pub use session::enhanced_security::{
    EnhancedFingerprintRepository, EnhancedSessionFingerprint,
    PostgresEnhancedFingerprintRepository, PostgresRiskAssessmentRepository,
    PostgresSessionLocationRepository, RiskAssessmentRepository, SessionLocation,
    SessionLocationRepository, SessionRiskAssessment,
};
pub use session::{
    Session, SessionError, SessionFilter, SessionRepository,
    types::{DeviceFingerprint, SessionInvalidationReason},
};
pub use utils::{
    jwt::{Claims, JwtError, JwtUtils},
    password::{PasswordError, check_password_strength, hash_password, verify_password},
};

use acci_core::error::Result;
use std::sync::Arc;

/// Create a verification service with configured providers
pub fn create_verification_service(
    config: &AuthConfig,
    verification_repository: Arc<dyn VerificationCodeRepository>,
) -> Result<Arc<VerificationService>> {
    // Create verification config from auth config
    let verification_config = models::VerificationConfig {
        code_length: config.verification.code_length,
        expiration_seconds: config.verification.expiration_seconds,
        max_attempts: config.verification.max_attempts,
        throttle_seconds: config.verification.throttle_seconds,
    };

    // Setup message providers if configured
    let mut email_provider = None;
    let mut sms_provider = None;

    if let Some(ref message_config) = config.message_providers {
        // Setup email provider
        if let Ok(provider) = create_email_provider(message_config.email.clone()) {
            email_provider = Some(provider);
        }

        // Setup SMS provider
        if let Ok(provider) = create_sms_provider(message_config.sms.clone()) {
            sms_provider = Some(provider);
        }
    }

    // Create verification service
    let verification_service = VerificationService::new(
        verification_repository,
        verification_config,
        sms_provider,
        email_provider,
    );

    Ok(Arc::new(verification_service))
}
