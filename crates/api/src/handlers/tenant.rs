use crate::middleware::tenant::TenantContext;
use crate::monitoring;
use crate::response::{ApiError, ApiResponse};
use crate::validation::{generate_request_id, validate_json_payload};
use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
// lazy_static is imported in the regex module below
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;
use validator::Validate;

use acci_auth::{
    CreateTenantDto, CreateTenantWithAdminDto, TenantPlanType, TenantService, TenantServiceError,
    UpdateTenantDto, utils::jwt::Claims,
};

/// Module with regex patterns
pub mod regex {
    use lazy_static::lazy_static;
    use ::regex::Regex;

    lazy_static! {
        pub static ref SUBDOMAIN_REGEX: Regex = Regex::new(r"^[a-zA-Z][a-zA-Z0-9\-]*$").unwrap();
    }
}

/// API application state for tenant operations
#[derive(Clone)]
pub struct TenantAppState {
    /// Tenant service for tenant management
    pub tenant_service: Arc<TenantService>,
}

/// Create tenant request DTO
#[derive(Debug, Deserialize, Validate)]
pub struct CreateTenantRequest {
    #[validate(length(
        min = 3,
        max = 100,
        message = "Name must be between 3 and 100 characters"
    ))]
    pub name: String,

    #[validate(length(
        min = 3,
        max = 63,
        message = "Subdomain must be between 3 and 63 characters"
    ))]
    pub subdomain: String,

    pub metadata: Option<serde_json::Value>,
}

impl CreateTenantRequest {
    pub fn validate_subdomain(&self) -> Result<(), String> {
        if !regex::SUBDOMAIN_REGEX.is_match(&self.subdomain) {
            return Err("Subdomain can only contain letters, numbers, and hyphens, and must start with a letter".to_string());
        }
        Ok(())
    }
}

/// Create tenant with admin user request DTO
#[derive(Debug, Deserialize, Validate)]
pub struct CreateTenantWithAdminRequest {
    #[validate(nested)]
    pub tenant: CreateTenantRequest,

    #[validate(email(message = "Invalid admin email format"))]
    pub admin_email: String,

    #[validate(length(min = 8, message = "Admin password must be at least 8 characters long"))]
    pub admin_password: String,

    #[validate(must_match(
        other = "admin_password",
        message = "Password confirmation does not match"
    ))]
    pub admin_password_confirmation: String,

    pub plan: Option<String>,
}

/// Tenant response DTO
#[derive(Debug, Serialize)]
pub struct TenantResponse {
    pub id: String,
    pub name: String,
    pub subdomain: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
    pub metadata: Option<serde_json::Value>,
}

/// Create tenant handler
#[axum::debug_handler]
pub async fn create_tenant(
    State(state): State<TenantAppState>,
    Json(request): Json<CreateTenantRequest>,
) -> Response {
    debug!("Processing create tenant request");
    let start = std::time::Instant::now();

    // Generate a unique request ID
    let request_id = generate_request_id();

    // Validate the request
    let validated = match validate_json_payload(Json(request)).await {
        Ok(data) => data,
        Err(validation_error) => {
            return validation_error.into_response();
        },
    };
    
    // Validate the subdomain
    if let Err(message) = validated.validate_subdomain() {
        return ApiResponse::<()>::error(
            message,
            "INVALID_SUBDOMAIN",
            request_id,
        ).into_response();
    }

    // Convert to domain DTO
    let create_tenant = CreateTenantDto {
        name: validated.name,
        subdomain: validated.subdomain,
        metadata: validated.metadata,
    };

    // Create tenant
    match state.tenant_service.create_tenant(create_tenant).await {
        Ok(tenant) => {
            // Record success
            monitoring::record_tenant_operation("create", "success");

            // Record duration
            let duration = start.elapsed();
            monitoring::record_request_duration(duration.as_secs_f64(), "POST", "/tenants");

            // Successful creation
            let response = TenantResponse {
                id: tenant.id.to_string(),
                name: tenant.name,
                subdomain: tenant.subdomain,
                is_active: tenant.is_active,
                created_at: tenant.created_at.to_string(),
                updated_at: tenant.updated_at.to_string(),
                metadata: tenant.metadata,
            };

            info!(
                request_id = %request_id,
                tenant_id = %tenant.id,
                "Tenant created successfully"
            );

            let api_response = ApiResponse::success(response, request_id);
            (StatusCode::CREATED, Json(api_response)).into_response()
        },
        Err(err) => {
            // Record failure
            monitoring::record_tenant_operation("create", "failure");

            // Map error to appropriate response
            let (status, message, code) = match err {
                TenantServiceError::Tenant(ref tenant_err) => match tenant_err {
                    acci_auth::TenantError::AlreadyExists => (
                        StatusCode::CONFLICT,
                        "Tenant with this subdomain already exists",
                        "TENANT_ALREADY_EXISTS",
                    ),
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "An error occurred creating the tenant",
                        "TENANT_CREATION_ERROR",
                    ),
                },
                TenantServiceError::InvalidInput(ref msg) => {
                    (StatusCode::BAD_REQUEST, msg.as_str(), "INVALID_INPUT")
                },
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An error occurred creating the tenant",
                    "TENANT_CREATION_ERROR",
                ),
            };

            warn!(
                request_id = %request_id,
                error = %err,
                "Tenant creation failed"
            );

            ApiError::new(status, message, code, request_id).into_response()
        },
    }
}

