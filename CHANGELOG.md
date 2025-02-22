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

### Added

- Mutation testing pipeline with GitHub Actions
  - Configurable test target selection (single package or workspace)
  - Adjustable timeout and thread count
  - Optional file exclusion patterns
  - Automatic mutation score calculation
  - Warning threshold at 80% mutation coverage
  - Weekly scheduled runs and manual trigger support

### Changed

- Optimized mutation testing configuration for better package-specific testing
- Improved mutation test reporting with detailed JSON output analysis
- Enhanced health check endpoint to return JSON response with service status, version, and timestamp using `OffsetDateTime`
- Added `time` crate as a workspace dependency for `acci-api` crate
- Standardized DateTime handling across the codebase:
  - Migrated all DateTime usage to `time::OffsetDateTime`
  - Removed chrono dependency and conversion utilities
  - Updated test infrastructure to use `OffsetDateTime`
  - Improved timestamp handling in database operations
  - Enhanced session management with consistent DateTime usage

### Fixed

- Corrected package-specific mutation testing command structure
- Fixed mutation score calculation from JSON output

#### Documentation 📚

- **Milestone Documentation:**
  - Standardized task completion format across all milestones using both Markdown checkboxes and emojis:
    - Completed tasks: `[x]` with ✅
    - Incomplete tasks: `[ ]`
    - Completed sections: Header with ✅
    - In-progress/planned sections: Header with 🚧
  - Moved API Documentation, Observability, and Security Hardening tasks from M1.2 to M2.1
  - Enhanced visual clarity and consistency in milestone tracking
  - Improved progress tracking with standardized status indicators
  - Added detailed SSR implementation plan for frontend development
  - Enhanced M1.3 frontend milestone with comprehensive task breakdown
  - Added cross-references between milestone documents and implementation plans
  - Improved documentation structure with clear links between related documents

## [0.1.27] - 2024-03-28

### Changed

#### Code Quality 🔧

- **Authentication Improvements:**
  - Enhanced username search with case-insensitive ILIKE queries in user repository
  - Improved session context handling with derive macro for Default implementation
  - Added comprehensive error documentation for all authentication-related functions
  - Enhanced password verification with better error messages

- **Code Structure:**
  - Improved pattern matching in shutdown signal handler using unit type
  - Extracted database configuration into dedicated module for better organization
  - Enhanced SQLx query caching with updated JSON files
  - Added new database migration for user activation status

- **Documentation:**
  - Added detailed error documentation for database operations
  - Enhanced panic documentation for UTF-8 validation in database functions
  - Improved function documentation with clear error scenarios
  - Added comprehensive documentation for authentication flows

- **Error Handling:**
  - Improved error handling in mock repositories by replacing `unwrap()` calls with proper error propagation
  - Added error injection capabilities to `MockSessionRepository` and `MockUserRepository` for better testing
  - Enhanced error messages with more context and details
  - Added configurable error injection for all repository methods

- **Code Cleanup:**
  - Removed unused imports in `acci-auth` and `acci-api` crates
  - Cleaned up unused dependencies
  - Added tracing dependency for improved logging capabilities

### Fixed

#### Code Quality 🛠️

- **Clippy Warnings:**
  - Fixed unsafe integer cast in session repository by implementing proper error handling with `try_from`
  - Added appropriate allow attribute for logging-related large stack arrays
  - Resolved all clippy warnings across the codebase (excluding integration tests)

#### Security Testing 🔒

- **Token Validation Mutation Tests:**
  - Implemented comprehensive mutation testing suite for token validation
  - Added timestamp validation mutation tests to verify expiration handling
  - Added signature verification mutation tests for various token formats
  - Added algorithm validation mutation tests to prevent algorithm downgrade attacks
  - Added claim validation mutation tests for required and optional claims
  - Enhanced test coverage for edge cases and security-critical paths
  - Improved test organization with modular test functions

- **Password Hash Mutation Tests:**
  - Implemented comprehensive mutation testing suite for password hashing
  - Added property-based testing for password validation rules
  - Enhanced timing attack detection with statistical analysis
  - Added salt uniqueness verification using HashSet
  - Implemented Argon2 parameter validation tests
  - Added explicit error type assertions for validation failures
  - Enhanced test documentation with hardware-specific timing thresholds

#### Test Infrastructure 🧪

- **Session Repository:**
  - Added `cleanup_expired_sessions` implementation to mock session repository
  - Enhanced session cleanup with proper error handling and type safety
  - Improved test coverage for session expiration scenarios

- **Code Quality:**
  - Added selective clippy warning suppression for `unused_mut` in auth tests to improve code clarity while maintaining intended test structure

#### CI/CD Pipeline 🔄

- **Test Pipeline Enhancements:**
  - Reorganized test workflow into separate jobs for better clarity and maintainability
  - Enhanced unit test job with improved coverage reporting and threshold checks
  - Added comprehensive integration test job with detailed logging and artifact collection
  - Implemented mutation testing with configurable thresholds and detailed reporting
  - Added performance testing with baseline comparison and regression detection
  - Created consolidated metrics job for comprehensive test quality monitoring
  - Integrated with Prometheus for metric collection and monitoring
  - Added detailed HTML reports for coverage, mutation testing, and benchmarks
  - Implemented threshold checks for coverage, mutation score, and performance regressions

- **CI Pipeline Improvements:**
  - Added validation job for CHANGELOG.md updates, EditorConfig, and Markdown files
  - Enhanced security checks with comprehensive audit reports and SBOM generation
  - Improved documentation deployment with proper GitHub Pages configuration
  - Added caching for Rust dependencies to speed up builds
  - Enhanced artifact handling with better organization and retention
  - Added detailed reporting for all pipeline stages
  - Improved error handling and status reporting across all jobs

