use crate::components::auth::{SendVerificationRequest, VerificationForm};
use crate::pages::verify::{SendVerifyQuery, VerifyQuery};
use crate::pages::verify::{render_send_verify_page, render_verify_page};
use crate::services::auth::{AuthError, MfaStatus, VerificationRequest};
use axum::{
    extract::{Form, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Redirect},
};

use super::AppState;

/// Handler für die Anzeige der Verifikationsseite
pub async fn verify_page_handler(
    State(state): State<AppState>,
    Query(query): Query<VerifyQuery>,
) -> impl IntoResponse {
    let html = render_verify_page(
        &state.leptos_options,
        query.user_id,
        query.verification_type,
        query.tenant_id,
        query.session_token,
        query.error,
        query.message,
    );

    (StatusCode::OK, [(header::CONTENT_TYPE, "text/html")], html)
}

/// Handler für die Anzeige der Seite zum Senden eines Verifikationscodes
pub async fn send_verify_page_handler(
    State(state): State<AppState>,
    Query(query): Query<SendVerifyQuery>,
) -> impl IntoResponse {
    let html = render_send_verify_page(
        &state.leptos_options,
        query.user_id,
        query.verification_type,
        query.tenant_id,
        query.session_token,
        query.error,
        query.message,
    );

    (StatusCode::OK, [(header::CONTENT_TYPE, "text/html")], html)
}

/// Handler für die Verarbeitung des Verifikationsformulars
pub async fn handle_verification(
    State(state): State<AppState>,
    Form(form): Form<VerificationForm>,
) -> impl IntoResponse {
    // Erstelle Verifikationsanfrage
    let verification_request = VerificationRequest {
        user_id: form.user_id.clone(),
        verification_type: form.verification_type.clone(),
        tenant_id: form.tenant_id.clone(),
        code: form.code,
        session_token: form.session_token.clone(),
    };

    // Versuche den Code zu verifizieren
    match state.auth_service.verify_code(&verification_request).await {
        Ok(_) => {
            // Wenn ein Session-Token vorhanden ist, aktualisiere den MFA-Status
            if let Some(token) = verification_request.session_token {
                if let Err(e) = state
                    .auth_service
                    .update_session_mfa_status(&token, MfaStatus::Verified)
                    .await
                {
                    // Fehler beim Aktualisieren des MFA-Status
                    return create_error_redirect(
                        &verification_request.user_id,
                        &verification_request.verification_type,
                        &verification_request.tenant_id,
                        form.session_token,
                        &format!("Fehler beim Aktualisieren des MFA-Status: {}", e),
                    );
                }
            }

            // Erfolgreiche Verifikation, leite zur Startseite weiter
            Redirect::to("/").into_response()
        },
        Err(e) => {
            // Fehler bei der Verifikation
            let (error_message, update_status) = match e {
                AuthError::InvalidVerificationCode => {
                    ("Ungültiger Verifikationscode", MfaStatus::Failed)
                },
                AuthError::ExpiredVerificationCode => {
                    ("Verifikationscode ist abgelaufen", MfaStatus::Failed)
                },
                AuthError::TooManyVerificationAttempts => {
                    ("Zu viele Verifikationsversuche", MfaStatus::Failed)
                },
                AuthError::RateLimitExceeded => (
                    "Rate-Limit überschritten. Bitte versuchen Sie es später erneut.",
                    MfaStatus::Pending,
                ),
                _ => ("Ein Fehler ist aufgetreten", MfaStatus::Failed),
            };

            // Wenn ein Session-Token vorhanden ist, aktualisiere den MFA-Status entsprechend
            if let Some(token) = verification_request.session_token.clone() {
                let _ = state
                    .auth_service
                    .update_session_mfa_status(&token, update_status)
                    .await;
            }

            // Leite zurück zur Verifikationsseite mit Fehlermeldung
            create_error_redirect(
                &verification_request.user_id,
                &verification_request.verification_type,
                &verification_request.tenant_id,
                verification_request.session_token,
                error_message,
            )
        },
    }
}

