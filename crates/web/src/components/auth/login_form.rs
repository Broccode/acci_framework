use crate::prelude::*;
use crate::view;
use serde::{Deserialize, Serialize};

/// LoginForm-Struktur für die Datenverarbeitung
#[derive(Serialize, Deserialize, Clone)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
    pub error: Option<String>,
}

/// Server-side rendered Login-Formular Komponente
///
/// Diese Komponente stellt ein HTML-Formular für den Login bereit und
/// wird ausschließlich auf dem Server gerendert (SSR).
///
/// # Parameter
///
/// * `cx` - Der Leptos-Scope
/// * `action_path` - Der Pfad, an den das Formular gesendet wird
/// * `error` - Eine optionale Fehlermeldung, die angezeigt werden soll
pub fn login_form_ssr(cx: Scope, action_path: String, error: Option<String>) -> impl IntoView {
    view! { cx,
        <form method="post" action={action_path} class="auth-form login-form">
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

            {
                match error {
                    Some(err) => view! { cx, <div class="error-message">{err}</div> },
                    None => view! { cx, <> </> }
                }
            }

            <div class="form-actions">
                <button type="submit" class="btn btn-primary">Anmelden</button>
            </div>

            <div class="form-links">
                <a href="/register" class="register-link">Konto erstellen</a>
            </div>
        </form>
    }
}

// Legacy-Funktion um Kompatibilität zu wahren
#[deprecated(note = "Verwende login_form_ssr stattdessen")]
pub fn LoginFormSSR(cx: Scope, action_path: String, error: Option<String>) -> impl IntoView {
    login_form_ssr(cx, action_path, error)
}
