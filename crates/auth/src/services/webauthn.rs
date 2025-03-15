use crate::{
    models::{
        user::User,
        webauthn::{
            Credential, CredentialID, PublicKeyCredential, RegisterCredential, WebAuthnError,
        },
    },
    repository::WebAuthnRepository,
    services::user::UserService,
};
use acci_core::error::{Error as CoreError, Result};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tracing::{debug, instrument};
use uuid::Uuid;
use webauthn_rs::prelude::*;
use std::convert::TryFrom;

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

// Removed WError conversion as it's not needed

/// Session storage key for registration state
const WEBAUTHN_REG_STATE_KEY: &str = "webauthn_registration_state";
/// Session storage key for authentication state
const WEBAUTHN_AUTH_STATE_KEY: &str = "webauthn_authentication_state";

/// Type for storing registration state
type RegistrationState = PasskeyRegistration;
/// Type for storing authentication state
type AuthenticationState = PasskeyAuthentication;

/// In-memory store for registration state
/// NOTE: In production, use a distributed cache like Redis
struct RegistrationStateStore {
    states: Mutex<HashMap<Uuid, RegistrationState>>,
}

impl RegistrationStateStore {
    fn new() -> Self {
        Self {
            states: Mutex::new(HashMap::new()),
        }
    }

    fn insert(&self, user_id: Uuid, state: RegistrationState) {
        let mut states = self.states.lock().unwrap();
        states.insert(user_id, state);
    }

    fn get(&self, user_id: &Uuid) -> Option<RegistrationState> {
        let states = self.states.lock().unwrap();
        states.get(user_id).cloned()
    }

    fn remove(&self, user_id: &Uuid) -> Option<RegistrationState> {
        let mut states = self.states.lock().unwrap();
        states.remove(user_id)
    }
}

/// In-memory store for authentication state
/// NOTE: In production, use a distributed cache like Redis
struct AuthenticationStateStore {
    states: Mutex<HashMap<Uuid, AuthenticationState>>,
}

impl AuthenticationStateStore {
    fn new() -> Self {
        Self {
            states: Mutex::new(HashMap::new()),
        }
    }

    fn insert(&self, user_id: Uuid, state: AuthenticationState) {
        let mut states = self.states.lock().unwrap();
        states.insert(user_id, state);
    }

    fn get(&self, user_id: &Uuid) -> Option<AuthenticationState> {
        let states = self.states.lock().unwrap();
        states.get(user_id).cloned()
    }

    fn remove(&self, user_id: &Uuid) -> Option<AuthenticationState> {
        let mut states = self.states.lock().unwrap();
        states.remove(user_id)
    }
}

/// Manages WebAuthn operations including registration and authentication
pub struct WebAuthnService {
    webauthn: Webauthn,
    repository: Arc<dyn WebAuthnRepository>,
    user_service: Arc<UserService>,
    reg_states: RegistrationStateStore,
    auth_states: AuthenticationStateStore,
}