/// Create tenant with admin handler
#[axum::debug_handler]
pub async fn create_tenant_with_admin(
    State(state): State<TenantAppState>,
    Json(request): Json<CreateTenantWithAdminRequest>,
) -> Response {
    debug!("Processing create tenant with admin request");
    let start = std::time::Instant::now();

    // Generate a unique request ID
    let request_id = generate_request_id();

    // Validate the request
    let validated = match validate_json_payload(Json(request)).await {
        Ok(data) => data,
        Err(validation_error) => {
            return validation_error.into_response();
        },
    };

    // Parse plan type if provided
    let plan_type = if let Some(plan_str) = validated.plan {
        match plan_str.to_uppercase().as_str() {
            "FREE" => Some(TenantPlanType::Free),
            "BASIC" => Some(TenantPlanType::Basic),
            "PROFESSIONAL" => Some(TenantPlanType::Professional),
            "ENTERPRISE" => Some(TenantPlanType::Enterprise),
            "CUSTOM" => Some(TenantPlanType::Custom),
            _ => {
                let error = ApiError::new(
                    StatusCode::BAD_REQUEST,
                    "Invalid plan type",
                    "INVALID_PLAN_TYPE",
                    request_id,
                );
                return error.into_response();
            },
        }
    } else {
        Some(TenantPlanType::Free) // Default to free plan
    };

    // Convert to domain DTO
    let create_tenant = CreateTenantDto {
        name: validated.tenant.name,
        subdomain: validated.tenant.subdomain,
        metadata: validated.tenant.metadata,
    };

    let create_dto = CreateTenantWithAdminDto {
        tenant: create_tenant,
        admin_email: validated.admin_email,
        admin_password: validated.admin_password,
        initial_plan: plan_type,
    };

    // Create tenant with admin
    match state
        .tenant_service
        .create_tenant_with_admin(create_dto)
        .await
    {
        Ok(result) => {
            // Record success
            monitoring::record_tenant_operation("create_with_admin", "success");

            // Record duration
            let duration = start.elapsed();
            monitoring::record_request_duration(
                duration.as_secs_f64(),
                "POST",
                "/tenants/with-admin",
            );

            // Construct response
            let tenant_response = TenantResponse {
                id: result.tenant.id.to_string(),
                name: result.tenant.name,
                subdomain: result.tenant.subdomain,
                is_active: result.tenant.is_active,
                created_at: result.tenant.created_at.to_string(),
                updated_at: result.tenant.updated_at.to_string(),
                metadata: result.tenant.metadata,
            };

            // Structure response data
            let response_data = serde_json::json!({
                "tenant": tenant_response,
                "admin_user_id": result.admin_user.id.to_string(),
                "admin_email": result.admin_user.email,
                "has_subscription": result.subscription.is_some(),
                "subscription_plan": result.subscription.map(|s| format!("{:?}", s.plan_type)),
            });

            info!(
                request_id = %request_id,
                tenant_id = %result.tenant.id,
                user_id = %result.admin_user.id,
                "Tenant with admin created successfully"
            );

            let api_response = ApiResponse::success(response_data, request_id);
            (StatusCode::CREATED, Json(api_response)).into_response()
        },
        Err(err) => {
            // Record failure
            monitoring::record_tenant_operation("create_with_admin", "failure");

            // Map error to appropriate response
            let (status, message, code) = match err {
                TenantServiceError::Tenant(ref tenant_err) => match tenant_err {
                    acci_auth::TenantError::AlreadyExists => (
                        StatusCode::CONFLICT,
                        "Tenant with this subdomain already exists",
                        "TENANT_ALREADY_EXISTS",
                    ),
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "An error occurred creating the tenant",
                        "TENANT_CREATION_ERROR",
                    ),
                },
                TenantServiceError::User(ref user_err) => match user_err {
                    acci_auth::UserError::AlreadyExists => (
                        StatusCode::CONFLICT,
                        "User with this email already exists",
                        "USER_ALREADY_EXISTS",
                    ),
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "An error occurred creating the admin user",
                        "USER_CREATION_ERROR",
                    ),
                },
                TenantServiceError::InvalidInput(ref msg) => {
                    (StatusCode::BAD_REQUEST, msg.as_str(), "INVALID_INPUT")
                },
                TenantServiceError::Password(_) => (
                    StatusCode::BAD_REQUEST,
                    "Password does not meet security requirements",
                    "WEAK_PASSWORD",
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An error occurred creating the tenant with admin",
                    "TENANT_CREATION_ERROR",
                ),
            };

            warn!(
                request_id = %request_id,
                error = %err,
                "Tenant with admin creation failed"
            );

            ApiError::new(status, message, code, request_id).into_response()
        },
    }
}

