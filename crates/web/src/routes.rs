use crate::handlers::verification::{
    handle_send_verification, handle_verification, send_verify_page_handler, verify_page_handler,
};
use crate::handlers::{
    AppState, handle_login, handle_logout, handle_registration, login_page_handler,
};
use crate::pages::home::render_home_page;
use crate::pages::register::{RegisterQuery, render_register_page};
use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use tower_http::services::ServeDir;

/// Erstellt den Router für die Anwendung
///
/// Diese Funktion definiert alle Routen für die Anwendung, einschließlich:
/// - Seitenrouten für das Server-Side Rendering
/// - API-Endpunkte für Formularübermittlungen
/// - Statische Dateien
pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        // Seitenrouten - mit Leptos serverseitig gerendert
        .route("/", get(home_page_handler))
        .route("/login", get(login_page_handler))
        .route("/register", get(register_page_handler))
        .route("/verify", get(verify_page_handler))
        .route("/verify/send", get(send_verify_page_handler))

        // API-Endpunkte - für Formularübermittlungen
        .route("/api/auth/login", post(handle_login))
        .route("/api/auth/register", post(handle_registration))
        .route("/api/auth/logout", post(handle_logout))
        .route("/api/auth/verify/code", post(handle_verification))
        .route("/api/auth/verify/send", post(handle_send_verification))

        // Statische Dateien
        .nest_service("/static", ServeDir::new("static"))
        .with_state(app_state)
}

/// Handler für die Home-Seite
async fn home_page_handler(State(state): State<AppState>) -> impl IntoResponse {
    // In einer echten Anwendung würden wir hier die Authentifizierung prüfen
    // und entsprechende Benutzerinformationen einfügen
    let is_authenticated = false;
    let user_name = None;

    let html = render_home_page(&state.leptos_options, is_authenticated, user_name);

    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/html")],
        html,
    )
}

/// Handler für die Registrierungsseite
async fn register_page_handler(
    State(state): State<AppState>,
    Query(query): Query<RegisterQuery>,
) -> impl IntoResponse {
    let html = render_register_page(&state.leptos_options, query.error, query.message);

    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "text/html")],
        html,
    )
}
