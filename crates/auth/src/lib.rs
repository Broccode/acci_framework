pub mod config;
pub mod models;
pub mod repository;
pub mod services;
pub mod session;
pub mod utils;

pub use config::AuthConfig;
pub use models::tenant::{
    CreateTenantDto, Tenant, TenantError, TenantPlanType, TenantRepository, TenantSubscription,
    TenantUser, UpdateTenantDto,
};
pub use models::totp::{Algorithm, TotpConfig, TotpSecret, TotpSecretInfo};
pub use models::user::{CreateUser, LoginCredentials, User, UserError, UserRepository};
pub use repository::{
    PostgresTenantRepository, PostgresTotpRepository, PostgresUserRepository, RepositoryConfig,
    RepositoryError, TenantAwareContext, TenantAwareRepository, TotpSecretRepository,
};
pub use services::{
    session::{SessionService, SessionServiceError},
    tenant::{
        CreateTenantWithAdminDto, TenantService, TenantServiceError, TenantWithAdminResponse,
    },
    totp::{TotpError, TotpService},
    user::{UserService, UserServiceError},
};
pub use session::{
    Session, SessionError, SessionFilter, SessionRepository,
    types::{DeviceFingerprint, SessionInvalidationReason},
};
pub use utils::{
    jwt::{Claims, JwtError, JwtUtils},
    password::{PasswordError, check_password_strength, hash_password, verify_password},
};
