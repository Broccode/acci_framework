# PostgreSQL Repository Implementation Plan

## Overview

This document outlines the implementation plan for the PostgreSQL-based user repository in the authentication system.

## Current Status

The database infrastructure and repository foundation have been implemented, including:

- Database migration system with up/down migrations
- Version tracking table implementation
- Basic PostgresUserRepository structure
- Connection pool management
- Basic error handling and mapping system
- Core user operations (create, read, update)
- Integration with the authentication service

Implementation of advanced features is in progress:

- Rate limiting for sensitive operations
- Audit logging system
- Email verification flow
- Password reset functionality

## Database Schema

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    last_login TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    verification_token UUID,
    verification_token_expires_at TIMESTAMPTZ,
    reset_token UUID,
    reset_token_expires_at TIMESTAMPTZ,
    failed_login_attempts INTEGER NOT NULL DEFAULT 0,
    last_failed_login_at TIMESTAMPTZ
);

-- Indices for common queries
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_active ON users(is_active) WHERE is_active = true;
CREATE INDEX idx_users_verification_token ON users(verification_token) WHERE verification_token IS NOT NULL;
CREATE INDEX idx_users_reset_token ON users(reset_token) WHERE reset_token IS NOT NULL;

-- Audit log table
CREATE TABLE user_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    action VARCHAR(50) NOT NULL,
    details JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_action CHECK (action IN (
        'REGISTRATION',
        'LOGIN_SUCCESS',
        'LOGIN_FAILED',
        'PASSWORD_RESET_REQUEST',
        'PASSWORD_RESET_SUCCESS',
        'EMAIL_VERIFICATION_REQUEST',
        'EMAIL_VERIFICATION_SUCCESS',
        'ACCOUNT_DEACTIVATED',
        'ACCOUNT_ACTIVATED'
    ))
);

CREATE INDEX idx_audit_user_id ON user_audit_log(user_id);
CREATE INDEX idx_audit_action ON user_audit_log(action);
CREATE INDEX idx_audit_created_at ON user_audit_log(created_at);
```

## Implementation Phases

### Phase 1: Database Migration System

- [x] Create migration system structure
- [x] Implement up/down migrations
- [x] Version tracking table
- [x] Migration CLI commands
- [x] Test migration process
- [x] Schema validation

### Phase 2: Repository Structure

- [x] Implement PostgresUserRepository
- [x] Connection pool management
- [x] Error mapping system
- [x] Query builder integration
- [ ] Logging and metrics setup
- [ ] Rate limiting implementation
- [ ] Audit logging system

### Phase 3: Core Operations

- [x] Create user with conflict handling
- [x] Read user by various criteria
- [x] Update user with optimistic locking
- [ ] Soft delete implementation
- [ ] Status management operations
- [ ] Password reset flow
- [ ] Email verification flow
- [ ] Rate limiting for sensitive operations

## Technical Details

### Dependencies

```toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "time", "json", "ipnetwork"] }
sea-query = "0.30"
tokio = { version = "1.36", features = ["full"] }
governor = "0.6"  # Rate limiting
secrecy = "0.8"   # Secure secret handling
lettre = "0.11"   # Email sending
validator = "0.16" # Input validation
```

### Repository Structure

```rust
pub struct PostgresUserRepository {
    pool: PgPool,
    query_builder: QueryBuilder,
    rate_limiter: Arc<RateLimiter>,
    email_client: Arc<EmailClient>,
    config: RepositoryConfig,
}

impl PostgresUserRepository {
    pub async fn new(config: DatabaseConfig) -> Result<Self, Error> {
        // Implementation
    }

    // Rate limiting
    async fn check_rate_limit(&self, action: RateLimitAction, key: &str) -> Result<(), Error> {
        // Implementation
    }

