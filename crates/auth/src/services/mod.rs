pub mod email_provider;
pub mod message_provider;
pub mod session;
pub mod sms_provider;
pub mod tenant;
pub mod totp;
pub mod user;
pub mod verification;
#[cfg(feature = "enable_webauthn")]
pub mod webauthn;

#[cfg(test)]
mod tests;

#[cfg(feature = "enable_webauthn")]
pub use crate::models::webauthn::WebAuthnError;
pub use email_provider::{SendGridEmailProvider, SmtpEmailProvider, create_email_provider};
pub use message_provider::{
    EmailProviderConfig, Message, MessageProvider, MessageProviderConfig, SmsProviderConfig,
    SmtpConfig,
};
pub use sms_provider::{TwilioSmsProvider, VonageSmsProvider, create_sms_provider};
pub use verification::{VerificationError, VerificationService};
#[cfg(feature = "enable_webauthn")]
pub use webauthn::{WebAuthnConfig, WebAuthnService};