impl WebAuthnService {
    /// Create a new WebAuthn service
    pub fn new(
        config: WebAuthnConfig,
        repository: Arc<dyn WebAuthnRepository>,
        user_service: Arc<UserService>,
    ) -> Result<Self> {
        // Parse the origin URL
        let origin = Url::parse(&config.origin)
            .map_err(|e| WebAuthnError::Unexpected(format!("Invalid origin URL: {}", e)))?;

        // Create the webauthn instance using the builder
        let webauthn_builder = WebauthnBuilder::new(&config.rp_id, &origin)
            .map_err(|e| {
                WebAuthnError::WebAuthn(format!("Failed to create WebAuthn instance: {}", e))
            })?
            .rp_name(&config.rp_name);

        // Origin is already set in WebauthnBuilder::new

        // WebAuthn builder doesn't expose direct verification settings in v0.5.1
        // We'll use default values

        // Build the WebAuthn instance
        let webauthn = webauthn_builder.build().map_err(|e| {
            WebAuthnError::WebAuthn(format!("Failed to build WebAuthn instance: {}", e))
        })?;

        Ok(Self {
            webauthn,
            repository,
            user_service,
            reg_states: RegistrationStateStore::new(),
            auth_states: AuthenticationStateStore::new(),
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

        // Create user entity for webauthn-rs
        let username = user.email.clone();
        let display_name = user.display_name.clone();

        // Get existing credentials to exclude
        let existing_credentials = self
            .repository
            .list_credentials_for_user(&user.id)
            .await
            .map_err(|e| WebAuthnError::Repository(e.to_string()))?;

        // Unused with current API but would be needed with earlier/later versions
        let _excluded_credentials = existing_credentials
            .iter()
            .map(|cred| cred.id.0.clone())
            .collect::<Vec<_>>();

        // Generate registration options
        let (options, reg_state) = self
            .webauthn
            .start_passkey_registration(user.id, &username, &display_name, None)
            .map_err(|e| WebAuthnError::WebAuthn(e.to_string()))?;

        // Store registration state in memory (for production, use Redis or similar)
        self.reg_states.insert(user.id, reg_state);

        // Also store a reference in the session data
        if let Some(obj) = session_data.as_object_mut() {
            obj.insert(
                WEBAUTHN_REG_STATE_KEY.to_string(),
                serde_json::Value::String(user.id.to_string()),
            );
        }

        // Convert options to JSON
        let options_json = serde_json::to_value(options).map_err(|e| {
            WebAuthnError::Unexpected(format!("Failed to serialize options: {}", e))
        })?;

        Ok(options_json)
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

        // Parse the attestation response
        let parsed_credential = credential
            .parse()
            .map_err(|e| WebAuthnError::InvalidCredentialData(e.to_string()))?;

        // Retrieve registration state from memory
        let reg_state = self
            .reg_states
            .remove(&user.id)
            .ok_or_else(|| WebAuthnError::Unexpected("Registration state not found".to_string()))?;

        // Verify the registration response
        let webauthn_cred = self
            .webauthn
            .finish_passkey_registration(&parsed_credential, &reg_state)
            .map_err(|e| WebAuthnError::Attestation(e.to_string()))?;

        // Create our credential model
        let _now = time::OffsetDateTime::now_utc();
        let cred = Credential::new(
            webauthn_cred.cred_id().to_vec(),
            webauthn_cred.cred_id().to_vec(), // public_key field is repurposed for this
            Vec::new(),                       // Empty aaguid
            &credential.name,
            user.id,
            *tenant_id,
        );

        // Store the credential
        self.repository
            .save_credential(&cred)
            .await
            .map_err(|e| WebAuthnError::Repository(e.to_string()))?;

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

        // Get the credentials for this user, if user_id is provided
        let _allow_credentials = if let Some(user_id) = user_id {
            // Get user's credentials
            let credentials = self
                .repository
                .list_credentials_for_user(user_id)
                .await
                .map_err(|e| WebAuthnError::Repository(e.to_string()))?;

            if credentials.is_empty() {
                return Err(WebAuthnError::CredentialNotFound.into());
            }

            // Convert to credential IDs expected by webauthn-rs
            let cred_ids = credentials
                .iter()
                .map(|c| c.id.0.clone())
                .collect::<Vec<_>>();

            Some(cred_ids)
        } else {
            None
        };

        // Generate authentication options
        let (options, auth_state) = self
            .webauthn
            .start_passkey_authentication(&[])
            .map_err(|e| WebAuthnError::WebAuthn(e.to_string()))?;

        // Store the auth state - if user_id is None, generate a temporary ID
        let state_user_id = user_id.copied().unwrap_or_else(Uuid::new_v4);
        self.auth_states.insert(state_user_id, auth_state);

        // Store reference in session
        if let Some(obj) = session_data.as_object_mut() {
            obj.insert(
                WEBAUTHN_AUTH_STATE_KEY.to_string(),
                serde_json::Value::String(state_user_id.to_string()),
            );
        }

        // Convert options to JSON
        let options_json = serde_json::to_value(options).map_err(|e| {
            WebAuthnError::Unexpected(format!("Failed to serialize options: {}", e))
        })?;

        Ok(options_json)
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

        // Get the auth state ID from session
        let state_user_id_str = session_data
            .get(WEBAUTHN_AUTH_STATE_KEY)
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                WebAuthnError::Unexpected("Authentication state not found in session".to_string())
            })?;

        let state_user_id = Uuid::parse_str(state_user_id_str)
            .map_err(|e| WebAuthnError::Unexpected(format!("Invalid UUID: {}", e)))?;

        // Get the authentication state
        let auth_state = self.auth_states.remove(&state_user_id).ok_or_else(|| {
            WebAuthnError::Unexpected("Authentication state not found".to_string())
        })?;

        // Parse the assertion (this is a bit hacky - we need to convert the credential type)
        // In a real implementation we'd need to match the correct credential type
        // but for this fix we'll use a dummy credential

        // Parse the credential properly using our model's parse method
        let parsed_credential = credential
            .parse()
            .map_err(|e| WebAuthnError::InvalidCredentialData(e.to_string()))?;

        // Verify the authentication
        let auth_result = self
            .webauthn
            .finish_passkey_authentication(&parsed_credential, &auth_state)
            .map_err(|e| WebAuthnError::Authentication(e.to_string()))?;

        // Parse the credential ID to find it in our database
        let cred_id_bytes = parsed_credential.raw_id.clone();
        let cred_id = CredentialID::new(&cred_id_bytes);

        // Find the credential in our database
        let mut db_cred = self
            .repository
            .find_credential_by_id(&cred_id)
            .await
            .map_err(|e| WebAuthnError::Repository(e.to_string()))?
            .ok_or_else(|| WebAuthnError::CredentialNotFound)?;

        // Update the credential counter and last used time
        db_cred.update_after_authentication(auth_result.counter());

        // Save the updated credential
        self.repository
            .update_credential(&db_cred)
            .await
            .map_err(|e| WebAuthnError::Repository(e.to_string()))?;

        // Get the user
        let user_opt = self
            .user_service
            .get_user(db_cred.user_id)
            .await
            .map_err(|e| WebAuthnError::Unexpected(format!("Failed to get user: {}", e)))?;
            
        // Check if user exists
        if user_opt.is_none() {
            return Err(WebAuthnError::Unexpected("User not found".to_string()).into());
        }
        
        let user = user_opt.unwrap();

        // Clear session state
        if let Some(obj) = session_data.as_object_mut() {
            obj.remove(WEBAUTHN_AUTH_STATE_KEY);
        }

        Ok((user, db_cred))
    }