/// Handler für die Verarbeitung des Formulars zum Senden eines Verifikationscodes
pub async fn handle_send_verification(
    State(state): State<AppState>,
    Form(form): Form<SendVerificationRequest>,
) -> impl IntoResponse {
    // Erstelle Anfrage zum Senden eines Verifikationscodes
    let send_request = crate::services::auth::SendVerificationRequest {
        user_id: form.user_id.clone(),
        verification_type: form.verification_type.clone(),
        recipient: form.recipient.clone(),
        tenant_id: form.tenant_id.clone(),
        session_token: form.session_token.clone(),
    };

    // Versuche den Verifikationscode zu senden
    match state.auth_service.send_verification(&send_request).await {
        Ok(_) => {
            // Wenn ein Session-Token vorhanden ist, aktualisiere den MFA-Status auf 'Pending'
            if let Some(token) = send_request.session_token.clone() {
                if let Err(e) = state
                    .auth_service
                    .update_session_mfa_status(&token, MfaStatus::Pending)
                    .await
                {
                    // Fehler beim Aktualisieren des MFA-Status
                    return create_error_redirect_for_send(
                        &send_request.user_id,
                        &send_request.verification_type,
                        &send_request.tenant_id,
                        send_request.session_token,
                        &format!("Fehler beim Aktualisieren des MFA-Status: {}", e),
                    );
                }
            }

            // Erfolgreiche Sendung, leite zur Verifikationsseite weiter mit Erfolgsmeldung
            let redirect_url = format!(
                "/verify?user_id={}&verification_type={}&tenant_id={}{}&message=Verifikationscode+wurde+gesendet",
                send_request.user_id,
                send_request.verification_type,
                send_request.tenant_id,
                send_request
                    .session_token
                    .map(|token| format!("&session_token={}", token))
                    .unwrap_or_default()
            );

            Redirect::to(&redirect_url).into_response()
        },
        Err(e) => {
            // Fehler beim Senden des Verifikationscodes
            let error_message = match e {
                AuthError::RateLimitExceeded => {
                    "Rate-Limit überschritten. Bitte versuchen Sie es später erneut."
                },
                _ => "Fehler beim Senden des Verifikationscodes",
            };

            // Leite zurück zur Seite zum Senden eines Verifikationscodes mit Fehlermeldung
            create_error_redirect_for_send(
                &send_request.user_id,
                &send_request.verification_type,
                &send_request.tenant_id,
                send_request.session_token,
                error_message,
            )
        },
    }
}

/// Erstellt eine Redirect-URL mit Fehlermeldung für die Verifikationsseite
fn create_error_redirect(
    user_id: &str,
    verification_type: &str,
    tenant_id: &str,
    session_token: Option<String>,
    error_message: &str,
) -> axum::http::Response<axum::body::Body> {
    let redirect_url = format!(
        "/verify?user_id={}&verification_type={}&tenant_id={}{}&error={}",
        user_id,
        verification_type,
        tenant_id,
        session_token
            .map(|token| format!("&session_token={}", token))
            .unwrap_or_default(),
        error_message
    );

    Redirect::to(&redirect_url).into_response()
}

/// Erstellt eine Redirect-URL mit Fehlermeldung für die Seite zum Senden eines Verifikationscodes
fn create_error_redirect_for_send(
    user_id: &str,
    verification_type: &str,
    tenant_id: &str,
    session_token: Option<String>,
    error_message: &str,
) -> axum::http::Response<axum::body::Body> {
    let redirect_url = format!(
        "/verify/send?user_id={}&verification_type={}&tenant_id={}{}&error={}",
        user_id,
        verification_type,
        tenant_id,
        session_token
            .map(|token| format!("&session_token={}", token))
            .unwrap_or_default(),
        error_message
    );

    Redirect::to(&redirect_url).into_response()
}
