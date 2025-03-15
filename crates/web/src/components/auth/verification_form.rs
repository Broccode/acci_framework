use crate::prelude::*;
use crate::view;
use serde::{Deserialize, Serialize};

/// VerificationForm-Struktur für die Datenverarbeitung
#[derive(Serialize, Deserialize, Clone)]
pub struct VerificationForm {
    pub user_id: String,
    pub verification_type: String,
    pub code: String,
    pub tenant_id: String,
    pub session_token: Option<String>,
    pub error: Option<String>,
}

/// Server-side rendered Verification-Formular Komponente
///
/// Diese Komponente stellt ein HTML-Formular für die Verifikation bereit und
/// wird ausschließlich auf dem Server gerendert (SSR).
///
/// # Parameter
///
/// * `cx` - Der Leptos-Scope
/// * `action_path` - Der Pfad, an den das Formular gesendet wird
/// * `verification_type` - Art der Verifikation (email oder sms)
/// * `user_id` - Die Benutzer-ID für die Verifikation
/// * `tenant_id` - Die Mandanten-ID für die Verifikation
/// * `session_token` - Das Session-Token für Authentifizierung (optional)
/// * `error` - Eine optionale Fehlermeldung, die angezeigt werden soll
#[allow(unused_variables)]
pub fn verification_form_ssr(
    cx: Scope,
    _action_path: String,
    verification_type: String,
    _user_id: String,
    _tenant_id: String,
    _session_token: Option<String>,
    _error: Option<String>,
) -> impl IntoView {
    // Anzeigename für den Verifikationstyp
    let _verification_type_display = match verification_type.to_lowercase().as_str() {
        "email" => "E-Mail",
        "sms" => "SMS",
        _ => "Verifikation",
    };

    view! { cx,
        <form method="post" action={_action_path} class="auth-form verification-form">
            <input type="hidden" name="user_id" value={_user_id} />
            <input type="hidden" name="verification_type" value={verification_type.clone()} />
            <input type="hidden" name="tenant_id" value={_tenant_id} />

            {
                _session_token.clone().map(|token| {
                    view! { cx,
                        <input type="hidden" name="session_token" value={token} />
                    }
                })
            }

            <div class="verification-info">
                <p>Bitte geben Sie den Code ein, den wir Ihnen per {_verification_type_display} zugesendet haben.</p>
            </div>

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

            {
                match _error {
                    Some(err) => view! { cx, <div class="error-message">{err}</div> },
                    None => view! { cx, <> </> }
                }
            }

            <div class="form-actions">
                <button type="submit" class="btn btn-primary">Bestätigen</button>
            </div>

            <div class="form-links">
                <a href="#" class="resend-link" data-verification-type={verification_type}>Code erneut senden</a>
            </div>
        </form>
    }
}

/// Send Verification Request DTO
#[derive(Serialize, Deserialize, Clone)]
pub struct SendVerificationRequest {
    pub user_id: String,
    pub verification_type: String,
    pub recipient: String,
    pub tenant_id: String,
    pub session_token: Option<String>,
    pub error: Option<String>,
}

/// Server-side rendered Formular zum Senden eines Verifikationscodes
///
/// Diese Komponente stellt ein HTML-Formular zum Senden eines Verifikationscodes bereit
/// und wird ausschließlich auf dem Server gerendert (SSR).
///
/// # Parameter
///
/// * `cx` - Der Leptos-Scope
/// * `action_path` - Der Pfad, an den das Formular gesendet wird
/// * `verification_type` - Art der Verifikation (email oder sms)
/// * `user_id` - Die Benutzer-ID für die Verifikation
/// * `tenant_id` - Die Mandanten-ID für die Verifikation
/// * `session_token` - Das Session-Token für Authentifizierung (optional)
/// * `error` - Eine optionale Fehlermeldung, die angezeigt werden soll
#[allow(unused_variables)]
pub fn send_verification_form_ssr(
    cx: Scope,
    _action_path: String,
    verification_type: String,
    _user_id: String,
    _tenant_id: String,
    _session_token: Option<String>,
    _error: Option<String>,
) -> impl IntoView {
    // Feldbezeichnungen basierend auf dem Verifikationstyp
    let (_recipient_label, _recipient_type, _input_mode) =
        match verification_type.to_lowercase().as_str() {
            "sms" => ("Telefonnummer", "tel", "tel"),
            _ => ("E-Mail-Adresse", "email", "email"),
        };

    view! { cx,
        <form method="post" action={_action_path} class="auth-form send-verification-form">
            <input type="hidden" name="user_id" value={_user_id} />
            <input type="hidden" name="verification_type" value={verification_type} />
            <input type="hidden" name="tenant_id" value={_tenant_id} />

            {
                _session_token.clone().map(|token| {
                    view! { cx,
                        <input type="hidden" name="session_token" value={token} />
                    }
                })
            }

            <div class="form-group">
                <label for="recipient">{_recipient_label}</label>
                <input
                    type={_recipient_type}
                    id="recipient"
                    name="recipient"
                    inputmode={_input_mode}
                    required
                />
            </div>

            {
                match _error {
                    Some(err) => view! { cx, <div class="error-message">{err}</div> },
                    None => view! { cx, <> </> }
                }
            }

            <div class="form-actions">
                <button type="submit" class="btn btn-primary">Code senden</button>
            </div>
        </form>
    }
}

