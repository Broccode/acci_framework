pub mod bruteforce;
pub mod config;
pub mod credstuffing;
pub mod fingerprint;
pub mod ratelimit;
pub mod replay;
pub mod types;

// Re-exports
pub use bruteforce::BruteForceProtection;
pub use config::{
    BruteForceConfig, CredentialStuffingConfig, FingerprintingConfig as FingerprintConfig,
    RateLimitingConfig as RateLimitConfig, ReplayProtectionConfig, SecurityConfig,
};
pub use credstuffing::{ChallengeProvider, CredentialStuffingProtection, PatternDetector};
pub use ratelimit::{RateLimitInfo, RateLimitLayer, RateLimitMiddleware, RateStore};
pub use types::{BruteForceError, RateLimitError};
pub use types::{
    Challenge, GeoLocation, LoginAttempt, RiskLevel, SecurityError, create_tenant_redis_key,
};
// Fingerprinting module exports
pub use fingerprint::{
    BrowserFingerprint, FingerprintComparison, FingerprintRepository, FingerprintService,
    PostgresFingerprintRepository, StoredFingerprint,
};
pub use replay::{NonceStore, ReplayProtectionLayer, ReplayProtectionMiddleware};

use redis::Client;
use std::sync::Arc;
use tracing::info;

/// Creates a new SecurityProtection instance with all security features
pub fn create_security_protection(
    redis_client: Arc<Client>,
    db_pool: sqlx::PgPool,
    config: SecurityConfig,
) -> anyhow::Result<Arc<SecurityProtection>> {
    // Create the fingerprint repository if configured
    let fingerprint_repo = if config.fingerprinting.enabled {
        Some(Arc::new(fingerprint::PostgresFingerprintRepository::new(
            db_pool.clone(),
        )) as Arc<dyn fingerprint::FingerprintRepository>)
    } else {
        None
    };

    // Create the security protection service
    let protection = SecurityProtection::new(redis_client, fingerprint_repo, config);

    info!("Security protection services initialized successfully");

    Ok(Arc::new(protection))
}

/// Main security protection service that combines all security features
pub struct SecurityProtection {
    /// Brute force protection service
    pub brute_force: BruteForceProtection,
    /// Credential stuffing protection service
    pub cred_stuffing: CredentialStuffingProtection,
    /// Rate limit store
    pub rate_store: Arc<RateStore>,
    /// Nonce store for replay protection
    pub nonce_store: Arc<NonceStore>,
    /// Redis client
    pub redis_client: Arc<Client>,
    /// Fingerprint service (optional)
    pub fingerprint_service: Option<Arc<fingerprint::FingerprintService>>,
    /// Security configuration
    pub config: SecurityConfig,
}

impl SecurityProtection {
    /// Create a new security protection service
    pub fn new(
        redis_client: Arc<Client>,
        fingerprint_repo: Option<Arc<dyn fingerprint::FingerprintRepository>>,
        config: SecurityConfig,
    ) -> Self {
        let brute_force =
            BruteForceProtection::new(redis_client.clone(), config.brute_force.clone());

        let pattern_detector = Arc::new(PatternDetector::new(redis_client.clone()));
        let challenge_provider = Arc::new(ChallengeProvider::new());

        let cred_stuffing = CredentialStuffingProtection::new(
            pattern_detector,
            challenge_provider,
            config.credential_stuffing.clone(),
        );

        let rate_store = Arc::new(RateStore::new(redis_client.clone()));

        let nonce_store = Arc::new(NonceStore::new(
            redis_client.clone(),
            config.replay_protection.clone(),
        ));

        // Initialize fingerprint service if repo is provided
        let fingerprint_service = fingerprint_repo.map(|repo| {
            Arc::new(fingerprint::FingerprintService::new(
                repo,
                config.fingerprinting.clone(),
            ))
        });

        info!("Security protection service initialized");

        Self {
            brute_force,
            cred_stuffing,
            rate_store,
            nonce_store,
            redis_client,
            fingerprint_service,
            config,
        }
    }

    /// Get a rate limit middleware
    pub fn rate_limit_middleware(&self) -> RateLimitLayer {
        RateLimitLayer::new(self.rate_store.clone(), self.config.rate_limiting.clone())
    }

    /// Get a replay protection middleware
    pub fn replay_protection_middleware(&self) -> ReplayProtectionLayer {
        ReplayProtectionLayer::new(self.nonce_store.clone())
    }
}
