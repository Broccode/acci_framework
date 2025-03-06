# Audit Logging System Implementation

## Overview

This document describes the comprehensive audit logging system for the ACCI Framework, which is a key component of Milestone 2. The audit logging system provides a complete, tamper-evident record of all security-relevant events across the platform, supporting compliance requirements, security investigations, and operational oversight.

## Current Status (Pre-Implementation)

This is a planning document for Milestone 2. The implementation will follow the architecture and approaches described here.

## Audit Logging Architecture

### Core Components

The audit logging system consists of several interconnected components:

1. **Audit Event Generation**: Captures events from all system components
2. **Event Processing**: Validates, enriches, and standardizes events
3. **Storage Engine**: Securely persists events with tamper protection
4. **Query Engine**: Retrieves and filters events for reporting
5. **Compliance Export**: Formats events for external compliance systems

```rust
/// Core audit logging service
pub struct AuditLogService {
    repository: Arc<dyn AuditLogRepository>,
    processor: Arc<EventProcessor>,
    export_handlers: HashMap<ExportFormat, Box<dyn ExportHandler>>,
    config: AuditLogConfig,
}

/// Configuration for audit logging
pub struct AuditLogConfig {
    pub event_retention_days: u32,
    pub min_log_level: LogLevel,
    pub tenant_isolation: bool,
    pub include_sensitive_data: bool,
    pub signature_verification: bool,
    pub storage_encryption: bool,
    pub compress_old_events: bool,
}

/// Supported export formats
pub enum ExportFormat {
    Json,
    Csv,
    Xml,
    Syslog,
    CloudTrail,
    Cef, // Common Event Format
}
```

### Event Schema

Audit events follow a standardized schema for consistency:

```rust
/// Comprehensive audit event structure
pub struct AuditEvent {
    pub id: EventId,
    pub timestamp: DateTime<Utc>,
    pub tenant_id: Option<TenantId>,
    
    // Actor information (who)
    pub actor: Actor,
    
    // Action details (what)
    pub event_type: EventType,
    pub action: String,
    pub status: EventStatus,
    pub severity: EventSeverity,
    
    // Target information (on what)
    pub target: Target,
    
    // Context information
    pub context: EventContext,
    
    // Detailed information
    pub metadata: HashMap<String, Value>,
    
    // Integrity verification
    pub signature: Option<String>,
}

/// Actor who performed the action
pub struct Actor {
    pub type_: ActorType,
    pub id: String,
    pub name: Option<String>,
    pub ip_address: Option<String>,
    pub location: Option<GeoLocation>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
}

/// Type of actor
pub enum ActorType {
    User,
    System,
    Service,
    Anonymous,
    Administrator,
}

/// Target of the action
pub struct Target {
    pub type_: TargetType,
    pub id: Option<String>,
    pub name: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
}

/// Type of target
pub enum TargetType {
    User,
    System,
    Data,
    Resource,
    Tenant,
    Configuration,
}

/// Context of the event
pub struct EventContext {
    pub request_id: Option<String>,
    pub correlation_id: Option<String>,
    pub source: String,
    pub component: String,
    pub trace_id: Option<String>,
}

/// Status of the event
pub enum EventStatus {
    Success,
    Failure,
    Warning,
    Information,
}

/// Severity level
pub enum EventSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Type of event
pub enum EventType {
    Authentication,
    Authorization,
    DataAccess,
    DataModification,
    UserManagement,
    TenantManagement,
    SystemConfiguration,
    SystemOperation,
    SecurityAlert,
    ApiAccess,
}
```

### Implementation Details

#### Event Generation

Events are generated from various components using a simple logging API:

```rust
impl AuditLogService {
    /// Log an authentication event
    pub async fn log_authentication_event(
        &self,
        actor: Actor,
        action: &str,
        status: EventStatus,
        details: HashMap<String, Value>,
        context: &RequestContext,
    ) -> Result<EventId, LoggingError> {
        let event = AuditEvent {
            id: EventId::new(),
            timestamp: Utc::now(),
            tenant_id: context.tenant_id.clone(),
            actor,
            event_type: EventType::Authentication,
            action: action.to_string(),
            status,
            severity: match status {
                EventStatus::Failure => EventSeverity::High,
                _ => EventSeverity::Info,
            },
            target: Target {
                type_: TargetType::System,
                id: None,
                name: Some("Authentication Service".to_string()),
                resource_type: None,
                resource_id: None,
            },
            context: EventContext {
                request_id: context.request_id.clone(),
                correlation_id: context.correlation_id.clone(),
                source: "auth-service".to_string(),
                component: "authentication".to_string(),
                trace_id: context.trace_id.clone(),
            },
            metadata: details,
            signature: None, // Will be added during processing
        };
        
        self.log_event(event).await
    }
    
    /// Log any audit event
    pub async fn log_event(&self, mut event: AuditEvent) -> Result<EventId, LoggingError> {
        // Check if we should log this event based on config
        if !self.should_log_event(&event) {
            return Ok(event.id);
        }
        
        // Process the event (enrichment, validation, etc.)
        event = self.processor.process_event(event).await?;
        
        // Generate signature if configured
        if self.config.signature_verification {
            event.signature = Some(self.generate_event_signature(&event)?);
        }
        
        // Persist the event
        self.repository.save(&event).await?;
        
        // Return the event ID
        Ok(event.id)
    }
    
    /// Determine if the event should be logged
    fn should_log_event(&self, event: &AuditEvent) -> bool {
        // Check log level
        let event_level = match event.severity {
            EventSeverity::Critical => LogLevel::Critical,
            EventSeverity::High => LogLevel::High,
            EventSeverity::Medium => LogLevel::Medium,
            EventSeverity::Low => LogLevel::Low,
            EventSeverity::Info => LogLevel::Info,
        };
        
        if event_level < self.config.min_log_level {
            return false;
        }
        
        // Additional checks can be added here
        
        true
    }
    
    /// Generate a cryptographic signature for the event
    fn generate_event_signature(&self, event: &AuditEvent) -> Result<String, LoggingError> {
        // Serialize event to JSON without the signature field
        let mut event_clone = event.clone();
        event_clone.signature = None;
        
        let event_json = serde_json::to_string(&event_clone)
            .map_err(|e| LoggingError::SerializationError(e.to_string()))?;
        
        // Generate HMAC signature using the service's secret key
        let mut mac = Hmac::<Sha256>::new_from_slice(self.config.signature_key.as_bytes())
            .map_err(|_| LoggingError::CryptographicError("Invalid key length".to_string()))?;
            
        mac.update(event_json.as_bytes());
        
        let result = mac.finalize();
        let signature = base64::encode(result.into_bytes());
        
        Ok(signature)
    }
}
```

