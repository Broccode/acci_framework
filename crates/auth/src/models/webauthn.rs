use acci_core::error::{Error as CoreError, Result};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;
use webauthn_rs::error::WebauthnError as WnError;
use webauthn_rs::prelude::*;

/// Represents the WebAuthn credential ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CredentialID(pub String);

impl CredentialID {
    pub fn new(credential_id: &[u8]) -> Self {
        let encoded = URL_SAFE_NO_PAD.encode(credential_id);
        Self(encoded)
    }

    pub fn from_base64(encoded: &str) -> Result<Self> {
        // Validate that this is valid base64
        URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| CoreError::Validation("Invalid credential ID".to_string()))?;
        Ok(Self(encoded.to_string()))
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>> {
        URL_SAFE_NO_PAD
            .decode(&self.0)
            .map_err(|_| CoreError::Validation("Invalid credential ID".to_string()))
    }

    pub fn to_webauthn_credential_id(&self) -> Result<CredentialID> {
        let _ = self.as_bytes()?; // Validate the bytes can be decoded
        Ok(self.clone())
    }
}

impl Display for CredentialID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Vec<u8>> for CredentialID {
    fn from(bytes: Vec<u8>) -> Self {
        Self::new(&bytes)
    }
}

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

impl Credential {
    /// Create a new credential from registration data
    pub fn new(
        attestation: &AttestationObject,
        credential_id_bytes: Vec<u8>,
        public_key: Vec<u8>,
        credential_name: &str,
        user_id: Uuid,
        tenant_id: Uuid,
    ) -> Self {
        let now = OffsetDateTime::now_utc();

        // Get AAGUID from attestation data if available, or use empty bytes
        let aaguid: Vec<u8> = match &attestation.aaguid {
            Some(aaguid_bytes) => aaguid_bytes.clone(),
            None => vec![0u8; 16],
        };

        Self {
            id: CredentialID::new(&credential_id_bytes),
            uuid: Uuid::new_v4(),
            user_id,
            tenant_id,
            name: credential_name.to_string(),
            aaguid,
            public_key,
            counter: 0,
            created_at: now,
            last_used_at: None,
        }
    }

    /// Update the counter and last_used timestamp after successful authentication
    pub fn update_after_authentication(&mut self, counter: u32) {
        self.counter = counter;
        self.last_used_at = Some(OffsetDateTime::now_utc());
    }

    /// Get a description of the authenticator model if available
    pub fn authenticator_description(&self) -> Option<String> {
        // This needs to be implemented based on AAGUID registry
        // For now, just return a static description
        if !self.aaguid.iter().all(|&b| b == 0) {
            Some(format!(
                "FIDO2 Security Key (AAGUID: {})",
                hex::encode(&self.aaguid)
            ))
        } else {
            None
        }
    }
}

/// Represents the public key credential used for registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterCredential {
    /// The raw attestation response from the client
    pub attestation: String,
    /// User-friendly name for this credential
    pub name: String,
}

impl RegisterCredential {
    pub fn parse(&self) -> Result<RegisterPublicKeyCredential> {
        serde_json::from_str(&self.attestation).map_err(|e| {
            CoreError::Validation(format!(
                "Failed to parse registration data: {}",
                e
            ))
        })
    }
}

/// Represents the public key credential used for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyCredential {
    /// The raw assertion response from the client
    pub assertion: String,
}

impl PublicKeyCredential {
    pub fn parse(&self) -> Result<RegisterPublicKeyCredential> {
        serde_json::from_str(&self.assertion).map_err(|e| {
            CoreError::Validation(format!("Failed to parse assertion data: {}", e))
        })
    }
}

#[derive(Debug, Error)]
pub enum WebAuthnError {
    #[error("Invalid credential ID")]
    InvalidCredentialID,

    #[error("Invalid credential data: {0}")]
    InvalidCredentialData(String),

    #[error("Credential not found")]
    CredentialNotFound,

    #[error("WebAuthn error: {0}")]
    WebAuthn(String),

    #[error("User verification required")]
    UserVerificationRequired,

    #[error("Attestation error: {0}")]
    Attestation(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

// Implementation moved to services/webauthn.rs due to orphan rule
