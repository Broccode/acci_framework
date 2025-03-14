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
use tracing::{debug, info, instrument, warn};
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
    /// User verification requirement (discouraged, preferred, or required)
    pub user_verification: UserVerificationPolicy,
}

// Using the WebAuthnError from models/webauthn.rs instead

/// Session storage key for registration state
const WEBAUTHN_REG_STATE_KEY: &str = "webauthn_registration_state";
/// Session storage key for authentication state
const WEBAUTHN_AUTH_STATE_KEY: &str = "webauthn_authentication_state";

/// Manages WebAuthn operations including registration and authentication
pub struct WebAuthnService {
    webauthn: Webauthn,
    repository: Arc<dyn WebAuthnRepository>,
}

impl WebAuthnService {
    /// Create a new WebAuthn service
    pub fn new(config: WebAuthnConfig, repository: Arc<dyn WebAuthnRepository>) -> Result<Self> {
        let builder = WebauthnBuilder::new()
            .rp_id(&config.rp_id)
            .rp_name(&config.rp_name)
            .rp_origin_list(&[Url::parse(&config.origin).map_err(|e| {
                WebAuthnError::Unexpected(format!("Invalid origin URL: {}", e))
            })?]);

        let builder = match config.user_verification {
            UserVerificationPolicy::Discouraged => builder.user_verification_discouraged(),
            UserVerificationPolicy::Preferred => builder.user_verification_preferred(),
            UserVerificationPolicy::Required => builder.user_verification_required(),
        };

        let webauthn = builder.build().map_err(|e| {
            WebAuthnError::WebAuthn(format!("Failed to create WebAuthn instance: {}", e))
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

        // Get existing credentials for exclusion
        let existing_credentials = self.repository.list_credentials_for_user(&user.id).await?;
        let exclude_credentials = existing_credentials
            .iter()
            .filter_map(|cred| {
                cred.id.as_bytes().ok().map(|bytes| {
                    CreationChallengeResponse {
                        id: Base64UrlSafeData(bytes.clone()),
                        type_: "public-key".to_string(),
                    }
                })
            })
            .collect::<Vec<_>>();

        // Create a user entity for this registration
        let user_entity = UserEntity {
            name: user.email.clone(),
            display_name: user.display_name.clone(),
            id: Base64UrlSafeData(user.id.as_bytes().to_vec()),
        };
        
        // Generate registration options
        let challenge = match self.webauthn.start_passkey_registration(
            user_entity, 
            if !exclude_credentials.is_empty() { Some(exclude_credentials) } else { None },
        ) {
            Ok(c) => c,
            Err(e) => return Err(WebAuthnError::WebAuthn(format!("Failed to start registration: {}", e)).into()),
        };

        // Convert to JSON
        let challenge_json = serde_json::to_value(&challenge)
            .map_err(|e| WebAuthnError::Unexpected(format!("Failed to serialize challenge: {}", e)))?;

        // Store registration state in session
        if let Some(obj) = session_data.as_object_mut() {
            obj.insert(WEBAUTHN_REG_STATE_KEY.to_string(), challenge_json.clone());
        } else {
            return Err(WebAuthnError::Unexpected("Invalid session data format".to_string()).into());
        }

        // Return challenge as JSON for the frontend
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

        // Get the registration state from session
        let reg_challenge: PasskeyRegistrationChallenge = if let Some(obj) = session_data.as_object() {
            if let Some(state) = obj.get(WEBAUTHN_REG_STATE_KEY) {
                serde_json::from_value(state.clone()).map_err(|e| {
                    WebAuthnError::Unexpected(format!("Invalid registration state: {}", e))
                })?
            } else {
                return Err(WebAuthnError::Unexpected(
                    "No registration state found in session".to_string()
                ).into());
            }
        } else {
            return Err(WebAuthnError::Unexpected("Invalid session data".to_string()).into());
        };

        // Parse the client response
        let reg_credential = credential.parse()?;

        // Complete the registration
        let result = match self.webauthn.finish_passkey_registration(&reg_credential, &reg_challenge) {
            Ok(r) => r,
            Err(e) => return Err(WebAuthnError::WebAuthn(format!("Registration failed: {}", e)).into()),
        };

        // Create and save the credential
        let attobj = match result.attestation_object() {
            Some(attobj) => attobj,
            None => return Err(WebAuthnError::Unexpected("Missing attestation object".to_string()).into()),
        };

        let cred = Credential::new(
            attobj,
            result.cred_id().to_vec(),
            result.cred_pk().to_vec(),
            &credential.name,
            user.id,
            *tenant_id,
        );
        self.repository.save_credential(&cred).await?;

        // Clear the registration state from session
        if let Some(obj) = session_data.as_object_mut() {
            obj.remove(WEBAUTHN_REG_STATE_KEY);
        }

        info!("WebAuthn registration completed for user: {}", user.id);
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

        let allow_credentials = if let Some(user_id) = user_id {
            // If user is known, only allow their credentials
            let user_credentials = self.repository.list_credentials_for_user(user_id).await?;
            
            let allow_list = user_credentials
                .iter()
                .filter_map(|cred| {
                    cred.id.as_bytes().ok().map(|bytes| {
                        RequestChallengeResponse {
                            id: Base64UrlSafeData(bytes.clone()),
                            type_: "public-key".to_string(),
                        }
                    })
                })
                .collect::<Vec<_>>();
                
            if !allow_list.is_empty() {
                Some(allow_list)
            } else {
                None
            }
        } else {
            // If user is unknown, allow any credential (passwordless flow)
            None
        };

        // Generate authentication challenge
        let challenge = match self.webauthn.start_passkey_authentication(allow_credentials) {
            Ok(c) => c,
            Err(e) => return Err(WebAuthnError::WebAuthn(format!("Failed to start authentication: {}", e)).into()),
        };

        // Convert to JSON
        let challenge_json = serde_json::to_value(&challenge)
            .map_err(|e| WebAuthnError::Unexpected(format!("Failed to serialize challenge: {}", e)))?;

        // Store authentication state in session
        if let Some(obj) = session_data.as_object_mut() {
            obj.insert(WEBAUTHN_AUTH_STATE_KEY.to_string(), challenge_json.clone());
        } else {
            return Err(WebAuthnError::Unexpected("Invalid session data format".to_string()).into());
        }

        // Return challenge as JSON for the frontend
        Ok(challenge_json)
    }

    /// Complete the authentication process with the client response
    #[instrument(skip(self, session_data, credential), level = "debug")]
    pub async fn complete_authentication(
        &self,
        tenant_id: &Uuid,
        session_data: &mut serde_json::Value,
        credential: PublicKeyCredential,
    ) -> Result<(User, Credential)> {
        debug!("Completing WebAuthn authentication");

        // Get the authentication state from session
        let auth_challenge: PasskeyAuthenticationChallenge = if let Some(obj) = session_data.as_object() {
            if let Some(state) = obj.get(WEBAUTHN_AUTH_STATE_KEY) {
                serde_json::from_value(state.clone()).map_err(|e| {
                    WebAuthnError::Unexpected(format!("Invalid authentication state: {}", e))
                })?
            } else {
                return Err(WebAuthnError::Unexpected(
                    "No authentication state found in session".to_string(),
                )
                .into());
            }
        } else {
            return Err(WebAuthnError::Unexpected("Invalid session data".to_string()).into());
        };

        // Parse the client response
        let auth_credential = credential.parse()?;

        // Verify the credential ID (in auth_credential.id) is base64 URL-safe encoded
        let credential_id = CredentialID(
            String::from_utf8(auth_credential.id.into_vec())
                .map_err(|_| WebAuthnError::InvalidCredentialID)?
        );

        // Find the credential by ID
        let mut user_credential = match self.repository.find_credential_by_id(&credential_id).await? {
            Some(cred) => cred,
            None => return Err(WebAuthnError::CredentialNotFound.into()),
        };

        // Complete the authentication
        let auth_result = match self.webauthn.finish_passkey_authentication(
            &auth_credential,
            &auth_challenge,
            &user_credential.public_key,
            user_credential.counter,
        ) {
            Ok(r) => r,
            Err(e) => return Err(WebAuthnError::WebAuthn(format!("Authentication failed: {}", e)).into()),
        };

        // Update the credential counter
        user_credential.update_after_authentication(auth_result.counter);
        self.repository.update_credential(&user_credential).await?;

        // Find the associated user
        let user = self.get_user_by_id(&user_credential.user_id).await?;

        // Clear the authentication state from session
        if let Some(obj) = session_data.as_object_mut() {
            obj.remove(WEBAUTHN_AUTH_STATE_KEY);
        }

        info!("WebAuthn authentication completed for user: {}", user.id);
        Ok((user, user_credential))
    }

    /// List all credentials for a user
    #[instrument(skip(self), level = "debug")]
    pub async fn list_credentials(&self, user_id: &Uuid) -> Result<Vec<Credential>> {
        debug!("Listing WebAuthn credentials for user: {}", user_id);
        self.repository.list_credentials_for_user(user_id).await
    }

    /// Delete a credential
    #[instrument(skip(self), level = "debug")]
    pub async fn delete_credential(&self, credential_uuid: &Uuid, user_id: &Uuid) -> Result<()> {
        debug!("Deleting WebAuthn credential: {}", credential_uuid);

        // Verify the credential belongs to this user before deleting
        let credential = match self
            .repository
            .find_credential_by_uuid(credential_uuid)
            .await?
        {
            Some(cred) => cred,
            None => return Err(WebAuthnError::CredentialNotFound.into()),
        };

        if credential.user_id != *user_id {
            warn!(
                "Unauthorized attempt to delete credential: {}",
                credential_uuid
            );
            return Err(WebAuthnError::CredentialNotFound.into());
        }

        self.repository.delete_credential(credential_uuid).await?;
        info!("WebAuthn credential deleted: {}", credential_uuid);
        Ok(())
    }

    // This would be replaced with a proper user lookup in a real implementation
    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<User> {
        // In a real implementation, this would query the user repository
        // For now, just return a dummy user
        Err(WebAuthnError::Unexpected("User lookup not implemented".to_string()).into())
    }
}