#### Event Processing

Events go through a processing pipeline for enrichment and validation:

```rust
pub struct EventProcessor {
    enrichers: Vec<Box<dyn EventEnricher>>,
    validators: Vec<Box<dyn EventValidator>>,
    sanitizers: Vec<Box<dyn EventSanitizer>>,
}

impl EventProcessor {
    /// Process an event through the pipeline
    pub async fn process_event(&self, mut event: AuditEvent) -> Result<AuditEvent, LoggingError> {
        // Run through validators
        for validator in &self.validators {
            validator.validate(&event)?;
        }
        
        // Run through enrichers
        for enricher in &self.enrichers {
            event = enricher.enrich(event).await?;
        }
        
        // Run through sanitizers
        for sanitizer in &self.sanitizers {
            event = sanitizer.sanitize(event)?;
        }
        
        Ok(event)
    }
}

/// Enriches events with additional information
#[async_trait]
pub trait EventEnricher: Send + Sync {
    async fn enrich(&self, event: AuditEvent) -> Result<AuditEvent, LoggingError>;
}

/// Validates events meet requirements
pub trait EventValidator: Send + Sync {
    fn validate(&self, event: &AuditEvent) -> Result<(), LoggingError>;
}

/// Sanitizes sensitive information from events
pub trait EventSanitizer: Send + Sync {
    fn sanitize(&self, event: AuditEvent) -> Result<AuditEvent, LoggingError>;
}

/// Geo-location enricher implementation
pub struct GeoLocationEnricher {
    geo_service: Arc<dyn GeoLocationService>,
}

#[async_trait]
impl EventEnricher for GeoLocationEnricher {
    async fn enrich(&self, mut event: AuditEvent) -> Result<AuditEvent, LoggingError> {
        // Skip if IP is not available or location already exists
        if event.actor.ip_address.is_none() || event.actor.location.is_some() {
            return Ok(event);
        }
        
        // Lookup IP address
        if let Some(ip) = &event.actor.ip_address {
            if let Ok(location) = self.geo_service.lookup_ip(ip).await {
                event.actor.location = Some(location);
            }
        }
        
        Ok(event)
    }
}

/// Sensitive data sanitizer implementation
pub struct SensitiveDataSanitizer {
    config: SanitizationConfig,
}

impl EventSanitizer for SensitiveDataSanitizer {
    fn sanitize(&self, mut event: AuditEvent) -> Result<AuditEvent, LoggingError> {
        // Sanitize sensitive fields from metadata
        for field in &self.config.sensitive_fields {
            if let Some(value) = event.metadata.get_mut(field) {
                *value = json!("*REDACTED*");
            }
        }
        
        // Handle nested fields
        for (key, value) in event.metadata.iter_mut() {
            if let Some(obj) = value.as_object_mut() {
                for field in &self.config.sensitive_fields {
                    if obj.contains_key(field) {
                        obj.insert(field.clone(), json!("*REDACTED*"));
                    }
                }
            }
        }
        
        Ok(event)
    }
}
```

#### Storage Implementation

Audit logs are stored in a secure, append-only repository:

```rust
/// Repository abstraction layer
#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    async fn save(&self, event: &AuditEvent) -> Result<(), RepositoryError>;
    async fn get_by_id(&self, id: &EventId) -> Result<Option<AuditEvent>, RepositoryError>;
    async fn search(&self, query: &EventQuery) -> Result<(Vec<AuditEvent>, u64), RepositoryError>;
    async fn export(&self, query: &EventQuery, format: ExportFormat) -> Result<Vec<u8>, RepositoryError>;
    async fn cleanup_old_events(&self, older_than: DateTime<Utc>) -> Result<u64, RepositoryError>;
}

/// PostgreSQL implementation of the repository
pub struct PostgresAuditLogRepository {
    pool: PgPool,
    config: RepositoryConfig,
}

#[async_trait]
impl AuditLogRepository for PostgresAuditLogRepository {
    async fn save(&self, event: &AuditEvent) -> Result<(), RepositoryError> {
        // Serialize metadata and context
        let metadata_json = serde_json::to_value(&event.metadata)
            .map_err(|e| RepositoryError::SerializationError(e.to_string()))?;
            
        let context_json = serde_json::to_value(&event.context)
            .map_err(|e| RepositoryError::SerializationError(e.to_string()))?;
            
        // Begin transaction
        let mut tx = self.pool.begin().await
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
        
        // Insert into main audit log table
        sqlx::query(
            "INSERT INTO audit_logs (
                id, timestamp, tenant_id, 
                actor_type, actor_id, actor_name, actor_ip, 
                event_type, action, status, severity,
                target_type, target_id, target_name, resource_type, resource_id,
                context, metadata, signature
            ) VALUES (
                $1, $2, $3, 
                $4, $5, $6, $7, 
                $8, $9, $10, $11,
                $12, $13, $14, $15, $16,
                $17, $18, $19
            )"
        )
        .bind(&event.id.to_string())
        .bind(&event.timestamp)
        .bind(event.tenant_id.as_ref().map(|t| t.to_string()))
        .bind(&event.actor.type_.to_string())
        .bind(&event.actor.id)
        .bind(&event.actor.name)
        .bind(&event.actor.ip_address)
        .bind(&event.event_type.to_string())
        .bind(&event.action)
        .bind(&event.status.to_string())
        .bind(&event.severity.to_string())
        .bind(&event.target.type_.to_string())
        .bind(&event.target.id)
        .bind(&event.target.name)
        .bind(&event.target.resource_type)
        .bind(&event.target.resource_id)
        .bind(&context_json)
        .bind(&metadata_json)
        .bind(&event.signature)
        .execute(&mut tx)
        .await
        .map_err(|e| RepositoryError::QueryError(e.to_string()))?;
        
        // If tenant isolation is enabled, also write to tenant-specific table
        if self.config.tenant_isolation && event.tenant_id.is_some() {
            let tenant_id = event.tenant_id.as_ref().unwrap();
            let table_name = format!("tenant_{}_audit_logs", tenant_id);
            
            // Create tenant-specific table if it doesn't exist
            let create_table_query = format!(
                "CREATE TABLE IF NOT EXISTS {} (
                    id UUID PRIMARY KEY,
                    event_data JSONB NOT NULL,
                    timestamp TIMESTAMPTZ NOT NULL,
                    event_type TEXT NOT NULL,
                    actor_id TEXT NOT NULL,
                    status TEXT NOT NULL
                )", 
                table_name
            );
            
            sqlx::query(&create_table_query)
                .execute(&mut tx)
                .await
                .map_err(|e| RepositoryError::QueryError(e.to_string()))?;
                
            // Serialize entire event
            let event_json = serde_json::to_value(event)
                .map_err(|e| RepositoryError::SerializationError(e.to_string()))?;
                
            // Insert into tenant-specific table
            let insert_query = format!(
                "INSERT INTO {} (id, event_data, timestamp, event_type, actor_id, status)
                 VALUES ($1, $2, $3, $4, $5, $6)",
                table_name
            );
            
            sqlx::query(&insert_query)
                .bind(&event.id.to_string())
                .bind(&event_json)
                .bind(&event.timestamp)
                .bind(&event.event_type.to_string())
                .bind(&event.actor.id)
                .bind(&event.status.to_string())
                .execute(&mut tx)
                .await
                .map_err(|e| RepositoryError::QueryError(e.to_string()))?;
        }
        
        // Commit transaction
        tx.commit()
            .await
            .map_err(|e| RepositoryError::TransactionError(e.to_string()))?;
            
        Ok(())
    }
    
    async fn search(&self, query: &EventQuery) -> Result<(Vec<AuditEvent>, u64), RepositoryError> {
        // Build the WHERE clause based on query parameters
        let mut conditions = Vec::new();
        let mut params = vec![];
        let mut param_index = 1;
        
        // Add filters based on query parameters
        if let Some(tenant_id) = &query.tenant_id {
            conditions.push(format!("tenant_id = ${}", param_index));
            params.push(tenant_id.to_string());
            param_index += 1;
        }
        
        if let Some(actor_id) = &query.actor_id {
            conditions.push(format!("actor_id = ${}", param_index));
            params.push(actor_id.clone());
            param_index += 1;
        }
        
        if let Some(start_time) = query.start_time {
            conditions.push(format!("timestamp >= ${}", param_index));
            params.push(start_time.to_rfc3339());
            param_index += 1;
        }
        
        if let Some(end_time) = query.end_time {
            conditions.push(format!("timestamp <= ${}", param_index));
            params.push(end_time.to_rfc3339());
            param_index += 1;
        }
        
        if !query.event_types.is_empty() {
            let placeholders: Vec<String> = (0..query.event_types.len())
                .map(|i| format!("${}", param_index + i))
                .collect();
                
            conditions.push(format!("event_type IN ({})", placeholders.join(", ")));
            
            for event_type in &query.event_types {
                params.push(event_type.to_string());
            }
            
            param_index += query.event_types.len();
        }
        
        // Build the WHERE clause
        let where_clause = if conditions.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };
        
        // Build the full query with pagination
        let query_sql = format!(
            "SELECT 
                id, timestamp, tenant_id, 
                actor_type, actor_id, actor_name, actor_ip, 
                event_type, action, status, severity,
                target_type, target_id, target_name, resource_type, resource_id,
                context, metadata, signature
             FROM audit_logs
             {}
             ORDER BY timestamp DESC
             LIMIT {} OFFSET {}",
            where_clause, query.limit, query.offset
        );
        
        // Count query for pagination
        let count_sql = format!(
            "SELECT COUNT(*) FROM audit_logs {}",
            where_clause
        );
        
        // Execute count query
        let count: i64 = sqlx::query_scalar(&count_sql)
            .bind_all(sqlx::postgres::PgArguments::from_iter(params.clone()))
            .fetch_one(&self.pool)
            .await
            .map_err(|e| RepositoryError::QueryError(e.to_string()))?;
            
        // Execute main query
        let rows = sqlx::query(&query_sql)
            .bind_all(sqlx::postgres::PgArguments::from_iter(params))
            .fetch_all(&self.pool)
            .await
            .map_err(|e| RepositoryError::QueryError(e.to_string()))?;
            
        // Convert rows to events
        let mut events = Vec::with_capacity(rows.len());
        
        for row in rows {
            // Extract fields from row
            let id = EventId::from_str(row.get::<&str, _>("id"))
                .map_err(|_| RepositoryError::DataError("Invalid event ID".to_string()))?;
                
            let timestamp: DateTime<Utc> = row.get("timestamp");
            
            let tenant_id_str: Option<String> = row.get("tenant_id");
            let tenant_id = tenant_id_str.map(|s| TenantId::from_str(&s))
                .transpose()
                .map_err(|_| RepositoryError::DataError("Invalid tenant ID".to_string()))?;
                
            let actor_type_str: String = row.get("actor_type");
            let actor_type = ActorType::from_str(&actor_type_str)
                .map_err(|_| RepositoryError::DataError("Invalid actor type".to_string()))?;
                
            let actor = Actor {
                type_: actor_type,
                id: row.get("actor_id"),
                name: row.get("actor_name"),
                ip_address: row.get("actor_ip"),
                location: None, // Will be reconstructed from metadata
                user_agent: None, // Will be reconstructed from metadata
                session_id: None, // Will be reconstructed from metadata
            };
            
            let event_type_str: String = row.get("event_type");
            let event_type = EventType::from_str(&event_type_str)
                .map_err(|_| RepositoryError::DataError("Invalid event type".to_string()))?;
                
            let status_str: String = row.get("status");
            let status = EventStatus::from_str(&status_str)
                .map_err(|_| RepositoryError::DataError("Invalid status".to_string()))?;
                
            let severity_str: String = row.get("severity");
            let severity = EventSeverity::from_str(&severity_str)
                .map_err(|_| RepositoryError::DataError("Invalid severity".to_string()))?;
                
            let target_type_str: String = row.get("target_type");
            let target_type = TargetType::from_str(&target_type_str)
                .map_err(|_| RepositoryError::DataError("Invalid target type".to_string()))?;
                
            let target = Target {
                type_: target_type,
                id: row.get("target_id"),
                name: row.get("target_name"),
                resource_type: row.get("resource_type"),
                resource_id: row.get("resource_id"),
            };
            
            let context_json: serde_json::Value = row.get("context");
            let context: EventContext = serde_json::from_value(context_json)
                .map_err(|e| RepositoryError::DeserializationError(e.to_string()))?;
                
            let metadata_json: serde_json::Value = row.get("metadata");
            let metadata: HashMap<String, Value> = serde_json::from_value(metadata_json)
                .map_err(|e| RepositoryError::DeserializationError(e.to_string()))?;
                
            let signature: Option<String> = row.get("signature");
            
            let event = AuditEvent {
                id,
                timestamp,
                tenant_id,
                actor,
                event_type,
                action: row.get("action"),
                status,
                severity,
                target,
                context,
                metadata,
                signature,
            };
            
            events.push(event);
        }
        
        Ok((events, count as u64))
    }
    
    // Other method implementations...
}
```

