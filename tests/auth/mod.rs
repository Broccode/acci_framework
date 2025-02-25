//! Authentication integration tests
//!
//! This module contains tests for the authentication system.

// Example:
// pub mod login_tests;
// pub mod registration_tests;

pub mod repository_test;
mod session_test;

pub use repository_test::*;
pub use session_test::*;
