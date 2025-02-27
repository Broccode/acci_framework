use crate::prelude::*;
use crate::view;

/// Server-side rendered Footer-Komponente
/// 
/// Diese Komponente stellt den Footer-Bereich der Anwendung dar, der
/// Copyright-Informationen und Links enthält.
/// 
/// # Parameter
/// 
/// * `cx` - Der Leptos-Scope
pub fn footer_ssr(cx: Scope) -> impl IntoView {
    let current_year = 2025; // In einer realen Anwendung würde das dynamisch ermittelt werden
    
    view! { cx,
        <footer class="main-footer">
            <div class="footer-content">
                <div class="copyright">
                    <p>&copy; {current_year} ACCI Framework. Alle Rechte vorbehalten.</p>
                </div>
                <div class="footer-links">
                    <ul>
                        <li><a href="/impressum">Impressum</a></li>
                        <li><a href="/datenschutz">Datenschutz</a></li>
                        <li><a href="/hilfe">Hilfe</a></li>
                    </ul>
                </div>
            </div>
        </footer>
    }
}

// Legacy-Funktion um Kompatibilität zu wahren
#[deprecated(note = "Verwende footer_ssr stattdessen")]
pub fn FooterSSR(cx: Scope) -> impl IntoView {
    footer_ssr(cx)
} 