#### Query Interface

A powerful query interface enables flexible searching:

```rust
/// Query parameters for audit log search
pub struct EventQuery {
    pub tenant_id: Option<TenantId>,
    pub actor_id: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub event_types: Vec<EventType>,
    pub actions: Vec<String>,
    pub statuses: Vec<EventStatus>,
    pub severities: Vec<EventSeverity>,
    pub target_id: Option<String>,
    pub target_type: Option<TargetType>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub full_text_search: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

impl AuditLogService {
    /// Search audit logs with a flexible query interface
    pub async fn search(
        &self,
        tenant_id: &TenantId,
        query: EventQuery,
    ) -> Result<SearchResult, LoggingError> {
        // Verify tenant authorization - in multi-tenant systems, users from one tenant
        // should not be able to access logs from another tenant
        if let Some(query_tenant_id) = &query.tenant_id {
            if query_tenant_id != tenant_id {
                return Err(LoggingError::AccessDenied("Cannot access logs from another tenant".to_string()));
            }
        }
        
        // Apply tenant ID filter if not specified in query
        let mut tenant_query = query.clone();
        if tenant_query.tenant_id.is_none() {
            tenant_query.tenant_id = Some(tenant_id.clone());
        }
        
        // Search repository
        let (events, total) = self.repository.search(&tenant_query).await?;
        
        // Apply additional filtering if needed
        let filtered_events = if self.config.signature_verification {
            // Verify signatures for all events
            events.into_iter()
                .filter(|event| self.verify_event_signature(event).unwrap_or(false))
                .collect()
        } else {
            events
        };
        
        // Calculate pagination info
        let total_pages = (total as f64 / tenant_query.limit as f64).ceil() as u64;
        let current_page = (tenant_query.offset / tenant_query.limit) as u64 + 1;
        
        Ok(SearchResult {
            events: filtered_events,
            total,
            total_pages,
            current_page,
            limit: tenant_query.limit as u64,
            offset: tenant_query.offset as u64,
        })
    }
    
    /// Verify the cryptographic signature on an event
    fn verify_event_signature(&self, event: &AuditEvent) -> Result<bool, LoggingError> {
        // Skip verification if no signature
        let signature = match &event.signature {
            Some(s) => s,
            None => return Ok(false),
        };
        
        // Clone and remove signature for verification
        let mut event_clone = event.clone();
        event_clone.signature = None;
        
        // Serialize for verification
        let event_json = serde_json::to_string(&event_clone)
            .map_err(|e| LoggingError::SerializationError(e.to_string()))?;
            
        // Create HMAC
        let mut mac = Hmac::<Sha256>::new_from_slice(self.config.signature_key.as_bytes())
            .map_err(|_| LoggingError::CryptographicError("Invalid key length".to_string()))?;
            
        mac.update(event_json.as_bytes());
        
        // Decode expected signature
        let expected_signature = base64::decode(signature)
            .map_err(|e| LoggingError::CryptographicError(format!("Invalid signature encoding: {}", e)))?;
            
        // Verify
        mac.verify_slice(&expected_signature)
            .map(|_| true)
            .map_err(|_| LoggingError::CryptographicError("Signature verification failed".to_string()))
    }
}
```

#### Compliance Reporting

The system supports compliance reporting with various output formats:

```rust
/// Export handler for compliance reporting
pub trait ExportHandler: Send + Sync {
    fn export(&self, events: &[AuditEvent], options: &ExportOptions) -> Result<Vec<u8>, LoggingError>;
}

/// Export options
pub struct ExportOptions {
    pub include_metadata: bool,
    pub redact_sensitive_fields: bool,
    pub format_options: HashMap<String, String>,
}

/// CSV export handler
pub struct CsvExportHandler;

impl ExportHandler for CsvExportHandler {
    fn export(&self, events: &[AuditEvent], options: &ExportOptions) -> Result<Vec<u8>, LoggingError> {
        let mut csv_writer = csv::Writer::from_writer(Vec::new());
        
        // Write header
        csv_writer.write_record(&[
            "Timestamp", "Tenant", "Actor Type", "Actor ID", "Event Type", 
            "Action", "Status", "Severity", "Target Type", "Target ID", 
            "Resource Type", "Resource ID", "Request ID",
        ])?;
        
        // Write rows
        for event in events {
            let mut record = Vec::new();
            
            // Add standard fields
            record.push(event.timestamp.to_rfc3339());
            record.push(event.tenant_id.as_ref().map_or("", |t| t.as_ref()));
            record.push(event.actor.type_.to_string());
            record.push(&event.actor.id);
            record.push(event.event_type.to_string());
            record.push(&event.action);
            record.push(event.status.to_string());
            record.push(event.severity.to_string());
            record.push(event.target.type_.to_string());
            record.push(event.target.id.as_deref().unwrap_or(""));
            record.push(event.target.resource_type.as_deref().unwrap_or(""));
            record.push(event.target.resource_id.as_deref().unwrap_or(""));
            record.push(event.context.request_id.as_deref().unwrap_or(""));
            
            // Add metadata if requested
            if options.include_metadata {
                let metadata_json = if options.redact_sensitive_fields {
                    // Redact sensitive fields
                    let mut sanitized = event.metadata.clone();
                    for key in &["password", "token", "secret", "key", "credential"] {
                        sanitized.retain(|k, _| !k.to_lowercase().contains(key));
                    }
                    serde_json::to_string(&sanitized)?
                } else {
                    serde_json::to_string(&event.metadata)?
                };
                
                record.push(metadata_json);
            }
            
            csv_writer.write_record(record)?;
        }
        
        // Get the CSV data
        let csv_data = csv_writer.into_inner()?;
        
        Ok(csv_data)
    }
}

impl AuditLogService {
    /// Export audit logs in various formats
    pub async fn export(
        &self,
        tenant_id: &TenantId,
        query: EventQuery,
        format: ExportFormat,
        options: ExportOptions,
    ) -> Result<Vec<u8>, LoggingError> {
        // Perform search
        let search_result = self.search(tenant_id, query).await?;
        
        // Get export handler
        let handler = self.export_handlers.get(&format)
            .ok_or_else(|| LoggingError::UnsupportedFormat(format!("{:?}", format)))?;
            
        // Export data
        handler.export(&search_result.events, &options)
    }
}
```

## Advanced Features

### Immutable Audit Trail

To ensure audit logs cannot be tampered with:

```rust
pub struct ImmutableAuditTrail {
    repository: Arc<dyn AuditLogRepository>,
    blockchain_service: Option<Arc<dyn BlockchainService>>,
    config: ImmutableAuditConfig,
}

impl ImmutableAuditTrail {
    /// Anchors a batch of audit logs to a tamper-proof storage
    pub async fn anchor_logs(&self, batch_id: &str) -> Result<AnchorProof, LoggingError> {
        // Get the batch
        let events = self.repository
            .get_batch(batch_id)
            .await?;
            
        if events.is_empty() {
            return Err(LoggingError::InvalidBatch("Empty batch".to_string()));
        }
        
        // Create Merkle tree from events
        let merkle_tree = self.create_merkle_tree(&events)?;
        
        // Get the Merkle root
        let merkle_root = merkle_tree.root();
        
        // Generate proof timestamp
        let timestamp = Utc::now();
        
        // Anchor to blockchain if configured
        let blockchain_tx_id = if let Some(blockchain) = &self.blockchain_service {
            Some(blockchain.anchor_data(merkle_root, timestamp).await?)
        } else {
            None
        };
        
        // Create cryptographic proof
        let proof = self.create_cryptographic_proof(&events, &merkle_tree)?;
        
        // Create anchor record
        let anchor = AnchorProof {
            id: Uuid::new_v4().to_string(),
            batch_id: batch_id.to_string(),
            merkle_root: merkle_root.to_string(),
            timestamp,
            blockchain_tx_id,
            proof,
            event_count: events.len() as u64,
        };
        
        // Store anchor proof
        self.repository
            .save_anchor_proof(&anchor)
            .await?;
            
        Ok(anchor)
    }
    
    /// Verifies the integrity of a batch of logs
    pub async fn verify_logs(&self, batch_id: &str) -> Result<VerificationResult, LoggingError> {
        // Get the events
        let events = self.repository
            .get_batch(batch_id)
            .await?;
            
        // Get the anchor proof
        let anchor = self.repository
            .get_anchor_proof(batch_id)
            .await?
            .ok_or_else(|| LoggingError::InvalidBatch("No anchor proof found".to_string()))?;
            
        // Recreate Merkle tree
        let merkle_tree = self.create_merkle_tree(&events)?;
        
        // Verify Merkle root
        let calculated_root = merkle_tree.root();
        let stored_root = anchor.merkle_root.parse()
            .map_err(|_| LoggingError::VerificationError("Invalid Merkle root".to_string()))?;
            
        let root_valid = calculated_root == stored_root;
        
        // Verify individual events
        let mut invalid_events = Vec::new();
        
        for event in &events {
            if !self.verify_event_signature(event)? {
                invalid_events.push(event.id.to_string());
            }
        }
        
        // Verify blockchain anchor if available
        let blockchain_valid = if let (Some(blockchain), Some(tx_id)) = (
            &self.blockchain_service, 
            &anchor.blockchain_tx_id
        ) {
            blockchain.verify_anchor(tx_id, &stored_root, anchor.timestamp).await?
        } else {
            true // If no blockchain anchor, consider it valid
        };
        
        Ok(VerificationResult {
            batch_id: batch_id.to_string(),
            timestamp: anchor.timestamp,
            events_count: events.len() as u64,
            merkle_root_valid: root_valid,
            blockchain_valid,
            invalid_events,
            is_valid: root_valid && blockchain_valid && invalid_events.is_empty(),
        })
    }
    
    /// Creates a Merkle tree from event hashes
    fn create_merkle_tree(&self, events: &[AuditEvent]) -> Result<MerkleTree<Sha256>, LoggingError> {
        // Create hashes for each event
        let leaves: Vec<[u8; 32]> = events.iter()
            .map(|event| {
                let serialized = serde_json::to_string(event)
                    .map_err(|e| LoggingError::SerializationError(e.to_string()))?;
                    
                let mut hasher = Sha256::new();
                hasher.update(serialized.as_bytes());
                
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&hasher.finalize());
                
                Ok(hash)
            })
            .collect::<Result<Vec<_>, LoggingError>>()?;
            
        // Create Merkle tree
        Ok(MerkleTree::new(leaves))
    }
}
```

### Compliance Alerting

The system can alert on compliance violations:

