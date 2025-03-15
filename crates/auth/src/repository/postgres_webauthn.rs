use crate::{
    models::webauthn::{Credential, CredentialID},
    repository::{
        WebAuthnRepository,
        tenant_aware::{RepositoryError, TenantAwareContext},
    },
};
use async_trait::async_trait;
use sqlx::{Pool, Postgres, query};
use tracing::{debug, error, instrument};
use uuid::Uuid;

/// PostgreSQL implementation of the WebAuthn repository
pub struct PostgresWebAuthnRepository {
    pool: Pool<Postgres>,
    tenant_id: Option<Uuid>,
}

impl PostgresWebAuthnRepository {
    /// Create a new PostgreSQL WebAuthn repository
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            pool,
            tenant_id: None,
        }
    }

    /// Get the current tenant ID
    fn get_tenant_id(&self) -> Result<Uuid, RepositoryError> {
        self.tenant_id
            .ok_or_else(|| RepositoryError::TenantRequired)
    }

    /// Helper to map database rows to Credential objects
    async fn map_row_to_credential(
        &self,
        row: sqlx::postgres::PgRow,
    ) -> Result<Credential, RepositoryError> {
        let credential = Credential {
            uuid: row.try_get("uuid").map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to get uuid: {}", e))
            })?,
            id: CredentialID(row.try_get("credential_id").map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to get credential_id: {}", e))
            })?),
            user_id: row.try_get("user_id").map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to get user_id: {}", e))
            })?,
            tenant_id: row.try_get("tenant_id").map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to get tenant_id: {}", e))
            })?,
            name: row.try_get("name").map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to get name: {}", e))
            })?,
            aaguid: row.try_get("aaguid").map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to get aaguid: {}", e))
            })?,
            public_key: row.try_get("public_key").map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to get public_key: {}", e))
            })?,
            counter: row.try_get("counter").map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to get counter: {}", e))
            })?,
            created_at: row.try_get("created_at").map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to get created_at: {}", e))
            })?,
            last_used_at: row.try_get("last_used_at").map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to get last_used_at: {}", e))
            })?,
        };

        Ok(credential)
    }
}

impl TenantAwareContext for PostgresWebAuthnRepository {
    fn set_tenant_context(&self, tenant_id: &Uuid) -> Result<(), RepositoryError> {
        let tenant_id_str = tenant_id.to_string();

        // We need to set tenant_id here, then set the PostgreSQL setting at query time
        // First create a mutable copy of self
        let mut this = Self {
            pool: self.pool.clone(),
            tenant_id: Some(*tenant_id),
        };

        std::mem::swap(self, &mut this);

        debug!("Set tenant context to {}", tenant_id_str);
        Ok(())
    }
}

