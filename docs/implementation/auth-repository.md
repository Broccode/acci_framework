# PostgreSQL Repository Implementation Plan

## Overview

This document outlines the implementation plan for the PostgreSQL-based user repository in the authentication system.

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

- Create migration system structure
- Implement up/down migrations
- Version tracking table
- Migration CLI commands
- Test migration process
- Schema validation

### Phase 2: Repository Structure

- Implement PostgresUserRepository
- Connection pool management
- Error mapping system
- Query builder integration
- Logging and metrics setup
- Rate limiting implementation
- Audit logging system

### Phase 3: Core Operations

- Create user with conflict handling
- Read user by various criteria
- Update user with optimistic locking
- Soft delete implementation
- Status management operations
- Password reset flow
- Email verification flow
- Rate limiting for sensitive operations

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
