pub mod config;
pub mod models;
pub mod repository;
pub mod services;
pub mod session;
pub mod utils;

pub use config::AuthConfig;
pub use models::user::{CreateUser, LoginCredentials, User, UserError, UserRepository};
pub use repository::{PostgresUserRepository, RepositoryConfig};
pub use services::{
    session::{SessionService, SessionServiceError},
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
