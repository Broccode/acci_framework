use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::models::{TenantId, UserId, VerificationType};
use acci_core::error::Result;

/// Configuration for message providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageProviderConfig {
    /// Email provider configuration
    pub email: EmailProviderConfig,
    /// SMS provider configuration
    pub sms: SmsProviderConfig,
}

/// Email provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailProviderConfig {
    /// Email service provider to use
    pub provider: String,
    /// SMTP server configuration (if using SMTP)
    pub smtp: Option<SmtpConfig>,
    /// API key (if using API-based service)
    pub api_key: Option<String>,
    /// Sender email address
    pub sender_email: String,
    /// Sender name
    pub sender_name: String,
    /// Template for verification emails
    pub verification_template: String,
}

/// SMTP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    /// SMTP server hostname
    pub host: String,
    /// SMTP server port
    pub port: u16,
    /// SMTP username
    pub username: String,
    /// SMTP password
    pub password: String,
    /// Use TLS
    pub use_tls: bool,
}

/// SMS provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsProviderConfig {
    /// SMS service provider to use
    pub provider: String,
    /// API key for the SMS service
    pub api_key: String,
    /// API secret for the SMS service (if needed)
    pub api_secret: Option<String>,
    /// Sender phone number or ID
    pub sender: String,
}

/// Message to be sent
#[derive(Debug, Clone)]
pub struct Message {
    /// Tenant ID
    pub tenant_id: TenantId,
    /// User ID
    pub user_id: UserId,
    /// Recipient (email or phone number)
    pub recipient: String,
    /// Subject (for emails)
    pub subject: Option<String>,
    /// Message body
    pub body: String,
    /// Message type
    pub message_type: VerificationType,
}

/// Trait for message providers
#[async_trait]
pub trait MessageProvider: Send + Sync {
    /// Get the type of verification this provider handles
    fn verification_type(&self) -> VerificationType;

    /// Send a message
    async fn send_message(&self, message: Message) -> Result<String>;
}

/// Mock message provider for testing
#[cfg(test)]
pub struct MockMessageProvider {
    /// Last message sent
    pub last_message: std::sync::Arc<std::sync::Mutex<Option<Message>>>,
    /// Type of verification this provider handles
    pub verification_type: VerificationType,
    /// Provider response
    pub response: String,
}

#[cfg(test)]
impl MockMessageProvider {
    /// Create a new mock message provider
    pub fn new(verification_type: VerificationType) -> Self {
        Self {
            last_message: std::sync::Arc::new(std::sync::Mutex::new(None)),
            verification_type,
            response: "message_id".to_string(),
        }
    }
}

#[cfg(test)]
#[async_trait]
impl MessageProvider for MockMessageProvider {
    fn verification_type(&self) -> VerificationType {
        self.verification_type
    }

    async fn send_message(&self, message: Message) -> Result<String> {
        let mut last_message = self.last_message.lock().unwrap();
        *last_message = Some(message);
        Ok(self.response.clone())
    }
}
