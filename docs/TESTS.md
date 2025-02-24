# Testing Strategy Documentation

## Overview

This document provides a comprehensive overview of our testing strategy and links to detailed documentation for each testing aspect. Our testing approach ensures high code quality, reliability, and maintainability of the Enterprise Application Framework.

## Test Categories

### [Unit Tests](unit-tests.md)

- Individual function and component testing
- Co-located with production code
- No external dependencies
- Fast execution for immediate feedback
- Target coverage: 90% line coverage, 85% branch coverage

### [Integration Tests](integration-tests.md)

- Testing component interactions
- Located in `/tests` directory
- Controlled external dependencies
- Container-based testing
- Target coverage: 80% line coverage

### [Property-Based Tests](property-tests.md)

- Automated test case generation
- System invariant verification
- Comprehensive edge case coverage
- Using proptest framework
- Example state machine testing:

  ```rust
  use proptest::prelude::*;
  use proptest::strategy::Strategy;

  #[derive(Debug, Clone)]
  enum UserAction {
      Register { email: String },
      Login { email: String },
      Logout,
  }

  proptest! {
      #[test]
      fn test_user_state_machine(actions: Vec<UserAction>) {
          let mut state = UserState::new();
          for action in actions {
              state = state.apply(action)?;
              // Verify invariants after each action
              assert!(state.invariants_hold());
          }
      }
  }
  ```

### [Performance Tests](performance-tests.md)

