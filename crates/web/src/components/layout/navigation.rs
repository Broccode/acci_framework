use crate::prelude::*;
use crate::view;

/// Server-side rendered Navigationskomponente
///
/// Diese Komponente zeigt die Hauptnavigation der Anwendung an und passt
/// die angezeigten Links basierend auf dem Authentifizierungsstatus des Benutzers an.
///
/// # Parameter
///
/// * `cx` - Der Leptos-Scope
/// * `is_authenticated` - Gibt an, ob der Benutzer angemeldet ist
/// * `user_name` - Der Name des angemeldeten Benutzers (falls vorhanden)
pub fn navigation_ssr(
    cx: Scope,
    is_authenticated: bool,
    user_name: Option<String>,
) -> impl IntoView {
    view! { cx,
        <nav class="main-navigation">
            <div class="logo">
                <a href="/">ACCI Framework</a>
            </div>
            <ul class="nav-links">
                <li><a href="/">Home</a></li>
                {if is_authenticated {
                    view! { cx,
                        <>
                            <li><a href="/dashboard">Dashboard</a></li>
                            <li>
                                <form method="post" action="/api/auth/logout">
                                    <button type="submit" class="btn-link">Abmelden</button>
                                </form>
                            </li>
                            <li class="user-info">{user_name.unwrap_or_else(|| "Benutzer".to_string())}</li>
                        </>
                    }
                } else {
                    view! { cx,
                        <>
                            <li><a href="/login">Anmelden</a></li>
                            <li><a href="/register">Registrieren</a></li>
                        </>
                    }
                }}
            </ul>
        </nav>
    }
}

// Legacy-Funktion um Kompatibilität zu wahren
#[deprecated(note = "Verwende navigation_ssr stattdessen")]
pub fn NavigationSSR(
    cx: Scope,
    is_authenticated: bool,
    user_name: Option<String>,
) -> impl IntoView {
    navigation_ssr(cx, is_authenticated, user_name)
}
