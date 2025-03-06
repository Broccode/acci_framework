use crate::models::tenant::{
    CreateSubscriptionDto, CreateTenantDto, CreateTenantUserDto, Tenant, TenantError,
    TenantPlanType, TenantRepository, TenantSubscription, TenantUser, UpdateSubscriptionDto,
    UpdateTenantDto, UpdateTenantUserDto,
};
use crate::models::user::{User, UserError, UserRepository};
use crate::repository::RepositoryError;
use crate::services::user::{UserService, UserServiceError};
use crate::utils::password::PasswordError;
use std::sync::Arc;
use thiserror::Error;
use time::OffsetDateTime;
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

/// Error types for tenant service operations
#[derive(Debug, Error)]
pub enum TenantServiceError {
    #[error("Tenant error: {0}")]
    Tenant(#[from] TenantError),

    #[error("User error: {0}")]
    User(#[from] UserError),

    #[error("User service error: {0}")]
    UserService(#[from] UserServiceError),

    #[error("Password error: {0}")]
    Password(#[from] PasswordError),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Feature not available: {0}")]
    FeatureNotAvailable(String),

    #[error("Tenant limit exceeded: {0}")]
    TenantLimitExceeded(String),
}

impl From<RepositoryError> for TenantServiceError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::TenantError(e) => TenantServiceError::Tenant(e),
            RepositoryError::NotFound => TenantServiceError::NotFound("Resource not found".into()),
            _ => TenantServiceError::Database(err.to_string()),
        }
    }
}

/// Data transfer object for tenant creation with admin user
#[derive(Debug)]
pub struct CreateTenantWithAdminDto {
    /// Tenant creation data
    pub tenant: CreateTenantDto,
    /// Admin user email
    pub admin_email: String,
    /// Admin user password
    pub admin_password: String,
    /// Initial subscription plan (optional)
    pub initial_plan: Option<TenantPlanType>,
}

/// Response for tenant creation with admin user
#[derive(Debug)]
pub struct TenantWithAdminResponse {
    /// Created tenant
    pub tenant: Tenant,
    /// Created admin user
    pub admin_user: User,
    /// Initial subscription (if created)
    pub subscription: Option<TenantSubscription>,
}

/// Service for managing tenants
pub struct TenantService {
    tenant_repository: Arc<dyn TenantRepository>,
    user_repository: Arc<dyn UserRepository>,
    user_service: Arc<UserService>,
}

impl TenantService {
    /// Creates a new tenant service
    pub fn new(
        tenant_repository: Arc<dyn TenantRepository>,
        user_repository: Arc<dyn UserRepository>,
        user_service: Arc<UserService>,
    ) -> Self {
        Self {
            tenant_repository,
            user_repository,
            user_service,
        }
    }

    /// Creates a new tenant
    #[instrument(skip(self, tenant), fields(tenant_name = %tenant.name))]
    pub async fn create_tenant(
        &self,
        tenant: CreateTenantDto,
    ) -> Result<Tenant, TenantServiceError> {
        debug!("Creating new tenant: {}", tenant.name);

        // Validate subdomain format
        self.validate_subdomain(&tenant.subdomain)?;

        // Create tenant
        let tenant = self.tenant_repository.create_tenant(tenant).await?;

        info!("Tenant created successfully: {}", tenant.id);
        Ok(tenant)
    }

