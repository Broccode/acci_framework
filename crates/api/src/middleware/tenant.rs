use acci_auth::models::tenant::{Tenant, TenantError, TenantRepository};
use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::monitoring;
use crate::response::ApiError;

/// Tenant context for the current request
#[derive(Debug, Clone)]
pub struct TenantContext {
    /// Unique identifier for the tenant
    pub id: Uuid,
    /// Name of the tenant
    pub name: String,
    /// Subdomain for the tenant
    pub subdomain: String,
    /// Database schema name for the tenant
    pub database_schema: String,
    /// Whether the tenant is active
    pub is_active: bool,
}

impl TenantContext {
    /// Creates a new tenant context from a tenant entity
    pub fn from_tenant(tenant: Tenant) -> Self {
        Self {
            id: tenant.id,
            name: tenant.name,
            subdomain: tenant.subdomain,
            database_schema: format!("tenant_{}", tenant.id),
            is_active: tenant.is_active,
        }
    }
}

/// Configuration for tenant resolution
#[derive(Debug, Clone)]
pub struct TenantResolutionConfig {
    /// The default domain for the application (e.g., "acci.io")
    pub default_domain: String,
    /// Whether to check for tenant in subdomain
    pub check_subdomain: bool,
    /// Whether to check for tenant in custom header
    pub check_header: bool,
    /// Name of the custom header to check
    pub header_name: String,
    /// Whether to check for tenant in JWT
    pub check_jwt: bool,
    /// Whether to check for tenant in path
    pub check_path: bool,
    /// Path prefix for tenant identification
    pub path_prefix: String,
}

impl Default for TenantResolutionConfig {
    fn default() -> Self {
        Self {
            default_domain: "localhost".to_string(),
            check_subdomain: true,
            check_header: true,
            header_name: "X-Tenant-ID".to_string(),
            check_jwt: true,
            check_path: false,
            path_prefix: "/api/tenants/".to_string(),
        }
    }
}

/// Claims struct for JWT with tenant information
#[derive(Debug, Serialize, Deserialize)]
pub struct TenantClaims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
    pub email: String,
    pub tenant_id: Option<Uuid>,
}

/// State for the tenant resolution middleware
#[derive(Clone)]
pub struct TenantState {
    /// Repository for tenant operations
    pub tenant_repository: Arc<dyn TenantRepository>,
    /// Configuration for tenant resolution
    pub config: TenantResolutionConfig,
}

/// Middleware for resolving tenant from the request
pub async fn tenant_resolution_middleware(
    State(state): State<TenantState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let _start = std::time::Instant::now();
    let request_id = request
        .extensions()
        .get::<String>()
        .map(|id| id.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    debug!(request_id = %request_id, "Resolving tenant for request");

    // Extract all necessary data from the request first
    let host = request
        .headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_owned());
    let tenant_header = request
        .headers()
        .get(&state.config.header_name)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_owned());
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_owned());
    let path = request.uri().path().to_owned();

    // Try to resolve tenant ID from various sources
    let tenant_id =
        match resolve_tenant_id_from_all_sources(&state, host, tenant_header, auth_header, path)
            .await
        {
            Ok(Some(id)) => Some(id),
            Ok(None) => None,
            Err(err) => {
                // Failed to resolve tenant
                error!(request_id = %request_id, error = %err, "Failed to resolve tenant");
                monitoring::record_auth_operation("tenant_resolution", "failure");

                let status_code = match err {
                    TenantError::NotFound => StatusCode::NOT_FOUND,
                    TenantError::InactiveTenant => StatusCode::FORBIDDEN,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };

                let error_code = match err {
                    TenantError::NotFound => "TENANT_NOT_FOUND",
                    TenantError::InactiveTenant => "TENANT_INACTIVE",
                    _ => "TENANT_RESOLUTION_ERROR",
                };

                let error_message = match err {
                    TenantError::NotFound => "Tenant not found",
                    TenantError::InactiveTenant => "Tenant account is inactive",
                    _ => "Internal server error",
                };

                let error = ApiError::new(status_code, error_message, error_code, request_id);
                return Ok(error.into_response());
            },
        };

    match tenant_id {
        Some(tenant_id) => {
            // Tenant ID found, load tenant information
            match state.tenant_repository.find_tenant_by_id(tenant_id).await {
                Ok(Some(tenant)) => {
                    if !tenant.is_active {
                        // Tenant exists but is inactive
                        error!(
                            request_id = %request_id,
                            tenant_id = %tenant_id,
                            "Tenant is inactive"
                        );
                        monitoring::record_auth_operation("tenant_resolution", "failure");

                        let error = ApiError::new(
                            StatusCode::FORBIDDEN,
                            "Tenant account is inactive",
                            "TENANT_INACTIVE",
                            request_id.clone(),
                        );
                        return Ok(error.into_response());
                    }

                    // Create tenant context and add it to request extensions
                    let tenant_context = TenantContext::from_tenant(tenant);
                    request.extensions_mut().insert(tenant_context);

                    // Record successful tenant resolution
                    info!(
                        request_id = %request_id,
                        tenant_id = %tenant_id,
                        "Tenant successfully resolved"
                    );
                    monitoring::record_auth_operation("tenant_resolution", "success");

                    // Continue with the request
                    Ok(next.run(request).await)
                },
                Ok(None) => {
                    // Tenant ID found but tenant doesn't exist (should not happen)
                    error!(
                        request_id = %request_id,
                        tenant_id = %tenant_id,
                        "Tenant not found"
                    );
                    monitoring::record_auth_operation("tenant_resolution", "failure");

                    let error = ApiError::new(
                        StatusCode::NOT_FOUND,
                        "Tenant not found",
                        "TENANT_NOT_FOUND",
                        request_id.clone(),
                    );
                    Ok(error.into_response())
                },
                Err(err) => {
                    // Error looking up tenant
                    error!(
                        request_id = %request_id,
                        tenant_id = %tenant_id,
                        error = %err,
                        "Error looking up tenant"
                    );
                    monitoring::record_auth_operation("tenant_resolution", "failure");

                    let error = ApiError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Internal server error",
                        "TENANT_LOOKUP_ERROR",
                        request_id.clone(),
                    );
                    Ok(error.into_response())
                },
            }
        },
        None => {
            // No tenant found but not required (public route)
            debug!(request_id = %request_id, "No tenant identified, continuing as public route");
            Ok(next.run(request).await)
        },
    }
}

