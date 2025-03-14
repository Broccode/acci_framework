use async_trait::async_trait;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::models::{TenantId, UserId, VerificationCode, VerificationType};
use crate::repository::tenant_aware::TenantAwareContext;
use acci_core::error::Result;

/// Repository for managing verification codes
#[async_trait]
pub trait VerificationCodeRepository: Sync + Send {
    /// Save a verification code
    async fn save(&self, code: &VerificationCode, context: &dyn TenantAwareContext) -> Result<()>;

    /// Get a verification code by its ID
    async fn get_by_id(
        &self,
        id: Uuid,
        tenant_id: TenantId,
        context: &dyn TenantAwareContext,
    ) -> Result<Option<VerificationCode>>;

    /// Get a verification code by the code value
    async fn get_by_code(
        &self,
        code: &str,
        user_id: UserId,
        verification_type: VerificationType,
        tenant_id: TenantId,
        context: &dyn TenantAwareContext,
    ) -> Result<Option<VerificationCode>>;

    /// Get all pending verification codes for a user
    async fn get_pending_by_user(
        &self,
        user_id: UserId,
        verification_type: VerificationType,
        tenant_id: TenantId,
        context: &dyn TenantAwareContext,
    ) -> Result<Vec<VerificationCode>>;

    /// Update an existing verification code
    async fn update(&self, code: &VerificationCode, context: &dyn TenantAwareContext)
    -> Result<()>;

    /// Delete a verification code
    async fn delete(
        &self,
        id: Uuid,
        tenant_id: TenantId,
        context: &dyn TenantAwareContext,
    ) -> Result<()>;

    /// Delete all expired verification codes
    async fn delete_expired(
        &self,
        before: OffsetDateTime,
        tenant_id: TenantId,
        context: &dyn TenantAwareContext,
    ) -> Result<u64>;

    /// Invalidate all pending verification codes for a user
    async fn invalidate_pending(
        &self,
        user_id: UserId,
        verification_type: VerificationType,
        tenant_id: TenantId,
        context: &dyn TenantAwareContext,
    ) -> Result<u64>;

    /// Count recent verification attempts for a user within a timeframe
    async fn count_recent_attempts(
        &self,
        user_id: UserId,
        verification_type: VerificationType,
        since: OffsetDateTime,
        tenant_id: TenantId,
        context: &dyn TenantAwareContext,
    ) -> Result<u64>;
}
