use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{
    Decode, Encode, Postgres, Type,
    encode::IsNull,
    error::BoxDynError,
    postgres::{PgArgumentBuffer, PgTypeInfo},
};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionInvalidationReason {
    UserLogout,
    AdminAction,
    PasswordChanged,
    SecurityBreach,
    InactivityTimeout,
    TokenExpired,
    DeviceChanged,
    ManualInvalidation,
}

// Add SQLx Type implementation for PostgreSQL
impl Type<Postgres> for SessionInvalidationReason {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("session_invalidation_reason")
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        ty.to_string() == "session_invalidation_reason" || ty.to_string() == "text"
    }
}

// Add SQLx Encode implementation for PostgreSQL
impl Encode<'_, Postgres> for SessionInvalidationReason {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        // Convert to the string representation used in the database
        let s = match self {
            SessionInvalidationReason::UserLogout => "USER_LOGOUT",
            SessionInvalidationReason::AdminAction => "ADMIN_ACTION",
            SessionInvalidationReason::PasswordChanged => "PASSWORD_CHANGED",
            SessionInvalidationReason::SecurityBreach => "SECURITY_BREACH",
            SessionInvalidationReason::InactivityTimeout => "INACTIVITY_TIMEOUT",
            SessionInvalidationReason::TokenExpired => "TOKEN_EXPIRED",
            SessionInvalidationReason::DeviceChanged => "DEVICE_CHANGED",
            SessionInvalidationReason::ManualInvalidation => "MANUAL_INVALIDATION",
        };

        // Encode as a string with explicit type annotation for Postgres
        <&str as Encode<Postgres>>::encode_by_ref(&s, buf)
    }
}

// Add SQLx Decode implementation for PostgreSQL
impl<'r> Decode<'r, Postgres> for SessionInvalidationReason {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as Decode<Postgres>>::decode(value)?;
        match s {
            "USER_LOGOUT" => Ok(SessionInvalidationReason::UserLogout),
            "ADMIN_ACTION" => Ok(SessionInvalidationReason::AdminAction),
            "PASSWORD_CHANGED" => Ok(SessionInvalidationReason::PasswordChanged),
            "SECURITY_BREACH" => Ok(SessionInvalidationReason::SecurityBreach),
            "INACTIVITY_TIMEOUT" => Ok(SessionInvalidationReason::InactivityTimeout),
            "TOKEN_EXPIRED" => Ok(SessionInvalidationReason::TokenExpired),
            "DEVICE_CHANGED" => Ok(SessionInvalidationReason::DeviceChanged),
            "MANUAL_INVALIDATION" => Ok(SessionInvalidationReason::ManualInvalidation),
            _ => Err(format!("Unknown session invalidation reason: {}", s).into()),
        }
    }
}

