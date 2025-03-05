use crate::prelude::*;
use crate::view;

/// Komponente zur Anzeige von Fehlermeldungen
///
/// # Parameter
/// * `cx` - Der Leptos-Scope
/// * `message` - Die anzuzeigende Fehlermeldung
/// * `error_type` - Der Typ des Fehlers (optional, Standard ist "error")
pub fn error_display_ssr(cx: Scope, message: String, error_type: Option<String>) -> impl IntoView {
    let type_class = error_type.unwrap_or_else(|| "error".to_string());

    view! { cx,
        <div class={format!("error-display {}", type_class)}>
            <div class="error-icon">!</div>
            <div class="error-message">{message}</div>
        </div>
    }
}

// Legacy-Funktion um Kompatibilität zu wahren
#[deprecated(note = "Verwende error_display_ssr stattdessen")]
pub fn error_display_ssr_legacy(
    cx: Scope,
    message: String,
    error_type: Option<String>,
) -> impl IntoView {
    error_display_ssr(cx, message, error_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::leptos::ssr::test_utils;

    #[test]
    fn test_error_display_renders_with_message() {
        // Given an error message
        let message = "Ein Fehler ist aufgetreten".to_string();

        // When rendering the error display
        let html = test_utils::render_to_html(|cx| error_display_ssr(cx, message.clone(), None));

        // Then it should contain the error message
        assert!(test_utils::assert_has_class(&html, "error-display"));
        assert!(test_utils::assert_has_class(&html, "error")); // Default error type
        assert!(test_utils::assert_contains_text(&html, &message));
    }

    #[test]
    fn test_error_display_with_custom_error_type() {
        // Given an error message and a custom error type
        let message = "Warnung".to_string();
        let error_type = "warning".to_string();

        // When rendering the error display with the custom type
        let html = test_utils::render_to_html(|cx| {
            error_display_ssr(cx, message.clone(), Some(error_type.clone()))
        });

        // Then it should have the custom error type class
        assert!(test_utils::assert_has_class(&html, "error-display"));
        assert!(test_utils::assert_has_class(&html, &error_type));
        assert!(test_utils::assert_contains_text(&html, &message));
    }

    #[test]
    fn test_error_display_contains_error_icon() {
        // Given an error message
        let message = "Fehler".to_string();

        // When rendering the error display
        let html = test_utils::render_to_html(|cx| error_display_ssr(cx, message.clone(), None));

        // Then it should contain the error icon
        assert!(test_utils::assert_has_class(&html, "error-icon"));
        assert!(test_utils::assert_contains_text(&html, "!"));
    }
}