- **EditorConfig Validation:**
  - Enhanced EditorConfig checker configuration with proper regex-based exclusions
  - Added GitHub Actions specific output formatting for better CI integration
  - Enabled colored output for improved readability
  - Optimized bin directory exclusion pattern using `.*\/bin\/.*`

- **SBOM Generation:**
  - Updated cargo-cyclonedx command to use correct `--override-filename` argument instead of deprecated `--output-file`
  - Added fallback for component count in case of JSON parsing errors
  - Improved error handling in SBOM report generation

- **Test Database Configuration:**
  - Updated database connection settings to use standardized credentials
  - Changed database user from 'postgres' to 'acci'
  - Updated database name to match development environment
  - Ensured consistent database configuration across all test jobs

- **Test Database Management:**
  - Added database reset step before running unit and integration tests
  - Ensured clean database state for each test run
  - Improved test isolation and reliability
  - Standardized database initialization process

- **Mutation Testing:**
  - Fixed command line arguments for cargo-mutants
  - Updated output handling to use correct JSON output location
  - Added proper file copying for mutation reports
  - Improved error handling and status tracking
  - Enhanced mutation test reliability

- **Performance Testing:**
  - Added Gnuplot installation for benchmark visualization
  - Improved benchmark environment setup
  - Enhanced plotting capabilities for performance reports
  - Ensured proper tool dependencies for performance analysis

### Fixed

#### CI/CD Pipeline 🔄

- **Coverage Report Generation:**
  - Split coverage report generation into separate steps for each output format
  - Fixed incompatibility between `--lcov` and `--json` output formats
  - Improved organization of coverage reports in dedicated directory structure
  - Added separate steps for LCOV, HTML, JSON, and summary reports

## [0.1.26] - 2024-03-28

### Changed

#### Infrastructure 🏗️

- **Docker Build Process:**
  - Added `acci-cli` crate to production Dockerfile build process
  - Updated manifest copying section to include `acci-cli` Cargo.toml
  - Added dummy source file creation for `acci-cli` crate
  - Included `acci-cli` source code in final build stage

### Fixed

#### Test Infrastructure 🧪

- **Code Quality:**
  - Cleaned up unused imports in test helper files
  - Removed redundant imports in `tests/src/helpers/auth.rs`
  - Removed unused `CoreError` import in `tests/src/mocks/session.rs`
  - Fixed CI pipeline code coverage step

## [0.1.25] - 2024-03-28

### Changed

#### Project Structure 📁

- **Database Migrations:**
  - Removed redundant `/migrations` directory
  - Consolidated all migrations in `/crates/acci-db/migrations/`
  - Ensured consistent migration structure and naming
  - Maintained single source of truth for database schema changes

### Fixed

#### Integration Tests 🧪

- **Password Hash Consistency:**
  - Fixed inconsistent Argon2 parameters between test and migration environments
  - Unified Argon2 configuration (m=19456, t=2, p=1) across all environments
  - Added parameter logging for better debugging visibility
  - Updated pre-computed hashes in migrations to match runtime parameters
  - Enhanced test coverage for password verification
  - Added isolated test case for password hashing verification
  - Improved error handling and logging in auth tests

- **Database Migrations:**
  - Fixed default admin user migration to use correct password hash
  - Updated migration files in both main and test environments
  - Added helper program for generating consistent password hashes
  - Improved migration test reliability
  - Enhanced error handling in migration tests

- **Test Infrastructure:**
  - Added proper database reset functionality using make db-reset
  - Improved test isolation and reliability
  - Enhanced error reporting in authentication tests
  - Added comprehensive logging for cryptographic operations
  - Improved test organization and maintainability

### Added

#### Security Enhancements 🔒

- **Cryptographic Parameter Management:**
  - Added new rule `304-crypto-params.mdc` for consistent parameter handling
  - Implemented comprehensive parameter validation
  - Added debugging guidelines for cryptographic operations
  - Enhanced documentation for security-critical configurations
  - Improved test coverage for security parameters

### Changed

#### Security Enhancements 🔒

- **Session Invalidation for Account Changes:**
  - Added `invalidate_user_sessions` method to `SessionRepository` trait
  - Implemented atomic session invalidation in `PgSessionRepository`
  - Added support in `BasicAuthProvider` for account-wide session invalidation
  - Added comprehensive test coverage in `auth_flow.rs`
  - Enhanced logging with structured context for better debugging
  - Added metrics for session invalidation monitoring
  - Implemented proper error handling and logging
  - Added documentation for all new functionality

- **Enhanced Session Hijacking Prevention Tests:**
  - Token reuse scenarios (after logout, password change)
  - Token manipulation detection (header, payload, signature)
  - Context manipulation tests (IP address, User-Agent)
  - Concurrent session manipulation tests
  - Session enumeration prevention
  - Timing attack prevention
  - Race condition handling
  - Atomic operation verification
  - Comprehensive security assertions
  - Detailed test scenarios
- **Admin Session Management:**
  - Comprehensive admin-initiated session invalidation tests
  - Admin session invalidation functionality with role verification
  - Permission verification for non-admin users with detailed error handling
  - Prevention of self-session invalidation with clear error messages
  - Invalid session ID handling with graceful error recovery
  - Concurrent session operations with race condition prevention
  - Atomic operation verification for session state changes
  - Granular error handling with specific error types
  - Security context validation for admin operations
  - Detailed test scenarios with edge cases