/// Get tenant handler
#[axum::debug_handler]
pub async fn get_tenant(
    State(state): State<TenantAppState>,
    Extension(tenant_context): Extension<TenantContext>,
) -> Response {
    debug!("Processing get tenant request");
    let start = std::time::Instant::now();

    // Generate a unique request ID
    let request_id = generate_request_id();

    // Get tenant from context
    let tenant_id = tenant_context.id;

    // Get tenant details
    match state.tenant_service.get_tenant(&tenant_id).await {
        Ok(tenant) => {
            // Record success
            monitoring::record_tenant_operation("get", "success");

            // Record duration
            let duration = start.elapsed();
            monitoring::record_request_duration(duration.as_secs_f64(), "GET", "/tenant");

            // Successful retrieval
            let response = TenantResponse {
                id: tenant.id.to_string(),
                name: tenant.name,
                subdomain: tenant.subdomain,
                is_active: tenant.is_active,
                created_at: tenant.created_at.to_string(),
                updated_at: tenant.updated_at.to_string(),
                metadata: tenant.metadata,
            };

            debug!(
                request_id = %request_id,
                tenant_id = %tenant.id,
                "Tenant retrieved successfully"
            );

            let api_response = ApiResponse::success(response, request_id);
            (StatusCode::OK, Json(api_response)).into_response()
        },
        Err(err) => {
            // Record failure
            monitoring::record_tenant_operation("get", "failure");

            // Map error to appropriate response
            let (status, message, code) = match err {
                TenantServiceError::NotFound(_) => (
                    StatusCode::NOT_FOUND,
                    "Tenant not found",
                    "TENANT_NOT_FOUND",
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An error occurred retrieving the tenant",
                    "TENANT_RETRIEVAL_ERROR",
                ),
            };

            warn!(
                request_id = %request_id,
                error = %err,
                tenant_id = %tenant_id,
                "Tenant retrieval failed"
            );

            ApiError::new(status, message, code, request_id).into_response()
        },
    }
}

