// Models Module
// Dieses Modul enthält Datenstrukturen für die Web-Anwendung

use serde::{Deserialize, Serialize};

/// Benutzermodell für die Anzeige in der UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserViewModel {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub is_admin: bool,
}

/// Session-Modell für die Authentifizierung
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub token: String,
    pub user_id: String,
    pub expires_at: i64,
}

/// Ergebnis einer API-Operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

/// Implementierung für ApiResult
impl<T> ApiResult<T> {
    /// Erstellt ein erfolgreiches Ergebnis
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// Erstellt ein Fehlerergebnis
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
} 