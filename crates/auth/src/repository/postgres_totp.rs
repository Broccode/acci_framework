use crate::models::{TenantId, TotpSecret, UserId};
use crate::repository::{RepositoryError, TotpSecretRepository};
use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

/// PostgreSQL implementation of the TotpSecretRepository
pub struct PostgresTotpRepository {
    pool: Pool<Postgres>,
}

impl PostgresTotpRepository {
    /// Create a new PostgresTotpRepository
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TotpSecretRepository for PostgresTotpRepository {
    async fn save(&self, secret: &TotpSecret) -> Result<(), RepositoryError> {
        // Check if a secret already exists for this user in this tenant
        let existing = sqlx::query!(
            r#"
            SELECT id FROM totp_secrets 
            WHERE user_id = $1 AND tenant_id = $2
            "#,
            secret.user_id,
            secret.tenant_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        if let Some(row) = existing {
            // Update existing secret
            sqlx::query!(
                r#"
                UPDATE totp_secrets
                SET secret = $1, algorithm = $2, digits = $3, period = $4, 
                    recovery_codes = $5, enabled = $6, last_used_at = $7
                WHERE id = $8
                "#,
                secret.secret,
                secret.algorithm,
                secret.digits as i32,
                secret.period as i64,
                serde_json::to_value(&secret.recovery_codes)
                    .map_err(|e| RepositoryError::SerializationError(e.to_string()))?,
                secret.enabled,
                secret.last_used_at,
                row.id,
            )
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        } else {
            // Insert new secret
            sqlx::query!(
                r#"
                INSERT INTO totp_secrets (
                    id, user_id, tenant_id, secret, algorithm, digits, period, 
                    recovery_codes, enabled, created_at, last_used_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                "#,
                secret.id,
                secret.user_id,
                secret.tenant_id,
                secret.secret,
                secret.algorithm,
                secret.digits as i32,
                secret.period as i64,
                serde_json::to_value(&secret.recovery_codes)
                    .map_err(|e| RepositoryError::SerializationError(e.to_string()))?,
                secret.enabled,
                secret.created_at,
                secret.last_used_at,
            )
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

            // If enabled, update user's MFA settings
            if secret.enabled {
                sqlx::query!(
                    r#"
                    UPDATE users 
                    SET has_mfa_enabled = true, 
                        mfa_methods = 
                            CASE 
                                WHEN NOT mfa_methods::jsonb ? 'totp' 
                                THEN jsonb_set(mfa_methods, '{totp}', 'true'::jsonb, true)
                                ELSE mfa_methods
                            END
                    WHERE id = $1
                    "#,
                    secret.user_id,
                )
                .execute(&self.pool)
                .await
                .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
            }
        }

        Ok(())
    }

    async fn get_by_user_id(
        &self,
        user_id: &UserId,
        tenant_id: &TenantId,
    ) -> Result<Option<TotpSecret>, RepositoryError> {
        let row = sqlx::query!(
            r#"
            SELECT 
                id, user_id, tenant_id, secret, algorithm, digits, period, 
                recovery_codes, enabled, created_at, last_used_at
            FROM totp_secrets
            WHERE user_id = $1 AND tenant_id = $2
            "#,
            user_id,
            tenant_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let recovery_codes: Vec<String> = serde_json::from_value(row.recovery_codes)
                .map_err(|e| RepositoryError::DeserializationError(e.to_string()))?;

            Ok(Some(TotpSecret {
                id: row.id,
                user_id: row.user_id.try_into().map_err(|_| {
                    RepositoryError::DeserializationError("Invalid user ID".to_string())
                })?,
                tenant_id: row.tenant_id.try_into().map_err(|_| {
                    RepositoryError::DeserializationError("Invalid tenant ID".to_string())
                })?,
                secret: row.secret,
                algorithm: row.algorithm,
                digits: row.digits as u32,
                period: row.period as u64,
                recovery_codes,
                enabled: row.enabled,
                created_at: row.created_at,
                last_used_at: row.last_used_at,
            }))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, user_id: &UserId, tenant_id: &TenantId) -> Result<(), RepositoryError> {
        // Delete TOTP secret
        sqlx::query!(
            r#"
            DELETE FROM totp_secrets
            WHERE user_id = $1 AND tenant_id = $2
            "#,
            user_id,
            tenant_id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        // Update user's MFA settings
        sqlx::query!(
            r#"
            UPDATE users 
            SET has_mfa_enabled = EXISTS(
                    SELECT 1 FROM jsonb_object_keys(mfa_methods) k
                    WHERE k <> 'totp' AND mfa_methods->k::text = 'true'
                ), 
                mfa_methods = mfa_methods - 'totp'
            WHERE id = $1
            "#,
            user_id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_all_for_tenant(
        &self,
        tenant_id: &TenantId,
    ) -> Result<Vec<TotpSecret>, RepositoryError> {
        let rows = sqlx::query!(
            r#"
            SELECT 
                id, user_id, tenant_id, secret, algorithm, digits, period, 
                recovery_codes, enabled, created_at, last_used_at
            FROM totp_secrets
            WHERE tenant_id = $1
            "#,
            tenant_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let mut secrets = Vec::with_capacity(rows.len());
        for row in rows {
            let recovery_codes: Vec<String> = serde_json::from_value(row.recovery_codes)
                .map_err(|e| RepositoryError::DeserializationError(e.to_string()))?;

            secrets.push(TotpSecret {
                id: row.id,
                user_id: row.user_id.try_into().map_err(|_| {
                    RepositoryError::DeserializationError("Invalid user ID".to_string())
                })?,
                tenant_id: row.tenant_id.try_into().map_err(|_| {
                    RepositoryError::DeserializationError("Invalid tenant ID".to_string())
                })?,
                secret: row.secret,
                algorithm: row.algorithm,
                digits: row.digits as u32,
                period: row.period as u64,
                recovery_codes,
                enabled: row.enabled,
                created_at: row.created_at,
                last_used_at: row.last_used_at,
            });
        }

        Ok(secrets)
    }

    async fn get_by_id(
        &self,
        id: &Uuid,
        tenant_id: &TenantId,
    ) -> Result<Option<TotpSecret>, RepositoryError> {
        let row = sqlx::query!(
            r#"
            SELECT 
                id, user_id, tenant_id, secret, algorithm, digits, period, 
                recovery_codes, enabled, created_at, last_used_at
            FROM totp_secrets
            WHERE id = $1 AND tenant_id = $2
            "#,
            id,
            tenant_id,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        if let Some(row) = row {
            let recovery_codes: Vec<String> = serde_json::from_value(row.recovery_codes)
                .map_err(|e| RepositoryError::DeserializationError(e.to_string()))?;

            Ok(Some(TotpSecret {
                id: row.id,
                user_id: row.user_id.try_into().map_err(|_| {
                    RepositoryError::DeserializationError("Invalid user ID".to_string())
                })?,
                tenant_id: row.tenant_id.try_into().map_err(|_| {
                    RepositoryError::DeserializationError("Invalid tenant ID".to_string())
                })?,
                secret: row.secret,
                algorithm: row.algorithm,
                digits: row.digits as u32,
                period: row.period as u64,
                recovery_codes,
                enabled: row.enabled,
                created_at: row.created_at,
                last_used_at: row.last_used_at,
            }))
        } else {
            Ok(None)
        }
    }
}
