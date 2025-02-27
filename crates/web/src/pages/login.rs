use leptos::*;
use serde::Deserialize;
use crate::components::auth::LoginFormSSR;
use crate::components::layout::NavigationSSR;
use crate::components::layout::FooterSSR;
use crate::services::leptos::LeptosOptions;
use crate::services::leptos::ssr;

/// Struktur für Query-Parameter der Login-Seite
#[derive(Deserialize)]
pub struct LoginQuery {
    pub error: Option<String>,
    pub redirect: Option<String>,
}

/// Rendert die gesamte Login-Seite als SSR
/// 
/// Diese Funktion rendert die vollständige HTML-Seite für den Login,
/// einschließlich Header, Navigation, Formular und Footer.
pub fn render_login_page(
    renderer: &LeptosOptions,
    error: Option<String>,
    redirect: Option<String>
) -> String {
    let _redirect_path = redirect.unwrap_or_else(|| "/".to_string());
    
    // Die gesamte Seite wird serverseitig gerendert
    ssr::render_to_string_with_context(
        renderer,
        move |_cx| {
            // Erstelle den Error-Display-String
            let error_display = if let Some(err_msg) = error {
                format!(r#"<div class="error-message">{}</div>"#, err_msg)
            } else {
                "".to_string()
            };
            
            // Erstelle den HTML-String mit dem Error-Display
            let html_string = format!(r#"
            <html>
                <head>
                    <title>Anmelden - ACCI Framework</title>
                    <meta charset="UTF-8"/>
                    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
                    <link rel="stylesheet" href="/static/styles/main.css"/>
                </head>
                <body>
                    <main class="container">
                        <h1>Anmelden</h1>
                        <p class="page-description">
                            Bitte melden Sie sich mit Ihren Zugangsdaten an.
                        </p>
                        <form method="post" action="/api/auth/login" class="auth-form login-form">
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
                            {error_display}
                            <div class="form-actions">
                                <button type="submit" class="btn btn-primary">Anmelden</button>
                            </div>
                            
                            <div class="form-links">
                                <a href="/register" class="register-link">Konto erstellen</a>
                            </div>
                        </form>
                        <div class="form-footer">
                            <p>Noch kein Konto? <a href="/register">Registrieren</a></p>
                        </div>
                    </main>
                    <script src="/static/js/validation.js"></script>
                </body>
            </html>
            "#, error_display = error_display);
            
            html_string
        }
    )
} 