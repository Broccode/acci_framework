use crate::prelude::*;
use crate::view;
use serde::Deserialize;

/// Struktur für Query-Parameter der Registrierungsseite
#[derive(Deserialize)]
pub struct RegisterQuery {
    pub error: Option<String>,
    pub message: Option<String>,
}

/// Rendert die gesamte Registrierungsseite als SSR
///
/// Diese Funktion rendert die vollständige HTML-Seite für die Registrierung,
/// einschließlich Header, Navigation, Formular und Footer.
pub fn render_register_page(
    renderer: &LeptosOptions,
    _error: Option<String>,
    _message: Option<String>,
) -> String {
    // Die gesamte Seite wird serverseitig gerendert
    ssr::render_to_string_with_context(renderer, move |cx| {
        view! { cx,
            <html>
                <head>
                    <title>Registrieren - ACCI Framework</title>
                    <meta charset="UTF-8"/>
                    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
                    <link rel="stylesheet" href="/static/styles/main.css"/>
                </head>
                <body>
                    <NavigationSSR is_authenticated=false user_name=None />
                    <main class="container">
                        <h1>Registrieren</h1>
                        <p class="page-description">
                            "Erstellen Sie ein neues Konto für das ACCI Framework."
                        </p>

                        // Nachricht anzeigen, falls vorhanden
                        {_message.map(|msg| view! { cx, <div class="success-message">{msg}</div> })}

                        <RegistrationFormSSR
                            action_path="/api/auth/register".to_string()
                            error={_error}
                        />
                        <div class="form-footer">
                            <p>"Bereits registriert? " <a href="/login">Anmelden</a></p>
                        </div>
                    </main>
                    <FooterSSR />
                    <script src="/static/js/validation.js"></script>
                </body>
            </html>
        }
    })
}
