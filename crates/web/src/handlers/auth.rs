use crate::components::auth::{LoginForm, RegistrationForm};
use crate::pages::login::LoginQuery;
use crate::pages::login::render_login_page;
use crate::services::auth::{AuthError, AuthService, CreateUser, LoginCredentials};
use crate::services::leptos::LeptosOptions;
use axum::{
    extract::{Form, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Redirect},
};

/// AppState Struktur, die die gemeinsam genutzten Anwendungszustände enthält
#[derive(Clone)]
pub struct AppState {
    pub auth_service: AuthService,
    pub leptos_options: LeptosOptions,
}

/// Handler für die Anzeige der Login-Seite
pub async fn login_page_handler(
    State(state): State<AppState>,
    Query(query): Query<LoginQuery>,
) -> impl IntoResponse {
    let html = render_login_page(&state.leptos_options, query.error, query.redirect);

    (StatusCode::OK, [(header::CONTENT_TYPE, "text/html")], html)
}

/// Handler für die Verarbeitung des Login-Formulars
pub async fn handle_login(
    State(state): State<AppState>,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    let credentials = LoginCredentials {
        email: form.email,
        password: form.password,
    };

    match state.auth_service.login(&credentials).await {
        Ok(session) => {
            // Erfolgreicher Login, Cookie setzen und zur Startseite weiterleiten
            let cookie = format!(
                "auth_token={}; HttpOnly; Path=/; Max-Age=86400",
                session.token
            );

            let mut response = Redirect::to("/").into_response();
            response.headers_mut().insert(
                header::SET_COOKIE,
                header::HeaderValue::from_str(&cookie).unwrap(),
            );
            response
        },
        Err(e) => {
            // Fehler bei der Anmeldung, zurück zur Login-Seite mit Fehlermeldung
            let error_message = match e {
                AuthError::InvalidCredentials => "Ungültige Anmeldedaten".to_string(),
                AuthError::AccountLocked => "Konto gesperrt".to_string(),
                _ => "Ein Fehler ist aufgetreten".to_string(),
            };

            Redirect::to(&format!("/login?error={}", error_message)).into_response()
        },
    }
}

/// Handler für die Verarbeitung des Registrierungsformulars
pub async fn handle_registration(
    State(state): State<AppState>,
    Form(form): Form<RegistrationForm>,
) -> impl IntoResponse {
    // Überprüfe, ob die Passwörter übereinstimmen
    if form.password != form.password_confirmation {
        return Redirect::to("/register?error=Passwörter+stimmen+nicht+überein").into_response();
    }

    // Erstelle den neuen Benutzer
    let create_user = CreateUser {
        email: form.email,
        password: form.password,
    };

    // Registriere den Benutzer mit dem Auth-Service
    match state.auth_service.register(&create_user).await {
        Ok(_) => {
            // Erfolgreiche Registrierung, leite zur Login-Seite weiter
            Redirect::to("/login?message=Registrierung+erfolgreich").into_response()
        },
        Err(e) => {
            // Fehler bei der Registrierung, zurück zur Registrierungsseite mit Fehlermeldung
            let error_message = match e {
                AuthError::UserAlreadyExists => "E-Mail-Adresse wird bereits verwendet".to_string(),
                AuthError::ValidationError(_) => "Ungültige Eingaben".to_string(),
                _ => "Ein Fehler ist aufgetreten".to_string(),
            };

            Redirect::to(&format!("/register?error={}", error_message)).into_response()
        },
    }
}

/// Handler für die Verarbeitung des Logout
pub async fn handle_logout() -> impl IntoResponse {
    // Lösche das Session-Cookie und leite zur Login-Seite weiter
    let cookie = "auth_token=; HttpOnly; Path=/; Max-Age=0";

    let mut response = Redirect::to("/login").into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        header::HeaderValue::from_str(cookie).unwrap(),
    );
    response
}
