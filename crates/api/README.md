# ACCI Framework API Module

The API module implements the basic infrastructure and routing logic for RESTful APIs in the ACCI Framework.

## Error Handling in the API Module

### Standardized API Errors

The API module uses a standardized error format for all responses:

```json
{
  "status": "error",
  "message": "The error description",
  "code": "ERROR_CODE",
  "request_id": "req-12345"
}
```

For validation errors, additional details are provided:

```json
{
  "status": "error",
  "message": "Validation error: email: Invalid email format, password: Password must be at least 8 characters long",
  "code": "VALIDATION_ERROR",
  "request_id": "req-12345",
  "errors": {
    "email": ["Invalid email format"],
    "password": ["Password must be at least 8 characters long"]
  }
}
```

### Error Types

| HTTP Status | Code | Description |
|-------------|------|-------------|
| 400 | INVALID_REQUEST | General error for invalid requests |
| 400 | INVALID_JSON | Invalid JSON format in the request |
| 401 | AUTHENTICATION_REQUIRED | Authentication required |
| 403 | AUTHORIZATION_ERROR | No permission for the requested resource |
| 404 | RESOURCE_NOT_FOUND | The requested resource was not found |
| 422 | VALIDATION_ERROR | The request contains invalid data |
| 429 | RATE_LIMIT_EXCEEDED | Too many requests were sent |
| 500 | INTERNAL_SERVER_ERROR | An internal server error occurred |

### Using Error Handling

There are several ways to use the API error handling:

1. **Using ApiError Structure**:

```rust
use acci_api::response::ApiError;
use axum::http::StatusCode;

// In a handler:
fn error_example() -> impl IntoResponse {
    let request_id = generate_request_id();
    ApiError::new(
        StatusCode::BAD_REQUEST,
        "Invalid parameters",
        "INVALID_PARAMETERS",
        request_id
    )
}
```

2. **Using API Error Helpers**:

```rust
use acci_api::response::ApiError;

// Predefined error types
fn not_found_example() -> impl IntoResponse {
    let request_id = generate_request_id();
    ApiError::not_found_error("User", request_id)
}
```

3. **Error Handling Middleware**:

The API module contains an error handling middleware that automatically standardizes error responses.

```rust
use axum::{Router, middleware::from_fn};
use acci_api::middleware::error_handling::error_handling_middleware;

let app = Router::new()
    // ... add routes ...
    .layer(from_fn(error_handling_middleware));
```

4. **Validation with error_handling**:

```rust
use acci_api::validation::validate_json_payload;
use axum::Json;

async fn create_item(
    Json(payload): Json<CreateItemRequest>
) -> impl IntoResponse {
    // Validation with detailed error reports
    let validated = validate_json_payload(Json(payload)).await?;
    
    // Access validated data with validated.0
    let item = create_item_in_db(validated.0).await?;
    
    // Return success response
    // ...
}
```

### Metrics and Logging

The API error handling automatically records error metrics and creates log entries:

1. **Error Metrics**: `api.errors.client`, `api.errors.server`, `api.validation.errors`
2. **Logging**: Depending on the error type, errors are logged as ERROR, WARN, or INFO
3. **Request IDs**: Each error receives a unique request ID for traceability

### Extensions

With the feature flag `extended_errors`, additional error details can be included in the response:

```toml
[dependencies]
acci_api = { version = "0.1.0", features = ["extended_errors"] }
```