#### Test Infrastructure 🧪

- **Documentation:**
  - Added detailed documentation for pre-commit hook setup in `scripts/SETUP_PRE_COMMIT.md`
  - Added comprehensive test strategy integration overview with detailed phase transitions
  - Added resource allocation details including FTE requirements and infrastructure costs
  - Added expanded risk management section with concrete mitigation strategies
- **Observability:**
  - Added observability integration configuration for test metrics
  - Added alert configuration for test quality monitoring
  - Added links to related testing documentation
  - Added maintenance and update guidelines for testing documentation
- **Test Coverage:**
  - Added comprehensive authentication test suite improvements:
    - Property-based testing for auth flows with QuickCheck
    - Detailed error scenario coverage with specific assertions
    - Configuration testing for rate limiting with metrics validation
    - Concurrent authentication flow testing with race condition detection
    - Enhanced token tampering tests with detailed security checks
    - Registration validation tests with input validation
  - Added enhanced CLI tool test coverage:
    - Comprehensive test suite for database CLI tool
    - Improved password hashing tool tests with Argon2id
    - Enhanced test user management tool tests
    - Property-based testing for password hashing
    - Detailed output format validation
    - Extensive error scenario coverage
    - Performance benchmarking for critical operations

#### CI/CD Improvements 🚀

- **Pipeline Enhancements:**
  - Added mutation testing with cargo-mutants for thorough test validation
  - Added performance benchmark tracking with criterion.rs
  - Added detailed coverage reporting for critical paths
  - Added metrics collection and reporting to Prometheus
- **Metrics and Reporting:**
  - Added performance regression checking scripts
  - Added test metrics report generation with detailed analysis
  - Added Prometheus metrics push capability
  - Added configurable test metrics thresholds:
    - JSON configuration for all test metrics
    - Component-specific coverage thresholds
    - Performance and memory usage limits
  - Added enhanced metrics reporting:
    - Detailed component coverage tracking
    - Performance percentiles (p50, p90, p99)
    - Memory usage monitoring
    - Status tracking for coverage, mutation, and performance

#### Documentation 📚

- **Test Metrics:**
  - Added detailed system overview with architecture diagrams
  - Added configuration guide with examples
  - Added usage instructions with common scenarios
  - Added maintenance procedures with checklists
  - Added troubleshooting guide with common issues
  - Added system architecture diagram with component relationships
  - Added customization examples for different use cases
- **Alerting:**
  - Added Grafana alerting rules:
    - Coverage alerts with thresholds
    - Performance regression alerts with baselines
    - Memory usage alerts with limits
    - Component-specific alerts with owners
    - Customized notification policies
    - Mutation score regression alerts
    - Granular performance alerts (p50, p90, p99)
    - Enhanced flaky test detection
  - Added alert resolution documentation:
    - Detailed investigation procedures
    - Step-by-step resolution guides
    - Escalation procedures with contacts
    - Contact information for teams
    - Response time requirements
    - Alert-specific troubleshooting
    - Severity level definitions
    - Investigation tools and commands
    - Automation opportunities
    - Dashboard links for quick access
    - Runbook references with examples

#### User Management 👤

- **User Profile Enhancement:**
  - Added `full_name` field to User model for better user identification
  - Added database migration for `full_name` field
  - Updated User repository implementation to handle the new field
  - Added validation for full_name field in user creation and updates

### Changed

#### Test Infrastructure 🔧

- **Documentation Generation:** Refined documentation generation process to include details on property-based testing and mutation testing in the generated documentation, enhancing clarity and discoverability of advanced testing strategies
- **Component-Specific Coverage:** Modified test infrastructure to enforce coverage thresholds at the component level (auth, core, db) with detailed reporting
- **Security Audit Integration:** Integrated security audit reports into test infrastructure for automated vulnerability detection and reporting
- **Enhanced Metrics Collection:** Refined metrics collection to include detailed performance and quality metrics with improved Grafana visualization
- **Test Performance Monitoring:** Enhanced test execution time tracking for more effective performance regression monitoring with historical data
- **Artifact Organization:** Improved organization and accessibility of test artifacts in CI/CD pipelines with clear naming conventions
- **Documentation Generation:** Refined documentation generation process with detailed test infrastructure documentation and examples

#### Error Handling ⚠️

- **CLI Tools:**
  - Replaced generic error handling with specific error types for better error reporting
  - Added NotFound error variant for resource lookup failures with clear messages
  - Improved user-facing error messages with actionable information
  - Enhanced error handling documentation with examples

#### Documentation Structure 📋

- **Repository Organization:**
  - Moved all test-related documentation from `docs/infrastructure/` to `docs/tests/` for better organization
  - Moved action plans from `docs/infrastructure/` to root `docs/` directory for easier access
  - Improved documentation organization for better maintainability with clear structure
  - Updated document cross-references to reflect new structure with proper links

#### CI Pipeline 🔄

- **Quality Checks:**
  - Updated workflow to include additional test quality checks with specific thresholds
  - Improved security audit reporting with JSON output for better integration
  - Enhanced artifact collection for test metrics with proper organization
  - Improved Prometheus metric naming and documentation with clear descriptions

#### Metrics and Alerting 📊

- **Configuration:**
  - Updated metrics collection to use external configuration for flexibility
  - Enhanced performance metrics collection with percentiles for better analysis
  - Refined alerting thresholds and policies with team input
  - Improved metrics documentation structure with examples
