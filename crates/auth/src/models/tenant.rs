use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

/// Tenant identifier type
pub type TenantId = Uuid;

/// Represents a tenant organization in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    /// Unique identifier for the tenant
    pub id: Uuid,
    /// Name of the tenant organization
    pub name: String,
    /// Unique subdomain for tenant access
    pub subdomain: String,
    /// Whether the tenant is currently active
    pub is_active: bool,
    /// When the tenant was created
    pub created_at: OffsetDateTime,
    /// When the tenant was last updated
    pub updated_at: OffsetDateTime,
    /// Additional tenant metadata as JSON
    pub metadata: Option<JsonValue>,
}

/// Available subscription plans for tenants
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TenantPlanType {
    Free,
    Basic,
    Professional,
    Enterprise,
    Custom,
}

// Add SQLx type mapping for PostgreSQL
impl sqlx::Type<sqlx::Postgres> for TenantPlanType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("tenant_plan_type")
    }
}

// Implement Display for TenantPlanType
impl std::fmt::Display for TenantPlanType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TenantPlanType::Free => write!(f, "FREE"),
            TenantPlanType::Basic => write!(f, "BASIC"),
            TenantPlanType::Professional => write!(f, "PROFESSIONAL"),
            TenantPlanType::Enterprise => write!(f, "ENTERPRISE"),
            TenantPlanType::Custom => write!(f, "CUSTOM"),
        }
    }
}

// Implement From/Into for converting between string and enum
impl From<&str> for TenantPlanType {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "FREE" => TenantPlanType::Free,
            "BASIC" => TenantPlanType::Basic,
            "PROFESSIONAL" => TenantPlanType::Professional,
            "ENTERPRISE" => TenantPlanType::Enterprise,
            _ => TenantPlanType::Custom,
        }
    }
}

// Implement Decode for TenantPlanType
impl<'r> sqlx::Decode<'r, sqlx::Postgres> for TenantPlanType {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Ok(Self::from(value))
    }
}

// Implement Encode for TenantPlanType
impl sqlx::Encode<'_, sqlx::Postgres> for TenantPlanType {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = self.to_string();
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s.as_str(), buf)
    }
}

/// Represents a tenant's subscription plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantSubscription {
    /// Unique identifier for the subscription
    pub id: Uuid,
    /// Associated tenant ID
    pub tenant_id: Uuid,
    /// Type of subscription plan
    pub plan_type: TenantPlanType,
    /// When the subscription starts
    pub starts_at: OffsetDateTime,
    /// When the subscription expires (if applicable)
    pub expires_at: Option<OffsetDateTime>,
    /// Whether the subscription is currently active
    pub is_active: bool,
    /// Current payment status
    pub payment_status: Option<String>,
    /// Maximum number of users allowed
    pub max_users: Option<i32>,
    /// Features available in this subscription as JSON
    pub features: Option<JsonValue>,
    /// When the subscription was created
    pub created_at: OffsetDateTime,
    /// When the subscription was last updated
    pub updated_at: OffsetDateTime,
}

/// Represents the association between a user and a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantUser {
    /// Associated tenant ID
    pub tenant_id: Uuid,
    /// Associated user ID
    pub user_id: Uuid,
    /// Role of the user within this tenant
    pub tenant_role: String,
    /// Whether the user is active within this tenant
    pub is_active: bool,
    /// When the association was created
    pub created_at: OffsetDateTime,
    /// When the association was last updated
    pub updated_at: OffsetDateTime,
}

/// Tenant creation data transfer object
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTenantDto {
    /// Name of the tenant organization
    pub name: String,
    /// Unique subdomain for tenant access
    pub subdomain: String,
    /// Optional initial metadata
    pub metadata: Option<JsonValue>,
}

/// Tenant update data transfer object
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTenantDto {
    /// Optional new name for the tenant
    pub name: Option<String>,
    /// Optional new subdomain for the tenant
    pub subdomain: Option<String>,
    /// Optional active status update
    pub is_active: Option<bool>,
    /// Optional metadata update
    pub metadata: Option<JsonValue>,
}