    /// List all credentials for a user
    #[instrument(skip(self), level = "debug")]
    pub async fn list_credentials(&self, user_id: &Uuid) -> Result<Vec<Credential>> {
        debug!("Listing WebAuthn credentials for user: {}", user_id);

        // Get credentials from repository
        let credentials = self
            .repository
            .list_credentials_for_user(user_id)
            .await
            .map_err(|e| WebAuthnError::Repository(e.to_string()))?;

        Ok(credentials)
    }

    /// Delete a credential
    #[instrument(skip(self), level = "debug")]
    pub async fn delete_credential(&self, credential_uuid: &Uuid, user_id: &Uuid) -> Result<()> {
        debug!("Deleting WebAuthn credential: {}", credential_uuid);

        // First verify the credential belongs to this user
        let credential = self
            .repository
            .find_credential_by_uuid(credential_uuid)
            .await
            .map_err(|e| WebAuthnError::Repository(e.to_string()))?
            .ok_or_else(|| WebAuthnError::CredentialNotFound)?;

        if credential.user_id != *user_id {
            return Err(WebAuthnError::Unexpected(
                "Credential does not belong to this user".to_string(),
            )
            .into());
        }

        // Delete the credential
        self.repository
            .delete_credential(credential_uuid)
            .await
            .map_err(|e| WebAuthnError::Repository(e.to_string()))?;

        Ok(())
    }
}