/// Get tenant by ID handler (admin operation)
#[axum::debug_handler]
pub async fn get_tenant_by_id(
    State(state): State<TenantAppState>,
    Path(tenant_id): Path<String>,
    Extension(_claims): Extension<Claims>,
) -> Response {
    debug!("Processing get tenant by ID request");
    let start = std::time::Instant::now();

    // Generate a unique request ID
    let request_id = generate_request_id();

    // Parse tenant ID
    let tenant_id = match Uuid::parse_str(&tenant_id) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::new(
                StatusCode::BAD_REQUEST,
                "Invalid tenant ID format",
                "INVALID_TENANT_ID",
                request_id,
            )
            .into_response();
        },
    };

    // Get tenant details
    match state.tenant_service.get_tenant(&tenant_id).await {
        Ok(tenant) => {
            // Record success
            monitoring::record_tenant_operation("get_by_id", "success");

            // Record duration
            let duration = start.elapsed();
            monitoring::record_request_duration(duration.as_secs_f64(), "GET", "/tenants/:id");

            // Successful retrieval
            let response = TenantResponse {
                id: tenant.id.to_string(),
                name: tenant.name,
                subdomain: tenant.subdomain,
                is_active: tenant.is_active,
                created_at: tenant.created_at.to_string(),
                updated_at: tenant.updated_at.to_string(),
                metadata: tenant.metadata,
            };

            debug!(
                request_id = %request_id,
                tenant_id = %tenant.id,
                "Tenant retrieved successfully by ID"
            );

            let api_response = ApiResponse::success(response, request_id);
            (StatusCode::OK, Json(api_response)).into_response()
        },
        Err(err) => {
            // Record failure
            monitoring::record_tenant_operation("get_by_id", "failure");

            // Map error to appropriate response
            let (status, message, code) = match err {
                TenantServiceError::NotFound(_) => (
                    StatusCode::NOT_FOUND,
                    "Tenant not found",
                    "TENANT_NOT_FOUND",
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An error occurred retrieving the tenant",
                    "TENANT_RETRIEVAL_ERROR",
                ),
            };

            warn!(
                request_id = %request_id,
                error = %err,
                tenant_id = %tenant_id,
                "Tenant retrieval by ID failed"
            );

            ApiError::new(status, message, code, request_id).into_response()
        },
    }
}

/// Update tenant request DTO
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTenantRequest {
    #[validate(length(
        min = 3,
        max = 100,
        message = "Name must be between 3 and 100 characters"
    ))]
    pub name: Option<String>,

    #[validate(length(
        min = 3,
        max = 63,
        message = "Subdomain must be between 3 and 63 characters"
    ))]
    pub subdomain: Option<String>,

    pub is_active: Option<bool>,

    pub metadata: Option<serde_json::Value>,
}

impl UpdateTenantRequest {
    pub fn validate_subdomain(&self) -> Result<(), String> {
        if let Some(subdomain) = &self.subdomain {
            if !regex::SUBDOMAIN_REGEX.is_match(subdomain) {
                return Err("Subdomain can only contain letters, numbers, and hyphens, and must start with a letter".to_string());
            }
        }
        Ok(())
    }
}

