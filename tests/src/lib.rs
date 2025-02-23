//! Integration test suite for the ACCI Framework
//!
//! This crate contains all integration and end-to-end tests,
//! along with shared test utilities, fixtures, and mocks.

pub mod fixtures;
pub mod helpers;
pub mod mocks;

// Integration test modules
// pub mod api;  // TODO: Implement API tests
pub mod auth;
pub mod database;
// pub mod e2e;  // TODO: Implement E2E tests
