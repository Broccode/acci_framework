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
pub fn registration_form_ssr(
    cx: Scope,
    action_path: String,
    error: Option<String>,
) -> impl IntoView {
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
pub fn registration_form_ssr_legacy(
    cx: Scope,
    action_path: String,
    error: Option<String>,
) -> impl IntoView {
    registration_form_ssr(cx, action_path, error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::leptos::ssr::test_utils;

    #[test]
    fn test_registration_form_renders_with_correct_structure() {
        // Given a registration form with a specific action path
        let action_path = "/api/auth/register".to_string();

        // When rendering the form without an error
        let html =
            test_utils::render_to_html(|cx| registration_form_ssr(cx, action_path.clone(), None));

        // Then it should contain the proper form elements
        assert!(test_utils::assert_has_class(&html, "auth-form"));
        assert!(test_utils::assert_has_class(&html, "registration-form"));
        assert!(test_utils::assert_contains_text(&html, "E-Mail"));
        assert!(test_utils::assert_contains_text(&html, "Passwort"));
        assert!(test_utils::assert_contains_text(
            &html,
            "Passwort bestätigen"
        ));
        assert!(test_utils::assert_contains_text(&html, "Registrieren"));
        assert!(test_utils::assert_contains_text(&html, "/login"));
    }

    #[test]
    fn test_registration_form_displays_error_when_provided() {
        // Given a registration form with an error message
        let action_path = "/api/auth/register".to_string();
        let error_message = "Passwörter stimmen nicht überein".to_string();

        // When rendering the form with the error
        let html = test_utils::render_to_html(|cx| {
            registration_form_ssr(cx, action_path.clone(), Some(error_message.clone()))
        });

        // Then it should display the error message
        assert!(test_utils::assert_has_class(&html, "error-message"));
        assert!(test_utils::assert_contains_text(&html, &error_message));
    }

    #[test]
    fn test_registration_form_with_custom_action_path() {
        // Given a registration form with a custom action path
        let custom_path = "/custom/register/path".to_string();

        // When rendering the form
        let html =
            test_utils::render_to_html(|cx| registration_form_ssr(cx, custom_path.clone(), None));

        // Then it should have the custom action path
        assert!(test_utils::assert_contains_text(&html, &custom_path));
    }

    #[test]
    fn test_registration_form_has_password_confirmation_field() {
        // Given a registration form
        let action_path = "/api/auth/register".to_string();

        // When rendering the form
        let html =
            test_utils::render_to_html(|cx| registration_form_ssr(cx, action_path.clone(), None));

        // Then it should have a password confirmation field
        assert!(test_utils::assert_contains_text(
            &html,
            "password_confirmation"
        ));
    }
}