    /// Creates a new tenant with an admin user
    #[instrument(skip(self, create_dto), fields(tenant_name = %create_dto.tenant.name))]
    pub async fn create_tenant_with_admin(
        &self,
        create_dto: CreateTenantWithAdminDto,
    ) -> Result<TenantWithAdminResponse, TenantServiceError> {
        debug!("Creating new tenant with admin: {}", create_dto.tenant.name);

        // Start by creating the tenant
        let tenant = self.create_tenant(create_dto.tenant).await?;

        // Create admin user
        let user = self
            .user_service
            .register(crate::models::user::CreateUser {
                email: create_dto.admin_email.clone(),
                password: create_dto.admin_password.clone(),
            })
            .await?;

        // Create the tenant-user association with admin role
        let _tenant_user = self
            .tenant_repository
            .add_user_to_tenant(
                tenant.id,
                CreateTenantUserDto {
                    user_id: user.id,
                    tenant_role: "ADMIN".to_string(),
                    is_active: Some(true),
                },
            )
            .await?;

        // Create subscription if initial plan is specified
        let subscription = if let Some(plan_type) = create_dto.initial_plan {
            let now = OffsetDateTime::now_utc();

            // Set expiration 1 year from now for paid plans, none for free
            let expires_at = match plan_type {
                TenantPlanType::Free => None,
                _ => Some(now + time::Duration::days(365)),
            };

            // Set user limits based on plan
            let max_users = match plan_type {
                TenantPlanType::Free => Some(5),
                TenantPlanType::Basic => Some(20),
                TenantPlanType::Professional => Some(100),
                TenantPlanType::Enterprise => Some(1000),
                TenantPlanType::Custom => None,
            };

            let subscription = self
                .tenant_repository
                .create_subscription(
                    tenant.id,
                    CreateSubscriptionDto {
                        plan_type,
                        starts_at: now,
                        expires_at,
                        is_active: Some(true),
                        payment_status: Some("PAID".to_string()),
                        max_users,
                        features: None,
                    },
                )
                .await?;

            Some(subscription)
        } else {
            None
        };

        info!("Tenant with admin created successfully: {}", tenant.id);
        Ok(TenantWithAdminResponse {
            tenant,
            admin_user: user,
            subscription,
        })
    }

    /// Gets a tenant by ID
    #[instrument(skip(self))]
    pub async fn get_tenant(&self, id: &Uuid) -> Result<Tenant, TenantServiceError> {
        debug!("Getting tenant: {}", id);

        let tenant = self
            .tenant_repository
            .find_tenant_by_id(*id)
            .await?
            .ok_or_else(|| TenantServiceError::NotFound(format!("Tenant not found: {}", id)))?;

        debug!("Tenant retrieved: {}", id);
        Ok(tenant)
    }

    /// Gets a tenant by subdomain
    #[instrument(skip(self))]
    pub async fn get_tenant_by_subdomain(
        &self,
        subdomain: &str,
    ) -> Result<Tenant, TenantServiceError> {
        debug!("Getting tenant by subdomain: {}", subdomain);

        let tenant = self
            .tenant_repository
            .find_tenant_by_subdomain(subdomain)
            .await?
            .ok_or_else(|| {
                TenantServiceError::NotFound(format!(
                    "Tenant not found for subdomain: {}",
                    subdomain
                ))
            })?;

        debug!("Tenant retrieved by subdomain: {}", subdomain);
        Ok(tenant)
    }

    /// Updates a tenant
    #[instrument(skip(self, update))]
    pub async fn update_tenant(
        &self,
        id: &Uuid,
        update: UpdateTenantDto,
    ) -> Result<Tenant, TenantServiceError> {
        debug!("Updating tenant: {}", id);

        // Validate subdomain if provided
        if let Some(subdomain) = &update.subdomain {
            self.validate_subdomain(subdomain)?;
        }

        let tenant = self.tenant_repository.update_tenant(*id, update).await?;

        info!("Tenant updated: {}", id);
        Ok(tenant)
    }

    /// Deletes a tenant
    #[instrument(skip(self))]
    pub async fn delete_tenant(&self, id: &Uuid) -> Result<(), TenantServiceError> {
        debug!("Deleting tenant: {}", id);

        self.tenant_repository.delete_tenant(*id).await?;

        info!("Tenant deleted: {}", id);
        Ok(())
    }

    /// Gets users for a tenant
    #[instrument(skip(self))]
    pub async fn get_tenant_users(
        &self,
        tenant_id: &Uuid,
    ) -> Result<Vec<TenantUser>, TenantServiceError> {
        debug!("Getting users for tenant: {}", tenant_id);

        let users = self.tenant_repository.get_tenant_users(*tenant_id).await?;

        debug!("Retrieved {} users for tenant {}", users.len(), tenant_id);
        Ok(users)
    }

    /// Gets tenants for a user
    #[instrument(skip(self))]
    pub async fn get_user_tenants(
        &self,
        user_id: &Uuid,
    ) -> Result<Vec<TenantUser>, TenantServiceError> {
        debug!("Getting tenants for user: {}", user_id);

        let tenants = self.tenant_repository.get_user_tenants(*user_id).await?;

        debug!("Retrieved {} tenants for user {}", tenants.len(), user_id);
        Ok(tenants)
    }