- **Alert Handling:**
  - Enhanced flaky test handling with rate and count thresholds
  - Updated notification policies with team-specific channels
  - Improved resolution procedures with detailed guides
  - Enhanced documentation with tool-specific guidance
  - Updated severity classifications with clear definitions
  - Improved runbook organization and references
- **Alert Conditions:**
  - Updated regression detection thresholds based on historical data
  - Improved trend analysis timeframes for better accuracy
  - Enhanced resource usage monitoring with specific limits
  - Optimized alert timing parameters to reduce noise
- **Alert Annotations:**
  - Added runbook links for all alerts with clear instructions
  - Added dashboard links for quick access to metrics
  - Improved alert descriptions with context
  - Added team ownership information for clear responsibility

#### Code Improvements 💻

- **Token Handling:**
  - Refactored token tampering functionality in `tests/src/auth/auth_flow.rs` into reusable helper functions to improve maintainability and reduce code duplication in security tests
  - Improved error handling in token validation tests with specific error types
  - Enhanced test coverage for security-critical scenarios with comprehensive assertions

### Fixed

- **SQLx Configuration:** Fixed SQLx dependency configuration by removing the non-existent 'offline' feature from version 0.8.x, ensuring proper compilation and dependency resolution
- **Test User Authentication:** Fixed an issue where test user password hashes in database migration did not match expected test user passwords, causing authentication failures in test environments and preventing proper test execution
- **Docker Build Process:** Fixed Docker production build failures by adding missing root `main.rs` file and resolving build errors related to missing source files, ensuring a stable and reproducible production build process
- **Session Repository Implementation:** Fixed session repository implementation in `crates/acci-db/src/repositories/session.rs` to properly handle session lifecycle and prevent memory leaks
- **Error Handling:** Fixed error handling in `crates/acci-core/src/error.rs` to provide more specific error types and improve error reporting
- **Authentication Routes:** Fixed authentication route implementation in `crates/acci-api/src/routes/auth.rs` to handle all error cases properly and provide clear error messages

### Security

#### Authentication Testing 🔐

- **Test Coverage:**
  - Added extensive security-focused tests for authentication:
    - Token tampering detection with comprehensive checks
    - Rate limiting validation with metrics
    - Concurrent session handling with race condition prevention
    - Error scenario coverage with specific assertions
  - Added detailed security vulnerability reporting in CI
  - Implemented critical vulnerability checks in security audit
  - Enhanced security metrics tracking with alerts
  - Added security-specific alerting rules
  - Improved security incident response procedures
  - Enhanced security tool integration documentation

#### Token Security 🔑

- **Validation:**
  - Added comprehensive testing for JWT security vulnerabilities
  - Improved validation of token claims and headers
  - Enhanced detection of token tampering attempts
  - Enhanced rate limiting test coverage for authentication endpoints
  - Added distributed attack simulation tests

#### Session Security 🛡️

- **Improvements:**
  - Improved session security validation with context checks
  - Added session ID property validation with UUID v4
  - Enhanced session invalidation tests with edge cases
  - Added granular IP and User-Agent validation
  - Implemented timing attack prevention checks

### Ongoing Tasks 📝

#### High Priority Tasks 🔥

- [ ] **Implement** session invalidation for account changes in `crates/acci-core/src/auth/session.rs`
      - Implement automatic session invalidation for password resets and email changes
      - Ensure atomic operations for concurrent session handling
      - Add comprehensive integration tests in `tests/src/auth/session_hijacking_test.rs`
      - Required for OWASP compliance and security best practices
      - Dependencies: None

- [ ] **Document** security assumptions for session management in `docs/ARCHITECTURE.md`
      - Document token lifecycle and validation process
      - Detail session context validation (IP, User-Agent)
      - Specify rate limiting strategies and configurations
      - Critical for security audit preparation
      - Dependencies: Current implementation details

#### Medium Priority Tasks ⚡

- [ ] **Implement** comprehensive timing attack prevention test suite in `tests/src/auth/security_test.rs`
      - Add constant-time comparison tests for token validation
      - Implement timing analysis for session lookups
      - Test rate limiting strategies from `rate_limit_test.rs`
      - Dependencies: Session invalidation implementation

- [ ] **Develop** session security test data parameterization in `tests/src/auth/session_hijacking_test.rs`
      - Extend proptest scenarios for token manipulation
      - Add property-based tests for session validation
      - Implement comprehensive edge case coverage
      - Dependencies: Basic test suite completion

- [ ] **Add** negative test cases for session invalidation
      - Test invalid session IDs and token formats
      - Verify concurrent operation handling
      - Ensure proper error propagation
      - Dependencies: Session invalidation implementation

#### Standard Priority Tasks 📋

- [ ] **Implement** advanced timing attack analysis
      - Add statistical analysis of response times
      - Implement sophisticated timing detection
      - Verify constant-time operations
      - Dependencies: Basic timing attack prevention

- [ ] **Develop** property-based test suite for session ID generation
      - Verify UUID v4 compliance and randomness
      - Test collision probability
      - Ensure cryptographic security
      - Dependencies: None

- [ ] **Implement** session security integration tests
      - Add end-to-end security validation
      - Test multi-session scenarios
      - Verify admin session management
      - Dependencies: All core features

- [ ] **Document** security requirements in `docs/SECURITY.md`
      - Detail session security model
      - Document rate limiting strategies
      - Specify security configurations
      - Dependencies: Architecture documentation

## [0.1.24] - 2024-03-28

### Changed

- Restructured documentation directory:
  - Moved all documentation to root docs/ directory
  - Removed language-specific subdirectories (de/, sq/)
  - Unified infrastructure documentation in single location
  - Simplified documentation structure for better maintainability