/// Subscription creation data transfer object
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSubscriptionDto {
    /// Type of subscription plan
    pub plan_type: TenantPlanType,
    /// When the subscription starts
    pub starts_at: OffsetDateTime,
    /// When the subscription expires (if applicable)
    pub expires_at: Option<OffsetDateTime>,
    /// Whether the subscription is active
    pub is_active: Option<bool>,
    /// Current payment status
    pub payment_status: Option<String>,
    /// Maximum number of users allowed
    pub max_users: Option<i32>,
    /// Features available in this subscription
    pub features: Option<JsonValue>,
}

/// Subscription update data transfer object
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSubscriptionDto {
    /// Optional new plan type
    pub plan_type: Option<TenantPlanType>,
    /// Optional new expiration date
    pub expires_at: Option<OffsetDateTime>,
    /// Optional active status update
    pub is_active: Option<bool>,
    /// Optional payment status update
    pub payment_status: Option<String>,
    /// Optional max users update
    pub max_users: Option<i32>,
    /// Optional features update
    pub features: Option<JsonValue>,
}

/// Tenant user association creation data transfer object
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTenantUserDto {
    /// User ID to associate with tenant
    pub user_id: Uuid,
    /// Role for the user within this tenant
    pub tenant_role: String,
    /// Whether the user should be active
    pub is_active: Option<bool>,
}

/// Tenant user association update data transfer object
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTenantUserDto {
    /// Optional new role for the user
    pub tenant_role: Option<String>,
    /// Optional active status update
    pub is_active: Option<bool>,
}

/// Possible errors that can occur during tenant operations
#[derive(Error, Debug)]
pub enum TenantError {
    #[error("Tenant not found")]
    NotFound,

    #[error("Tenant already exists")]
    AlreadyExists,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Invalid data: {0}")]
    ValidationError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Unauthorized action")]
    Unauthorized,

    #[error("Service unavailable")]
    ServiceUnavailable,

    #[error("Inactive tenant")]
    InactiveTenant,

    #[error("Subscription expired")]
    SubscriptionExpired,

    #[error("User limit exceeded")]
    UserLimitExceeded,
}

/// Repository trait for tenant operations
#[async_trait]
pub trait TenantRepository: Send + Sync {
    /// Creates a new tenant
    async fn create_tenant(&self, tenant: CreateTenantDto) -> Result<Tenant, TenantError>;

    /// Finds a tenant by ID
    async fn find_tenant_by_id(&self, id: Uuid) -> Result<Option<Tenant>, TenantError>;

    /// Finds a tenant by subdomain
    async fn find_tenant_by_subdomain(
        &self,
        subdomain: &str,
    ) -> Result<Option<Tenant>, TenantError>;

    /// Updates a tenant
    async fn update_tenant(&self, id: Uuid, tenant: UpdateTenantDto)
    -> Result<Tenant, TenantError>;

    /// Deletes a tenant
    async fn delete_tenant(&self, id: Uuid) -> Result<(), TenantError>;

    /// Creates a subscription for a tenant
    async fn create_subscription(
        &self,
        tenant_id: Uuid,
        subscription: CreateSubscriptionDto,
    ) -> Result<TenantSubscription, TenantError>;

    /// Gets the active subscription for a tenant
    async fn get_active_subscription(
        &self,
        tenant_id: Uuid,
    ) -> Result<Option<TenantSubscription>, TenantError>;

    /// Updates a subscription
    async fn update_subscription(
        &self,
        id: Uuid,
        subscription: UpdateSubscriptionDto,
    ) -> Result<TenantSubscription, TenantError>;

    /// Adds a user to a tenant
    async fn add_user_to_tenant(
        &self,
        tenant_id: Uuid,
        user: CreateTenantUserDto,
    ) -> Result<TenantUser, TenantError>;

    /// Gets users for a tenant
    async fn get_tenant_users(&self, tenant_id: Uuid) -> Result<Vec<TenantUser>, TenantError>;

    /// Gets tenants for a user
    async fn get_user_tenants(&self, user_id: Uuid) -> Result<Vec<TenantUser>, TenantError>;

    /// Updates a user's tenant association
    async fn update_tenant_user(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
        update: UpdateTenantUserDto,
    ) -> Result<TenantUser, TenantError>;

    /// Removes a user from a tenant
    async fn remove_user_from_tenant(
        &self,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), TenantError>;
}
