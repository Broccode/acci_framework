//! API integration tests
//!
//! This module contains tests for the API endpoints.

#[cfg(test)]
pub mod auth_handler_test;
#[cfg(test)]
pub mod middleware_test;
#[cfg(test)]
pub mod router_test;
#[cfg(test)]
pub mod validation_test;

#[cfg(test)]
pub use auth_handler_test::*;
