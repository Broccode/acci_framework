use crate::prelude::*;
use crate::view;
use serde::{Deserialize, Serialize};

/// RegistrationForm-Struktur für die Datenverarbeitung
#[derive(Serialize, Deserialize, Clone)]
pub struct RegistrationForm {
    pub email: String,
    pub password: String,
    pub password_confirmation: String,
    pub error: Option<String>,
}

/// Server-side rendered Registrierungsformular Komponente
/// 
/// Diese Komponente stellt ein HTML-Formular für die Benutzerregistrierung bereit und
/// wird ausschließlich auf dem Server gerendert (SSR).
/// 
/// # Parameter
/// 
/// * `cx` - Der Leptos-Scope
/// * `action_path` - Der Pfad, an den das Formular gesendet wird
/// * `error` - Eine optionale Fehlermeldung, die angezeigt werden soll
pub fn registration_form_ssr(cx: Scope, action_path: String, error: Option<String>) -> impl IntoView {
    view! { cx,
        <form method="post" action={action_path} class="auth-form registration-form">
            <div class="form-group">
                <label for="email">E-Mail</label>
                <input 
                    type="email" 
                    id="email" 
                    name="email" 
                    required
                />
            </div>
            <div class="form-group">
                <label for="password">Passwort</label>
                <input 
                    type="password" 
                    id="password" 
                    name="password" 
                    required
                />
            </div>
            <div class="form-group">
                <label for="password_confirmation">Passwort bestätigen</label>
                <input 
                    type="password" 
                    id="password_confirmation" 
                    name="password_confirmation" 
                    required
                />
            </div>
            
            // Bedingte Anzeige einer Fehlermeldung, falls vorhanden
            {error.map(|err| view! { cx, <div class="error-message">{err}</div> })}
            
            <div class="form-actions">
                <button type="submit" class="btn btn-primary">Registrieren</button>
            </div>
            
            <div class="form-links">
                <a href="/login" class="login-link">Zurück zum Login</a>
            </div>
        </form>
    }
}

// Legacy-Funktion um Kompatibilität zu wahren
#[deprecated(note = "Verwende registration_form_ssr stattdessen")]
pub fn RegistrationFormSSR(cx: Scope, action_path: String, error: Option<String>) -> impl IntoView {
    registration_form_ssr(cx, action_path, error)
} 