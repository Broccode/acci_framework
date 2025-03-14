use crate::models::webauthn::{Credential, CredentialID};
use crate::repository::tenant_aware::RepositoryError;
use async_trait::async_trait;
use uuid::Uuid;

/// Repository interface for WebAuthn operations
#[async_trait]
pub trait WebAuthnRepository: Send + Sync + 'static {
    /// Store a newly registered credential
    async fn save_credential(&self, credential: &Credential) -> Result<(), RepositoryError>;

    /// Update an existing credential (e.g., after successful authentication)
    async fn update_credential(&self, credential: &Credential) -> Result<(), RepositoryError>;

    /// Find a credential by its ID
    async fn find_credential_by_id(&self, id: &CredentialID) -> Result<Option<Credential>, RepositoryError>;

    /// Find a credential by its UUID
    async fn find_credential_by_uuid(&self, uuid: &Uuid) -> Result<Option<Credential>, RepositoryError>;

    /// List all credentials for a user
    async fn list_credentials_for_user(&self, user_id: &Uuid) -> Result<Vec<Credential>, RepositoryError>;

    /// Delete a credential
    async fn delete_credential(&self, uuid: &Uuid) -> Result<(), RepositoryError>;

    /// Delete all credentials for a user
    async fn delete_credentials_for_user(&self, user_id: &Uuid) -> Result<u64, RepositoryError>;
}