- Updated test execution documentation in Integration_Tests.md to use Make targets instead of direct cargo commands for better consistency and maintainability
- Modified Makefile targets to exclude integration tests:
  - Added `--exclude acci-tests` to `clippy` target to prevent unwanted changes in test files
  - Added `--exclude acci-tests` to `fix` target to preserve test-specific imports
- Enhanced pre-commit hook script to improve development workflow:
  - Added proper error handling with `set -e`
  - Improved script organization with clear comments
  - Added automatic directory change to repository root
  - Added clear status messages for better user feedback

### Added

- Added French and Spanish translations to README.md
- Added new helper module `tests/src/helpers/auth.rs` for authentication test utilities
- Added new mock implementation `tests/src/mocks/user.rs` for user testing
- Added pre-commit hook script in `scripts/pre-commit.sh` that runs code formatting, fixing, linting and unit tests before each commit

### Changed

- Translated testing section in .cursorrules from German to English for consistency with the rest of the documentation
- Enhanced test infrastructure:
  - Improved session repository mocking in `tests/src/mocks/session.rs`
  - Updated authentication test cases in `tests/src/api/auth_test.rs`
  - Refined test user testing in `tests/src/api/test_users_test.rs`
  - Updated migration tests in `tests/src/api/migrations_test.rs`
  - Enhanced test helper organization in `tests/src/helpers/mod.rs`

### Fixed

- Fixed test user password hashes in database migration to match the actual test user passwords
- Added helper program to generate correct Argon2 password hashes
- Fixed test targets in Makefile to properly separate unit and integration tests:
  - test-unit now excludes acci-tests crate
  - test-integration now correctly runs tests in acci-tests crate
  - Fixed incorrect test pattern in integration test target
- Fixed Clippy warnings:
  - Added missing error documentation for hash_password function
  - Added #[allow(clippy::large_stack_arrays)] at crate level for acci-db
  - Added Eq implementation for Environment enum
  - Used Self instead of type name in Environment::default implementation

## [0.1.23] - 2024-03-28

### Technical

- Updated dependencies in Cargo.lock to latest compatible versions:
  - cc v1.2.13 -> v1.2.14
  - clap v4.5.28 -> v4.5.29
  - clap_builder v4.5.27 -> v4.5.29
  - equivalent v1.0.1 -> v1.0.2
  - miniz_oxide v0.8.3 -> v0.8.4
  - ring v0.17.8 -> v0.17.9
  - rustls v0.23.22 -> v0.23.23
  - smallvec v1.13.2 -> v1.14.0

## [0.1.22] - 2024-03-28

### Fixed

- Improved database initialization and migrations:
  - Schema and extensions are now created in the first migration
  - Ensured correct migration order
  - Sessions table is created in the correct schema
  - UUID generation explicitly uses public schema
  - Enhanced error handling during database initialization
  - Applied DRY principle in Makefile by using existing targets

### Changed

- Switched from `chrono::DateTime<Utc>` to `time::OffsetDateTime` for better SQLx compatibility
- Unified database commands in Makefile
- Enhanced integration tests:
  - Improved connection pool configuration for better stability
  - Increased connection pool limits and timeouts
  - Added proper error handling in email case sensitivity tests
  - Added small delay after user creation to ensure transaction completion
  - Improved test assertions with better error messages
  - Reduced test execution time from 16s to 9s

## [0.1.21] - 2024-03-28

### Added

- Test user configuration in acci-core
- Database migration for predefined test users
- Password hash generation tool in acci-db
- Mock repository implementation for integration testing
- Comprehensive test coverage for test user authentication

### Fixed

- Session invalidation and concurrent sessions
- Case-sensitive email search in user repository

### Changed

- Improved documentation and code structure
- Switched code coverage reporting from Codecov to Coveralls.io

## [0.1.20] - 2024-03-28

### Fixed

- Fixed Docker production build by adding missing root main.rs file

## [0.1.19] - 2024-03-28

### Fixed

- Fixed Docker production build by adding missing test_users.rs binary file for acci-db crate

## [0.1.18] - 2024-03-28

### Fixed

- Fixed Docker production build by adding missing dummy files for all crates:
  - Added main.rs and bin files for acci-db
  - Ensured all required source files are present for dependency resolution

## [0.1.17] - 2024-03-28

### Fixed

- Fixed Docker production build by adding missing lib.rs dummy file for acci-api crate

## [0.1.16] - 2024-03-28

### Fixed

- Fixed Docker production build by removing tests from workspace members in Dockerfile.prod

## [0.1.15] - 2024-03-28

### Fixed

- Fixed Docker production build by excluding test workspace from release builds to prevent missing Cargo.toml errors

## [0.1.14] - 2024-03-28

### Fixed

- Fixed Docker production build by excluding test workspace from release builds to prevent missing Cargo.toml errors

## [0.1.13] - 2024-03-28

### Fixed

- Fixed Docker production build by excluding test workspace from release builds to prevent missing Cargo.toml errors

## [0.1.12] - 2024-03-28

### Changed

- Updated database migrations to use Argon2 instead of Blowfish for password hashing:
  - Changed default admin user migration to use pre-computed Argon2 hash
  - Updated test users migration to use pre-computed Argon2 hashes
  - Ensured consistent password hashing across codebase
- Removed unused migrate binary from acci-db crate to simplify the codebase
- Updated project description to correctly reflect ACCI as an enterprise application framework rather than just a license management system:
  - Updated README.md in all languages (EN, DE, SQ)
  - Adjusted feature descriptions to show license management as one of many features
  - Maintained consistent terminology across all documentation
