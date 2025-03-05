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
pub fn login_form_ssr_legacy(
    cx: Scope,
    action_path: String,
    error: Option<String>,
) -> impl IntoView {
    login_form_ssr(cx, action_path, error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::leptos::ssr::test_utils;

    #[test]
    fn test_login_form_renders_with_correct_structure() {
        // Given a login form with a specific action path
        let action_path = "/api/auth/login".to_string();

        // When rendering the form without an error
        let html = test_utils::render_to_html(|cx| login_form_ssr(cx, action_path.clone(), None));

        // Then it should contain the proper form elements
        assert!(test_utils::assert_has_class(&html, "auth-form"));
        assert!(test_utils::assert_has_class(&html, "login-form"));
        assert!(test_utils::assert_contains_text(&html, "E-Mail"));
        assert!(test_utils::assert_contains_text(&html, "Passwort"));
        assert!(test_utils::assert_contains_text(&html, "Anmelden"));
        assert!(test_utils::assert_contains_text(&html, "/register"));
    }

    #[test]
    fn test_login_form_displays_error_when_provided() {
        // Given a login form with an error message
        let action_path = "/api/auth/login".to_string();
        let error_message = "Ungültige Anmeldedaten".to_string();

        // When rendering the form with the error
        let html = test_utils::render_to_html(|cx| {
            login_form_ssr(cx, action_path.clone(), Some(error_message.clone()))
        });

        // Then it should display the error message
        assert!(test_utils::assert_has_class(&html, "error-message"));
        assert!(test_utils::assert_contains_text(&html, &error_message));
    }

    #[test]
    fn test_login_form_with_custom_action_path() {
        // Given a login form with a custom action path
        let custom_path = "/custom/login/path".to_string();

        // When rendering the form
        let html = test_utils::render_to_html(|cx| login_form_ssr(cx, custom_path.clone(), None));

        // Then it should have the custom action path
        assert!(test_utils::assert_contains_text(&html, &custom_path));
    }
}
