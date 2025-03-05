# API Specification

## Overview

This document defines the comprehensive API structure for the ACCI Framework, detailing both REST and GraphQL endpoints, data models, versioning strategy, and integration guidelines. It serves as the authoritative reference for internal development teams and external integrators working with our platform.

## API Endpoints

### REST API Endpoints

#### Authentication

| Endpoint | Method | Description | Request Body | Response |
|----------|--------|-------------|--------------|----------|
| `/api/auth/login` | POST | Authenticates a user and creates a session | `{ "username": string, "password": string }` | `{ "status": "success", "data": { "token": string, "user_id": string, "expires_at": string } }` |
| `/api/auth/logout` | POST | Terminates the current session | None | `{ "status": "success" }` |
| `/api/auth/register` | POST | Creates a new user account | `{ "username": string, "email": string, "password": string }` | `{ "status": "success", "data": { "user_id": string } }` |
| `/api/auth/refresh` | POST | Refreshes an existing session token | `{ "refresh_token": string }` | `{ "status": "success", "data": { "token": string, "expires_at": string } }` |
| `/api/auth/password/reset` | POST | Initiates password reset flow | `{ "email": string }` | `{ "status": "success" }` |
| `/api/auth/password/change` | POST | Changes user password | `{ "old_password": string, "new_password": string }` | `{ "status": "success" }` |

#### User Management

| Endpoint | Method | Description | Request Body/Params | Response |
|----------|--------|-------------|--------------|----------|
| `/api/users` | GET | Retrieves a list of users | Query params: `limit`, `offset`, `search` | `{ "status": "success", "data": { "users": [ ... ], "total": number } }` |
| `/api/users/{id}` | GET | Retrieves a specific user | Path param: `id` | `{ "status": "success", "data": { "user": { ... } } }` |
| `/api/users` | POST | Creates a new user | `{ "username": string, "email": string, "role": string, ... }` | `{ "status": "success", "data": { "user_id": string } }` |
| `/api/users/{id}` | PUT | Updates a user | Path param: `id`, Body: user fields | `{ "status": "success" }` |
| `/api/users/{id}` | DELETE | Deletes a user | Path param: `id` | `{ "status": "success" }` |
| `/api/users/me` | GET | Gets current user profile | None | `{ "status": "success", "data": { "user": { ... } } }` |

#### Session Management

| Endpoint | Method | Description | Request Body/Params | Response |
|----------|--------|-------------|--------------|----------|
| `/api/sessions` | GET | Lists active sessions | None | `{ "status": "success", "data": { "sessions": [ ... ] } }` |
| `/api/sessions/{id}` | DELETE | Terminates a specific session | Path param: `id` | `{ "status": "success" }` |
| `/api/sessions` | DELETE | Terminates all sessions except current | None | `{ "status": "success", "data": { "terminated_count": number } }` |

### GraphQL API

Our GraphQL API provides a flexible alternative to the REST API, allowing clients to request exactly the data they need.

#### Queries

```graphql
# User queries
query GetUser($id: ID!) {
  user(id: $id) {
    id
    username
    email
    created_at
    last_login
    roles {
      id
      name
    }
  }
}

query GetUsers($limit: Int, $offset: Int, $search: String) {
  users(limit: $limit, offset: $offset, search: $search) {
    total
    users {
      id
      username
      email
    }
  }
}

# Session queries
query GetSessions {
  sessions {
    id
    created_at
    expires_at
    last_active_at
    device_info
    ip_address
    current
  }
}
```

#### Mutations

```graphql
# Authentication mutations
mutation Login($username: String!, $password: String!) {
  login(username: $username, password: $password) {
    token
    user_id
    expires_at
  }
}

mutation Register($input: RegisterInput!) {
  register(input: $input) {
    user_id
  }
}

# User mutations
mutation CreateUser($input: CreateUserInput!) {
  createUser(input: $input) {
    id
    username
    email
  }
}

mutation UpdateUser($id: ID!, $input: UpdateUserInput!) {
  updateUser(id: $id, input: $input) {
    id
    username
    email
  }
}
```

## Data Models

### User Model

```json
{
  "id": "uuid",
  "username": "string (3-100 chars)",
  "email": "string (valid email format)",
  "password_hash": "string (internal only)",
  "first_name": "string (optional)",
  "last_name": "string (optional)",
  "created_at": "datetime (ISO-8601)",
  "updated_at": "datetime (ISO-8601)",
  "last_login": "datetime (ISO-8601, optional)",
  "status": "string (active, inactive, locked)",
  "roles": ["string"],
  "settings": {
    "notification_preferences": {},
    "ui_preferences": {}
  }
}
```

### Session Model

```json
{
  "id": "uuid",
  "user_id": "uuid (ref to User)",
  "token": "string (JWT token, internal only)",
  "created_at": "datetime (ISO-8601)",
  "expires_at": "datetime (ISO-8601)",
  "last_active_at": "datetime (ISO-8601)",
  "ip_address": "string (IP address)",
  "user_agent": "string (browser/client info)",
  "device_fingerprint": "string (unique device ID)",
  "is_mfa_completed": "boolean"
}
```