- Restructured milestone M3.4 to better reflect the role of license management as a feature:
  - Renamed from "License Management System" to "Enterprise Features and License Management"
  - Adjusted subtasks to align with the framework's broader scope
  - Updated task descriptions to maintain consistency with overall architecture
- Updated milestone documentation to reflect authentication progress in all supported languages (EN, DE, SQ)
- Improved dependency management:
  - Moved all dependency definitions to workspace level
  - Implemented strict workspace inheritance for shared dependencies
  - Added dependency management guidelines to .cursorrules
  - Removed redundant version specifications in individual crates
  - Centralized feature configuration in workspace
- Updated acci-db binary to use DATABASE_URL environment variable:
  - Added proper environment variable handling
  - Improved error messages and logging
  - Added fallback to default configuration
- Added missing documentation for Environment enum and variants
- Moved auth integration tests from `acci-auth/tests` to central integration test suite
- Fixed password hashing in auth integration tests
- Cleaned up test module exports in integration tests

### Added

- Added comprehensive README.md in three languages (EN, DE, SQ):
  - Project description and key features
  - Quick start guide with make commands
  - Links to language-specific documentation
  - License and security information
  - Contributing guidelines
- Added database migration for default admin user:
  - Secure password hashing using pgcrypto's blowfish
  - Default credentials: admin/whiskey
  - Idempotent migration with conflict handling
  - Integration tests for migration and authentication
  - Test infrastructure with Docker containers
- Implemented default test user for development:
  - Username: admin
  - Password: whiskey
  - Secure password hashing with Argon2
  - Comprehensive test coverage for authentication flow
  - Mock repository implementation for testing
- Basic Authentication Provider Infrastructure:
  - Modular provider system for extensible authentication methods
  - Trait-based approach for provider implementations
  - Support for multiple authentication strategies
  - Password security with Argon2 implementation
  - JWT token management and validation
  - User authentication traits and repository integration
  - Comprehensive test coverage for auth components
  - Security-first implementation following best practices
- Login endpoint implementation:
  - REST API endpoint for user authentication ✅
  - Request validation and error handling ✅
  - Proper error mapping between core and API layers ✅
  - Integration with BasicAuthProvider ✅
  - Comprehensive test coverage with mock repositories ✅
  - CORS support for authentication endpoints ✅
  - Structured logging with sensitive data masking ✅
  - Proper dependency injection for database access ✅
  - Unit tests for invalid credentials scenario ✅
- Updated milestone documentation to reflect authentication progress in all supported languages (EN, DE, SQ)
- Added test-users make targets for managing test users in development:
  - test-users-list: List all test users and their status
  - test-users-reset: Reset test users to default configuration
  - test-users-clean: Delete all test users

### Technical

- Enhanced test infrastructure:
  - Added mock repositories for unit testing
  - Implemented proper dependency injection in tests
  - Added test coverage for error scenarios
  - Improved test isolation and maintainability
  - Added structured test organization
- Improved code quality:
  - Fixed clippy warnings
  - Added proper documentation
  - Implemented proper error handling
  - Added structured logging
  - Improved type safety with proper imports
- Fixed dependency configuration in acci-api:
  - Moved rand_core from dev-dependencies to dependencies
  - Ensured proper workspace inheritance for cryptographic dependencies
  - Fixed import resolution for OsRng in auth tests
  - Resolved version conflicts in rand and rand_core dependencies
  - Updated getrandom to version 0.3.1 for better compatibility
- Fixed cyclic dependency between acci-auth and acci-db:
  - Removed direct dependency from acci-db to acci-auth
  - Moved password hashing functionality to acci-core
  - Improved crate architecture by centralizing core functionality
- Fixed dependency configuration in acci-db:
  - Added acci-auth dependency for test-users binary
  - Resolved import resolution for password hashing functionality

### Fixed

- Fixed test user password hashes in database migration to match the actual test user passwords
- Added helper program to generate correct Argon2 password hashes
- Fixed test targets in Makefile to properly separate unit and integration tests:
  - test-unit now excludes acci-tests crate
  - test-integration now correctly runs tests in acci-tests crate
  - Fixed incorrect test pattern in integration test target
- Fixed Clippy warnings:
  - Added missing error documentation for hash_password function
  - Added #[allow(clippy::large_stack_arrays)] at crate level for acci-db
  - Added Eq implementation for Environment enum
  - Used Self instead of type name in Environment::default implementation

## [0.1.11] - 2024-03-27

### Added

- Added `.editorconfig` file for consistent code formatting across different editors and IDEs:
  - Configured specific rules for Rust files matching rustfmt settings
  - Added specialized configurations for TOML, Markdown, YAML, and JSON files
  - Set up proper Git commit message formatting
  - Configured documentation-specific rules
  - Added Makefile-specific tab configuration

### Changed

- Completed database integration milestone (M1.2):
  - Finalized PostgreSQL setup with migrations system
  - Completed user schema design with UUID and timestamp support
  - Implemented full Repository pattern with CRUD operations
  - Added comprehensive test coverage using testcontainers
  - Integrated CLI tools for database management
  - Updated milestone documentation to reflect completion

## [0.1.10] - 2024-03-27

### Changed

- Moved user repository tests from `user.rs` to integration tests in `user_test.rs`
- Fixed UUID import in tests to use SQLx's UUID type instead of direct uuid crate

## [0.1.9] - 2024-03-27

### Technical

- Updated workspace version to match package version
- Synchronized version numbers across workspace crates

## [0.1.8] - 2024-03-27

### Fixed