    /// Adds a user to a tenant
    #[instrument(skip(self, user))]
    pub async fn add_user_to_tenant(
        &self,
        tenant_id: &Uuid,
        user: CreateTenantUserDto,
    ) -> Result<TenantUser, TenantServiceError> {
        debug!("Adding user {} to tenant {}", user.user_id, tenant_id);

        // First, check tenant limits
        self.check_tenant_user_limits(tenant_id).await?;

        // Add user to tenant
        let tenant_user = self
            .tenant_repository
            .add_user_to_tenant(*tenant_id, user)
            .await?;

        info!(
            "User added to tenant: {} -> {}",
            tenant_user.user_id, tenant_id
        );
        Ok(tenant_user)
    }

    /// Updates a user's tenant association
    #[instrument(skip(self, update))]
    pub async fn update_tenant_user(
        &self,
        tenant_id: &Uuid,
        user_id: &Uuid,
        update: UpdateTenantUserDto,
    ) -> Result<TenantUser, TenantServiceError> {
        debug!("Updating user {} in tenant {}", user_id, tenant_id);

        let tenant_user = self
            .tenant_repository
            .update_tenant_user(*tenant_id, *user_id, update)
            .await?;

        info!("User updated in tenant: {} -> {}", user_id, tenant_id);
        Ok(tenant_user)
    }

    /// Removes a user from a tenant
    #[instrument(skip(self))]
    pub async fn remove_user_from_tenant(
        &self,
        tenant_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), TenantServiceError> {
        debug!("Removing user {} from tenant {}", user_id, tenant_id);

        // First check if this is the last admin user
        self.check_if_last_admin(tenant_id, user_id).await?;

        self.tenant_repository
            .remove_user_from_tenant(*tenant_id, *user_id)
            .await?;

        info!("User removed from tenant: {} -> {}", user_id, tenant_id);
        Ok(())
    }

    /// Gets the active subscription for a tenant
    #[instrument(skip(self))]
    pub async fn get_active_subscription(
        &self,
        tenant_id: &Uuid,
    ) -> Result<Option<TenantSubscription>, TenantServiceError> {
        debug!("Getting active subscription for tenant: {}", tenant_id);

        let subscription = self
            .tenant_repository
            .get_active_subscription(*tenant_id)
            .await?;

        if let Some(ref sub) = subscription {
            debug!(
                "Found active subscription for tenant {}: {}",
                tenant_id, sub.id
            );
        } else {
            debug!("No active subscription found for tenant {}", tenant_id);
        }

        Ok(subscription)
    }

    /// Creates a subscription for a tenant
    #[instrument(skip(self, subscription))]
    pub async fn create_subscription(
        &self,
        tenant_id: &Uuid,
        subscription: CreateSubscriptionDto,
    ) -> Result<TenantSubscription, TenantServiceError> {
        debug!("Creating subscription for tenant: {}", tenant_id);

        // Deactivate any current subscriptions
        self.deactivate_existing_subscriptions(tenant_id).await?;

        // Create new subscription
        let subscription = self
            .tenant_repository
            .create_subscription(*tenant_id, subscription)
            .await?;

        info!(
            "Subscription created for tenant {}: {}",
            tenant_id, subscription.id
        );
        Ok(subscription)
    }

    /// Updates a subscription
    #[instrument(skip(self, update))]
    pub async fn update_subscription(
        &self,
        id: &Uuid,
        update: UpdateSubscriptionDto,
    ) -> Result<TenantSubscription, TenantServiceError> {
        debug!("Updating subscription: {}", id);

        let subscription = self
            .tenant_repository
            .update_subscription(*id, update)
            .await?;

        info!("Subscription updated: {}", id);
        Ok(subscription)
    }

    /// Private utility functions

    // Validates a subdomain
    fn validate_subdomain(&self, subdomain: &str) -> Result<(), TenantServiceError> {
        // Only allow alphanumeric characters and hyphens
        if !subdomain.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err(TenantServiceError::InvalidInput(
                "Subdomain can only contain alphanumeric characters and hyphens".into(),
            ));
        }

        // Must start with a letter
        if !subdomain.chars().next().is_some_and(|c| c.is_alphabetic()) {
            return Err(TenantServiceError::InvalidInput(
                "Subdomain must start with a letter".into(),
            ));
        }

        // Must be at least 3 characters
        if subdomain.len() < 3 {
            return Err(TenantServiceError::InvalidInput(
                "Subdomain must be at least 3 characters long".into(),
            ));
        }

        // Must be at most 63 characters (DNS limit)
        if subdomain.len() > 63 {
            return Err(TenantServiceError::InvalidInput(
                "Subdomain must be at most 63 characters long".into(),
            ));
        }