    // Audit logging
    async fn log_audit_event(&self, event: AuditEvent) -> Result<(), Error> {
        // Implementation
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    // Implementation of trait methods
}
```

### Error Handling

- Map SQLx errors to domain errors
- Handle unique constraint violations
- Connection pool error handling
- Transaction management
- Rate limit error handling
- Audit logging errors
- Email sending errors

## Testing Strategy

### Integration Tests

- Use TestContainers for PostgreSQL
- Isolated test database
- Parallel test execution
- Transaction rollback for tests
- Rate limit testing
- Audit log verification
- Email verification testing

### Security Tests

- Password hash verification
- Token generation/validation
- Rate limiting effectiveness
- SQL injection prevention
- Input validation
- Audit log completeness

### Performance Tests

- Connection pool optimization
- Query performance benchmarks
- Load testing scenarios
- Rate limiting impact
- Concurrent operations

## API Integration

### Axum Integration

```rust
// User repository API integration
pub fn create_auth_router() -> Router {
    Router::new()
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/logout", post(handlers::logout))
        .route("/api/auth/reset-password", post(handlers::reset_password))
        .route("/api/auth/verify-email/:token", get(handlers::verify_email))
        .layer(middleware::from_fn(extract_auth_session))
        .layer(middleware::from_fn(enforce_rate_limits))
        .layer(middleware::from_fn(log_requests))
}
```

### Middleware Implementation

The API includes several middleware layers:

- **Authentication Session Extraction**: Extracts session details from requests
- **Rate Limiting**: Prevents abuse through IP-based and user-based rate limits
- **Request Logging**: Logs all requests for auditing and monitoring
- **Error Handling**: Consistent error responses across all endpoints
- **Validation**: Input validation using validator crate

### Request Validation

All incoming requests are validated before processing:

```rust
#[derive(Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    #[validate(must_match = "password", message = "Passwords do not match")]
    pub password_confirmation: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, ApiError> {
    payload.validate()?;
    
    // Process registration
    // ...
}
```

### Response Formatting

API responses follow a consistent format:

```rust
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub meta: Option<HashMap<String, Value>>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: None,
        }
    }
    
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            meta: None,
        }
    }
}
```

## Monitoring & Observability

### Metrics

- Query execution times
- Connection pool statistics
- Error rates and types
- Rate limit hits
- Authentication success/failure rates
- Password reset statistics
- Email verification rates

### Logging

- SQL query logging
- Error context logging
- Audit trail logging
- Performance logging
- Rate limit events
- Security events

## Security Considerations

### Data Protection

- Password hash storage
- Email address encryption
- Audit logging
- Access control
- Rate limiting
- Token expiration
- Secure secret handling

### Database Security

- Connection encryption
- Minimal privileges
- Prepared statements
- Input validation
- Transaction isolation
- Connection timeouts

## Implementation Schedule

1. **Week 1: Foundation**
   - Set up migration system
   - Implement basic repository structure
   - Create test infrastructure
   - Set up security measures

2. **Week 2: Core Operations**
   - Implement CRUD operations
   - Add error handling
   - Implement rate limiting
   - Set up audit logging
   - Write integration tests

3. **Week 3: Advanced Features**
   - Implement password reset
   - Add email verification
   - Set up monitoring
   - Security hardening
   - Performance optimization

## Success Criteria

- All UserRepository trait methods implemented
- Integration tests passing
- Performance benchmarks met
- Security requirements satisfied
- Migration system working
- Error handling complete
- Rate limiting effective
- Audit logging comprehensive
- Email flows working
- Password reset secure

## Future Improvements

- Query caching layer
- Read replicas support
- Sharding capability
- Advanced monitoring
- Automated backups
- OAuth integration
- Two-factor authentication
- Hardware security module integration
- Advanced anomaly detection
- Automated security testing

## Next Steps

Based on the current implementation status, the following next steps are prioritized:

1. **Complete Error Handling**
   - Finish implementing validation error handling
   - Add comprehensive error logging
   - Integrate with monitoring systems

2. **Implement Rate Limiting**
   - Add IP-based rate limiting for login attempts
   - Implement user-based rate limiting for sensitive operations
   - Create configurable rate limit policies

3. **Enhance Audit Logging**
   - Complete the audit logging system for all authentication actions
   - Add detailed context to audit logs
   - Implement audit log querying functionality

4. **Password Reset Flow**
   - Build complete password reset functionality
   - Implement secure token generation and validation
   - Create email notification for password reset requests

5. **Email Verification**
   - Implement email verification flow
   - Add secure token handling
   - Create resend verification functionality

The implementation will continue to follow our security-first approach, with comprehensive testing at each stage.