// Implement Display for error messages
impl fmt::Display for SessionInvalidationReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionInvalidationReason::UserLogout => f.write_str("USER_LOGOUT"),
            SessionInvalidationReason::AdminAction => f.write_str("ADMIN_ACTION"),
            SessionInvalidationReason::PasswordChanged => f.write_str("PASSWORD_CHANGED"),
            SessionInvalidationReason::SecurityBreach => f.write_str("SECURITY_BREACH"),
            SessionInvalidationReason::InactivityTimeout => f.write_str("INACTIVITY_TIMEOUT"),
            SessionInvalidationReason::TokenExpired => f.write_str("TOKEN_EXPIRED"),
            SessionInvalidationReason::DeviceChanged => f.write_str("DEVICE_CHANGED"),
            SessionInvalidationReason::ManualInvalidation => f.write_str("MANUAL_INVALIDATION"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceFingerprint {
    pub user_agent_hash: String,
    pub platform: Option<String>,
    pub browser: Option<String>,
    pub screen_resolution: Option<String>,
    pub color_depth: Option<u8>,
    pub timezone: Option<String>,
    pub language: Option<String>,
    pub do_not_track: Option<bool>,
    pub hardware_concurrency: Option<u8>,
    pub additional_data: Option<Value>,
}

impl DeviceFingerprint {
    pub fn new(user_agent_hash: String) -> Self {
        Self {
            user_agent_hash,
            platform: None,
            browser: None,
            screen_resolution: None,
            color_depth: None,
            timezone: None,
            language: None,
            do_not_track: None,
            hardware_concurrency: None,
            additional_data: None,
        }
    }

    pub fn with_platform(mut self, platform: String) -> Self {
        self.platform = Some(platform);
        self
    }

    pub fn with_browser(mut self, browser: String) -> Self {
        self.browser = Some(browser);
        self
    }

    pub fn with_screen_resolution(mut self, resolution: String) -> Self {
        self.screen_resolution = Some(resolution);
        self
    }

    pub fn with_color_depth(mut self, depth: u8) -> Self {
        self.color_depth = Some(depth);
        self
    }

    pub fn with_timezone(mut self, timezone: String) -> Self {
        self.timezone = Some(timezone);
        self
    }

    pub fn with_language(mut self, language: String) -> Self {
        self.language = Some(language);
        self
    }

    pub fn with_do_not_track(mut self, dnt: bool) -> Self {
        self.do_not_track = Some(dnt);
        self
    }

    pub fn with_hardware_concurrency(mut self, concurrency: u8) -> Self {
        self.hardware_concurrency = Some(concurrency);
        self
    }

    pub fn with_additional_data(mut self, data: Value) -> Self {
        self.additional_data = Some(data);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_session_invalidation_reason_serialization() {
        let reason = SessionInvalidationReason::UserLogout;
        let serialized = serde_json::to_string(&reason).unwrap();
        assert_eq!(serialized, "\"USER_LOGOUT\"");

        let deserialized: SessionInvalidationReason = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, SessionInvalidationReason::UserLogout);
    }

    #[test]
    fn test_device_fingerprint_builder() {
        let fingerprint = DeviceFingerprint::new("test_hash".to_string())
            .with_platform("Linux".to_string())
            .with_browser("Firefox".to_string())
            .with_screen_resolution("1920x1080".to_string())
            .with_color_depth(24)
            .with_timezone("UTC".to_string())
            .with_language("en-US".to_string())
            .with_do_not_track(false)
            .with_hardware_concurrency(8)
            .with_additional_data(json!({
                "webgl_vendor": "NVIDIA",
                "webgl_renderer": "GeForce RTX 3080"
            }));

        assert_eq!(fingerprint.user_agent_hash, "test_hash");
        assert_eq!(fingerprint.platform, Some("Linux".to_string()));
        assert_eq!(fingerprint.browser, Some("Firefox".to_string()));
        assert_eq!(fingerprint.screen_resolution, Some("1920x1080".to_string()));
        assert_eq!(fingerprint.color_depth, Some(24));
        assert_eq!(fingerprint.timezone, Some("UTC".to_string()));
        assert_eq!(fingerprint.language, Some("en-US".to_string()));
        assert_eq!(fingerprint.do_not_track, Some(false));
        assert_eq!(fingerprint.hardware_concurrency, Some(8));

        let additional_data = fingerprint.additional_data.unwrap();
        assert_eq!(additional_data["webgl_vendor"], "NVIDIA");
        assert_eq!(additional_data["webgl_renderer"], "GeForce RTX 3080");
    }

    #[test]
    fn test_device_fingerprint_serialization() {
        let fingerprint = DeviceFingerprint::new("test_hash".to_string())
            .with_platform("Linux".to_string())
            .with_browser("Firefox".to_string());

        let serialized = serde_json::to_string(&fingerprint).unwrap();
        let deserialized: DeviceFingerprint = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, fingerprint);
    }

    #[test]
    fn test_device_fingerprint_partial_data() {
        let fingerprint =
            DeviceFingerprint::new("test_hash".to_string()).with_platform("Linux".to_string());

        assert_eq!(fingerprint.platform, Some("Linux".to_string()));
        assert_eq!(fingerprint.browser, None);
        assert_eq!(fingerprint.screen_resolution, None);
        assert_eq!(fingerprint.additional_data, None);
    }
}