// Damit die Komponente in Tests verwendet werden kann
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::leptos::ssr::test_utils;

    #[test]
    fn test_verification_form_renders_with_correct_structure() {
        // Given a verification form with specific parameters
        let action_path = "/api/auth/verify/code".to_string();
        let verification_type = "email".to_string();
        let user_id = "user123".to_string();
        let tenant_id = "tenant456".to_string();
        let session_token = Some("sessionabc".to_string());

        // When rendering the form without an error
        let html = test_utils::render_to_html(|cx| {
            verification_form_ssr(
                cx,
                action_path.clone(),
                verification_type.clone(),
                user_id.clone(),
                tenant_id.clone(),
                session_token.clone(),
                None,
            )
        });

        // Then it should contain the proper form elements
        assert!(test_utils::assert_has_class(&html, "auth-form"));
        assert!(test_utils::assert_has_class(&html, "verification-form"));
        assert!(test_utils::assert_contains_text(&html, "Verifikationscode"));
        assert!(test_utils::assert_contains_text(&html, "E-Mail"));
        assert!(test_utils::assert_contains_text(&html, "Bestätigen"));
        assert!(test_utils::assert_contains_text(
            &html,
            "Code erneut senden"
        ));

        // Check hidden fields
        assert!(test_utils::assert_contains_text(
            &html,
            &format!("value=\"{}\"", user_id)
        ));
        assert!(test_utils::assert_contains_text(
            &html,
            &format!("value=\"{}\"", tenant_id)
        ));
        assert!(test_utils::assert_contains_text(
            &html,
            &format!("value=\"{}\"", verification_type)
        ));
        assert!(test_utils::assert_contains_text(
            &html,
            &format!("value=\"{}\"", session_token.unwrap())
        ));
    }

    #[test]
    fn test_verification_form_displays_error_when_provided() {
        // Given a verification form with an error message
        let action_path = "/api/auth/verify/code".to_string();
        let verification_type = "email".to_string();
        let user_id = "user123".to_string();
        let tenant_id = "tenant456".to_string();
        let session_token = Some("sessionabc".to_string());
        let error_message = "Ungültiger Verifikationscode".to_string();

        // When rendering the form with the error
        let html = test_utils::render_to_html(|cx| {
            verification_form_ssr(
                cx,
                action_path.clone(),
                verification_type.clone(),
                user_id.clone(),
                tenant_id.clone(),
                session_token.clone(),
                Some(error_message.clone()),
            )
        });

        // Then it should display the error message
        assert!(test_utils::assert_has_class(&html, "error-message"));
        assert!(test_utils::assert_contains_text(&html, &error_message));
    }

    #[test]
    fn test_send_verification_form_renders_with_email_type() {
        // Given a send verification form for email
        let action_path = "/api/auth/verify/send".to_string();
        let verification_type = "email".to_string();
        let user_id = "user123".to_string();
        let tenant_id = "tenant456".to_string();
        let session_token = Some("sessionabc".to_string());

        // When rendering the form
        let html = test_utils::render_to_html(|cx| {
            send_verification_form_ssr(
                cx,
                action_path.clone(),
                verification_type.clone(),
                user_id.clone(),
                tenant_id.clone(),
                session_token.clone(),
                None,
            )
        });

        // Then it should contain email-specific elements
        assert!(test_utils::assert_has_class(
            &html,
            "send-verification-form"
        ));
        assert!(test_utils::assert_contains_text(&html, "E-Mail-Adresse"));
        assert!(test_utils::assert_contains_text(&html, "type=\"email\""));
    }

    #[test]
    fn test_send_verification_form_renders_with_sms_type() {
        // Given a send verification form for SMS
        let action_path = "/api/auth/verify/send".to_string();
        let verification_type = "sms".to_string();
        let user_id = "user123".to_string();
        let tenant_id = "tenant456".to_string();
        let session_token = Some("sessionabc".to_string());

        // When rendering the form
        let html = test_utils::render_to_html(|cx| {
            send_verification_form_ssr(
                cx,
                action_path.clone(),
                verification_type.clone(),
                user_id.clone(),
                tenant_id.clone(),
                session_token.clone(),
                None,
            )
        });

        // Then it should contain SMS-specific elements
        assert!(test_utils::assert_has_class(
            &html,
            "send-verification-form"
        ));
        assert!(test_utils::assert_contains_text(&html, "Telefonnummer"));
        assert!(test_utils::assert_contains_text(&html, "type=\"tel\""));
    }
}
