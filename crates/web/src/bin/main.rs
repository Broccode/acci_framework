use acci_web::routes::create_router;
use acci_web::handlers::AppState;
use acci_web::services::auth::AuthService;
use acci_web::services::leptos::LeptosOptions;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Initialisiere Logger
    println!("ACCI Web Server wird gestartet...");
    
    // Stelle sicher, dass der statische Ordner existiert
    std::fs::create_dir_all("static").unwrap_or_else(|e| {
        eprintln!("Fehler beim Erstellen des static-Verzeichnisses: {}", e);
    });
    
    // Initialisiere den Auth-Service und die Leptos-Options
    let app_state = AppState {
        auth_service: AuthService::new(),
        leptos_options: LeptosOptions::new(),
    };
    
    // Erstelle den Router mit den definierten Routes
    let app = create_router(app_state);
    
    // Binde den Server an die Adresse
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server läuft auf http://{}", addr);
    
    // Starte den Server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
} 