# Integration Testing Guide

## Overview

Integration tests verify the interaction between multiple components of the system. These tests ensure that different parts of the application work together correctly under real-world conditions.

## Key Principles

1. **Test Location**
   - All integration tests are located in the `/tests` directory
   - Clear separation from unit tests
   - Organized by feature or component

2. **External Dependencies**
   - Use of containerized dependencies via `testcontainers-rs`
   - Controlled network access
   - Isolated test databases
   - Mocked external services when appropriate

3. **Test Isolation**
   - Each test suite runs in isolation
   - Clean environment for each test
   - No interference between tests

## Directory Structure

```
/tests
├── src/
│   ├── helpers/       # Shared test utilities
│   ├── fixtures/      # Test data
│   └── mocks/         # Mock implementations
├── api/               # API integration tests
├── auth/              # Authentication tests
├── database/          # Database integration tests
└── e2e/              # End-to-end tests
```

## Test Setup

### Database Tests

```rust
use testcontainers::*;

#[tokio::test]
async fn test_database_integration() {
    let docker = clients::Cli::default();
    let postgres = docker.run(images::postgres::Postgres::default());
    
    let connection_string = format!(
        "postgres://postgres:postgres@localhost:{}/postgres",
        postgres.get_host_port_ipv4(5432)
    );
    
    let db = Database::connect(&connection_string).await?;
    // Test implementation
}
```

### API Tests

```rust
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn test_api_integration() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/api/resource"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;
    
    let client = ApiClient::new(mock_server.uri());
    let response = client.get_resource().await?;
    assert_eq!(response.status(), 200);
}
```

## Test Categories

### 1. Component Integration

```rust
#[tokio::test]
async fn test_auth_with_database() {
    let db = setup_test_database().await?;
    let auth_service = AuthService::new(db);
    
    let result = auth_service
        .authenticate("user", "password")
        .await?;
    
    assert!(result.is_authenticated());
}
```

### 2. API Integration

#### Auth Flow Integration Test

We've implemented comprehensive tests for the API layer, focusing on the authentication flow:

```rust
#[tokio::test]
async fn test_e2e_auth_flow() {
    // Setup mock services for a complete auth flow:
    // 1. Register a new user
    // 2. Login with that user
    // 3. Validate the token
    
    let mut mock_user_service = MockUserService::new();
    let user_id = Uuid::new_v4();
    let email = "new_user@example.com".to_string();
    let session_token = "generated_token_123".to_string();
    
    // Setup registration mock
    mock_user_service
        .expect_register()
        .withf(move |create_user| {
            create_user.email == "new_user@example.com" && create_user.password == "secure_pass"
        })
        .returning(move |_| {
            Ok(User {
                id: user_id,
                email: email.clone(),
                // Other fields...
            })
        });
    
    // Setup login mock
    mock_user_service
        .expect_login()
        .returning(move |_, _, _, _, _, _| {
            Ok(LoginResult {
                user: login_user,
                session_token: session_token.clone(),
            })
        });
    
    // Setup token validation mock
    mock_user_service
        .expect_validate_session()
        .returning(move |_| {
            Ok(Some(User { /* ... */ }))
        });
    
    let app = create_test_router(Arc::new(mock_user_service));
    
    // 1. Register a new user
    let register_response = app.clone()
        .oneshot(register_request)
        .await
        .unwrap();
    assert_eq!(register_response.status(), StatusCode::CREATED);
    
    // 2. Login with that user
    let login_response = app.clone()
        .oneshot(login_request)
        .await
        .unwrap();
    assert_eq!(login_response.status(), StatusCode::OK);
    
    // 3. Validate the token
    let validate_response = app
        .oneshot(validate_request)
        .await
        .unwrap();
    assert_eq!(validate_response.status(), StatusCode::OK);
}
```

#### Middleware Integration Test

We've also implemented tests for the middleware components:

```rust
#[tokio::test]
async fn test_middleware_transforms_client_error() {
    let app = create_test_app_with_error_handling();
    
    let request = Request::builder()
        .uri("/error/400")
        .body(Body::empty())
        .unwrap();
        
    let response = app
        .oneshot(request)
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let json = extract_json_body(response).await;
    
    // Verify standardized error format
    assert_eq!(json["status"], "error");
    assert_eq!(json["code"], "BAD_REQUEST");
    assert!(json["message"].is_string());
    assert!(json["request_id"].is_string());
}
```

#### Router Integration Test

And tests for the API router configuration:

```rust
#[tokio::test]
async fn test_router_auth_endpoints_exist() {
    let (app_state, config) = create_test_dependencies();
    let router = ApiRouter::new(config.clone());
    let app = router.create_router_with_state(app_state);
    
    // Test login endpoint - expect method not allowed for GET
    let login_response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/login")
                .method(Method::GET) // Should be POST
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    
    // Assert method not allowed (since we used GET)
    assert_eq!(login_response.status(), StatusCode::METHOD_NOT_ALLOWED);
    
    // Similar tests for register and validate-token endpoints...
}
```

### 3. Database Integration

```rust
#[tokio::test]
async fn test_database_operations() {
    let db = setup_test_database().await?;
    
    // Create record
    let id = db.create_record(&new_record).await?;
    
    // Verify record
    let record = db.get_record(id).await?;
    assert_eq!(record.field, expected_value);
}
```

## Mocking Strategies

### 1. HTTP Services

```rust
#[tokio::test]
async fn test_external_service() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/external/api"))
        .and(json_body(expected_request))
        .respond_with(json_response(expected_response))
        .mount(&mock_server)
        .await;
    
    // Test implementation
}
```

### 2. Database Mocking

```rust
#[tokio::test]
async fn test_with_mock_database() {
    let mock_db = MockDatabase::new()
        .expect_query()
        .with(eq("SELECT * FROM users"))
        .returning(|_| Ok(mock_users()));
    
    let service = Service::new(mock_db);
    // Test implementation
}
```

## Best Practices

1. **Test Data Management**
   - Use fixtures for consistent test data
   - Clean up test data after each test
   - Use meaningful test data names

2. **Error Handling**
   - Test error conditions
   - Verify error responses
   - Test timeout scenarios

3. **Async Testing**
   - Proper handling of async operations
   - Test cancellation scenarios
   - Verify timeouts

4. **Container Management**
   - Efficient container lifecycle
   - Resource cleanup
   - Parallel test execution

## Running Tests

1. Run all integration tests:

   ```bash
   cargo test --test '*'
   ```

2. Run specific test suite:

   ```bash
   cargo test --test api_integration
   ```

3. Run with logging:

   ```bash
   RUST_LOG=debug cargo test --test '*'
   ```

## Common Patterns

### Test App Setup

```rust
async fn setup_test_app() -> TestApp {
    let db = setup_test_database().await?;
    let auth = setup_test_auth().await?;
    let api = setup_test_api(db.clone(), auth.clone()).await?;
    
    TestApp {
        db,
        auth,
        api,
        client: reqwest::Client::new(),
    }
}
```

### Cleanup

```rust
impl Drop for TestApp {
    fn drop(&mut self) {
        // Cleanup resources
        self.db.cleanup();
        self.auth.cleanup();
    }
}
```

## Further Reading

- [Testcontainers Documentation](https://docs.rs/testcontainers)
- [WireMock Documentation](https://docs.rs/wiremock)
- [Unit Testing Guide](unit-tests.md)
- [Test Configuration](test-configuration.md)
