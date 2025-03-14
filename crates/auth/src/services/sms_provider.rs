use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;
use tracing::{debug, error, info, instrument};
use urlencoding::encode;

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

        // Get API key and secret from config
        let api_key = &self.config.api_key;
        let api_secret = self
            .config
            .api_secret
            .clone()
            .ok_or_else(|| Error::Config("Twilio API secret is required".to_string()))?;

        // Extract account SID from the API key (in Twilio, the API key is usually the account SID)
        let account_sid = api_key;

        // Create request client
        let client = Client::new();

        // Build the Twilio API request
        let url = format!("{}/Accounts/{}/Messages.json", self.base_url, account_sid);

        debug!("Sending request to Twilio API: {}", url);

        // Send the request
        let response = client
            .post(&url)
            .basic_auth(api_key, Some(&api_secret))
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

        // Check response status
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                status = %status,
                error = %error_text,
                "Twilio API error"
            );
            return Err(Error::Other(anyhow::anyhow!(
                "Twilio API error: {} - {}",
                status,
                error_text
            )));
        }

        // Parse response
        let response_json: serde_json::Value = response.json().await.map_err(|err| {
            error!("Failed to parse Twilio response: {}", err);
            Error::Other(anyhow::anyhow!("Failed to parse Twilio response: {}", err))
        })?;

        // Extract message SID
        let message_sid = response_json["sid"].as_str().ok_or_else(|| {
            error!("Twilio response missing message SID");
            Error::Other(anyhow::anyhow!("Twilio response missing message SID"))
        })?;

        info!(
            recipient = %message.recipient,
            message_sid = %message_sid,
            "SMS verification message sent successfully via Twilio"
        );

        // Return message ID
        Ok(format!("twilio:{}", message_sid))
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
        Self { config }
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
            "Sending SMS verification message via Vonage"
        );

        // Get API credentials
        let api_key = &self.config.api_key;
        let api_secret = self
            .config
            .api_secret
            .clone()
            .ok_or_else(|| Error::Config("Vonage API secret is required".to_string()))?;

        // Create request client
        let client = Client::new();

        // Vonage API endpoint
        let url = "https://rest.nexmo.com/sms/json";

        // URL encode the message body and convert Cow<str> to String
        let encoded_body: String = encode(&message.body).into_owned();

        // Build and send the request
        let response = client
            .post(url)
            .form(&[
                ("api_key", api_key),
                ("api_secret", &api_secret),
                ("from", &self.config.sender),
                ("to", &message.recipient),
                ("text", &encoded_body),
            ])
            .send()
            .await
            .map_err(|err| {
                error!("Failed to send Vonage request: {}", err);
                Error::Other(anyhow::anyhow!("Failed to send Vonage request: {}", err))
            })?;

        // Check response status
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!(
                status = %status,
                error = %error_text,
                "Vonage API error"
            );
            return Err(Error::Other(anyhow::anyhow!(
                "Vonage API error: {} - {}",
                status,
                error_text
            )));
        }

        // Parse response
        let response_json: serde_json::Value = response.json().await.map_err(|err| {
            error!("Failed to parse Vonage response: {}", err);
            Error::Other(anyhow::anyhow!("Failed to parse Vonage response: {}", err))
        })?;

        // Vonage returns messages as an array
        let messages = response_json["messages"].as_array().ok_or_else(|| {
            error!("Vonage response missing messages array");
            Error::Other(anyhow::anyhow!("Vonage response missing messages array"))
        })?;

        if messages.is_empty() {
            return Err(Error::Other(anyhow::anyhow!(
                "Vonage response contains no messages"
            )));
        }

        // Get the first message info
        let message_id = messages[0]["message-id"].as_str().ok_or_else(|| {
            error!("Vonage response missing message ID");
            Error::Other(anyhow::anyhow!("Vonage response missing message ID"))
        })?;

        info!(
            recipient = %message.recipient,
            message_id = %message_id,
            "SMS verification message sent successfully via Vonage"
        );

        // Return message ID
        Ok(format!("vonage:{}", message_id))
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
            "Unsupported SMS provider: {}",
            config.provider
        ))),
    }
}
