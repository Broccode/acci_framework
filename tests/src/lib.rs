//! Integration test suite for the ACCI Framework
//!
//! This crate contains all integration and end-to-end tests,
//! along with shared test utilities, fixtures, and mocks.

#[cfg(test)]
pub mod api;
pub mod auth;
pub mod database;
pub mod fixtures;
pub mod helpers;
pub mod mocks;
pub mod security;

// All test categories are now implemented
// - API tests: Testing API endpoints and middleware
// - Auth tests: Authentication system component tests
// - Database tests: Testing database operations and migrations
// - Security tests: Security audit tests for authentication flow
