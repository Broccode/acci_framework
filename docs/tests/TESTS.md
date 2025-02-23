# Test Structure and Standards

## Directory Structure

```
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

## Test Categories

### Unit Tests

- Location: MUST be in the same file as the code being tested or in a `tests.rs` file next to the module
- Purpose: Test individual functions and components in isolation
- Dependencies: NO external services, NO mocks
- Example location: `crates/core/src/database/tests.rs`

### Integration Tests

- Location: MUST be in the `/tests` directory
- Purpose: Test interaction between multiple components
- Dependencies: MAY use external services via containers, MAY use mocks
- Example location: `/tests/api/auth_tests.rs`

### End-to-End Tests

- Location: MUST be in `/tests/e2e`
- Purpose: Test complete workflows
- Dependencies: MAY use all required services
- Example location: `/tests/e2e/workflows_tests.rs`

## Test Implementation

### Unit Tests

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

### Integration Tests

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

## Critical Rules

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

## Migration Guide

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