```rust
pub struct ComplianceMonitor {
    repository: Arc<dyn AuditLogRepository>,
    alert_service: Arc<dyn AlertService>,
    rules: Vec<ComplianceRule>,
}

pub struct ComplianceRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: AlertSeverity,
    pub conditions: Vec<RuleCondition>,
    pub threshold: u32,
    pub time_window: Duration,
    pub actions: Vec<AlertAction>,
}

pub enum RuleCondition {
    EventType(EventType),
    EventStatus(EventStatus),
    EventSeverity(EventSeverity),
    Action(String),
    ActorType(ActorType),
    TargetType(TargetType),
    Custom(Box<dyn Fn(&AuditEvent) -> bool + Send + Sync>),
}

impl ComplianceMonitor {
    /// Monitor audit logs for compliance violations
    pub async fn monitor(&self, tenant_id: &TenantId) -> Result<Vec<Alert>, LoggingError> {
        let mut alerts = Vec::new();
        
        // Process each rule
        for rule in &self.rules {
            // Create query for this rule's time window
            let end_time = Utc::now();
            let start_time = end_time - rule.time_window;
            
            let query = EventQuery {
                tenant_id: Some(tenant_id.clone()),
                start_time: Some(start_time),
                end_time: Some(end_time),
                // Other fields left default/empty
                limit: 1000,
                offset: 0,
                // ...
            };
            
            // Get events that match the time window
            let (events, _) = self.repository.search(&query).await?;
            
            // Filter events that match all conditions
            let matching_events: Vec<&AuditEvent> = events.iter()
                .filter(|event| self.matches_conditions(event, &rule.conditions))
                .collect();
                
            // Check if threshold is exceeded
            if matching_events.len() as u32 >= rule.threshold {
                // Create alert
                let alert = Alert {
                    id: Uuid::new_v4().to_string(),
                    tenant_id: tenant_id.clone(),
                    rule_id: rule.id.clone(),
                    rule_name: rule.name.clone(),
                    severity: rule.severity.clone(),
                    message: format!(
                        "{} detected: {} occurrences (threshold: {})",
                        rule.name, matching_events.len(), rule.threshold
                    ),
                    detected_at: Utc::now(),
                    matching_events: matching_events.iter().map(|e| e.id).collect(),
                    status: AlertStatus::New,
                };
                
                // Execute alert actions
                for action in &rule.actions {
                    self.alert_service.execute_action(action, &alert).await?;
                }
                
                alerts.push(alert);
            }
        }
        
        Ok(alerts)
    }
    
    /// Check if an event matches all conditions in a rule
    fn matches_conditions(&self, event: &AuditEvent, conditions: &[RuleCondition]) -> bool {
        conditions.iter().all(|condition| {
            match condition {
                RuleCondition::EventType(event_type) => &event.event_type == event_type,
                RuleCondition::EventStatus(status) => &event.status == status,
                RuleCondition::EventSeverity(severity) => &event.severity == severity,
                RuleCondition::Action(action) => event.action == *action,
                RuleCondition::ActorType(actor_type) => &event.actor.type_ == actor_type,
                RuleCondition::TargetType(target_type) => &event.target.type_ == target_type,
                RuleCondition::Custom(func) => func(event),
            }
        })
    }
}
```

### User Activity Timeline

Generate user activity timelines for investigations:

```rust
pub struct UserActivityTimeline {
    repository: Arc<dyn AuditLogRepository>,
}

pub struct TimelineEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub action: String,
    pub status: EventStatus,
    pub details: HashMap<String, Value>,
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub location: Option<GeoLocation>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl UserActivityTimeline {
    /// Generate a timeline for a specific user
    pub async fn generate_timeline(
        &self,
        tenant_id: &TenantId,
        user_id: &str,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        filter_types: Option<Vec<EventType>>,
    ) -> Result<Vec<TimelineEvent>, LoggingError> {
        // Create query
        let query = EventQuery {
            tenant_id: Some(tenant_id.clone()),
            actor_id: Some(user_id.to_string()),
            start_time,
            end_time,
            event_types: filter_types.unwrap_or_default(),
            // Other fields left as default
            limit: 1000, // Get up to 1000 events
            offset: 0,
            // ...
        };
        
        // Search events
        let (events, _) = self.repository.search(&query).await?;
        
        // Convert to timeline events
        let timeline = events.into_iter()
            .map(|event| {
                // Extract location from actor
                let location = event.actor.location;
                
                TimelineEvent {
                    timestamp: event.timestamp,
                    event_type: event.event_type,
                    action: event.action,
                    status: event.status,
                    details: event.metadata,
                    resource_type: event.target.resource_type,
                    resource_id: event.target.resource_id,
                    location,
                    ip_address: event.actor.ip_address,
                    user_agent: event.actor.user_agent,
                }
            })
            .collect();
            
        Ok(timeline)
    }
    
    /// Generate a session timeline
    pub async fn generate_session_timeline(
        &self,
        tenant_id: &TenantId,
        session_id: &str,
    ) -> Result<Vec<TimelineEvent>, LoggingError> {
        // Search for all events in this session
        let query = EventQuery {
            tenant_id: Some(tenant_id.clone()),
            // Use custom metadata field for session ID
            // This requires the audit log to consistently capture session ID in metadata
            full_text_search: Some(format!("\"session_id\":\"{session_id}\"")),
            // Other fields default
            limit: 1000,
            offset: 0,
            // ...
        };
        
        // Get events
        let (events, _) = self.repository.search(&query).await?;
        
        // Convert to timeline events (similar to previous method)
        let timeline = events.into_iter()
            .map(|event| {
                // Create timeline event (similar to above)
                TimelineEvent {
                    // ...fields from event...
                    timestamp: event.timestamp,
                    event_type: event.event_type,
                    action: event.action,
                    status: event.status,
                    details: event.metadata,
                    resource_type: event.target.resource_type,
                    resource_id: event.target.resource_id,
                    location: event.actor.location,
                    ip_address: event.actor.ip_address,
                    user_agent: event.actor.user_agent,
                }
            })
            .collect();
            
        Ok(timeline)
    }
    
    /// Generate a resource access timeline
    pub async fn generate_resource_timeline(
        &self,
        tenant_id: &TenantId,
        resource_type: &str,
        resource_id: &str,
    ) -> Result<Vec<TimelineEvent>, LoggingError> {
        // Search for all events affecting this resource
        let query = EventQuery {
            tenant_id: Some(tenant_id.clone()),
            resource_type: Some(resource_type.to_string()),
            resource_id: Some(resource_id.to_string()),
            // Other fields default
            limit: 1000,
            offset: 0,
            // ...
        };
        
        // Get events
        let (events, _) = self.repository.search(&query).await?;
        
        // Convert to timeline events (similar to previous methods)
        let timeline = events.into_iter()
            .map(|event| TimelineEvent {
                // fields as above
                timestamp: event.timestamp,
                event_type: event.event_type,
                action: event.action,
                status: event.status,
                details: event.metadata,
                resource_type: event.target.resource_type,
                resource_id: event.target.resource_id,
                location: event.actor.location,
                ip_address: event.actor.ip_address,
                user_agent: event.actor.user_agent,
            })
            .collect();
            
        Ok(timeline)
    }
}
```

