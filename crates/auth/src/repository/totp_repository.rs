use crate::models::{TenantId, TotpSecret, UserId};
use crate::repository::RepositoryError;
use async_trait::async_trait;

/// Repository interface for TOTP secrets
#[async_trait]
pub trait TotpSecretRepository: Send + Sync + 'static {
    /// Save a TOTP secret
    async fn save(&self, secret: &TotpSecret) -> Result<(), RepositoryError>;

    /// Get a TOTP secret by user ID and tenant ID
    async fn get_by_user_id(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> Result<Option<TotpSecret>, RepositoryError>;

    /// Delete a TOTP secret
    async fn delete(&self, user_id: &UserId, tenant_id: &TenantId) -> Result<(), RepositoryError>;

    /// Get all TOTP secrets for a tenant
    async fn get_all_for_tenant(
        &self,
        tenant_id: &TenantId,
    ) -> Result<Vec<TotpSecret>, RepositoryError>;

    /// Get TOTP secret by ID
    async fn get_by_id(
        &self,
        id: &uuid::Uuid,
        tenant_id: &TenantId,
    ) -> Result<Option<TotpSecret>, RepositoryError>;
}