### Error Model

```json
{
  "status": "error",
  "message": "string (human-readable error message)",
  "code": "string (machine-readable error code)",
  "request_id": "string (unique identifier for the request)",
  "details": {
    // Optional additional error details
    "fields": {
      "fieldName": ["error message 1", "error message 2"]
    }
  }
}
```

## API Response Format

All API responses follow a consistent format:

### Success Response

```json
{
  "status": "success",
  "data": {
    // Response data specific to the endpoint
  },
  "request_id": "unique-request-id"
}
```

### Error Response

```json
{
  "status": "error",
  "message": "Error message",
  "code": "ERROR_CODE",
  "request_id": "unique-request-id",
  "details": {
    // Optional additional error details
  }
}
```

## Common Error Codes

| Code | Description | HTTP Status |
|------|-------------|-------------|
| `VALIDATION_ERROR` | Input validation failed | 400 |
| `INVALID_CREDENTIALS` | Login credentials are invalid | 401 |
| `TOKEN_EXPIRED` | Authentication token has expired | 401 |
| `UNAUTHORIZED` | User is not authorized for this action | 403 |
| `RESOURCE_NOT_FOUND` | Requested resource does not exist | 404 |
| `RATE_LIMIT_EXCEEDED` | Too many requests | 429 |
| `INTERNAL_SERVER_ERROR` | Server encountered an unexpected error | 500 |

## Versioning Strategy

### REST API Versioning

We use URL-based versioning to maintain backward compatibility:

```
/api/v1/users      # Version 1 of users API
/api/v2/users      # Version 2 of users API
```

When implementing a new version:
1. The previous version remains available and unchanged
2. New features and breaking changes are introduced in the new version
3. Version deprecation is announced at least 6 months in advance

### GraphQL API Versioning

For our GraphQL API, we use schema evolution rather than explicit versions:

1. Field and type additions are non-breaking and can be added anytime
2. Fields and types are never removed; they are deprecated with `@deprecated` directive
3. Schema changes are documented in the changelog

## Authentication

### JWT Authentication

Our API uses JWT (JSON Web Tokens) for authentication:

1. Client authenticates with username/password and receives a JWT
2. The JWT is included in the `Authorization` header as a bearer token
3. Tokens have a default expiry of 15 minutes
4. Refresh tokens are provided for obtaining new access tokens

Example:

```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### OAuth 2.0 / OpenID Connect

For external integrations, we support OAuth 2.0 with the following flows:

1. **Authorization Code Flow**: For web applications
2. **PKCE Flow**: For mobile/native applications
3. **Client Credentials**: For service-to-service API access

## Rate Limiting

To prevent abuse, our API implements rate limiting:

1. Authentication endpoints: 10 requests per minute per IP
2. Regular API endpoints: 60 requests per minute per user
3. Bulk operations: 10 requests per minute per user

Rate limit headers are included in all responses:

```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 58
X-RateLimit-Reset: 1631022471
```

## Pagination

For endpoints returning multiple items, we use offset-based pagination:

```
GET /api/users?limit=20&offset=40
```

Response includes pagination metadata:

```json
{
  "status": "success",
  "data": {
    "users": [...],
    "pagination": {
      "total": 243,
      "limit": 20,
      "offset": 40,
      "next_offset": 60,
      "prev_offset": 20
    }
  }
}
```

## Data Filtering and Sorting

Endpoints support filtering and sorting via query parameters:

```
GET /api/users?search=john&sort=created_at:desc&status=active
```

## API Documentation

Interactive API documentation is available at `/api/docs` and includes:

1. **Swagger UI**: For REST API exploration and testing
2. **GraphiQL**: For GraphQL query building and testing
3. **Postman Collection**: Available for download

## Integration Guidelines

### Getting Started

1. Register for API access at our developer portal
2. Use the sandbox environment for testing
3. Implement proper error handling and retries
4. Follow rate limit guidelines to avoid service disruption

### Security Best Practices

1. Store client secrets securely
2. Implement proper token handling
3. Use HTTPS for all API requests
4. Validate all user inputs before sending to API
5. Implement proper error handling for security responses

### Webhooks

Our API supports webhooks for event notifications:

1. Register webhook endpoints at `/api/webhooks`
2. Events will be POSTed to your endpoint with a signature header
3. Verify the signature to ensure event authenticity
4. Return 2xx status code to acknowledge receipt

## SDKs and Client Libraries

Official SDKs are available for:

- JavaScript/TypeScript
- Python
- Rust
- Java
- C#

## API Status and Monitoring

Service status and API uptime information is available at `/api/status`.

## Conclusion

This API specification serves as the single source of truth for internal and external developers integrating with the ACCI Framework. All API endpoints are versioned, documented, and follow consistent patterns to ensure long-term compatibility and ease of use.
