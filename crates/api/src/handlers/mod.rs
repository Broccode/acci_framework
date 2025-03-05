// Handler-Module für die API
pub mod auth;
pub mod example;
pub mod example_router;

// Re-Export der Handler
pub use auth::*;
