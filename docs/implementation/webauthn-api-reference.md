# WebAuthn API Reference

## Introduction

This document provides a reference for the WebAuthn API implementation in the ACCI Framework. It covers both server-side APIs and client-side JavaScript interfaces for implementing WebAuthn functionality.

## Server-Side API Endpoints

### Registration Flow

#### Start Registration

Initiates the WebAuthn registration process by generating registration options.

**Endpoint**: `POST /api/auth/webauthn/register/start`

**Request**:
```json
{
  "user_id": "00000000-0000-0000-0000-000000000000"
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "publicKey": {
      "challenge": "base64url-encoded-challenge",
      "rp": {
        "name": "Example App",
        "id": "example.com"
      },
      "user": {
        "id": "base64url-encoded-user-id",
        "name": "user@example.com",
        "displayName": "User Name"
      },
      "pubKeyCredParams": [
        { "type": "public-key", "alg": -7 },
        { "type": "public-key", "alg": -257 }
      ],
      "timeout": 60000,
      "attestation": "direct",
      "excludeCredentials": [],
      "authenticatorSelection": {
        "authenticatorAttachment": "platform",
        "userVerification": "preferred"
      }
    }
  }
}
```

#### Complete Registration

Completes the WebAuthn registration process by verifying the attestation and storing the credential.

**Endpoint**: `POST /api/auth/webauthn/register/complete/:user_id`

**Request**:
```json
{
  "attestation": "{...attestation object as JSON string...}",
  "name": "My Security Key"
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "credential_id": "base64url-encoded-credential-id",
    "name": "My Security Key",
    "created_at": "2023-03-15T10:30:00Z"
  }
}
```

### Authentication Flow

#### Start Authentication

Initiates the WebAuthn authentication process by generating authentication options.

**Endpoint**: `POST /api/auth/webauthn/authenticate/start`

**Request**:
```json
{
  "user_id": "00000000-0000-0000-0000-000000000000"  // Optional, omit for usernameless flows
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "publicKey": {
      "challenge": "base64url-encoded-challenge",
      "timeout": 60000,
      "rpId": "example.com",
      "allowCredentials": [
        {
          "type": "public-key",
          "id": "base64url-encoded-credential-id"
        }
      ],
      "userVerification": "preferred"
    }
  }
}
```

#### Complete Authentication

Completes the WebAuthn authentication process by verifying the assertion.

**Endpoint**: `POST /api/auth/webauthn/authenticate/complete`

**Request**:
```json
{
  "assertion": "{...assertion object as JSON string...}"
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "user_id": "00000000-0000-0000-0000-000000000000",
    "session_token": "jwt-session-token",
    "authenticated": true
  }
}
```

### Credential Management

#### List Credentials

Lists all WebAuthn credentials for a user.

**Endpoint**: `GET /api/auth/webauthn/credentials`

**Response**:
```json
{
  "success": true,
  "data": {
    "credentials": [
      {
        "id": "credential-uuid",
        "name": "My Security Key",
        "created_at": "2023-03-15T10:30:00Z",
        "last_used": "2023-03-16T14:45:00Z",
        "authenticator_type": "Security Key (FIDO2)"
      }
    ]
  }
}
```

#### Delete Credential

Deletes a WebAuthn credential.

**Endpoint**: `DELETE /api/auth/webauthn/credentials/:credential_id`

**Response**:
```json
{
  "success": true,
  "data": {
    "message": "Credential deleted successfully"
  }
}
```

## Client-Side JavaScript API

### Registration

```javascript
/**
 * Starts the WebAuthn registration process
 * @param {Object} options - Registration options from the server
 * @returns {Promise<Credential>} - The newly created credential
 */
async function startRegistration(options) {
  // Convert base64url challenge to ArrayBuffer
  options.publicKey.challenge = base64UrlToArrayBuffer(options.publicKey.challenge);
  options.publicKey.user.id = base64UrlToArrayBuffer(options.publicKey.user.id);
  
  // Convert excluded credentials if present
  if (options.publicKey.excludeCredentials) {
    options.publicKey.excludeCredentials = options.publicKey.excludeCredentials.map(cred => {
      return {
        ...cred,
        id: base64UrlToArrayBuffer(cred.id)
      };
    });
  }
  
  // Create credential with the browser's API
  return await navigator.credentials.create({
    publicKey: options.publicKey
  });
}

/**
 * Prepares credential for sending to server
 * @param {Credential} credential - The credential from navigator.credentials.create
 * @param {string} name - User-friendly name for this credential
 * @returns {Object} - Formatted credential for the server
 */
function prepareRegistrationCredential(credential, name) {
  return {
    attestation: JSON.stringify({
      id: credential.id,
      rawId: arrayBufferToBase64Url(credential.rawId),
      response: {
        attestationObject: arrayBufferToBase64Url(credential.response.attestationObject),
        clientDataJSON: arrayBufferToBase64Url(credential.response.clientDataJSON)
      },
      type: credential.type
    }),
    name: name
  };
}
```

