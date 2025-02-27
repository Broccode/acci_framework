// Utils Module
// Dieses Modul enthält Hilfsfunktionen für die Web-Anwendung

/// Validiert eine E-Mail-Adresse
pub fn validate_email(email: &str) -> bool {
    // Einfache E-Mail-Validierung
    // In einer produktiven Anwendung würde hier eine umfassendere Validierung stehen
    email.contains('@') && email.contains('.')
}

/// Validiert ein Passwort
pub fn validate_password(password: &str) -> bool {
    // Einfache Passwortvalidierung
    // In einer produktiven Anwendung würden hier komplexere Regeln stehen
    password.len() >= 8
}

/// Generiert eine URL mit zusätzlichen Query-Parametern
pub fn build_url_with_query(base_url: &str, params: &[(&str, &str)]) -> String {
    if params.is_empty() {
        return base_url.to_string();
    }

    let mut url = String::from(base_url);
    url.push('?');

    for (i, (key, value)) in params.iter().enumerate() {
        if i > 0 {
            url.push('&');
        }
        url.push_str(key);
        url.push('=');
        url.push_str(value);
    }

    url
} 