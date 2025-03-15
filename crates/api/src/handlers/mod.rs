// Handler modules for the API
pub mod auth;
pub mod example;
pub mod example_router;
pub mod tenant;
pub mod verification;
pub mod webauthn;

// Re-export handlers
pub use auth::*;
pub use tenant::*;
pub use verification::*;
pub use webauthn::*;
