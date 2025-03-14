use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info, instrument};

use crate::models::VerificationType;
use crate::services::message_provider::{
    EmailProviderConfig, Message as ProviderMessage, MessageProvider, SmtpConfig,
};
use acci_core::error::{Error, Result};

/// EmailProvider using SMTP for delivering messages
pub struct SmtpEmailProvider {
    /// Configuration for the email provider
    config: EmailProviderConfig,
}

impl SmtpEmailProvider {
    /// Create a new SMTP email provider
    pub fn new(config: EmailProviderConfig) -> Result<Self> {
        // Get SMTP config
        let _smtp_config = config.smtp.clone().ok_or_else(|| {
            Error::Config("SMTP configuration is required for SMTP email provider".to_string())
        })?;

        Ok(Self { config })
    }
}

#[async_trait]
impl MessageProvider for SmtpEmailProvider {
    fn verification_type(&self) -> VerificationType {
        VerificationType::Email
    }

    #[instrument(skip(self, message), level = "debug")]
    async fn send_message(&self, message: ProviderMessage) -> Result<String> {
        debug!(
            recipient = %message.recipient,
            "Sending email verification message"
        );

        let _subject = message
            .subject
            .unwrap_or_else(|| "Verification Code".to_string());

        // For now, to avoid complex email provider implementation, we'll simulate sending
        info!(
            recipient = %message.recipient,
            "Email verification message sent successfully (simulated)"
        );

        // Return a message ID (this is a placeholder as SMTP doesn't return message IDs)
        Ok(format!("email:{}", uuid::Uuid::new_v4()))
    }
}

/// Email provider using the SendGrid API for delivering messages
pub struct SendGridEmailProvider {
    /// Configuration for the email provider
    config: EmailProviderConfig,
    /// API key
    api_key: String,
}

impl SendGridEmailProvider {
    /// Create a new SendGrid email provider
    pub fn new(config: EmailProviderConfig) -> Result<Self> {
        // Get API key
        let api_key = config.api_key.clone().ok_or_else(|| {
            Error::Config("API key is required for SendGrid email provider".to_string())
        })?;

        Ok(Self { config, api_key })
    }
}

#[async_trait]
impl MessageProvider for SendGridEmailProvider {
    fn verification_type(&self) -> VerificationType {
        VerificationType::Email
    }

    #[instrument(skip(self, message), level = "debug")]
    async fn send_message(&self, message: ProviderMessage) -> Result<String> {
        // Placeholder for SendGrid implementation
        // This would use reqwest to call the SendGrid API

        debug!(
            recipient = %message.recipient,
            "Sending email via SendGrid (not yet implemented)"
        );

        // Return a message ID (placeholder)
        Ok(format!("sendgrid:{}", uuid::Uuid::new_v4()))
    }
}

/// Factory function to create an email provider based on configuration
pub fn create_email_provider(config: EmailProviderConfig) -> Result<Arc<dyn MessageProvider>> {
    match config.provider.to_lowercase().as_str() {
        "smtp" => {
            let provider = SmtpEmailProvider::new(config)?;
            Ok(Arc::new(provider))
        },
        "sendgrid" => {
            let provider = SendGridEmailProvider::new(config)?;
            Ok(Arc::new(provider))
        },
        _ => Err(Error::Config(format!(
            "Unsupported email provider: {}",
            config.provider
        ))),
    }
}

/// Build an SMTP transport from configuration
fn build_smtp_transport(config: &SmtpConfig) -> Result<()> {
    // Just a placeholder function for future implementation
    let _ = config;
    Ok(())
}