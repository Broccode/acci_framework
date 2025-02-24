# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Version Numbering

This project uses a three-number versioning system (X.Y.Z):

X (Major): Breaking changes, major feature overhauls
Y (Minor): New features, significant improvements
Z (Patch): Bug fixes, minor improvements

Example: Version 1.2.3

1: Major version
2: Minor version
3: Patch version

When to increment:

Major (X): When making incompatible changes that might break existing functionality
Minor (Y): When adding functionality in a backward-compatible manner
Patch (Z): When making backward-compatible bug fixes

## Making Changelog Entries For New Changes in Development

Add changes under the [Unreleased] section

Categorize them under appropriate headers:

Added for new features

Changed for changes in existing functionality

Deprecated for soon-to-be removed features

Removed for removed features

Fixed for bug fixes

Security for vulnerability fixes

Technical for technical changes/dependencies

Keep entries concise but descriptive

## When Releasing a Version

Convert the [Unreleased] section to a version number with date (e.g., [1.0.0] - 2024-01-20)

Create a new empty [Unreleased] section at the top

## General Rules

Newest changes always go at the top of the file

Each version should be in descending order (newest to oldest)

Group related changes under the same category

Use bullet points for each entry

## Development Workflow

For Every Code Change:

ALWAYS add an entry to the [Unreleased] section in this changelog

Write clear, descriptive change notes

Categorize changes appropriately using the headers above

Commit changes with meaningful commit messages

For Version Releases:

Move [Unreleased] changes to a new version section with today's date

Update version number in ProjectSettings.asset (bundleVersion)

Create a git tag for the version

Create a new empty [Unreleased] section at the top

## Release Process

When asked to make a release, follow these steps:

Review Changes:

Review all changes under [Unreleased]

Ensure all changes are properly categorized

Verify all changes are documented

Choose Version Number:

For new features: increment minor version (0.1.0 → 0.2.0)

For bug fixes: increment patch version (0.1.0 → 0.1.1)

For breaking changes: increment major version (0.1.0 → 1.0.0)

Update Files:

Move [Unreleased] changes to new version section with today's date

Update version in ProjectSettings.asset (bundleVersion)

Create new empty [Unreleased] section

Commit and Tag:

Commit all changes with message "release: Version X.Y.Z"

Create a git tag for the version (e.g., v0.2.0)

## [Unreleased]

### Changed

- Improved CI pipeline coverage reporting
  - Separated unit and integration test coverage reports
  - Fixed coverage report generation with multiple output formats
  - Added separate Coveralls reporting for unit and integration tests
  - Enhanced coverage threshold checks for both test types
- Fixed integration test execution in CI pipeline
  - Added cargo-nextest installation step
  - Created test-logs directory before test execution
  - Ensured proper test output capture
- Temporarily disabled E2E tests in CI pipeline
  - Removed E2E test execution step
  - Removed Playwright installation
  - Removed E2E test artifacts from upload
  - Will be re-enabled once E2E tests are implemented
- Temporarily disabled additional test types in CI pipeline
  - Property-based tests disabled until test suite is complete
  - Mutation tests disabled pending implementation
  - Performance tests (criterion and k6) disabled until benchmarks are defined
  - Security tests (fuzzing, audit, deny) disabled until security baseline is established
  - All test artifacts and reports adjusted accordingly

## [0.1.1] - 2025-02-24

### Fixed

- Package naming consistency
  - Changed all package names from kebab-case to snake_case
  - Updated all local dependencies to use snake_case names
  - Affected packages: acci_core, acci_auth, acci_api, acci_web
- Code quality improvements in auth module
  - Removed unsafe unwrap() calls in PostgresUserRepository
  - Fixed redundant pattern matching in user email checks
  - Improved error handling in configuration initialization
  - Enhanced JSON serialization safety in audit events
- CI/CD Pipeline improvements
  - Fixed nextest JUnit report generation by using NEXTEST_JUNIT_REPORT environment variable
  - Updated all test steps to use the correct configuration method
  - Ensured consistent report generation across unit, property, integration and E2E tests

### Added

- Initial CI/CD pipeline setup with GitHub Actions
  - Basic test workflow with PostgreSQL integration
  - Code coverage reporting using cargo-tarpaulin and Coveralls
  - Security audit workflow with cargo-audit and cargo-deny
  - Automated documentation generation and deployment to GitHub Pages
  - Release management workflow with automated changelog generation
  - Automated crate publishing to crates.io
- Core infrastructure setup
  - Database connection handling with SQLx
  - Error handling framework
  - Configuration management
  - Logging and metrics setup
  - Initial database migrations
    - Users table with UUID, email, and password hash
    - Automatic timestamp handling
    - Email indexing
- Database testing infrastructure
  - Integration with testcontainers for isolated PostgreSQL instances
  - Automatic database creation and migration for tests
  - Connection pool tests
  - Concurrent query tests
  - Transaction handling tests
  - Error handling tests
  - Test fixtures and utilities
  - Comprehensive user repository testing
    - Full CRUD operation coverage
    - Email uniqueness validation
    - Password hash handling verification
    - Timestamp management testing
    - User state transitions testing
- Enhanced test infrastructure with cargo-nextest
  - Custom test profiles for development, CI, and coverage
  - Optimized test execution with parallel testing
  - Slow test detection and reporting
  - Immediate failure reporting for better developer experience

### Changed

- Significantly enhanced CI/CD pipeline configuration
  - Introduced parallel test execution with cargo-nextest
  - Added matrix builds for multiple Rust versions (stable, nightly) and platforms
  - Implemented comprehensive caching strategy for dependencies and artifacts
  - Enhanced test result reporting with JUnit format
  - Added detailed performance metrics collection
  - Improved security scanning and reporting
  - Added SBOM generation and validation
  - Enhanced documentation deployment workflow
  - Added EditorConfig and Markdown validation
  - Improved artifact management and retention
  - Added automatic CHANGELOG.md validation
- Updated test execution infrastructure
  - Migrated from direct cargo test to cargo-nextest
  - Added make targets for different test categories
  - Enhanced test output configuration
  - Improved CI test execution with retries
- Updated test documentation
  - Added nextest configuration documentation
  - Updated test execution instructions
  - Clarified test profile configurations
  - Added make target documentation

### Technical

- Updated test dependencies and configuration
  - Configured nextest profiles in `.config/nextest.toml`
  - Removed unsupported nextest configuration options
  - Optimized coverage profile for accurate results
  - Enhanced CI profile with comprehensive output
