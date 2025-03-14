# WebAuthn Implementation for ACCI Framework

**NOTE: This document outlines the planned WebAuthn implementation for the ACCI Framework. The implementation is currently in progress with compatibility issues being addressed for webauthn-rs 0.5.1. This design document serves as a reference for the final implementation.**

## Overview

This document outlines the implementation details for WebAuthn (Web Authentication) support in the ACCI Framework. WebAuthn enables strong, phishing-resistant authentication using public key cryptography via hardware security keys, biometric sensors, or platform authenticators like Windows Hello or Touch ID.

## Architecture

The WebAuthn implementation follows the same layered architecture as the rest of the framework:

1. **Models Layer**: Defines the core data structures for WebAuthn credentials
2. **Repository Layer**: Provides data persistence for WebAuthn credentials
3. **Service Layer**: Implements WebAuthn business logic and operations
4. **API Layer**: Exposes WebAuthn functionality through REST endpoints
5. **Web Layer**: Provides user interface components for WebAuthn operations

## Components

### 1. Models

The `models/webauthn.rs` file defines the following key structures:

- **Credential**: Represents a registered WebAuthn credential with its metadata
- **CredentialID**: Wrapper around the unique identifier for a WebAuthn credential
- **RegisterCredential**: Container for registration attestation data
- **PublicKeyCredential**: Container for authentication assertion data
- **WebAuthnError**: Error types specific to WebAuthn operations

### 2. Repository

The WebAuthn repository layer provides persistence for credential data:

- **WebAuthnRepository**: Trait defining the interface for WebAuthn data operations
- **PostgresWebAuthnRepository**: Implementation of the repository using PostgreSQL

Key operations include:
- Saving new credentials
- Finding credentials by ID or UUID
- Listing all credentials for a user
- Deleting credentials

### 3. Service

The `services/webauthn.rs` file implements the core WebAuthn functionality:

- **WebAuthnService**: Manages authentication flows and credential lifecycle
- **WebAuthnConfig**: Configuration for the WebAuthn implementation

Key capabilities include:
- Registration flow (start_registration/complete_registration)
- Authentication flow (start_authentication/complete_authentication)
- Credential management

### 4. Database Schema

The WebAuthn implementation includes a database migration (`20240224005_create_webauthn_credentials.sql`) that creates the required schema:

```sql
CREATE TABLE webauthn_credentials (
  uuid UUID PRIMARY KEY,
  credential_id TEXT NOT NULL,
  user_id UUID NOT NULL,
  tenant_id UUID NOT NULL,
  name VARCHAR(255) NOT NULL,
  aaguid BYTEA NOT NULL,
  public_key BYTEA NOT NULL,
  counter INTEGER NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  last_used_at TIMESTAMPTZ
);
```

The schema includes multi-tenant support through tenant isolation with row-level security:

```sql
CREATE POLICY tenant_isolation_policy ON webauthn_credentials
  USING (tenant_id::text = current_setting('app.tenant_id', TRUE));
```

## User Flows

### Registration Flow

1. User initiates credential registration
2. Backend generates challenge with WebAuthnService.start_registration()
3. Frontend requests platform/security key to create a credential
4. User performs authentication gesture (touch, PIN, biometric)
5. Attestation data is sent to the backend
6. Backend verifies attestation with WebAuthnService.complete_registration()
7. Credential is stored in the database

### Authentication Flow

1. User initiates authentication with WebAuthn
2. Backend generates challenge with WebAuthnService.start_authentication()
3. Frontend requests platform/security key to sign the challenge
4. User performs authentication gesture
5. Assertion data is sent to the backend
6. Backend verifies assertion with WebAuthnService.complete_authentication()
7. User is authenticated and session is created

## Security Considerations

1. **Replay Protection**: The counter value in credentials prevents replay attacks
2. **Resident Keys**: Support for discoverable credentials enables passwordless flows
3. **User Verification**: Can be configured as required, preferred, or discouraged
4. **Attestation**: Optional support for attestation allows verifying authenticator security properties
5. **Multi-tenancy**: Credentials are isolated by tenant through row-level security

## Browser Support

The WebAuthn implementation supports all modern browsers:

- Chrome/Edge (version 67+)
- Firefox (version 60+)
- Safari (version 13+)
- Mobile browsers with platform authentication support

A feature detection system is implemented to provide graceful fallbacks for unsupported browsers.

## Future Enhancements

1. **User Verification Level**: Add support for different levels based on risk
2. **Conditional UI**: Implement support for conditional UI where available
3. **Cross-Device Usage**: Support for authentication from multiple devices
4. **Trusted Device Lists**: Allow users to see and manage their authenticators
5. **Recovery Mechanism**: Implement backup authentication methods

## Integration Points

1. **User Registration**: Option to add WebAuthn during user registration
2. **Account Settings**: Interface for users to manage their credentials
3. **Login Flow**: Integration with the main authentication workflow
4. **MFA Enforcement**: Policy-based enforcement of WebAuthn as a second factor
5. **Session Management**: Enhanced session security with WebAuthn verification