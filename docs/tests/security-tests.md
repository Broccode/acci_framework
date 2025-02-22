# Security Testing Guide

## Overview

Security testing is essential for identifying vulnerabilities and ensuring our application meets security requirements. This guide covers various security testing approaches, tools, and best practices for maintaining application security.

## Key Concepts

1. **Static Analysis**
   - Code scanning
   - Dependency auditing
   - SAST (Static Application Security Testing)

2. **Dynamic Analysis**
   - Penetration testing
   - Fuzzing
   - DAST (Dynamic Application Security Testing)

3. **Compliance Testing**
   - Security standards verification
   - Regulatory compliance checks
   - Security policy enforcement

## Static Analysis Tools

### Cargo Audit

```bash
# Check for known vulnerabilities in dependencies
cargo audit

# Update advisory database
cargo audit update

# Generate report
cargo audit --json > security-audit.json
```

### Clippy Security Lints

```rust
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]

// Security-specific lints
#![deny(clippy::missing_safety_doc)]
#![deny(clippy::undocumented_unsafe_blocks)]
```

## Test Categories

### 1. Input Validation Tests

```rust
#[test]
fn test_sql_injection_prevention() {
    let input = "'; DROP TABLE users; --";
    let query = sanitize_sql_input(input);
    assert!(!query.contains(';'));
    assert!(!query.contains("DROP"));
}
```

### 2. Authentication Tests

```rust
#[tokio::test]
async fn test_password_hashing() {
    let password = "secure_password123";
    let hash = hash_password(password).await?;
    
    // Verify hash is not plaintext
    assert_ne!(password, hash);
    
    // Verify hash validation
    assert!(verify_password(password, &hash).await?);
}
```

### 3. Authorization Tests

```rust
#[test]
fn test_rbac_permissions() {
    let user = User::new_with_role(Role::Standard);
    
    assert!(user.can_read());
    assert!(!user.can_admin());
    
    let admin = User::new_with_role(Role::Admin);
    assert!(admin.can_admin());
}
```

## Fuzzing Tests

### 1. Basic Fuzzing

```rust
use afl::fuzz;

#[cfg(fuzzing)]
fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(s) = std::str::from_utf8(data) {
            let _ = parse_user_input(s);
        }
    });
}
```

### 2. Structure-Aware Fuzzing

```rust
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct RequestData {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
}

#[test]
fn fuzz_http_request() {
    let mut fuzzer = arbitrary::Unstructured::new(&SEED);
    let request = RequestData::arbitrary(&mut fuzzer)?;
    process_request(&request);
}
```

## Penetration Testing

### 1. API Security Tests

```rust
#[tokio::test]
async fn test_api_security() {
    let client = reqwest::Client::new();
    
    // Test CORS
    let response = client
        .get("/api/resource")
        .header("Origin", "https://malicious.com")
        .send()
        .await?;
    
    assert!(!response.headers().contains_key("Access-Control-Allow-Origin"));
}
```

### 2. Session Security

```rust
#[test]
fn test_session_security() {
    let session = create_session();
    
    // Test session token strength
    assert!(session.token().len() >= 32);
    assert!(is_cryptographically_secure(session.token()));
    
    // Test session expiration
    assert!(session.expires_in() <= Duration::from_hours(24));
}
```

## Best Practices

1. **Secure Configuration**
   - Use secure defaults
   - Validate security settings
   - Regular security audits

2. **Data Protection**
   - Encryption at rest
   - Secure data transmission
   - Proper key management

3. **Error Handling**
   - Safe error messages
   - Proper logging
   - No sensitive data exposure

4. **Dependency Management**
   - Regular updates
   - Vulnerability scanning
   - Minimal dependency usage

## Running Tests

1. Run security checks:

   ```bash
   cargo audit && cargo clippy -- -D warnings
   ```

2. Run fuzzing tests:

   ```bash
   cargo afl build
   cargo afl fuzz -i input -o output target/debug/fuzz_target
   ```

3. Run security test suite:

   ```bash
   cargo test --test security_tests
   ```

## Common Patterns

### Secure Password Handling

```rust
use argon2::{self, Config};

fn hash_password(password: &[u8]) -> Result<String> {
    let salt = generate_salt();
    let config = Config::default();
    argon2::hash_encoded(password, &salt, &config)
}
```

### Input Sanitization

```rust
fn sanitize_input(input: &str) -> String {
    let mut sanitized = input.to_owned();
    sanitized.retain(|c| !c.is_control());
    html_escape::encode_text(&sanitized).to_string()
}
```

## Security Monitoring

### 1. Audit Logging

```rust
use tracing::{info, warn, error};

fn log_security_event(event: SecurityEvent) {
    info!(
        event_type = %event.type_str(),
        user = %event.user_id,
        "Security event occurred"
    );
}
```

### 2. Intrusion Detection

```rust
fn detect_suspicious_activity(request: &Request) -> bool {
    let rules = load_security_rules();
    rules.iter().any(|rule| rule.matches(request))
}
```

## Further Reading

- [OWASP Rust Security Guidelines](https://owasp.org/rust-security-framework/)
- [Rust Security Working Group](https://github.com/rust-secure-code/wg)
- [Common Security Tests](../security/COMMON_TESTS.md)
- [Security Policies](../security/POLICIES.md)