### Authentication

```javascript
/**
 * Starts the WebAuthn authentication process
 * @param {Object} options - Authentication options from the server
 * @returns {Promise<Credential>} - The assertion credential
 */
async function startAuthentication(options) {
  // Convert base64url challenge to ArrayBuffer
  options.publicKey.challenge = base64UrlToArrayBuffer(options.publicKey.challenge);
  
  // Convert allowed credentials if present
  if (options.publicKey.allowCredentials) {
    options.publicKey.allowCredentials = options.publicKey.allowCredentials.map(cred => {
      return {
        ...cred,
        id: base64UrlToArrayBuffer(cred.id)
      };
    });
  }
  
  // Get assertion with the browser's API
  return await navigator.credentials.get({
    publicKey: options.publicKey
  });
}

/**
 * Prepares assertion for sending to server
 * @param {Credential} credential - The credential from navigator.credentials.get
 * @returns {Object} - Formatted assertion for the server
 */
function prepareAuthenticationCredential(credential) {
  return {
    assertion: JSON.stringify({
      id: credential.id,
      rawId: arrayBufferToBase64Url(credential.rawId),
      response: {
        authenticatorData: arrayBufferToBase64Url(credential.response.authenticatorData),
        clientDataJSON: arrayBufferToBase64Url(credential.response.clientDataJSON),
        signature: arrayBufferToBase64Url(credential.response.signature),
        userHandle: credential.response.userHandle ? 
          arrayBufferToBase64Url(credential.response.userHandle) : null
      },
      type: credential.type
    })
  };
}
```

### Utility Functions

```javascript
/**
 * Converts a base64url string to an ArrayBuffer
 * @param {string} base64url - Base64url encoded string
 * @returns {ArrayBuffer} - Decoded ArrayBuffer
 */
function base64UrlToArrayBuffer(base64url) {
  const base64 = base64url.replace(/-/g, '+').replace(/_/g, '/');
  const padLen = (4 - (base64.length % 4)) % 4;
  const padded = base64 + '='.repeat(padLen);
  const binary = atob(padded);
  const buffer = new ArrayBuffer(binary.length);
  const view = new Uint8Array(buffer);
  
  for (let i = 0; i < binary.length; i++) {
    view[i] = binary.charCodeAt(i);
  }
  
  return buffer;
}

/**
 * Converts an ArrayBuffer to a base64url string
 * @param {ArrayBuffer} buffer - The ArrayBuffer to encode
 * @returns {string} - Base64url encoded string
 */
function arrayBufferToBase64Url(buffer) {
  const bytes = new Uint8Array(buffer);
  let binary = '';
  
  for (let i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  
  const base64 = btoa(binary);
  return base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}
```

## Frontend Leptos Components

### WebAuthnRegistrationForm Component

```rust
#[component]
pub fn WebAuthnRegistrationForm(
    cx: Scope,
    user_id: Uuid,
    on_register: Callback<Result<Credential, String>>,
) -> impl IntoView {
    let (is_loading, set_is_loading) = create_signal(cx, false);
    let (error, set_error) = create_signal(cx, None::<String>);
    let (credential_name, set_credential_name) = create_signal(cx, String::from("My Security Key"));
    
    let register = move |_| {
        set_is_loading.set(true);
        set_error.set(None);
        
        // Start registration process
        spawn_local(async move {
            match start_webauthn_registration(user_id).await {
                Ok(options) => {
                    // Call JavaScript to create credential
                    match start_registration(options).await {
                        Ok(credential) => {
                            // Complete registration
                            let result = complete_webauthn_registration(
                                user_id, 
                                credential, 
                                credential_name.get()
                            ).await;
                            
                            on_register.call(result);
                        },
                        Err(e) => set_error.set(Some(e.to_string())),
                    }
                },
                Err(e) => set_error.set(Some(e.to_string())),
            }
            
            set_is_loading.set(false);
        });
    };
    
    view! { cx,
        <div class="webauthn-form">
            <h3>"Register a Security Key or Biometric Authentication"</h3>
            
            <div class="form-group">
                <label for="credential-name">"Credential Name"</label>
                <input
                    id="credential-name"
                    type="text"
                    value=credential_name.get()
                    on:input=move |ev| set_credential_name.set(event_target_value(&ev))
                    placeholder="My Security Key"
                />
            </div>
            
            <button
                class="webauthn-button"
                on:click=register
                disabled=is_loading.get()
            >
                {move || if is_loading.get() { "Registering..." } else { "Register Security Key" }}
            </button>
            
            {move || error.get().map(|e| view! { cx, <div class="error">{e}</div> })}
        </div>
    }
}
```

