# Mutation Testing Guide

## Overview

Mutation testing is a method to evaluate the quality of your test suite by introducing small changes (mutations) to your code and verifying that your tests can detect these changes. This helps identify weak spots in your test coverage and improve test effectiveness.

## Key Concepts

1. **Mutations**
   - Small code changes
   - Simulated bugs
   - Boundary condition modifications

2. **Mutation Operators**
   - Arithmetic operators
   - Logical operators
   - Control flow changes
   - Boundary conditions

3. **Mutation Score**
   - Percentage of detected mutations
   - Test suite effectiveness
   - Coverage quality metric

## Using cargo-mutants

### Basic Configuration

```toml
# .mutants.toml
[mutants]
timeout = 300
jobs = 4

paths = [
    "src/core",
    "src/api",
    "src/auth"
]

exclude = [
    "**/tests/*",
    "**/benches/*"
]

operators = [
    "arithmetic",
    "comparison",
    "control_flow",
    "function_calls"
]
```

### Running Mutation Tests

```bash
# Install cargo-mutants
cargo install cargo-mutants

# Run mutation testing
cargo mutants

# Generate HTML report
cargo mutants --reporter html
```

## Mutation Categories

### 1. Arithmetic Mutations

```rust
// Original code
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Mutations
fn add(a: i32, b: i32) -> i32 {
    a - b    // Mutation 1
    a * b    // Mutation 2
    a / b    // Mutation 3
}
```

### 2. Comparison Mutations

```rust
// Original code
fn is_valid_age(age: u8) -> bool {
    age >= 18 && age <= 120
}

// Mutations
fn is_valid_age(age: u8) -> bool {
    age > 18 && age <= 120   // Mutation 1
    age >= 18 || age <= 120  // Mutation 2
    age >= 18 && age < 120   // Mutation 3
}
```

### 3. Control Flow Mutations

```rust
// Original code
fn process_list(items: &[i32]) -> Vec<i32> {
    let mut result = Vec::new();
    for item in items {
        if *item > 0 {
            result.push(*item);
        }
    }
    result
}

// Mutations
fn process_list(items: &[i32]) -> Vec<i32> {
    let mut result = Vec::new();
    for item in items {
        if *item >= 0 {          // Mutation 1
            result.push(*item);
        }
        if *item > 0 {           // Mutation 2
            continue;            // Mutation 3
        }
    }
    result
}
```

## Advanced Techniques

### 1. Custom Mutation Operators

```rust
use cargo_mutants::prelude::*;

#[derive(MutationOperator)]
struct CustomOperator;

impl Operator for CustomOperator {
    fn mutate(&self, expr: &Expr) -> Option<Expr> {
        // Implementation
    }
}
```

### 2. Mutation Filtering

```rust
// .mutants.toml
[mutants.filter]
paths = ["src/critical"]
min_coverage = 90
exclude_patterns = ["*_generated.rs"]
```

## Best Practices

1. **Test Selection**
   - Focus on critical code paths
   - Prioritize high-impact areas
   - Consider performance impact

2. **Mutation Strategy**
   - Choose appropriate operators
   - Set reasonable timeouts
   - Balance coverage and speed

3. **Result Analysis**
   - Review surviving mutations
   - Identify test gaps
   - Improve test cases

4. **Performance**
   - Use parallel execution
   - Filter unnecessary mutations
   - Optimize test runtime

## Running Tests

1. Basic mutation testing:

   ```bash
   cargo mutants
   ```

2. With specific configuration:

   ```bash
   cargo mutants --config custom-mutants.toml
   ```

3. Generate reports:

   ```bash
   cargo mutants --reporter json --output mutations.json
   ```

## Common Patterns

### Test Improvement

```rust
// Before mutation testing
#[test]
fn test_process_positive() {
    let result = process_numbers(&[1, 2, 3]);
    assert_eq!(result.len(), 3);
}

// After mutation testing
#[test]
fn test_process_positive() {
    let result = process_numbers(&[1, 2, 3]);
    assert_eq!(result.len(), 3);
    assert_eq!(result, vec![1, 2, 3]);  // Stronger assertion
}
```

### Mutation Resistance

```rust
// Mutation-prone code
fn validate_range(value: i32) -> bool {
    value >= 0 && value <= 100
}

// Mutation-resistant code
fn validate_range(value: i32) -> bool {
    let min = 0;
    let max = 100;
    value >= min && value <= max
}
```

## Integration with CI/CD

### 1. GitHub Actions

```yaml
name: Mutation Testing

on: [push, pull_request]

jobs:
  mutants:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Install cargo-mutants
        run: cargo install cargo-mutants
      - name: Run mutation tests
        run: cargo mutants --reporter github
```

### 2. Quality Gates

```rust
// mutation-check.rs
fn main() {
    let report = parse_mutation_report("mutations.json");
    if report.score < 0.80 {
        std::process::exit(1);
    }
}
```

## Further Reading

- [Mutation Testing Book](https://mutation-testing.org/)
- [cargo-mutants Documentation](https://docs.rs/cargo-mutants)
- [Test Quality Metrics](../testing/METRICS.md)
- [CI Integration Guide](../ci/MUTATION_TESTING.md)