## Integration with Other Systems

### API Integration

The audit logging system is integrated with the API layer:

```rust
pub struct AuditMiddleware<S> {
    audit_service: Arc<AuditLogService>,
    inner: S,
}

impl<S> Service<Request<Body>> for AuditMiddleware<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        // Clone things we need to move into the future
        let audit_service = self.audit_service.clone();
        let inner = self.inner.clone();
        
        // Extract data needed for audit log
        let method = req.method().clone();
        let uri = req.uri().clone();
        let path = uri.path().to_string();
        let tenant_id = extract_tenant_id(&req);
        let user_id = extract_user_id(&req);
        let request_id = extract_request_id(&req);
        let ip = extract_client_ip(&req);
        let user_agent = extract_user_agent(&req);
        
        // Create timing for request duration
        let start_time = Instant::now();
        
        // Determine event type based on path and method
        let event_type = determine_event_type(&path, &method);
        
        // Process request and log audit event
        Box::pin(async move {
            // Call the inner service and catch response
            let mut response = inner.call(req).await?;
            
            // Calculate duration
            let duration = start_time.elapsed();
            
            // Determine status based on response
            let status = if response.status().is_success() {
                EventStatus::Success
            } else if response.status().is_client_error() {
                EventStatus::Failure
            } else {
                EventStatus::Warning
            };
            
            // Create actor
            let actor = Actor {
                type_: ActorType::User,
                id: user_id.unwrap_or_else(|| "anonymous".to_string()),
                name: None,
                ip_address: ip,
                location: None, // Will be enriched by the service
                user_agent,
                session_id: None, // Could extract from request if available
            };
            
            // Create event context
            let context = RequestContext {
                tenant_id: tenant_id.clone(),
                request_id,
                correlation_id: extract_correlation_id(&response),
                trace_id: extract_trace_id(&response),
                user_agent: user_agent.clone(),
                ip_address: ip.clone(),
            };
            
            // Create metadata
            let mut metadata = HashMap::new();
            metadata.insert("method".to_string(), json!(method.as_str()));
            metadata.insert("path".to_string(), json!(path));
            metadata.insert("status_code".to_string(), json!(response.status().as_u16()));
            metadata.insert("duration_ms".to_string(), json!(duration.as_millis()));
            
            // Extract resource ID and type from path if possible
            let (resource_type, resource_id) = extract_resource_info(&path);
            
            // Log audit event
            let audit_result = audit_service.log_api_event(
                actor,
                format!("{} {}", method, path),
                status,
                metadata,
                resource_type,
                resource_id,
                &context,
            ).await;
            
            // If audit logging fails, log error but don't fail the request
            if let Err(e) = audit_result {
                eprintln!("Failed to log audit event: {}", e);
            }
            
            Ok(response)
        })
    }
}
```

### Database Integration

Audit critical database operations:

```rust
pub struct AuditedRepository<R> {
    inner: R,
    audit_service: Arc<AuditLogService>,
}

impl<R: UserRepository> UserRepository for AuditedRepository<R> {
    async fn create_user(&self, tenant_id: &TenantId, user: &CreateUser) -> Result<User, RepositoryError> {
        // Call the inner repository
        let result = self.inner.create_user(tenant_id, user).await;
        
        // Log the operation regardless of success/failure
        let status = if result.is_ok() {
            EventStatus::Success
        } else {
            EventStatus::Failure
        };
        
        // Create actor (system in this case, could be actual user if available)
        let actor = Actor {
            type_: ActorType::System,
            id: "system".to_string(),
            name: Some("Repository".to_string()),
            ip_address: None,
            location: None,
            user_agent: None,
            session_id: None,
        };
        
        // Create metadata
        let mut metadata = HashMap::new();
        metadata.insert("email".to_string(), json!(user.email));
        // Note: Don't log the password, even if hashed
        
        // Create context
        let context = RequestContext {
            tenant_id: tenant_id.clone(),
            request_id: None,
            correlation_id: None,
            trace_id: None,
            user_agent: None,
            ip_address: None,
        };
        
        // Create resource info if successful
        let (resource_id, resource_type) = if let Ok(ref user) = result {
            (Some(user.id.to_string()), Some("user".to_string()))
        } else {
            (None, Some("user".to_string()))
        };
        
        // Log audit event
        let _ = self.audit_service.log_data_event(
            actor,
            "CREATE_USER".to_string(),
            status,
            metadata,
            resource_type,
            resource_id,
            &context,
        ).await;
        
        // Return original result
        result
    }
    
    async fn update_user(&self, tenant_id: &TenantId, user_id: &UserId, update: &UpdateUser) -> Result<User, RepositoryError> {
        // Call inner repository
        let result = self.inner.update_user(tenant_id, user_id, update).await;
        
        // Log the operation (similar to create_user)
        // ...
        
        result
    }
    
    async fn delete_user(&self, tenant_id: &TenantId, user_id: &UserId) -> Result<(), RepositoryError> {
        // Call inner repository
        let result = self.inner.delete_user(tenant_id, user_id).await;
        
        // Log the operation (similar to create_user)
        // ...
        
        result
    }
    
    // Similarly implement other repository methods with audit logging
}
```

