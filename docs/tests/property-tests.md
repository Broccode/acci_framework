# Property-Based Testing Guide

## Overview

Property-based testing is a testing methodology where instead of writing individual test cases, you define properties that your code should satisfy and let the testing framework generate test cases automatically. This approach can find edge cases that you might not think of when writing traditional unit tests.

## Key Concepts

1. **Properties**
   - Invariants that should hold true for all inputs
   - Relationships between inputs and outputs
   - State transitions and their effects

2. **Generators**
   - Automatic test data generation
   - Custom data type generation
   - Shrinking for minimal failing cases

3. **Test Cases**
   - Automatically generated inputs
   - Multiple test runs with different values
   - Reproducible failures

## Using Proptest

### Basic Example

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_addition_commutative(a in 0..100i32, b in 0..100i32) {
        assert_eq!(a + b, b + a);
    }
}
```

### Custom Types

```rust
#[derive(Debug)]
struct User {
    name: String,
    age: u8,
}

prop_compose! {
    fn arbitrary_user()(
        name in "[A-Za-z]{1,10}",
        age in 0..120u8
    ) -> User {
        User { name, age }
    }
}

proptest! {
    #[test]
    fn test_user_creation(user in arbitrary_user()) {
        assert!(user.age <= 120);
        assert!(!user.name.is_empty());
    }
}
```

## Test Categories

### 1. Numeric Properties

```rust
proptest! {
    #[test]
    fn test_absolute_value(x in -1000i32..1000i32) {
        let abs_x = x.abs();
        assert!(abs_x >= 0);
        assert_eq!(abs_x.abs(), abs_x);
    }
}
```

### 2. String Properties

```rust
proptest! {
    #[test]
    fn test_string_reversal(s in ".*") {
        let reversed = s.chars().rev().collect::<String>();
        let double_reversed = reversed.chars().rev().collect::<String>();
        assert_eq!(s, double_reversed);
    }
}
```

### 3. Collection Properties

```rust
proptest! {
    #[test]
    fn test_vec_sorting(mut vec in prop::collection::vec(0..100i32, 0..100)) {
        vec.sort();
        for i in 1..vec.len() {
            assert!(vec[i-1] <= vec[i]);
        }
    }
}
```

## Advanced Techniques

### 1. State Machine Testing

```rust
#[derive(Debug)]
enum Action {
    Push(i32),
    Pop,
}

proptest! {
    #[test]
    fn test_stack_operations(actions in prop::collection::vec(
        prop_oneof![
            Just(Action::Pop),
            (0..100i32).prop_map(Action::Push)
        ],
        0..100
    )) {
        let mut stack = Vec::new();
        
        for action in actions {
            match action {
                Action::Push(x) => stack.push(x),
                Action::Pop => { stack.pop(); }
            }
            
            // Invariant: stack size is never negative
            assert!(stack.len() >= 0);
        }
    }
}
```

### 2. Async Property Testing

```rust
proptest! {
    #[test]
    fn test_async_operation(input in ".*") {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let result = async_operation(&input).await;
            assert!(result.is_ok());
        });
    }
}
```

## Best Practices

1. **Property Selection**
   - Choose properties that are always true
   - Focus on invariants and relationships
   - Consider edge cases and boundaries

2. **Generator Design**
   - Create focused generators
   - Use appropriate value ranges
   - Consider domain constraints

3. **Test Configuration**
   - Set appropriate test case counts
   - Configure timeout values
   - Handle non-deterministic behavior

4. **Failure Analysis**
   - Examine shrunk test cases
   - Look for patterns in failures
   - Document discovered properties

## Running Tests

1. Run all property tests:

   ```bash
   cargo test
   ```

2. Run with more test cases:

   ```bash
   PROPTEST_CASES=10000 cargo test
   ```

3. Run with debug output:

   ```bash
   RUST_LOG=debug cargo test
   ```

## Common Patterns

### Custom Generators

```rust
prop_compose! {
    fn valid_email()(
        local in "[a-zA-Z0-9._%+-]{1,64}",
        domain in "[a-zA-Z0-9.-]{1,255}"
    ) -> String {
        format!("{}@{}", local, domain)
    }
}
```

### Property Composition

```rust
fn is_sorted<T: Ord>(slice: &[T]) -> bool {
    slice.windows(2).all(|w| w[0] <= w[1])
}

proptest! {
    #[test]
    fn test_sort_idempotent(mut vec in prop::collection::vec(0..100i32, 0..100)) {
        vec.sort();
        assert!(is_sorted(&vec));
        
        let sorted = vec.clone();
        vec.sort();
        assert_eq!(vec, sorted);
    }
}
```

## Further Reading

- [Proptest Documentation](https://docs.rs/proptest)
- [QuickCheck for Rust](https://docs.rs/quickcheck)
- [Property-Based Testing in Practice](https://blog.rust-lang.org/2020/01/07/proptest.html)
- [Testing Strategies Guide](../TESTING.md)
