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

## [0.2.0] - 2025-03-06

### Added

- Comprehensive Authentication Documentation
  - Detailed multi-factor authentication implementation
  - OAuth 2.0/OpenID Connect integration specifications
  - Password strength validation using zxcvbn
  - Detailed audit logging implementation
  - Advanced session management features
- Complete API Specification
  - Comprehensive REST API endpoint documentation
  - GraphQL schema documentation
  - Detailed data models and validation rules
  - Authentication flow documentation
  - Error handling specifications
  - Versioning strategy
  - Rate limiting guidelines
  - Pagination implementation
  - Data filtering and sorting
  - Client library and SDK information
- Advanced API Implementation Examples
  - Authentication flow examples in multiple languages
  - Session management usage patterns
  - Error handling best practices
  - Structured logging implementation
  - Domain-specific error handling
- Session Management Infrastructure
  - Comprehensive session management tables
  - Secure token handling with rotation support
  - Device fingerprinting capabilities
  - Session audit logging with retention policies
  - Automated session cleanup
  - Optimized activity tracking
  - Enhanced security with typed session invalidation reasons
  - Support for multiple device management
- Leptos Frontend Implementation (SSR-only)
  - Login and Registration form components with server-side rendering
  - Navigation component with authentication state awareness
  - Error Display component for standardized error presentation
  - Loading Indicator component for asynchronous operations
  - Client-side form validation with unobtrusive JavaScript
  - Integration with Axum routing system
  - Proper error handling and display
  - Responsive styling with clean, accessible design
  - Comprehensive component unit tests
  - Integration tests for form submission handlers
- Comprehensive Project Documentation
  - Detailed project description with architecture details
  - Component descriptions (Core, Auth, API, Web)
  - Build, test, and lint command documentation
  - Code style guidelines
  - Async programming guidelines
  - Test organization documentation
  - Key documentation references

### Fixed

- Compilation issues in API and web modules
  - Resolved ambiguous glob imports in web module to prevent naming conflicts
  - Fixed missing imports in API validation and example handlers
  - Corrected function parameter types and usage in test code
  - Added proper error handling for JSON parsing in validation tests
  - Renamed API and web component functions to follow Rust's snake_case convention
  - Fixed Docker test availability detection to gracefully handle environments without Docker
  - Properly prefixed unused variables with underscores to prevent compiler warnings
  - Fixed initialization issues in the validation module tests
- Web component implementation maintainability issues
  - Corrected import paths in `prelude.rs` to fix IntoView trait resolution issues
  - Fixed incompatible trait implementations for HTML element functionality
  - Replaced direct macro usage with more reliable module imports
  - Ensured correct element class and child element trait implementations  
  - Added proper legacy component support with deprecated annotations
  - Refactored component functions to follow snake_case naming convention
  - Simplified component structure to improve reliability and reduce errors
  - Improved compatibility between Leptos components and server-side rendering
- CI pipeline and test execution issues
  - Fixed integration test execution with proper setup
  - Enhanced coverage reporting configuration
  - Improved test output capture and reporting

### Changed

- Enhanced API infrastructure implementation
  - Improved middleware stack organization
  - Enhanced error handling middleware
  - Standardized request validation
  - Unified response formatting
  - Added comprehensive metrics tracking
  - Implemented structured logging for requests
  - Added domain-specific error handling
- Improved documentation structure and organization
  - Standardized documentation format across all features
  - Enhanced code examples with syntax highlighting
  - Added diagrams for authentication and API flows
  - Improved navigation and cross-references
  - Added search functionality to documentation
- Updated milestone progress tracking
  - Marked all Milestone 1 tasks as complete
  - Updated test coverage metrics
  - Verified all functional requirements
  - Validated performance requirements
  - Confirmed security requirements
  - Achieved quality requirements

### Removed

- Deprecated API test directories
  - Removed `crates/api/tests/` directory with outdated test implementation
  - Removed `tests/api/` directory with incompatible test code

### Technical

- Restructured database migrations
  - Consolidated module-specific migrations into central `/migrations` directory
  - Implemented standardized migration structure
  - Updated database.rs to load migrations from central location
  - Modified test helpers to use new migration path
- Added new dependencies
  - Added `hex` for secure token handling
  - Added `ipnetwork` feature to sqlx for IP address management
  - Implemented conditional metrics support with feature flags
  - Added mock metrics implementations when feature is disabled

## [Unreleased]

### Added

- Multi-tenancy implementation
  - Added database schema for tenant management with proper constraints and indexing
  - Implemented tenant repository for CRUD operations
  - Created tenant service with business logic
  - Added tenant identification and resolution middleware
  - Implemented tenant-based request routing
  - Added tenant context passing through middleware
  - Created tenant isolation mechanics for data separation
  - Added token-based tenant identification

### Fixed

- Resolved async/sync issues in the tenant middleware implementation
  - Fixed future Send bounds for request references in middleware
  - Improved data extraction pattern for better thread safety
  - Resolved potential race conditions in tenant resolution process
  - Ensured proper error propagation across async boundaries
  - Fixed test failures related to missing tenant_id field

### Technical

- Added SQL query caching for tenant operations
  - Generated SQLx prepared query support files
  - Cached tenant lookup queries for improved performance
  - Added tenant user relationship query caching

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