        // Reserved subdomains
        let reserved = vec![
            "www",
            "mail",
            "smtp",
            "admin",
            "administrator",
            "blog",
            "dashboard",
            "api",
            "secure",
            "dev",
            "development",
            "staging",
            "prod",
            "production",
            "test",
            "billing",
            "support",
            "help",
            "sales",
            "connect",
            "login",
            "auth",
            "account",
        ];

        if reserved.contains(&subdomain.to_lowercase().as_str()) {
            return Err(TenantServiceError::InvalidInput(format!(
                "The subdomain '{}' is reserved and cannot be used",
                subdomain
            )));
        }

        Ok(())
    }

    // Deactivates existing subscriptions
    async fn deactivate_existing_subscriptions(
        &self,
        tenant_id: &Uuid,
    ) -> Result<(), TenantServiceError> {
        debug!(
            "Deactivating existing subscriptions for tenant: {}",
            tenant_id
        );

        // Get current active subscription
        if let Some(subscription) = self.get_active_subscription(tenant_id).await? {
            // Deactivate it
            self.update_subscription(
                &subscription.id,
                UpdateSubscriptionDto {
                    plan_type: None,
                    expires_at: None,
                    is_active: Some(false),
                    payment_status: None,
                    max_users: None,
                    features: None,
                },
            )
            .await?;
        }

        Ok(())
    }

    // Checks if user is the last admin in the tenant
    async fn check_if_last_admin(
        &self,
        tenant_id: &Uuid,
        user_id: &Uuid,
    ) -> Result<(), TenantServiceError> {
        debug!(
            "Checking if user {} is the last admin in tenant {}",
            user_id, tenant_id
        );

        // Get all tenant users
        let tenant_users = self.get_tenant_users(tenant_id).await?;

        // Find admins
        let admins: Vec<&TenantUser> = tenant_users
            .iter()
            .filter(|u| u.tenant_role.to_uppercase() == "ADMIN" && u.is_active)
            .collect();

        // If there's only one admin and it's this user, prevent removal
        if admins.len() == 1 && admins[0].user_id == *user_id {
            return Err(TenantServiceError::InvalidInput(
                "Cannot remove the last admin user from a tenant".into(),
            ));
        }

        Ok(())
    }

    // Checks if tenant has reached user limit
    async fn check_tenant_user_limits(&self, tenant_id: &Uuid) -> Result<(), TenantServiceError> {
        debug!("Checking user limits for tenant: {}", tenant_id);

        // Get active subscription
        let subscription = self.get_active_subscription(tenant_id).await?;

        // If there's an active subscription with a user limit
        if let Some(subscription) = subscription {
            if let Some(max_users) = subscription.max_users {
                // Count current users
                let current_users = self.get_tenant_users(tenant_id).await?;

                // Only count active users
                let active_users = current_users.iter().filter(|u| u.is_active).count() as i32;

                // Check if limit is reached
                if active_users >= max_users {
                    return Err(TenantServiceError::TenantLimitExceeded(format!(
                        "User limit of {} reached for tenant {}",
                        max_users, tenant_id
                    )));
                }
            }
        }

        Ok(())
    }

    /// Checks if user belongs to a tenant with specified role
    #[instrument(skip(self))]
    pub async fn check_user_tenant_role(
        &self,
        tenant_id: &Uuid,
        user_id: &Uuid,
        required_role: &str,
    ) -> Result<bool, TenantServiceError> {
        debug!(
            "Checking if user {} has role {} in tenant {}",
            user_id, required_role, tenant_id
        );

        // Get all tenant-user associations
        let tenant_users = self.tenant_repository.get_tenant_users(*tenant_id).await?;

        // Find this user's association
        let user_tenant = tenant_users.iter().find(|tu| tu.user_id == *user_id);

        // Check role if user association exists and is active
        match user_tenant {
            Some(tu) if tu.is_active => {
                // Check if roles match, case-insensitive
                let has_role = tu.tenant_role.to_uppercase() == required_role.to_uppercase();

                debug!(
                    "User {} has role {} in tenant {}: {}",
                    user_id, tu.tenant_role, tenant_id, has_role
                );

                Ok(has_role)
            },
            Some(_) => {
                debug!("User {} is inactive in tenant {}", user_id, tenant_id);
                Ok(false)
            },
            None => {
                debug!("User {} does not belong to tenant {}", user_id, tenant_id);
                Ok(false)
            },
        }
    }
}
