use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use time::OffsetDateTime;
use tracing::{instrument, trace};
use uuid::Uuid;

use crate::models::{TenantId, UserId, VerificationCode, VerificationStatus, VerificationType};
use crate::repository::tenant_aware::TenantAwareContext;
use crate::repository::verification_repository::VerificationCodeRepository;
use acci_core::error::{Error, Result};

/// PostgreSQL implementation of verification code repository
pub struct PostgresVerificationCodeRepository {
    pool: PgPool,
}

impl PostgresVerificationCodeRepository {
    /// Create a new PostgreSQL verification code repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Start a transaction
    async fn begin_transaction(&self) -> Result<Transaction<'_, Postgres>> {
        let tx = self.pool.begin().await.map_err(Error::Database)?;
        Ok(tx)
    }
}

#[async_trait]
impl VerificationCodeRepository for PostgresVerificationCodeRepository {
    #[instrument(skip(self, code, _context), level = "debug")]
    async fn save(&self, code: &VerificationCode, _context: &TenantAwareContext) -> Result<()> {
        let tenant_id = code.tenant_id;
        let user_id = code.user_id;
        let verification_type = format!("{:?}", code.verification_type);
        let status = format!("{:?}", code.status);

        sqlx::query!(
            r#"
            INSERT INTO verification_codes (
                id, tenant_id, user_id, code, verification_type, 
                created_at, expires_at, status, attempts
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9
            )
            "#,
            code.id,
            tenant_id,
            user_id,
            code.code,
            verification_type,
            code.created_at,
            code.expires_at,
            status,
            code.attempts as i32
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;

        trace!("Saved verification code with ID: {}", code.id);
        Ok(())
    }

    #[instrument(skip(self, _context), level = "debug")]
    async fn get_by_id(
        &self,
        id: Uuid,
        tenant_id: TenantId,
        _context: &TenantAwareContext,
    ) -> Result<Option<VerificationCode>> {
        let tenant_id_str = tenant_id.to_string();

        let record = sqlx::query!(
            r#"
            SELECT 
                id, tenant_id, user_id, code, verification_type, 
                created_at, expires_at, status, attempts
            FROM 
                verification_codes
            WHERE 
                id = $1 AND tenant_id::text = $2
            "#,
            id,
            tenant_id_str
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::Database)?;

        match record {
            Some(rec) => {
                let verification_type = match rec.verification_type.as_str() {
                    "Email" => VerificationType::Email,
                    "Sms" => VerificationType::Sms,
                    _ => {
                        return Err(Error::Validation(format!(
                            "Invalid verification type: {}",
                            rec.verification_type
                        )));
                    },
                };
                let status = match rec.status.as_str() {
                    "Pending" => VerificationStatus::Pending,
                    "Verified" => VerificationStatus::Verified,
                    "Expired" => VerificationStatus::Expired,
                    "Invalidated" => VerificationStatus::Invalidated,
                    _ => {
                        return Err(Error::Validation(format!(
                            "Invalid verification status: {}",
                            rec.status
                        )));
                    },
                };

                Ok(Some(VerificationCode {
                    id: rec.id,
                    tenant_id: rec.tenant_id,
                    user_id: rec.user_id,
                    code: rec.code,
                    verification_type,
                    created_at: rec.created_at,
                    expires_at: rec.expires_at,
                    status,
                    attempts: rec.attempts as usize,
                }))
            },
            None => Ok(None),
        }
    }

    #[instrument(skip(self, code, _context), level = "debug")]
    async fn get_by_code(
        &self,
        code: &str,
        user_id: UserId,
        verification_type: VerificationType,
        tenant_id: TenantId,
        _context: &TenantAwareContext,
    ) -> Result<Option<VerificationCode>> {
        let tenant_id_str = tenant_id.to_string();
        let user_id_str = user_id.to_string();
        let verification_type_str = format!("{:?}", verification_type);

        let record = sqlx::query!(
            r#"
            SELECT 
                id, tenant_id, user_id, code, verification_type, 
                created_at, expires_at, status, attempts
            FROM 
                verification_codes
            WHERE 
                code = $1 AND tenant_id::text = $2 AND user_id::text = $3 AND verification_type = $4
            "#,
            code,
            tenant_id_str,
            user_id_str,
            verification_type_str
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::Database)?;

        match record {
            Some(rec) => {
                let verification_type = match rec.verification_type.as_str() {
                    "Email" => VerificationType::Email,
                    "Sms" => VerificationType::Sms,
                    _ => {
                        return Err(Error::Validation(format!(
                            "Invalid verification type: {}",
                            rec.verification_type
                        )));
                    },
                };
                let status = match rec.status.as_str() {
                    "Pending" => VerificationStatus::Pending,
                    "Verified" => VerificationStatus::Verified,
                    "Expired" => VerificationStatus::Expired,
                    "Invalidated" => VerificationStatus::Invalidated,
                    _ => {
                        return Err(Error::Validation(format!(
                            "Invalid verification status: {}",
                            rec.status
                        )));
                    },
                };

                Ok(Some(VerificationCode {
                    id: rec.id,
                    tenant_id: rec.tenant_id,
                    user_id: rec.user_id,
                    code: rec.code,
                    verification_type,
                    created_at: rec.created_at,
                    expires_at: rec.expires_at,
                    status,
                    attempts: rec.attempts as usize,
                }))
            },
            None => Ok(None),
        }
    }

    #[instrument(skip(self, _context), level = "debug")]
    async fn get_pending_by_user(
        &self,
        user_id: UserId,
        verification_type: VerificationType,
        tenant_id: TenantId,
        _context: &TenantAwareContext,
    ) -> Result<Vec<VerificationCode>> {
        let tenant_id_str = tenant_id.to_string();
        let user_id_str = user_id.to_string();
        let verification_type_str = format!("{:?}", verification_type);
        let status = format!("{:?}", VerificationStatus::Pending);

        let records = sqlx::query!(
            r#"
            SELECT 
                id, tenant_id, user_id, code, verification_type, 
                created_at, expires_at, status, attempts
            FROM 
                verification_codes
            WHERE 
                tenant_id::text = $1 AND user_id::text = $2 AND verification_type = $3 AND status = $4
            "#,
            tenant_id_str,
            user_id_str,
            verification_type_str,
            status
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::Database)?;

        let mut codes = Vec::with_capacity(records.len());
        for rec in records {
            let verification_type = match rec.verification_type.as_str() {
                "Email" => VerificationType::Email,
                "Sms" => VerificationType::Sms,
                _ => {
                    return Err(Error::Validation(format!(
                        "Invalid verification type: {}",
                        rec.verification_type
                    )));
                },
            };
            let status = match rec.status.as_str() {
                "Pending" => VerificationStatus::Pending,
                "Verified" => VerificationStatus::Verified,
                "Expired" => VerificationStatus::Expired,
                "Invalidated" => VerificationStatus::Invalidated,
                _ => {
                    return Err(Error::Validation(format!(
                        "Invalid verification status: {}",
                        rec.status
                    )));
                },
            };

            codes.push(VerificationCode {
                id: rec.id,
                tenant_id: rec.tenant_id,
                user_id: rec.user_id,
                code: rec.code,
                verification_type,
                created_at: rec.created_at,
                expires_at: rec.expires_at,
                status,
                attempts: rec.attempts as usize,
            });
        }

        Ok(codes)
    }

    #[instrument(skip(self, code, _context), level = "debug")]
    async fn update(&self, code: &VerificationCode, _context: &TenantAwareContext) -> Result<()> {
        let status = format!("{:?}", code.status);

        let result = sqlx::query!(
            r#"
            UPDATE verification_codes
            SET 
                code = $1, 
                expires_at = $2, 
                status = $3, 
                attempts = $4
            WHERE 
                id = $5 AND tenant_id = $6
            "#,
            code.code,
            code.expires_at,
            status,
            code.attempts as i32,
            code.id,
            code.tenant_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;

        if result.rows_affected() == 0 {
            return Err(Error::Validation("Verification code not found".to_string()));
        }

        trace!("Updated verification code with ID: {}", code.id);
        Ok(())
    }

    #[instrument(skip(self, _context), level = "debug")]
    async fn delete(
        &self,
        id: Uuid,
        tenant_id: TenantId,
        _context: &TenantAwareContext,
    ) -> Result<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM verification_codes
            WHERE id = $1 AND tenant_id = $2
            "#,
            id,
            tenant_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;

        if result.rows_affected() == 0 {
            return Err(Error::Validation("Verification code not found".to_string()));
        }

        trace!("Deleted verification code with ID: {}", id);
        Ok(())
    }

    #[instrument(skip(self, _context), level = "debug")]
    async fn delete_expired(
        &self,
        before: OffsetDateTime,
        tenant_id: TenantId,
        _context: &TenantAwareContext,
    ) -> Result<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM verification_codes
            WHERE tenant_id = $1 AND expires_at < $2
            "#,
            tenant_id,
            before
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;

        trace!(
            "Deleted {} expired verification codes",
            result.rows_affected()
        );
        Ok(result.rows_affected())
    }