/// Resolves tenant ID from all possible sources in the request
async fn resolve_tenant_id_from_all_sources(
    state: &TenantState,
    host: Option<String>,
    tenant_header: Option<String>,
    auth_header: Option<String>,
    path: String,
) -> Result<Option<Uuid>, TenantError> {
    // Try to resolve from subdomain
    if state.config.check_subdomain && host.is_some() {
        let host = host.unwrap();
        if let Some(tenant_id) = resolve_from_subdomain(state, &host).await? {
            return Ok(Some(tenant_id));
        }
    }

    // Try to resolve from header
    if state.config.check_header && tenant_header.is_some() {
        let tenant_header = tenant_header.unwrap();
        if let Some(tenant_id) = resolve_from_header(state, &tenant_header).await? {
            return Ok(Some(tenant_id));
        }
    }

    // Try to resolve from JWT
    if state.config.check_jwt && auth_header.is_some() {
        let auth_header = auth_header.unwrap();
        if let Some(tenant_id) = resolve_from_jwt(&auth_header)? {
            return Ok(Some(tenant_id));
        }
    }

    // Try to resolve from path
    if state.config.check_path {
        if let Some(tenant_id) = resolve_from_path(state, &path).await? {
            return Ok(Some(tenant_id));
        }
    }

    // No tenant found, but this is not an error
    // The route might not require a tenant context
    Ok(None)
}


/// Resolves tenant ID from subdomain
async fn resolve_from_subdomain(
    state: &TenantState,
    host: &str,
) -> Result<Option<Uuid>, TenantError> {
    // Extract subdomain from host
    // Format: subdomain.domain.com
    let domain_parts: Vec<&str> = host.split('.').collect();

    // Need at least 3 parts for a subdomain (subdomain.domain.tld)
    if domain_parts.len() >= 3 {
        let subdomain = domain_parts[0].to_string();

        // Don't treat 'www' as a tenant subdomain
        if subdomain == "www" {
            return Ok(None);
        }

        // Find tenant by subdomain
        match state
            .tenant_repository
            .find_tenant_by_subdomain(&subdomain)
            .await?
        {
            Some(tenant) => Ok(Some(tenant.id)),
            None => Ok(None),
        }
    } else {
        Ok(None)
    }
}

/// Resolves tenant ID from custom header
async fn resolve_from_header(
    state: &TenantState,
    tenant_id_str: &str,
) -> Result<Option<Uuid>, TenantError> {
    // Try to parse as UUID
    if let Ok(tenant_id) = Uuid::parse_str(tenant_id_str) {
        return Ok(Some(tenant_id));
    }

    // If not a UUID, try to find by subdomain
    match state
        .tenant_repository
        .find_tenant_by_subdomain(tenant_id_str)
        .await?
    {
        Some(tenant) => Ok(Some(tenant.id)),
        None => Ok(None),
    }
}

/// Resolves tenant ID from JWT claims
fn resolve_from_jwt(auth_header: &str) -> Result<Option<Uuid>, TenantError> {
    // Handle "Bearer" prefix
    let token = if auth_header.starts_with("Bearer ") {
        &auth_header[7..] // Skip "Bearer " prefix
    } else {
        auth_header
    };

    // Attempt to decode JWT without validation
    // We're just looking for the tenant_id claim, and the actual validation
    // will happen in the authentication middleware
    let mut validation = Validation::default();
    validation.insecure_disable_signature_validation();

    let token_data = match decode::<TenantClaims>(
        token,
        &DecodingKey::from_secret(&[]), // Dummy key
        &validation,
    ) {
        Ok(data) => data,
        Err(_) => return Ok(None),
    };

    // Extract tenant_id from claims if present
    Ok(token_data.claims.tenant_id)
}

/// Resolves tenant ID from URL path
async fn resolve_from_path(state: &TenantState, path: &str) -> Result<Option<Uuid>, TenantError> {
    // Check if path starts with the configured prefix
    if !path.starts_with(&state.config.path_prefix) {
        return Ok(None);
    }

    // Extract tenant ID or subdomain from path
    let after_prefix = &path[state.config.path_prefix.len()..];
    let tenant_id_str = after_prefix.split('/').next().unwrap_or_default();

    if tenant_id_str.is_empty() {
        return Ok(None);
    }

    // Try to parse as UUID
    if let Ok(tenant_id) = Uuid::parse_str(tenant_id_str) {
        return Ok(Some(tenant_id));
    }

    // If not a UUID, try to find by subdomain
    match state
        .tenant_repository
        .find_tenant_by_subdomain(tenant_id_str)
        .await?
    {
        Some(tenant) => Ok(Some(tenant.id)),
        None => Ok(None),
    }
}
