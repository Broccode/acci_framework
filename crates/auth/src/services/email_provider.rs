use async_trait::async_trait;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use std::sync::Arc;
use tracing::{debug, error, info, instrument};

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

        // Get SMTP config
        let smtp_config = self.config.smtp.clone().ok_or_else(|| {
            Error::Config("SMTP configuration is required for SMTP email provider".to_string())
        })?;

        // Build email message
        let subject = message
            .subject
            .unwrap_or_else(|| "Verification Code".to_string());

        // Parse sender and recipient email addresses
        let sender = format!("{} <{}>", self.config.sender_name, self.config.sender_email)
            .parse::<Mailbox>()
            .map_err(|e| Error::Other(anyhow::anyhow!("Invalid sender address: {}", e)))?;

        let recipient = message
            .recipient
            .parse::<Mailbox>()
            .map_err(|e| Error::Other(anyhow::anyhow!("Invalid recipient address: {}", e)))?;

        // Create email structure
        let email = Message::builder()
            .from(sender)
            .to(recipient.clone())
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(message.body)
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to build email: {}", e)))?;

        // Create SMTP transport
        let mailer = build_smtp_transport(&smtp_config)?;

        // Send the email
        match mailer.send(email).await {
            Ok(_) => {
                info!(
                    recipient = %message.recipient,
                    "Email verification message sent successfully"
                );
                Ok(format!("email:{}", uuid::Uuid::new_v4()))
            },
            Err(e) => {
                error!(
                    recipient = %message.recipient,
                    error = %e,
                    "Failed to send email verification message"
                );
                Err(Error::Other(anyhow::anyhow!("Failed to send email: {}", e)))
            },
        }
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
        debug!(
            recipient = %message.recipient,
            "Sending email via SendGrid"
        );

        // Build the SendGrid V3 API request
        // This is a basic implementation of the SendGrid V3 API
        // For a production environment, consider using a dedicated SendGrid client

        // Get the subject and sender from config
        let subject = message
            .subject
            .unwrap_or_else(|| "Verification Code".to_string());
        let from_email = self.config.sender_email.clone();
        let from_name = self.config.sender_name.clone();

        // Construct the SendGrid API payload
        let payload = serde_json::json!({
            "personalizations": [{
                "to": [{
                    "email": message.recipient
                }]
            }],
            "from": {
                "email": from_email,
                "name": from_name
            },
            "subject": subject,
            "content": [{
                "type": "text/plain",
                "value": message.body
            }]
        });

        // Send the request using reqwest
        let client = reqwest::Client::new();
        let res = client
            .post("https://api.sendgrid.com/v3/mail/send")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::Other(anyhow::anyhow!("SendGrid API request failed: {}", e)))?;

        // Check response
        if res.status().is_success() {
            info!(
                recipient = %message.recipient,
                "Email sent successfully via SendGrid"
            );
            Ok(format!("sendgrid:{}", uuid::Uuid::new_v4()))
        } else {
            let status = res.status();
            let error_text = res
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                recipient = %message.recipient,
                status = %status,
                error = %error_text,
                "Failed to send email via SendGrid"
            );
            Err(Error::Other(anyhow::anyhow!(
                "SendGrid API error: {} - {}",
                status,
                error_text
            )))
        }
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
fn build_smtp_transport(config: &SmtpConfig) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
    // Create credentials
    let credentials = Credentials::new(config.username.clone(), config.password.clone());

    // Create the appropriate transport based on TLS configuration
    let mailer = if config.use_tls {
        // TLS transport
        AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
            .map_err(|e| Error::Other(anyhow::anyhow!("SMTP relay error: {}", e)))?
            .credentials(credentials)
            .port(config.port)
            .build()
    } else {
        // Plain transport (not recommended for production)
        AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host)
            .credentials(credentials)
            .port(config.port)
            .build()
    };

    Ok(mailer)
}