## Tenant-Specific Configuration

Support different audit requirements per tenant:

```rust
pub struct TenantAuditConfig {
    pub tenant_id: TenantId,
    pub retention_days: u32,
    pub min_log_level: LogLevel,
    pub compliance_frameworks: Vec<ComplianceFramework>,
    pub export_formats: Vec<ExportFormat>,
    pub alert_recipients: Vec<AlertRecipient>,
    pub sensitive_fields: Vec<String>,
    pub custom_event_types: HashMap<String, EventType>,
}

impl AuditLogService {
    /// Get tenant-specific configuration
    async fn get_tenant_config(&self, tenant_id: &TenantId) -> TenantAuditConfig {
        // Try to get tenant specific configuration
        if let Some(config) = self.tenant_configs.get(tenant_id) {
            return config.clone();
        }
        
        // Fall back to default
        TenantAuditConfig {
            tenant_id: tenant_id.clone(),
            retention_days: self.config.event_retention_days,
            min_log_level: self.config.min_log_level.clone(),
            compliance_frameworks: vec![],
            export_formats: vec![ExportFormat::Json, ExportFormat::Csv],
            alert_recipients: vec![],
            sensitive_fields: vec![
                "password".to_string(),
                "token".to_string(),
                "secret".to_string(),
                "api_key".to_string(),
            ],
            custom_event_types: HashMap::new(),
        }
    }
    
    /// Apply tenant-specific configuration to an event
    async fn apply_tenant_config(&self, event: &mut AuditEvent) -> Result<(), LoggingError> {
        if let Some(tenant_id) = &event.tenant_id {
            let config = self.get_tenant_config(tenant_id).await;
            
            // Apply tenant-specific event type mapping if exists
            if let Some(custom_type) = event.metadata.get("custom_event_type") {
                if let Some(json_str) = custom_type.as_str() {
                    if let Some(mapped_type) = config.custom_event_types.get(json_str) {
                        event.event_type = mapped_type.clone();
                    }
                }
            }
            
            // Redact sensitive fields if configured
            for field in &config.sensitive_fields {
                if let Some(value) = event.metadata.get_mut(field) {
                    *value = json!("*REDACTED*");
                }
            }
        }
        
        Ok(())
    }
}
```

## Performance Considerations

Audit logging is optimized for performance:

```rust
impl AuditLogService {
    /// Asynchronously log an event without waiting for persistence
    pub fn log_event_async(&self, event: AuditEvent) {
        // Clone what we need to move into the spawned task
        let repository = self.repository.clone();
        let processor = self.processor.clone();
        let config = self.config.clone();
        
        // Spawn a task to handle logging asynchronously
        tokio::spawn(async move {
            // Process the event
            let processed_event = match processor.process_event(event).await {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("Failed to process audit event: {}", err);
                    return;
                }
            };
            
            // Generate signature if configured
            let final_event = if config.signature_verification {
                // Generate signature (simplified for example)
                let mut event_with_sig = processed_event;
                // ... generate and add signature ...
                event_with_sig
            } else {
                processed_event
            };
            
            // Persist the event
            if let Err(err) = repository.save(&final_event).await {
                eprintln!("Failed to save audit event: {}", err);
            }
        });
    }
    
    /// Batch-log multiple events efficiently
    pub async fn log_events_batch(&self, events: Vec<AuditEvent>) -> Result<Vec<EventId>, LoggingError> {
        // Process events in parallel
        let processed_events = futures::future::join_all(
            events.into_iter().map(|event| {
                let processor = self.processor.clone();
                async move {
                    processor.process_event(event).await
                }
            })
        ).await;
        
        // Collect results, handling errors for individual events
        let mut valid_events = Vec::new();
        let mut event_ids = Vec::new();
        
        for result in processed_events {
            match result {
                Ok(event) => {
                    event_ids.push(event.id.clone());
                    valid_events.push(event);
                },
                Err(err) => {
                    eprintln!("Failed to process event: {}", err);
                }
            }
        }
        
        // Batch save events
        if !valid_events.is_empty() {
            self.repository.save_batch(&valid_events).await?;
        }
        
        Ok(event_ids)
    }
    
    /// Configure batching behavior
    pub fn configure_batching(&mut self, batch_size: usize, flush_interval_ms: u64) {
        // Set up background task to periodically flush events
        let repository = self.repository.clone();
        let event_queue = self.event_queue.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(flush_interval_ms));
            
            loop {
                interval.tick().await;
                
                // Get events from queue (up to batch_size)
                let mut events = Vec::with_capacity(batch_size);
                
                while events.len() < batch_size {
                    match event_queue.try_recv() {
                        Ok(event) => events.push(event),
                        Err(_) => break, // No more events in queue
                    }
                }
                
                // If we have events, save them
                if !events.is_empty() {
                    if let Err(err) = repository.save_batch(&events).await {
                        eprintln!("Failed to save event batch: {}", err);
                    }
                }
            }
        });
    }
}
```

## Conclusion

The Audit Logging System provides a comprehensive, secure, and high-performance solution for tracking and reviewing all security-relevant activities within the ACCI Framework. Key benefits include:

1. **Complete Coverage:** Captures all security-relevant events across the platform
2. **Tamper-Evident:** Cryptographic proof of audit log integrity
3. **High Performance:** Optimized for minimal impact on system performance
4. **Multi-Tenant:** Configurable per-tenant audit policies
5. **Compliance Ready:** Supports various compliance requirements with flexible reporting
6. **Searchable:** Powerful query capabilities for investigations
7. **Integrable:** Seamlessly integrates with other system components

By implementing this comprehensive audit logging system, we establish a foundation for security analysis, compliance reporting, and historical accountability that meets the most stringent enterprise requirements.

## Next Steps

1. Implement the core audit log repository and service
2. Create the event processing pipeline
3. Develop the cryptographic verification system
4. Integrate with API and database layers
5. Build compliance reporting capabilities