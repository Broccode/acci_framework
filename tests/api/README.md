# API Tests for ACCI Framework

This directory contains comprehensive tests for the API components of the ACCI Framework. The tests are designed to verify the functionality of the API endpoints, middleware, validation, and router components.

## Test Structure

The API tests are organized into several categories:

1. **Auth Handler Tests** (`auth_handler_test.rs`): Tests for the authentication handlers, including login, registration, and token validation.
2. **Router Tests** (`router_test.rs`): Tests for the API router configuration, including endpoint routing and base path handling.
3. **Middleware Tests** (`middleware_test.rs`): Tests for middleware components, including error handling and request logging.
4. **Validation Tests** (`validation_test.rs`): Tests for request validation functionality, ensuring proper validation of user input.

## Running the Tests

To run the API tests, use the following commands:

```bash
# Run all API tests
cargo test -p acci_tests --test api

# Run a specific test
cargo test -p acci_tests --test api::auth_handler_test::test_successful_login
```

## Test Categories

### Auth Handler Tests

- Unit tests for individual handlers
- End-to-end tests for complete authentication flows
- Error handling and validation tests

### Router Tests

- Endpoint configuration and routing
- Base path handling
- HTTP method handling
- Error responses for invalid routes

### Middleware Tests

- Error handling middleware
- Request logging middleware
- Middleware chaining
- Error transformation

### Validation Tests

- JSON payload validation
- Request parameter validation
- Validation error formatting

## Test Coverage

The tests are designed to provide comprehensive coverage of the API functionality:

- **Success paths**: Verifying that valid requests receive appropriate responses
- **Error paths**: Ensuring proper error handling for invalid inputs
- **Edge cases**: Testing boundary conditions and unexpected inputs
- **Integration**: Testing how components work together

## Dependencies

The tests utilize several testing utilities and mocks:

- **Mockall**: For creating mock implementations of services
- **Tower**: For testing HTTP services
- **Axum utilities**: For simulating HTTP requests and responses

## Future Improvements

- Performance tests for API endpoints
- Security tests for authentication mechanisms
- Load testing for concurrency handling
- Property-based tests for validation