#[async_trait]
impl WebAuthnRepository for PostgresWebAuthnRepository {
    #[instrument(skip(self, credential), level = "debug")]
    async fn save_credential(&self, credential: &Credential) -> Result<(), RepositoryError> {
        let tenant_id = self.get_tenant_id()?;
        debug!(
            "Saving new credential: {} for tenant: {}",
            credential.id, tenant_id
        );

        // Start a transaction
        let mut tx = self.pool.begin().await.map_err(|e| {
            RepositoryError::DatabaseError(format!("Failed to begin transaction: {}", e))
        })?;

        // Set tenant context for this transaction
        query("SET LOCAL app.tenant_id = $1")
            .bind(tenant_id.to_string())
            .execute(&mut tx)
            .await
            .map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to set tenant context: {}", e))
            })?;

        // Insert the credential
        let result = query(
            r#"
            INSERT INTO webauthn_credentials (
                uuid, credential_id, user_id, tenant_id, name, 
                aaguid, public_key, counter, created_at, last_used_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(credential.uuid)
        .bind(credential.id.0.clone())
        .bind(credential.user_id)
        .bind(tenant_id)
        .bind(&credential.name)
        .bind(&credential.aaguid)
        .bind(&credential.public_key)
        .bind(credential.counter)
        .bind(credential.created_at)
        .bind(credential.last_used_at)
        .execute(&mut tx)
        .await;

        // Check for errors, especially uniqueness violations
        if let Err(e) = result {
            error!("Failed to save credential: {}", e);
            tx.rollback().await.map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to rollback transaction: {}", e))
            })?;

            if let Some(db_err) = e.as_database_error() {
                if let Some(code) = db_err.code() {
                    if code == "23505" {
                        // Unique violation
                        return Err(RepositoryError::UniqueViolation(
                            "Credential already exists".to_string(),
                        ));
                    }
                }
            }

            return Err(RepositoryError::DatabaseError(format!(
                "Failed to save credential: {}",
                e
            )));
        }

        // Commit the transaction
        tx.commit().await.map_err(|e| {
            RepositoryError::DatabaseError(format!("Failed to commit transaction: {}", e))
        })?;

        Ok(())
    }

    #[instrument(skip(self, credential), level = "debug")]
    async fn update_credential(&self, credential: &Credential) -> Result<(), RepositoryError> {
        let tenant_id = self.get_tenant_id()?;
        debug!(
            "Updating credential: {} for tenant: {}",
            credential.id, tenant_id
        );

        // Start a transaction
        let mut tx = self.pool.begin().await.map_err(|e| {
            RepositoryError::DatabaseError(format!("Failed to begin transaction: {}", e))
        })?;

        // Set tenant context for this transaction
        query("SET LOCAL app.tenant_id = $1")
            .bind(tenant_id.to_string())
            .execute(&mut tx)
            .await
            .map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to set tenant context: {}", e))
            })?;

        // Update the credential
        let result = query(
            r#"
            UPDATE webauthn_credentials
            SET 
                name = $1,
                counter = $2,
                last_used_at = $3
            WHERE uuid = $4 AND tenant_id = $5
            "#,
        )
        .bind(&credential.name)
        .bind(credential.counter)
        .bind(credential.last_used_at)
        .bind(credential.uuid)
        .bind(tenant_id)
        .execute(&mut tx)
        .await
        .map_err(|e| {
            RepositoryError::DatabaseError(format!("Failed to update credential: {}", e))
        })?;

        // Check if any rows were affected
        if result.rows_affected() == 0 {
            tx.rollback().await.map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to rollback transaction: {}", e))
            })?;
            return Err(RepositoryError::NotFound(
                "Credential not found".to_string(),
            ));
        }

        // Commit the transaction
        tx.commit().await.map_err(|e| {
            RepositoryError::DatabaseError(format!("Failed to commit transaction: {}", e))
        })?;

        Ok(())
    }

    #[instrument(skip(self), level = "debug")]
    async fn find_credential_by_id(
        &self,
        id: &CredentialID,
    ) -> Result<Option<Credential>, RepositoryError> {
        let tenant_id = self.get_tenant_id()?;
        debug!("Finding credential by ID: {} for tenant: {}", id, tenant_id);

        // Start a transaction
        let mut tx = self.pool.begin().await.map_err(|e| {
            RepositoryError::DatabaseError(format!("Failed to begin transaction: {}", e))
        })?;

        // Set tenant context for this transaction
        query("SET LOCAL app.tenant_id = $1")
            .bind(tenant_id.to_string())
            .execute(&mut tx)
            .await
            .map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to set tenant context: {}", e))
            })?;

        // Query for the credential
        let row_opt = query(
            r#"
            SELECT 
                uuid, credential_id, user_id, tenant_id, name,
                aaguid, public_key, counter, created_at, last_used_at
            FROM webauthn_credentials
            WHERE credential_id = $1 AND tenant_id = $2
            "#,
        )
        .bind(&id.0)
        .bind(tenant_id)
        .fetch_optional(&mut tx)
        .await
        .map_err(|e| RepositoryError::DatabaseError(format!("Failed to find credential: {}", e)))?;

        // Map the row to a Credential if found
        let credential_opt = match row_opt {
            Some(row) => {
                let credential = self.map_row_to_credential(row).await?;
                Some(credential)
            },
            None => None,
        };

        // Commit the transaction
        tx.commit().await.map_err(|e| {
            RepositoryError::DatabaseError(format!("Failed to commit transaction: {}", e))
        })?;

        Ok(credential_opt)
    }

    #[instrument(skip(self), level = "debug")]
    async fn find_credential_by_uuid(
        &self,
        uuid: &Uuid,
    ) -> Result<Option<Credential>, RepositoryError> {
        let tenant_id = self.get_tenant_id()?;
        debug!(
            "Finding credential by UUID: {} for tenant: {}",
            uuid, tenant_id
        );

        // Start a transaction
        let mut tx = self.pool.begin().await.map_err(|e| {
            RepositoryError::DatabaseError(format!("Failed to begin transaction: {}", e))
        })?;

        // Set tenant context for this transaction
        query("SET LOCAL app.tenant_id = $1")
            .bind(tenant_id.to_string())
            .execute(&mut tx)
            .await
            .map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to set tenant context: {}", e))
            })?;

        // Query for the credential
        let row_opt = query(
            r#"
            SELECT 
                uuid, credential_id, user_id, tenant_id, name,
                aaguid, public_key, counter, created_at, last_used_at
            FROM webauthn_credentials
            WHERE uuid = $1 AND tenant_id = $2
            "#,
        )
        .bind(uuid)
        .bind(tenant_id)
        .fetch_optional(&mut tx)
        .await
        .map_err(|e| RepositoryError::DatabaseError(format!("Failed to find credential: {}", e)))?;

        // Map the row to a Credential if found
        let credential_opt = match row_opt {
            Some(row) => {
                let credential = self.map_row_to_credential(row).await?;
                Some(credential)
            },
            None => None,
        };

        // Commit the transaction
        tx.commit().await.map_err(|e| {
            RepositoryError::DatabaseError(format!("Failed to commit transaction: {}", e))
        })?;

        Ok(credential_opt)
    }

    #[instrument(skip(self), level = "debug")]
    async fn list_credentials_for_user(
        &self,
        user_id: &Uuid,
    ) -> Result<Vec<Credential>, RepositoryError> {
        let tenant_id = self.get_tenant_id()?;
        debug!(
            "Listing credentials for user: {} in tenant: {}",
            user_id, tenant_id
        );

        // Start a transaction
        let mut tx = self.pool.begin().await.map_err(|e| {
            RepositoryError::DatabaseError(format!("Failed to begin transaction: {}", e))
        })?;

        // Set tenant context for this transaction
        query("SET LOCAL app.tenant_id = $1")
            .bind(tenant_id.to_string())
            .execute(&mut tx)
            .await
            .map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to set tenant context: {}", e))
            })?;

        // Query for all credentials for this user
        let rows = query(
            r#"
            SELECT 
                uuid, credential_id, user_id, tenant_id, name,
                aaguid, public_key, counter, created_at, last_used_at
            FROM webauthn_credentials
            WHERE user_id = $1 AND tenant_id = $2
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .bind(tenant_id)
        .fetch_all(&mut tx)
        .await
        .map_err(|e| {
            RepositoryError::DatabaseError(format!("Failed to list credentials: {}", e))
        })?;

        // Map the rows to Credential objects
        let mut credentials = Vec::with_capacity(rows.len());
        for row in rows {
            let credential = self.map_row_to_credential(row).await?;
            credentials.push(credential);
        }

        // Commit the transaction
        tx.commit().await.map_err(|e| {
            RepositoryError::DatabaseError(format!("Failed to commit transaction: {}", e))
        })?;

        Ok(credentials)
    }

    #[instrument(skip(self), level = "debug")]
    async fn delete_credential(&self, uuid: &Uuid) -> Result<(), RepositoryError> {
        let tenant_id = self.get_tenant_id()?;
        debug!("Deleting credential: {} for tenant: {}", uuid, tenant_id);

        // Start a transaction
        let mut tx = self.pool.begin().await.map_err(|e| {
            RepositoryError::DatabaseError(format!("Failed to begin transaction: {}", e))
        })?;

        // Set tenant context for this transaction
        query("SET LOCAL app.tenant_id = $1")
            .bind(tenant_id.to_string())
            .execute(&mut tx)
            .await
            .map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to set tenant context: {}", e))
            })?;

        // Delete the credential
        let result = query(
            r#"
            DELETE FROM webauthn_credentials
            WHERE uuid = $1 AND tenant_id = $2
            "#,
        )
        .bind(uuid)
        .bind(tenant_id)
        .execute(&mut tx)
        .await
        .map_err(|e| {
            RepositoryError::DatabaseError(format!("Failed to delete credential: {}", e))
        })?;

        // Check if any rows were affected
        if result.rows_affected() == 0 {
            tx.rollback().await.map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to rollback transaction: {}", e))
            })?;
            return Err(RepositoryError::NotFound(
                "Credential not found".to_string(),
            ));
        }

        // Commit the transaction
        tx.commit().await.map_err(|e| {
            RepositoryError::DatabaseError(format!("Failed to commit transaction: {}", e))
        })?;

        Ok(())
    }

    #[instrument(skip(self), level = "debug")]
    async fn delete_credentials_for_user(&self, user_id: &Uuid) -> Result<u64, RepositoryError> {
        let tenant_id = self.get_tenant_id()?;
        debug!(
            "Deleting all credentials for user: {} in tenant: {}",
            user_id, tenant_id
        );

        // Start a transaction
        let mut tx = self.pool.begin().await.map_err(|e| {
            RepositoryError::DatabaseError(format!("Failed to begin transaction: {}", e))
        })?;

        // Set tenant context for this transaction
        query("SET LOCAL app.tenant_id = $1")
            .bind(tenant_id.to_string())
            .execute(&mut tx)
            .await
            .map_err(|e| {
                RepositoryError::DatabaseError(format!("Failed to set tenant context: {}", e))
            })?;

        // Delete all credentials for this user
        let result = query(
            r#"
            DELETE FROM webauthn_credentials
            WHERE user_id = $1 AND tenant_id = $2
            "#,
        )
        .bind(user_id)
        .bind(tenant_id)
        .execute(&mut tx)
        .await
        .map_err(|e| {
            RepositoryError::DatabaseError(format!("Failed to delete credentials: {}", e))
        })?;

        let rows_affected = result.rows_affected();

        // Commit the transaction
        tx.commit().await.map_err(|e| {
            RepositoryError::DatabaseError(format!("Failed to commit transaction: {}", e))
        })?;

        Ok(rows_affected)
    }
}