- Benchmark testing using criterion.rs ([Documentation](https://bheisler.github.io/criterion.rs/book/))
- Load testing with k6 ([Documentation](https://k6.io/docs/))
- Scalability verification
- RED metrics tracking:
  - Rate (requests/second)
  - Errors (failed requests/second)
  - Duration (response time percentiles)
- Example benchmark:

  ```rust
  use criterion::{criterion_group, criterion_main, Criterion};

  fn benchmark_api_endpoint(c: &mut Criterion) {
      c.bench_function("user_registration", |b| {
          b.iter(|| {
              // Test implementation
          })
      });
  }

  criterion_group!(benches, benchmark_api_endpoint);
  criterion_main!(benches);
  ```

### [Security Tests](security-tests.md)

- Fuzzing tests with cargo-fuzz ([Documentation](https://rust-fuzz.github.io/book/))
- Security boundary testing
- Vulnerability scanning
- Penetration testing patterns
- Focus areas:
  - Input validation
  - Authentication flows
  - Authorization checks
  - Data encryption
  - SQL injection prevention
- Example fuzz test:

  ```rust
  #[fuzz]
  fn fuzz_api_input(data: &[u8]) {
      if let Ok(input) = std::str::from_utf8(data) {
          let _ = validate_user_input(input);
      }
  }
  ```

### [End-to-End Tests](e2e-tests.md)

- Complete workflow testing
- Browser automation using Playwright ([Documentation](https://playwright.dev/))
- API chain testing
- Example E2E test:

  ```rust
  use playwright::Playwright;

  #[tokio::test]
  async fn test_user_registration_flow() {
      let playwright = Playwright::initialize().await?;
      let browser = playwright.chromium().launcher().launch().await?;
      let page = browser.new_page().await?;

      // Test implementation
      page.goto("http://localhost:3000/register").await?;
      page.fill("input[name='email']", "test@example.com").await?;
      page.click("button[type='submit']").await?;
      
      assert!(page.url().contains("/dashboard")).await?;
  }
  ```

### [Mutation Tests](mutation-tests.md)

- Code quality verification using cargo-mutants ([Documentation](https://github.com/sourcefrog/cargo-mutants))
- Test suite effectiveness measurement
- Target mutation score: 80%
- Workflow:
  1. Run mutation tests weekly
  2. Review surviving mutants
  3. Add tests to catch mutations
  4. Document uncatchable mutations

## Test Infrastructure

### Directory Structure

```plaintext
/
├── crates/
│   ├── core/
│   │   ├── src/
│   │   │   └── module/
│   │   │       ├── mod.rs          # Module code
│   │   │       └── tests.rs        # Unit tests for this module
│   │   └── tests/                  # DEPRECATED - DO NOT USE FOR NEW TESTS
│   └── other_crate/
│       └── src/
│           └── tests/              # Unit tests only
└── tests/                          # ALL Integration/E2E Tests
    ├── Cargo.toml                  # Test-specific dependencies
    ├── src/
    │   ├── helpers/               # Shared test utilities
    │   │   ├── mod.rs
    │   │   ├── database.rs
    │   │   └── http.rs
    │   ├── fixtures/              # Test data
    │   │   ├── mod.rs
    │   │   └── users.rs
    │   └── mocks/                 # Mock implementations
    │       ├── mod.rs
    │       └── services.rs
    ├── api/                       # API integration tests
    │   ├── mod.rs
    │   ├── auth_tests.rs
    │   └── user_tests.rs
    ├── auth/                      # Authentication integration tests
    │   ├── mod.rs
    │   └── login_tests.rs
    ├── database/                  # Database integration tests
    │   ├── mod.rs
    │   └── migration_tests.rs
    └── e2e/                       # End-to-end tests
        ├── mod.rs
        └── workflows_tests.rs
```

### Common Tools and Dependencies

```toml
[dev-dependencies]
tokio = { workspace = true, features = ["full", "test-util"] }
proptest = { workspace = true }
testcontainers = { workspace = true }
wiremock = { workspace = true }
fake = { workspace = true }
```

## Best Practices

1. **Test Organization**
   - Clear separation between unit and integration tests
   - Consistent naming conventions
   - Comprehensive documentation

2. **Test Quality**
   - High coverage requirements
   - Mutation testing verification
   - Regular test suite maintenance

3. **CI/CD Integration**
   - Automated test execution
   - Coverage reporting
   - Performance benchmark tracking

## Getting Started

### Installation

First, install cargo-nextest:

```bash
cargo install cargo-nextest
```

### Running Tests

1. Run the complete test suite (unit and integration tests):

   ```bash
   make test
   ```

2. Run only unit tests:

   ```bash
   make test-unit
   ```

3. Run only integration tests:

   ```bash
   make test-integration
   ```

4. Run E2E tests:

   ```bash
   make test-e2e
   ```

5. Generate test coverage report (LCOV format):

   ```bash
   make coverage
   ```

6. Generate HTML coverage report:

   ```bash
   make coverage-html
   ```

7. Before committing changes, run all checks including tests:

   ```bash
   make prepare-commit
   ```

For a complete list of available commands, run:

```bash
make help
```

### Nextest Configuration

The project uses cargo-nextest with custom configuration in `.config/nextest.toml`:

```toml
[profile.default]
# Configure the default test profile for local development
retries = 0
test-threads = "num-cpus"
status-level = "pass"
final-status-level = "fail"
failure-output = "immediate"
success-output = "never"
slow-timeout = { period = "60s", terminate-after = 3 }

[profile.ci]
# CI-specific configuration
retries = 2
test-threads = "num-cpus"
status-level = "all"
final-status-level = "all"
failure-output = "immediate-final"
success-output = "final"
slow-timeout = { period = "60s", terminate-after = 3 }

[profile.coverage]
# Profile for running tests with coverage
retries = 0
test-threads = 1  # Single threaded for accurate coverage
status-level = "all"
final-status-level = "all"
failure-output = "immediate"
success-output = "never"
```

The configuration defines three profiles:

- `default`: Optimized for local development with immediate failure output
- `ci`: Configured for CI environment with retries and comprehensive output
- `coverage`: Single-threaded execution for accurate coverage measurement

For more nextest configuration options, see the [official documentation](https://nexte.st/book/configuration.html).

## Test Implementation Standards

### Directory Structure

```plaintext
/
├── crates/
│   ├── core/
│   │   ├── src/
│   │   │   └── module/
│   │   │       ├── mod.rs          # Module code
│   │   │       └── tests.rs        # Unit tests for this module
│   │   └── tests/                  # DEPRECATED - DO NOT USE FOR NEW TESTS
│   └── other_crate/
│       └── src/
│           └── tests/              # Unit tests only
└── tests/                          # ALL Integration/E2E Tests
    ├── Cargo.toml                  # Test-specific dependencies
    ├── src/
    │   ├── helpers/               # Shared test utilities
    │   │   ├── mod.rs
    │   │   ├── database.rs
    │   │   └── http.rs
    │   ├── fixtures/              # Test data
    │   │   ├── mod.rs
    │   │   └── users.rs
    │   └── mocks/                 # Mock implementations
    │       ├── mod.rs
    │       └── services.rs
    ├── api/                       # API integration tests
    │   ├── mod.rs
    │   ├── auth_tests.rs
    │   └── user_tests.rs
    ├── auth/                      # Authentication integration tests
    │   ├── mod.rs
    │   └── login_tests.rs
    ├── database/                  # Database integration tests
    │   ├── mod.rs
    │   └── migration_tests.rs
    └── e2e/                       # End-to-end tests
        ├── mod.rs
        └── workflows_tests.rs
```

### Implementation Examples

#### Unit Tests

Unit tests must be co-located with the code they test:

```rust
// In crates/core/src/database/mod.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_string_parsing() {
        let result = parse_connection_string("postgres://localhost");
        assert!(result.is_ok());
    }
}
```

#### Integration Tests

Integration tests must be placed in the `/tests` directory:

```rust
// In /tests/database/migration_tests.rs
use crate::helpers::database::TestDb;

#[rstest]
#[tokio::test]
async fn test_migration_applies_successfully(#[future] db: TestDb) {
    let db = db.await;
    // Test implementation
}
```

### Critical Rules

1. Directory Structure
   - ✅ ALL integration tests MUST be in `/tests`
   - ✅ ALL unit tests MUST be with their code
   - ❌ NEVER put integration tests in crate-specific test directories
   - ❌ NEVER put unit tests in `/tests`

2. Test Organization
   - ✅ Use appropriate test categories
   - ✅ Follow the directory structure exactly
   - ✅ Use helper modules for shared code
   - ❌ NEVER mix test categories

3. Dependencies
   - ✅ Use testcontainers for external services
   - ✅ Keep mocks in `/tests/src/mocks`
   - ❌ NEVER use real external services in unit tests
   - ❌ NEVER use mocks in unit tests

### Migration Guide

If you find tests in the wrong location:

1. Identify the test type (unit/integration)
2. Move to correct location
3. Update imports and paths
4. Remove old test directories
5. Update documentation

Example:

```bash
# Moving integration tests to /tests
mv crates/core/tests/integration/* tests/database/
rm -rf crates/core/tests/integration

# Moving unit tests next to their code
mv crates/core/tests/unit/parser_test.rs crates/core/src/parser/tests.rs
```

## Test Fixtures and Mocks

### Fixture Best Practices

1. Use factories for complex objects:

   ```rust
   #[derive(Default)]
   pub struct UserBuilder {
       email: Option<String>,
       role: Option<Role>,
   }

   impl UserBuilder {
       pub fn email(mut self, email: impl Into<String>) -> Self {
           self.email = Some(email.into());
           self
       }

       pub fn build(self) -> User {
           User {
               email: self.email.unwrap_or_else(|| fake::faker::internet::en::SafeEmail().fake()),
               role: self.role.unwrap_or_default(),
           }
       }
   }
   ```

2. Keep fixtures minimal
3. Use fake data generators
4. Version control fixtures
5. Document fixture assumptions

### Mock Best Practices

1. Mock at architectural boundaries
2. Use trait-based mocks:

   ```rust
   #[async_trait]
   pub trait UserRepository: Send + Sync {
       async fn find_by_email(&self, email: &str) -> Result<User>;
   }

   #[derive(Default)]
   pub struct MockUserRepository {
       find_by_email_fn: Arc<Mutex<dyn FnMut(&str) -> Result<User> + Send>>,
   }

   impl MockUserRepository {
       pub fn with_find_by_email(
           mut self,
           f: impl FnMut(&str) -> Result<User> + Send + 'static,
       ) -> Self {
           self.find_by_email_fn = Arc::new(Mutex::new(f));
           self
       }
   }
   ```

3. Prefer explicit mocks over record/replay
4. Document mock behavior
5. Reset mocks between tests
