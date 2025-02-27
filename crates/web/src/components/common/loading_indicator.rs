use crate::prelude::*;
use crate::view;

/// Server-side rendered Ladeindikator-Komponente
/// 
/// Diese Komponente zeigt einen Ladeindikator mit einer optionalen Nachricht an.
/// 
/// # Parameter
/// 
/// * `cx` - Der Leptos-Scope
/// * `message` - Eine optionale Nachricht, die angezeigt werden soll
pub fn loading_indicator_ssr(cx: Scope, message: Option<String>) -> impl IntoView {
    let default_message = "Wird geladen...".to_string();
    let display_message = message.unwrap_or(default_message);
    
    view! { cx,
        <div class="loading-indicator">
            <div class="spinner"></div>
            <div class="loading-message">{display_message}</div>
        </div>
    }
}

// Legacy-Funktion um Kompatibilität zu wahren
#[deprecated(note = "Verwende loading_indicator_ssr stattdessen")]
pub fn LoadingIndicatorSSR(cx: Scope, message: Option<String>) -> impl IntoView {
    loading_indicator_ssr(cx, message)
} 