/// Update tenant handler
#[axum::debug_handler]
pub async fn update_tenant(
    State(state): State<TenantAppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(request): Json<UpdateTenantRequest>,
) -> Response {
    debug!("Processing update tenant request");
    let start = std::time::Instant::now();

    // Generate a unique request ID
    let request_id = generate_request_id();

    // Validate the request
    let validated = match validate_json_payload(Json(request)).await {
        Ok(data) => data,
        Err(validation_error) => {
            return validation_error.into_response();
        },
    };
    
    // Validate the subdomain if provided
    if let Err(message) = validated.validate_subdomain() {
        return ApiResponse::<()>::error(
            message,
            "INVALID_SUBDOMAIN",
            request_id,
        ).into_response();
    }

    // Get tenant ID from context
    let tenant_id = tenant_context.id;

    // Convert to domain DTO
    let update_tenant = UpdateTenantDto {
        name: validated.name,
        subdomain: validated.subdomain,
        is_active: validated.is_active,
        metadata: validated.metadata,
    };

    // Update tenant
    match state
        .tenant_service
        .update_tenant(&tenant_id, update_tenant)
        .await
    {
        Ok(tenant) => {
            // Record success
            monitoring::record_tenant_operation("update", "success");

            // Record duration
            let duration = start.elapsed();
            monitoring::record_request_duration(duration.as_secs_f64(), "PUT", "/tenant");

            // Successful update
            let response = TenantResponse {
                id: tenant.id.to_string(),
                name: tenant.name,
                subdomain: tenant.subdomain,
                is_active: tenant.is_active,
                created_at: tenant.created_at.to_string(),
                updated_at: tenant.updated_at.to_string(),
                metadata: tenant.metadata,
            };

            info!(
                request_id = %request_id,
                tenant_id = %tenant.id,
                "Tenant updated successfully"
            );

            let api_response = ApiResponse::success(response, request_id);
            (StatusCode::OK, Json(api_response)).into_response()
        },
        Err(err) => {
            // Record failure
            monitoring::record_tenant_operation("update", "failure");

            // Map error to appropriate response
            let (status, message, code) = match err {
                TenantServiceError::NotFound(_) => (
                    StatusCode::NOT_FOUND,
                    "Tenant not found",
                    "TENANT_NOT_FOUND",
                ),
                TenantServiceError::Tenant(ref tenant_err) => match tenant_err {
                    acci_auth::TenantError::AlreadyExists => (
                        StatusCode::CONFLICT,
                        "Tenant with this subdomain already exists",
                        "TENANT_ALREADY_EXISTS",
                    ),
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "An error occurred updating the tenant",
                        "TENANT_UPDATE_ERROR",
                    ),
                },
                TenantServiceError::InvalidInput(ref msg) => {
                    (StatusCode::BAD_REQUEST, msg.as_str(), "INVALID_INPUT")
                },
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An error occurred updating the tenant",
                    "TENANT_UPDATE_ERROR",
                ),
            };

            warn!(
                request_id = %request_id,
                error = %err,
                tenant_id = %tenant_id,
                "Tenant update failed"
            );

            ApiError::new(status, message, code, request_id).into_response()
        },
    }
}

/// Delete tenant handler (admin operation)
#[axum::debug_handler]
pub async fn delete_tenant(
    State(state): State<TenantAppState>,
    Path(tenant_id): Path<String>,
    Extension(_claims): Extension<Claims>,
) -> Response {
    debug!("Processing delete tenant request");
    let start = std::time::Instant::now();

    // Generate a unique request ID
    let request_id = generate_request_id();

    // Parse tenant ID
    let tenant_id = match Uuid::parse_str(&tenant_id) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::new(
                StatusCode::BAD_REQUEST,
                "Invalid tenant ID format",
                "INVALID_TENANT_ID",
                request_id,
            )
            .into_response();
        },
    };

    // Delete tenant
    match state.tenant_service.delete_tenant(&tenant_id).await {
        Ok(_) => {
            // Record success
            monitoring::record_tenant_operation("delete", "success");

            // Record duration
            let duration = start.elapsed();
            monitoring::record_request_duration(duration.as_secs_f64(), "DELETE", "/tenants/:id");

            info!(
                request_id = %request_id,
                tenant_id = %tenant_id,
                "Tenant deleted successfully"
            );

            let api_response = ApiResponse::success(true, request_id);
            (StatusCode::OK, Json(api_response)).into_response()
        },
        Err(err) => {
            // Record failure
            monitoring::record_tenant_operation("delete", "failure");

            // Map error to appropriate response
            let (status, message, code) = match err {
                TenantServiceError::NotFound(_) => (
                    StatusCode::NOT_FOUND,
                    "Tenant not found",
                    "TENANT_NOT_FOUND",
                ),
                _ => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An error occurred deleting the tenant",
                    "TENANT_DELETION_ERROR",
                ),
            };

            warn!(
                request_id = %request_id,
                error = %err,
                tenant_id = %tenant_id,
                "Tenant deletion failed"
            );

            ApiError::new(status, message, code, request_id).into_response()
        },
    }
}

/// Utility to validate tenant operations
pub async fn is_tenant_admin(
    tenant_service: &TenantService,
    tenant_id: &Uuid,
    user_id: &Uuid,
) -> bool {
    match tenant_service
        .check_user_tenant_role(tenant_id, user_id, "ADMIN")
        .await
    {
        Ok(is_admin) => is_admin,
        Err(_) => false,
    }
}
