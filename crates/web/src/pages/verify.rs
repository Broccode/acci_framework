use crate::services::leptos::LeptosOptions;
use crate::services::leptos::ssr;
use serde::Deserialize;

/// Struktur für Query-Parameter der Verifizierungsseite
#[derive(Deserialize)]
pub struct VerifyQuery {
    pub user_id: Option<String>,
    pub verification_type: Option<String>,
    pub tenant_id: Option<String>,
    pub session_token: Option<String>,
    pub error: Option<String>,
    pub message: Option<String>,
}

/// Struktur für Query-Parameter der Seite zum Senden von Verifikationscodes
#[derive(Deserialize)]
pub struct SendVerifyQuery {
    pub user_id: Option<String>,
    pub verification_type: Option<String>,
    pub tenant_id: Option<String>,
    pub session_token: Option<String>,
    pub error: Option<String>,
    pub message: Option<String>,
}

/// Rendert die Verifizierungsseite als SSR
///
/// Diese Funktion rendert die vollständige HTML-Seite für die Verifizierung,
/// einschließlich Header, Navigation, Formular und Footer.
pub fn render_verify_page(
    renderer: &LeptosOptions,
    user_id: Option<String>,
    verification_type: Option<String>,
    tenant_id: Option<String>,
    session_token: Option<String>,
    error: Option<String>,
    message: Option<String>,
) -> String {
    // Parameter mit Standardwerten versehen
    let user_id = user_id.unwrap_or_default();
    let verification_type = verification_type.unwrap_or_else(|| "email".to_string());
    let tenant_id = tenant_id.unwrap_or_default();

    // Vorbereitung der Anzeigetexte
    let verification_type_display = match verification_type.to_lowercase().as_str() {
        "email" => "E-Mail",
        "sms" => "SMS",
        _ => "Verifizierung",
    };

    let title = format!("{} verifizieren", verification_type_display);

    // Erstelle den Info- oder Error-Display-String
    let message_display = if let Some(msg) = message {
        format!(r#"<div class="info-message">{}</div>"#, msg)
    } else if let Some(err_msg) = error {
        format!(r#"<div class="error-message">{}</div>"#, err_msg)
    } else {
        "".to_string()
    };

    // Die gesamte Seite wird serverseitig gerendert
    ssr::render_to_string_with_context(renderer, move |_cx| {
        // Erstelle den HTML-String
        let html_string = format!(
            r#"
            <html>
                <head>
                    <title>{title} - ACCI Framework</title>
                    <meta charset="UTF-8"/>
                    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
                    <link rel="stylesheet" href="/static/styles/main.css"/>
                </head>
                <body>
                    <main class="container">
                        <h1>{title}</h1>
                        <p class="page-description">
                            Bitte geben Sie den Code ein, den wir Ihnen per {verification_type_display} zugesendet haben.
                        </p>
                        {message_display}
                        <form method="post" action="/api/auth/verify/code" class="auth-form verification-form">
                            <input type="hidden" name="user_id" value="{user_id}" />
                            <input type="hidden" name="verification_type" value="{verification_type}" />
                            <input type="hidden" name="tenant_id" value="{tenant_id}" />
                            {session_token_field}
                            
                            <div class="form-group">
                                <label for="code">Verifikationscode</label>
                                <input 
                                    type="text" 
                                    id="code" 
                                    name="code" 
                                    placeholder="123456"
                                    autocomplete="one-time-code"
                                    inputmode="numeric"
                                    pattern="[0-9]*"
                                    minlength="6"
                                    maxlength="6"
                                    required
                                />
                            </div>
                            
                            <div class="form-actions">
                                <button type="submit" class="btn btn-primary">Bestätigen</button>
                            </div>
                            
                            <div class="verification-info">
                                <p>Haben Sie keinen Code erhalten?</p>
                                <a href="/verify/send?verification_type={verification_type}&user_id={user_id}&tenant_id={tenant_id}{session_token_param}" class="resend-link">
                                    Code erneut senden
                                </a>
                            </div>
                        </form>
                    </main>
                    <script src="/static/js/validation.js"></script>
                </body>
            </html>
            "#,
            title = title,
            verification_type_display = verification_type_display,
            verification_type = verification_type,
            user_id = user_id,
            tenant_id = tenant_id,
            message_display = message_display,
            session_token_field = session_token.as_ref().map_or("".to_string(), |token| {
                format!(
                    r#"<input type="hidden" name="session_token" value="{}" />"#,
                    token
                )
            }),
            session_token_param = session_token.as_ref().map_or("".to_string(), |token| {
                format!("&session_token={}", token)
            }),
        );

        html_string
    })
}

/// Rendert die Seite zum Senden eines Verifikationscodes als SSR
pub fn render_send_verify_page(
    renderer: &LeptosOptions,
    user_id: Option<String>,
    verification_type: Option<String>,
    tenant_id: Option<String>,
    session_token: Option<String>,
    error: Option<String>,
    message: Option<String>,
) -> String {
    // Parameter mit Standardwerten versehen
    let user_id = user_id.unwrap_or_default();
    let verification_type = verification_type.unwrap_or_else(|| "email".to_string());
    let tenant_id = tenant_id.unwrap_or_default();

    // Vorbereitung der Anzeigetexte
    let verification_type_display = match verification_type.to_lowercase().as_str() {
        "email" => "E-Mail",
        "sms" => "SMS",
        _ => "Verifizierung",
    };

    let title = format!("{}-Code senden", verification_type_display);

    // Feldbezeichnungen basierend auf dem Verifikationstyp
    let (recipient_label, recipient_type, input_mode) =
        match verification_type.to_lowercase().as_str() {
            "sms" => ("Telefonnummer", "tel", "tel"),
            _ => ("E-Mail-Adresse", "email", "email"),
        };

    // Erstelle den Info- oder Error-Display-String
    let message_display = if let Some(msg) = message {
        format!(r#"<div class="info-message">{}</div>"#, msg)
    } else if let Some(err_msg) = error {
        format!(r#"<div class="error-message">{}</div>"#, err_msg)
    } else {
        "".to_string()
    };

    // Die gesamte Seite wird serverseitig gerendert
    ssr::render_to_string_with_context(renderer, move |_cx| {
        // Erstelle den HTML-String
        let html_string = format!(
            r#"
            <html>
                <head>
                    <title>{title} - ACCI Framework</title>
                    <meta charset="UTF-8"/>
                    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
                    <link rel="stylesheet" href="/static/styles/main.css"/>
                </head>
                <body>
                    <main class="container">
                        <h1>{title}</h1>
                        <p class="page-description">
                            Bitte geben Sie Ihre {recipient_label} ein, um einen Verifizierungscode zu erhalten.
                        </p>
                        {message_display}
                        <form method="post" action="/api/auth/verify/send" class="auth-form send-verification-form">
                            <input type="hidden" name="user_id" value="{user_id}" />
                            <input type="hidden" name="verification_type" value="{verification_type}" />
                            <input type="hidden" name="tenant_id" value="{tenant_id}" />
                            {session_token_field}
                            
                            <div class="form-group">
                                <label for="recipient">{recipient_label}</label>
                                <input 
                                    type="{recipient_type}" 
                                    id="recipient" 
                                    name="recipient" 
                                    inputmode="{input_mode}"
                                    required
                                />
                            </div>
                            
                            <div class="form-actions">
                                <button type="submit" class="btn btn-primary">Code senden</button>
                            </div>
                        </form>
                        
                        <div class="form-footer">
                            <a href="/login" class="back-link">Zurück zum Login</a>
                        </div>
                    </main>
                    <script src="/static/js/validation.js"></script>
                </body>
            </html>
            "#,
            title = title,
            recipient_label = recipient_label,
            recipient_type = recipient_type,
            input_mode = input_mode,
            verification_type = verification_type,
            user_id = user_id,
            tenant_id = tenant_id,
            message_display = message_display,
            session_token_field = session_token.as_ref().map_or("".to_string(), |token| {
                format!(
                    r#"<input type="hidden" name="session_token" value="{}" />"#,
                    token
                )
            }),
        );

        html_string
    })
}
