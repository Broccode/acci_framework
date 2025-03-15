// Auth Components Module
// Komponenten für die Authentifizierung (Login, Registrierung, Verifikation)

pub mod login_form;
pub mod registration_form;
pub mod verification_form;
#[cfg(feature = "enable_webauthn")]
pub mod webauthn_form;

// Re-exports für häufig verwendete Komponenten
pub use login_form::*;
pub use registration_form::*;
pub use verification_form::*;
#[cfg(feature = "enable_webauthn")]
pub use webauthn_form::*;