    #[instrument(skip(self, _context), level = "debug")]
    async fn invalidate_pending(
        &self,
        user_id: UserId,
        verification_type: VerificationType,
        tenant_id: TenantId,
        _context: &TenantAwareContext,
    ) -> Result<u64> {
        let tenant_id_str = tenant_id.to_string();
        let user_id_str = user_id.to_string();
        let verification_type_str = format!("{:?}", verification_type);
        let pending_status = format!("{:?}", VerificationStatus::Pending);
        let invalidated_status = format!("{:?}", VerificationStatus::Invalidated);

        let result = sqlx::query!(
            r#"
            UPDATE verification_codes
            SET status = $1
            WHERE 
                tenant_id::text = $2 AND 
                user_id::text = $3 AND 
                verification_type = $4 AND 
                status = $5
            "#,
            invalidated_status,
            tenant_id_str,
            user_id_str,
            verification_type_str,
            pending_status
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;

        trace!(
            "Invalidated {} pending verification codes",
            result.rows_affected()
        );
        Ok(result.rows_affected())
    }

    #[instrument(skip(self, _context), level = "debug")]
    async fn count_recent_attempts(
        &self,
        user_id: UserId,
        verification_type: VerificationType,
        since: OffsetDateTime,
        tenant_id: TenantId,
        _context: &TenantAwareContext,
    ) -> Result<u64> {
        let tenant_id_str = tenant_id.to_string();
        let user_id_str = user_id.to_string();
        let verification_type_str = format!("{:?}", verification_type);

        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM verification_codes
            WHERE 
                tenant_id::text = $1 AND 
                user_id::text = $2 AND 
                verification_type = $3 AND 
                created_at > $4
            "#,
            tenant_id_str,
            user_id_str,
            verification_type_str,
            since
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::Database)?;

        Ok(result.count.unwrap_or(0) as u64)
    }
}
