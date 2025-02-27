use crate::prelude::*;
use crate::view;

/// Rendert die gesamte Home-Seite als SSR
/// 
/// Diese Funktion rendert die vollständige HTML-Seite für die Startseite,
/// einschließlich Header, Navigation, Inhalt und Footer.
pub fn render_home_page(
    renderer: &LeptosOptions,
    _is_authenticated: bool,
    _user_name: Option<String>
) -> String {
    // Die gesamte Seite wird serverseitig gerendert
    ssr::render_to_string_with_context(
        renderer,
        move |cx| {
            view! { cx,
                <html>
                    <head>
                        <title>Willkommen - ACCI Framework</title>
                        <meta charset="UTF-8"/>
                        <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
                        <link rel="stylesheet" href="/static/styles/main.css"/>
                    </head>
                    <body>
                        <crate::components::layout::NavigationSSR 
                            is_authenticated={_is_authenticated} 
                            user_name={_user_name.clone()}
                        />
                        <main class="container">
                            <section class="hero">
                                <h1>Willkommen beim ACCI Framework</h1>
                                <p class="subtitle">
                                    "Ein modernes, sicheres Framework für Ihre Anwendungen"
                                </p>
                            </section>
                            
                            <section class="features">
                                <h2>Funktionen</h2>
                                <div class="feature-grid">
                                    <div class="feature-card">
                                        <h3>Sicherheit</h3>
                                        <p>
                                            "Moderne Authentifizierung und Autorisierung für Ihre Anwendungen"
                                        </p>
                                    </div>
                                    <div class="feature-card">
                                        <h3>Performance</h3>
                                        <p>
                                            "Hochperformante serverseitige Rendering-Implementierung mit Rust"
                                        </p>
                                    </div>
                                    <div class="feature-card">
                                        <h3>Skalierbarkeit</h3>
                                        <p>
                                            "Designt für Skalierbarkeit und Container-Umgebungen"
                                        </p>
                                    </div>
                                </div>
                            </section>
                            
                            {if !_is_authenticated {
                                view! { cx,
                                    <section class="cta">
                                        <h2>Jetzt loslegen</h2>
                                        <p>
                                            "Erstellen Sie ein Konto, um alle Funktionen des ACCI Frameworks zu nutzen."
                                        </p>
                                        <div class="cta-buttons">
                                            <a href="/register" class="btn btn-primary">Registrieren</a>
                                            <a href="/login" class="btn btn-secondary">Anmelden</a>
                                        </div>
                                    </section>
                                }
                            } else {
                                view! { cx,
                                    <section class="welcome-back">
                                        <h2>Willkommen zurück!</h2>
                                        <p>
                                            "Sie sind erfolgreich angemeldet und können alle Funktionen nutzen."
                                        </p>
                                        <a href="/dashboard" class="btn btn-primary">Zum Dashboard</a>
                                    </section>
                                }
                            }}
                        </main>
                        <crate::components::layout::FooterSSR />
                    </body>
                </html>
            }
        }
    )
} 