- Fixed Docker build process by creating proper dummy source files for each crate and maintaining correct directory structure during build phases

## [0.1.7] - 2024-03-27

### Added

- Completed core infrastructure setup (M1.1):
  - Basic repository structure with workspace configuration
  - Development environment with Docker setup
  - Initial linting configuration
  - Basic CI/CD pipeline with GitHub Actions
  - Test automation framework
- Partial completion of MVP Backend (M1.2):
  - Basic Axum setup with health check endpoint
  - Error handling structure with custom API errors
  - CORS and tracing middleware
  - Health check endpoint returning service status and version
  - Integration tests for health check endpoint:
    - Test coverage for HTTP status codes
    - Response payload validation
    - Middleware integration testing
  - Database integration:
    - PostgreSQL setup in Docker Compose
    - SQLx integration with offline mode support
    - Database migrations system
    - CLI tool for database management
    - Initial users table migration
    - Make commands for database operations
- Leptos frontend framework dependencies (leptos, leptos_meta, leptos_router) to workspace dependencies
- wasm-bindgen-test for frontend testing capabilities
- User Repository implementation in acci-db:
  - CRUD operations for user management
  - Email-based user lookup
  - Secure password hash storage
  - Automatic timestamp handling
  - Comprehensive test coverage
  - SQLx integration with type-safe queries
  - UUID-based user identification
- Enhanced test coverage for database layer:
  - Comprehensive unit tests for database connection handling
  - Test coverage for connection pool limits and timeouts
  - Error handling tests for invalid configurations
  - Migration error handling tests
  - Complete test coverage for DbError type
  - Environment-aware test configuration
  - Connection pool lifecycle tests

### Changed

- Switched Leptos frontend framework from CSR (Client-Side Rendering) to SSR (Server-Side Rendering) for improved performance and SEO capabilities
- Enhanced User Repository implementation:
  - Added comprehensive documentation for all public types and functions
  - Fixed schema usage to properly use 'acci' schema for all database operations
  - Improved error handling documentation
  - Added proper clippy configuration and fixed all warnings
  - Added documentation for potential panics and error conditions
- Improved test organization and separation:
  - Moved database-dependent tests to integration tests
  - Enhanced unit tests to be independent of external dependencies
  - Fixed body handling in API error tests
  - Simplified test database configuration
  - Improved test isolation and maintainability
- Refactored database integration tests:
  - Migrated to testcontainers for improved test isolation
  - Added proper database initialization with extensions
  - Improved connection pool testing with better timeout handling
  - Enhanced error condition testing for invalid configurations
  - Added proper cleanup of test resources

### Technical

- Updated workspace dependencies to latest versions:
  - tokio to 1.43.0
  - axum to 0.8.1
  - hyper to 1.6.0
  - serde to 1.0.217
  - Other dependencies updated to their latest stable versions
- Switched from chrono to time crate for timestamp handling in User repository
- Moved uuid dependency to workspace dependencies for better version management
- Enhanced development environment:
  - Added structured shell scripts in devbox configuration
  - Improved init_hook for better rustup integration
  - Added convenient scripts for testing and documentation
- Updated Rust toolchain configuration:
  - Set specific Rust version to 1.84.1
  - Added support for multiple targets including WebAssembly
  - Configured minimal profile with essential components
- Improved workspace configuration:
  - Moved all dependency versions to workspace
  - Added SQLx with runtime-tokio-rustls and macros support
  - Added Clap for CLI tools
  - Enabled acci-db crate in workspace
  - Added acci-db binary target for database management
- Enhanced development guidelines in `.cursorrules`:
  - Added clear AI assistant role and expertise definition
  - Added explicit references to project guideline files
  - Improved formatting and structure of guidelines
  - Enhanced markdown formatting for better readability
- Improved code quality through Clippy fixes:
  - Optimized error handling in API responses
  - Enhanced logging structure with proper allow attributes
  - Removed unnecessary imports
  - Improved JSON serialization without macro usage
  - Added proper test configuration for Clippy rules
  - Limited Clippy checks to libraries and binaries to avoid integration test issues
- Improved test organization and execution:
  - Separated unit tests and integration tests in CI pipeline
  - Moved database-dependent tests from repository modules to integration tests
  - Added separate make commands for running unit and integration tests
  - Enhanced test documentation and organization
  - Optimized CI pipeline to run tests in correct order with proper database setup
  - Switched to testcontainers for database integration tests
  - Added Docker-in-Docker support for CI pipeline
- Updated Leptos stack to version 0.7 to address unmaintained dependencies:
  - Resolved unmaintained `instant` dependency issue (RUSTSEC-2024-0384)
  - Resolved unmaintained `proc-macro-error` dependency issue (RUSTSEC-2024-0370)

### Security

- Updated sqlx to version 0.8.1 to fix Binary Protocol Misinterpretation vulnerability (RUSTSEC-2024-0363)

### Fixed

- Fixed database setup in integration tests:
  - Added pgcrypto extension for cryptographic functions
  - Added uuid-ossp extension for UUID generation
  - Ensured extensions are created before schema creation
  - Improved test database initialization reliability

## [0.1.6] - 2024-03-26

### Technical

- Modified production Docker build:
  - Temporarily disabled frontend assets copying to container
  - Simplified container image size by removing unused static files

## [0.1.5] - 2024-03-26

### Technical

- Enhanced Docker build process:
  - Added temporary main.rs for initial dependency build phase
  - Optimized two-phase build process: dependencies first, then full application
  - Improved build reliability by ensuring all required files exist during dependency resolution

## [0.1.4] - 2024-03-26

### Added