### WebAuthnAuthenticationForm Component

```rust
#[component]
pub fn WebAuthnAuthenticationForm(
    cx: Scope,
    user_id: Option<Uuid>,
    on_authenticate: Callback<Result<(User, String), String>>,
) -> impl IntoView {
    let (is_loading, set_is_loading) = create_signal(cx, false);
    let (error, set_error) = create_signal(cx, None::<String>);
    
    let authenticate = move |_| {
        set_is_loading.set(true);
        set_error.set(None);
        
        // Start authentication process
        spawn_local(async move {
            match start_webauthn_authentication(user_id).await {
                Ok(options) => {
                    // Call JavaScript to get assertion
                    match start_authentication(options).await {
                        Ok(credential) => {
                            // Complete authentication
                            let result = complete_webauthn_authentication(credential).await;
                            
                            on_authenticate.call(result);
                        },
                        Err(e) => set_error.set(Some(e.to_string())),
                    }
                },
                Err(e) => set_error.set(Some(e.to_string())),
            }
            
            set_is_loading.set(false);
        });
    };
    
    view! { cx,
        <div class="webauthn-form">
            <h3>"Sign in with Security Key or Biometric"</h3>
            
            <button
                class="webauthn-button"
                on:click=authenticate
                disabled=is_loading.get()
            >
                {move || if is_loading.get() { "Authenticating..." } else { "Sign in with Security Key" }}
            </button>
            
            {move || error.get().map(|e| view! { cx, <div class="error">{e}</div> })}
        </div>
    }
}
```

## Models and Data Types

### Credential Model

```rust
/// Represents a full WebAuthn credential with all necessary data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    /// Unique identifier for this credential
    pub id: CredentialID,
    /// The credential's unique ID assigned by the framework
    pub uuid: Uuid,
    /// The user ID that owns this credential
    pub user_id: Uuid,
    /// The tenant ID that this credential belongs to
    pub tenant_id: Uuid,
    /// User-friendly name for this credential
    pub name: String,
    /// The credential's AAGUID, identifying the authenticator model
    pub aaguid: Vec<u8>,
    /// Public key and other credential data
    pub public_key: Vec<u8>,
    /// Counter for signature use to prevent replay attacks
    pub counter: u32,
    /// When this credential was registered
    pub created_at: time::OffsetDateTime,
    /// Last time this credential was used
    pub last_used_at: Option<time::OffsetDateTime>,
}
```

### CredentialID Type

```rust
/// Represents the WebAuthn credential ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CredentialID(pub String);

impl CredentialID {
    pub fn new(credential_id: &[u8]) -> Self {
        let encoded = URL_SAFE_NO_PAD.encode(credential_id);
        Self(encoded)
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>> {
        URL_SAFE_NO_PAD
            .decode(&self.0)
            .map_err(|_| CoreError::Validation("Invalid credential ID".to_string()))
    }
}
```

### WebAuthnConfig

```rust
/// Configuration for WebAuthn
#[derive(Debug, Clone)]
pub struct WebAuthnConfig {
    /// The Relying Party ID (usually the domain name)
    pub rp_id: String,
    /// The Relying Party name (displayed to users)
    pub rp_name: String,
    /// Origin for the website (e.g. https://example.com)
    pub origin: String,
    /// User verification preference: "discouraged", "preferred", or "required"
    pub user_verification: String,
}
```

## See Also

- [WebAuthn Implementation Status](./webauthn-implementation-status.md)
- [WebAuthn Implementation Strategy](./webauthn-implementation-strategy.md)
- [W3C WebAuthn Specification](https://www.w3.org/TR/webauthn-2/)
- [webauthn-rs Documentation](https://docs.rs/webauthn-rs/0.5.1/webauthn_rs/)