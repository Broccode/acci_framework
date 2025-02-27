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
pub fn ErrorDisplaySSR(cx: Scope, message: String, error_type: Option<String>) -> impl IntoView {
    error_display_ssr(cx, message, error_type)
} 