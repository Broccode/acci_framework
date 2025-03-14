use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info, instrument};

use crate::models::VerificationType;
use crate::services::message_provider::{Message, MessageProvider, SmsProviderConfig};
use acci_core::error::{Error, Result};

/// SMS Provider using Twilio for delivering messages
pub struct TwilioSmsProvider {
    /// Configuration for the SMS provider
    config: SmsProviderConfig,
    /// Base URL for Twilio API
    base_url: String,
}

impl TwilioSmsProvider {
    /// Create a new Twilio SMS provider
    pub fn new(config: SmsProviderConfig) -> Self {
        Self {
            config,
            base_url: "https://api.twilio.com/2010-04-01".to_string(),
        }
    }
}

#[async_trait]
impl MessageProvider for TwilioSmsProvider {
    fn verification_type(&self) -> VerificationType {
        VerificationType::Sms
    }
    
    #[instrument(skip(self, message), level = "debug")]
    async fn send_message(&self, message: Message) -> Result<String> {
        debug!(
            recipient = %message.recipient,
            "Sending SMS verification message via Twilio"
        );
        
        // In a real implementation, we would use reqwest to call the Twilio API
        // For now, we'll return a placeholder message ID
        // The real implementation would look something like:
        
        /*
        let client = reqwest::Client::new();
        
        let response = client
            .post(&format!("{}/Accounts/{}/Messages.json", self.base_url, self.config.api_key))
            .basic_auth(&self.config.api_key, Some(&self.config.api_secret.clone().unwrap_or_default()))
            .form(&[
                ("From", &self.config.sender),
                ("To", &message.recipient),
                ("Body", &message.body),
            ])
            .send()
            .await
            .map_err(|err| {
                error!("Failed to send Twilio request: {}", err);
                Error::Other(anyhow::anyhow!("Failed to send Twilio request: {}", err))
            })?;
            
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Twilio API error: {}", error_text);
            return Err(Error::Other(anyhow::anyhow!("Twilio API error: {}", error_text)));
        }
        
        let response_json: serde_json::Value = response.json().await.map_err(|err| {
            error!("Failed to parse Twilio response: {}", err);
            Error::Other(anyhow::anyhow!("Failed to parse Twilio response: {}", err))
        })?;
        
        let message_sid = response_json["sid"].as_str().ok_or_else(|| {
            error!("Twilio response missing message SID");
            Error::Other(anyhow::anyhow!("Twilio response missing message SID"))
        })?;
        */
        
        info!(
            recipient = %message.recipient,
            "SMS verification message sent successfully (simulated)"
        );
        
        // Placeholder message ID
        Ok(format!("twilio:{}", uuid::Uuid::new_v4()))
    }
}

/// SMS Provider using Vonage (formerly Nexmo) for delivering messages
pub struct VonageSmsProvider {
    /// Configuration for the SMS provider
    config: SmsProviderConfig,
}

impl VonageSmsProvider {
    /// Create a new Vonage SMS provider
    pub fn new(config: SmsProviderConfig) -> Self {
        Self {
            config,
        }
    }
}

#[async_trait]
impl MessageProvider for VonageSmsProvider {
    fn verification_type(&self) -> VerificationType {
        VerificationType::Sms
    }
    
    #[instrument(skip(self, message), level = "debug")]
    async fn send_message(&self, message: Message) -> Result<String> {
        debug!(
            recipient = %message.recipient,
            "Sending SMS verification message via Vonage (not yet implemented)"
        );
        
        // Placeholder for Vonage implementation
        
        info!(
            recipient = %message.recipient,
            "SMS verification message sent successfully (simulated)"
        );
        
        // Placeholder message ID
        Ok(format!("vonage:{}", uuid::Uuid::new_v4()))
    }
}

/// Factory function to create an SMS provider based on configuration
pub fn create_sms_provider(config: SmsProviderConfig) -> Result<Arc<dyn MessageProvider>> {
    match config.provider.to_lowercase().as_str() {
        "twilio" => {
            let provider = TwilioSmsProvider::new(config);
            Ok(Arc::new(provider))
        },
        "vonage" | "nexmo" => {
            let provider = VonageSmsProvider::new(config);
            Ok(Arc::new(provider))
        },
        _ => Err(Error::Config(format!(
            "Unsupported SMS provider: {}", config.provider
        ))),
    }
}