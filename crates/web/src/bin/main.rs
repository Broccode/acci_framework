use acci_web::handlers::AppState;
use acci_web::routes::create_router;
use acci_web::services::auth::AuthService;
use acci_web::services::leptos::LeptosOptions;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Initialize logger
    println!("Starting ACCI Web Server...");

    // Ensure the static directory exists
    std::fs::create_dir_all("static").unwrap_or_else(|e| {
        eprintln!("Error creating static directory: {}", e);
    });

    // Initialize the Auth Service and Leptos Options
    let app_state = AppState {
        auth_service: AuthService::new(),
        leptos_options: LeptosOptions::new(),
    };

    // Create the router with defined routes
    let app = create_router(app_state);

    // Bind the server to the address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Error binding listener to address");
    axum::serve(listener, app)
        .await
        .expect("Error starting axum server");
}
