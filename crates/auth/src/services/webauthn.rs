use crate::{
    models::{
        user::User,
        webauthn::{
            Credential, CredentialID, PublicKeyCredential, RegisterCredential, WebAuthnError,
        },
    },
    repository::WebAuthnRepository,
};
use acci_core::error::{Error as CoreError, Result};
use std::sync::Arc;
use tracing::{debug, instrument};
use uuid::Uuid;
use webauthn_rs::prelude::*;

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

// Using the WebAuthnError from models/webauthn.rs instead
// But we need to implement the conversion to CoreError here

impl From<WebAuthnError> for CoreError {
    fn from(error: WebAuthnError) -> Self {
        CoreError::Validation(error.to_string())
    }
}

/// Session storage key for registration state
const WEBAUTHN_REG_STATE_KEY: &str = "webauthn_registration_state";
/// Session storage key for authentication state
const WEBAUTHN_AUTH_STATE_KEY: &str = "webauthn_authentication_state";

/// Manages WebAuthn operations including registration and authentication
pub struct WebAuthnService {
    #[allow(dead_code)]
    webauthn: Webauthn,
    #[allow(dead_code)]
    repository: Arc<dyn WebAuthnRepository>,
}

impl WebAuthnService {
    /// Create a new WebAuthn service
    pub fn new(config: WebAuthnConfig, repository: Arc<dyn WebAuthnRepository>) -> Result<Self> {
        // Parse the origin URL
        let origin = Url::parse(&config.origin)
            .map_err(|e| WebAuthnError::Unexpected(format!("Invalid origin URL: {}", e)))?;

        // Create the webauthn instance using the builder
        let webauthn_builder = WebauthnBuilder::new(&config.rp_id, &origin).map_err(|e| {
            WebAuthnError::WebAuthn(format!("Failed to create WebAuthn instance: {}", e))
        })?;

        let webauthn = webauthn_builder.build().map_err(|e| {
            WebAuthnError::WebAuthn(format!("Failed to build WebAuthn instance: {}", e))
        })?;

        Ok(Self {
            webauthn,
            repository,
        })
    }

    /// Start the registration process for a new credential
    #[instrument(skip(self, session_data), level = "debug")]
    pub async fn start_registration(
        &self,
        user: &User,
        tenant_id: &Uuid,
        session_data: &mut serde_json::Value,
    ) -> Result<serde_json::Value> {
        debug!("Starting WebAuthn registration for user: {}", user.id);

        // This is a stub implementation - in a real implementation, we would:
        // 1. Create and store a registration challenge
        // 2. Return WebAuthn options for the client

        let challenge_json = serde_json::json!({
            "status": "success",
            "message": "WebAuthn registration started - implementation in progress"
        });

        // Store dummy state in session just to demonstrate flow
        if let Some(obj) = session_data.as_object_mut() {
            obj.insert(WEBAUTHN_REG_STATE_KEY.to_string(), challenge_json.clone());
        }

        Ok(challenge_json)
    }

    /// Complete the registration process with the client response
    #[instrument(skip(self, session_data, credential), level = "debug")]
    pub async fn complete_registration(
        &self,
        user: &User,
        tenant_id: &Uuid,
        session_data: &mut serde_json::Value,
        credential: RegisterCredential,
    ) -> Result<Credential> {
        debug!("Completing WebAuthn registration for user: {}", user.id);

        // This is a stub implementation - in a real implementation, we would:
        // 1. Verify the attestation with the stored challenge
        // 2. Extract credential data
        // 3. Store the credential

        // For demonstration, create a dummy credential
        let now = time::OffsetDateTime::now_utc();
        let cred = Credential {
            id: CredentialID("dummy_credential_id".to_string()),
            uuid: Uuid::new_v4(),
            user_id: user.id,
            tenant_id: *tenant_id,
            name: credential.name,
            aaguid: vec![0u8; 16],
            public_key: vec![0u8; 32],
            counter: 0,
            created_at: now,
            last_used_at: None,
        };

        // Clear session state
        if let Some(obj) = session_data.as_object_mut() {
            obj.remove(WEBAUTHN_REG_STATE_KEY);
        }

        Ok(cred)
    }

    /// Start the authentication process
    #[instrument(skip(self, session_data), level = "debug")]
    pub async fn start_authentication(
        &self,
        user_id: Option<&Uuid>,
        tenant_id: &Uuid,
        session_data: &mut serde_json::Value,
    ) -> Result<serde_json::Value> {
        debug!("Starting WebAuthn authentication");

        // This is a stub implementation - in a real implementation, we would:
        // 1. Create and store an authentication challenge
        // 2. Return WebAuthn options for the client

        let challenge_json = serde_json::json!({
            "status": "success",
            "message": "WebAuthn authentication started - implementation in progress"
        });

        // Store dummy state in session just to demonstrate flow
        if let Some(obj) = session_data.as_object_mut() {
            obj.insert(WEBAUTHN_AUTH_STATE_KEY.to_string(), challenge_json.clone());
        }

        Ok(challenge_json)
    }

    /// Complete the authentication process with the client response
    #[instrument(skip(self, session_data, _credential), level = "debug")]
    pub async fn complete_authentication(
        &self,
        tenant_id: &Uuid,
        session_data: &mut serde_json::Value,
        _credential: PublicKeyCredential,
    ) -> Result<(User, Credential)> {
        debug!("Completing WebAuthn authentication");

        // This is a stub implementation - in a real implementation, we would:
        // 1. Verify the assertion with the stored challenge
        // 2. Verify the credential exists
        // 3. Update the credential counter
        // 4. Return the associated user

        // For demonstration, create a dummy user and credential
        let now = time::OffsetDateTime::now_utc();
        let user_id = Uuid::new_v4();

        let user = User {
            id: user_id,
            email: format!("user-{}@example.com", user_id),
            display_name: format!("User {}", user_id),
            password_hash: "dummy_hash".to_string(),
            created_at: now,
            updated_at: now,
            last_login: None,
            is_active: true,
            is_verified: true,
        };

        let cred = Credential {
            id: CredentialID("dummy_credential_id".to_string()),
            uuid: Uuid::new_v4(),
            user_id,
            tenant_id: *tenant_id,
            name: "Dummy Credential".to_string(),
            aaguid: vec![0u8; 16],
            public_key: vec![0u8; 32],
            counter: 1,
            created_at: now,
            last_used_at: Some(now),
        };

        // Clear session state
        if let Some(obj) = session_data.as_object_mut() {
            obj.remove(WEBAUTHN_AUTH_STATE_KEY);
        }

        Ok((user, cred))
    }

    /// List all credentials for a user
    #[instrument(skip(self), level = "debug")]
    pub async fn list_credentials(&self, user_id: &Uuid) -> Result<Vec<Credential>> {
        debug!("Listing WebAuthn credentials for user: {}", user_id);

        // In a real implementation, we would query the repository
        // For stub implementation, return an empty list
        Ok(Vec::new())
    }

    /// Delete a credential
    #[instrument(skip(self), level = "debug")]
    pub async fn delete_credential(&self, credential_uuid: &Uuid, user_id: &Uuid) -> Result<()> {
        debug!("Deleting WebAuthn credential: {}", credential_uuid);

        // In a real implementation, we would delete from the repository
        // For stub implementation, just return success
        Ok(())
    }
}
