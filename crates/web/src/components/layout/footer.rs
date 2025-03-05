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
pub fn footer_ssr_legacy(cx: Scope) -> impl IntoView {
    footer_ssr(cx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::leptos::ssr::test_utils;

    #[test]
    fn test_footer_renders_correctly() {
        // When rendering the footer component
        let html = test_utils::render_to_html(|cx| footer_ssr(cx));

        // Then it should contain the main footer elements
        assert!(test_utils::assert_has_class(&html, "main-footer"));
        assert!(test_utils::assert_has_class(&html, "footer-content"));
        assert!(test_utils::assert_has_class(&html, "copyright"));
        assert!(test_utils::assert_has_class(&html, "footer-links"));
    }

    #[test]
    fn test_footer_contains_copyright_with_year() {
        // When rendering the footer component
        let html = test_utils::render_to_html(|cx| footer_ssr(cx));

        // Then it should contain the copyright information with the current year
        assert!(test_utils::assert_contains_text(&html, "2025")); // Hardcoded in the component
        assert!(test_utils::assert_contains_text(&html, "ACCI Framework"));
        assert!(test_utils::assert_contains_text(
            &html,
            "Alle Rechte vorbehalten"
        ));
    }

    #[test]
    fn test_footer_contains_required_links() {
        // When rendering the footer component
        let html = test_utils::render_to_html(|cx| footer_ssr(cx));

        // Then it should contain all required links
        assert!(test_utils::assert_contains_text(&html, "/impressum"));
        assert!(test_utils::assert_contains_text(&html, "/datenschutz"));
        assert!(test_utils::assert_contains_text(&html, "/hilfe"));

        // And the appropriate link text
        assert!(test_utils::assert_contains_text(&html, "Impressum"));
        assert!(test_utils::assert_contains_text(&html, "Datenschutz"));
        assert!(test_utils::assert_contains_text(&html, "Hilfe"));
    }
}
