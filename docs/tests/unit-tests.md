# Unit Testing Guide

## Overview

Unit tests are the foundation of our testing strategy. They verify the correctness of individual functions and components in isolation. These tests are fast, reliable, and provide immediate feedback during development.

## Key Principles

1. **Co-location with Source Code**
   - Tests are placed in the same file as the code being tested
   - Use the `#[cfg(test)]` attribute for test modules

2. **Independence**
   - No external dependencies
   - No database connections
   - No file system operations
   - No network calls

3. **Fast Execution**
   - Tests should complete within milliseconds
   - Immediate feedback during development
   - Support for test-driven development (TDD)

## Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_valid_input_when_processing_then_succeeds() {
        // Arrange
        let input = prepare_test_input();

        // Act
        let result = process_input(input);

        // Assert
        assert_eq!(result, expected_output);
    }
}
```

## Naming Conventions

- Test names should follow the pattern: `given_[condition]_when_[action]_then_[result]`
- Test modules should be named `tests`
- Helper functions should have descriptive names indicating their purpose

## Test Categories

### 1. Function Tests

```rust
#[test]
fn test_add_numbers() {
    assert_eq!(add(2, 2), 4);
}
```

### 2. Error Cases

```rust
#[test]
fn test_division_by_zero() {
    assert!(divide(10, 0).is_err());
}
```

### 3. Edge Cases

```rust
#[test]
fn test_empty_input() {
    let result = process_list(vec![]);
    assert_eq!(result.len(), 0);
}
```

## Async Testing

```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_operation().await;
    assert!(result.is_ok());
}
```

## Test Documentation

```rust
/// Tests the addition of two numbers
///
/// # Examples
///
/// ```
/// let result = add(2, 2);
/// assert_eq!(result, 4);
/// ```
#[test]
fn test_addition() {
    // Test implementation
}
```

## Best Practices

1. **Test Coverage**
   - Aim for high test coverage
   - Test both success and failure paths
   - Include edge cases

2. **Test Independence**
   - Each test should be independent
   - No shared state between tests
   - No test order dependencies

3. **Test Readability**
   - Clear test names
   - Well-structured arrange-act-assert pattern
   - Descriptive error messages

4. **Test Maintenance**
   - Regular review and updates
   - Remove obsolete tests
   - Keep tests simple and focused

## Running Tests

1. Run all tests:

   ```bash
   cargo test
   ```

2. Run specific test:

   ```bash
   cargo test test_name
   ```

3. Show test output:

   ```bash
   cargo test -- --show-output
   ```

## Common Patterns

### Setup Code

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> TestStruct {
        TestStruct::new()
    }

    #[test]
    fn test_operation() {
        let test_struct = setup();
        // Test implementation
    }
}
```

### Test Utilities

```rust
#[cfg(test)]
mod test_utils {
    pub fn create_test_data() -> Vec<TestData> {
        // Create test data
    }
}
```

## Further Reading

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Test Organization](../CONTRIBUTING.md#testing)
- [Integration Testing](integration-tests.md)
