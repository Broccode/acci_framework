use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

/// Represents a geographic location where a session was accessed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLocation {
    pub id: Uuid,
    pub session_id: Uuid,
    pub ip_address: String,
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub created_at: SystemTime,
}

/// Enhanced fingerprint data for session security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSessionFingerprint {
    pub id: Uuid,
    pub session_id: Uuid,
    pub user_agent: String,
    pub browser: Option<String>,
    pub browser_version: Option<String>,
    pub os: Option<String>,
    pub os_version: Option<String>,
    pub device_type: Option<String>,
    pub device_vendor: Option<String>,
    pub device_model: Option<String>,
    pub created_at: SystemTime,
}

/// Risk assessment for session security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Represents a risk assessment for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRiskAssessment {
    pub id: Uuid,
    pub session_id: Uuid,
    pub risk_level: RiskLevel,
    pub risk_factors: Vec<String>,
    pub assessment_time: SystemTime,
    pub created_at: SystemTime,
}