- Added proper library configurations for all workspace crates:
  - Added [lib] sections with explicit name and path configurations
  - Created initial lib.rs files with placeholder implementations
  - Configured proper dependencies and workspace inheritance

### Technical

- Improved Docker build process by removing dummy file creation
- Updated Rust version to 1.84.1 in Docker build

## [0.1.3] - 2024-03-26

### Added

- Added binary target configuration in root Cargo.toml
- Created initial main.rs with basic logging setup and entry point

## [0.1.2] - 2024-03-26

### Fixed

- Added missing root crate target (lib.rs) to fix Docker build process

## [0.1.1] - 2024-03-26

### Added

- License Management Framework with enterprise licensing and tenant-specific feature control
  - Basic license validation for MVP phase
  - Feature flag system for license control
  - Tenant-specific resource control
  - Offline validation support
  - License expiration notifications
  - License key generation and validation
  - Usage analytics and reporting capabilities
  - Emergency override system for critical situations
  - Tenant quota management system
  - Resource allocation tracking

### Technical

- Fixed permissions in docs-sync GitHub Action workflow to properly create translation issues
- Added Cargo.toml configurations for all workspace crates:
  - acci-api: Added axum integration and dependencies on auth/db
  - acci-auth: Added core authentication dependencies
  - acci-db: Added SQLx integration for database access
  - acci-frontend: Added Leptos framework and WASM testing support
  - All crates inherit workspace-wide configuration and lints

## [0.1.0] - 2024-02-09

### Changed

- Simplified core module structure by temporarily disabling unused modules (models, traits, types)
- Adjusted core prelude exports to match current module structure
- Enhanced multi-tenancy architecture with license and feature management capabilities
- Extended tenant isolation system to support feature-based access control
- Defined comprehensive test organization structure:
  - Separated unit tests (inline) and integration tests (/tests)
  - Established clear test directory structure with dedicated categories
  - Standardized test file naming and organization
  - Implemented container management guidelines for integration tests
  - Added test helper utilities and fixtures organization
- Enhanced development guidelines in `.cursorrules`:
  - Added clear AI assistant role and expertise definition
  - Added explicit references to project guideline files
  - Improved formatting and structure of guidelines
  - Enhanced markdown formatting for better readability

### Technical

- Set up Rust workspace with initial dependencies
- Configured workspace-wide lints and MSRV
- Implemented basic error handling in core crate
- Added Docker Compose configuration for development environment
- Configured multi-stage Dockerfile for development
- Added GitHub Actions workflows for:
  - Testing and linting
  - Security auditing
  - SBOM generation
  - License compliance checking
  - Release automation
  - Documentation deployment
  - Translation synchronization
- Enhanced test infrastructure:
  - Defined `/tests` directory structure for integration tests
  - Added support for testcontainers-rs framework
  - Implemented test categories (api, database, e2e, services)
  - Created helper utilities for container management
  - Set up test fixtures organization
  - Established naming conventions for test files
  - Added guidelines for container lifecycle management
- Updated development guidelines:
  - Added explicit file references for PLAN.md, MILESTONES.md, and CHANGELOG.md
  - Improved markdown formatting in `.cursorrules`
  - Enhanced section organization in guidelines

### Fixed

- Fixed SBOM generation in CI pipeline by correcting cargo-cyclonedx command syntax
- Fixed Clippy lint group priorities for pedantic and nursery groups

### Fixed

- Fixed Clippy warnings:
  - Added missing error documentation for Result-returning functions
  - Added missing panic documentation for functions that may panic
  - Removed unused metrics in session repository
  - Fixed redundant pattern matching in auth provider
  - Removed unused self parameters in basic auth provider
  - Added allow attribute for large_stack_arrays in logging macros
  - Renamed unused variable ip to _ip in auth module

### Fixed

- Implemented case-insensitive username search in user repository to ensure consistent user lookup regardless of letter casing
- Added explicit case-insensitive username uniqueness check during user creation

### Changed

- Moved mutation testing into its own GitHub Actions workflow to improve CI/CD pipeline performance
- Mutation tests now run on push/PR to main/master and can be triggered manually
- Simplified test metrics collection in main test workflow

### Changed

#### Code Quality 🛠️

- **Documentation:**
  - Added comprehensive documentation for auth module in API middleware
  - Enhanced documentation for tasks module in authentication service
  - Improved documentation for session cleanup module
  - Added detailed module-level documentation across the codebase

### Fixed

#### Code Quality 🛠️

- **Clippy Warnings:**
  - Fixed unsafe integer cast in session repository by implementing proper error handling with `try_from`
  - Added appropriate allow attribute for logging-related large stack arrays
  - Resolved all clippy warnings across the codebase (excluding integration tests)

### Changed

#### Code Quality 🔧

- **Error Handling:**
  - Improved error handling in mock repositories by replacing `unwrap()` calls with proper error propagation
  - Added error injection capabilities to `MockSessionRepository` and `MockUserRepository` for better testing
  - Enhanced error messages with more context and details
  - Added configurable error injection for all repository methods

- **Code Cleanup:**
  - Removed unused imports in `acci-auth` and `acci-api` crates
  - Cleaned up unused dependencies
  - Added tracing dependency for improved logging capabilities

#### Code Improvements 💻

- **Code Improvements:**
  - Replaced manual `Default` implementation for `SessionContext` with derive macro
  - Improved pattern matching in `shutdown_signal` using unit type `()` instead of wildcard `_`
  - Enhanced SQLx query caching with updated JSON files
  - Added new database migration for user activation status
  - Extracted database configuration into separate module
