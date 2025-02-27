use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;
use thiserror::Error;

/// Fehler, die während der Authentifizierung auftreten können
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Ungültige Anmeldedaten")]
    InvalidCredentials,

    #[error("Benutzer existiert bereits")]
    UserAlreadyExists,

    #[error("Konto gesperrt")]
    AccountLocked,

    #[error("Validierungsfehler: {0}")]
    ValidationError(String),

    #[error("Interner Serverfehler: {0}")]
    InternalError(Box<dyn StdError + Send + Sync>),
}

/// Anmeldedaten
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}

/// Daten für die Benutzerregistrierung
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
}

/// Session-Informationen
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Session {
    pub token: String,
    pub user_id: String,
    pub expires_at: i64,
}

/// Vereinfachter Authentifizierungs-Service
#[derive(Clone)]
pub struct AuthService {
    // In einer realen Anwendung würden hier Verbindungen zur Datenbank und
    // andere benötigte Konfigurationen injiziert werden
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthService {
    /// Erstellt eine neue Instanz des Auth-Service
    pub fn new() -> Self {
        Self {}
    }

    /// Führt einen Login-Vorgang durch
    pub async fn login(&self, credentials: &LoginCredentials) -> Result<Session, AuthError> {
        // In einer echten Implementierung würde hier die Datenbank abgefragt werden
        // und ein echter Login-Prozess durchgeführt werden

        // Für Demonstrationszwecke simulieren wir eine erfolgreiche Anmeldung
        // wenn die E-Mail "demo@example.com" und das Passwort "password" ist
        if credentials.email == "demo@example.com" && credentials.password == "password" {
            Ok(Session {
                token: "demo-token-123".to_string(),
                user_id: "user-1".to_string(),
                expires_at: chrono::Utc::now().timestamp() + 86400, // 1 Tag
            })
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }

    /// Registriert einen neuen Benutzer
    pub async fn register(&self, user: &CreateUser) -> Result<(), AuthError> {
        // In einer echten Implementierung würde hier ein neuer Benutzer erstellt werden

        // Für Demonstrationszwecke simulieren wir, dass die Registrierung erfolgreich ist,
        // wenn die E-Mail nicht "demo@example.com" ist (da dieser "bereits existiert")
        if user.email == "demo@example.com" {
            Err(AuthError::UserAlreadyExists)
        } else if user.password.len() < 8 {
            Err(AuthError::ValidationError(
                "Passwort muss mindestens 8 Zeichen lang sein".to_string(),
            ))
        } else {
            Ok(())
        }
    }

    /// Überprüft die Gültigkeit eines Tokens
    pub async fn validate_token(&self, token: &str) -> Result<Session, AuthError> {
        // In einer echten Implementierung würde hier der Token validiert werden

        // Für Demonstrationszwecke simulieren wir, dass der Token gültig ist,
        // wenn er "demo-token-123" ist
        if token == "demo-token-123" {
            Ok(Session {
                token: token.to_string(),
                user_id: "user-1".to_string(),
                expires_at: chrono::Utc::now().timestamp() + 86400, // 1 Tag
            })
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }
}

/// Fehlertyp, der dem acci_auth::Error entspricht, damit wir die bestehenden Handler verwenden können
pub enum Error {
    InvalidCredentials,
    UserAlreadyExists,
    AccountLocked,
    ValidationError,
    InternalError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidCredentials => write!(f, "Ungültige Anmeldedaten"),
            Error::UserAlreadyExists => write!(f, "Benutzer existiert bereits"),
            Error::AccountLocked => write!(f, "Konto gesperrt"),
            Error::ValidationError => write!(f, "Validierungsfehler"),
            Error::InternalError => write!(f, "Interner Serverfehler"),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl From<AuthError> for Error {
    fn from(err: AuthError) -> Self {
        match err {
            AuthError::InvalidCredentials => Error::InvalidCredentials,
            AuthError::UserAlreadyExists => Error::UserAlreadyExists,
            AuthError::AccountLocked => Error::AccountLocked,
            AuthError::ValidationError(_) => Error::ValidationError,
            AuthError::InternalError(_) => Error::InternalError,
        }
    }
}
