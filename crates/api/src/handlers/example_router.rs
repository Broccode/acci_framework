use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;

use crate::handlers::example::{
    ProductAppState, ProductService, create_product, get_product, search_products,
};
use crate::middleware::error_handling::error_handling_middleware;

/// Beispiel für die Konfiguration eines Produkt-Routers mit Fehlerbehandlung
pub fn product_routes() -> Router {
    // Produkt-Service initialisieren
    let product_service = Arc::new(ProductService::new());

    // App-State erstellen
    let app_state = ProductAppState { product_service };

    // Router mit Fehlerbehandlung konfigurieren
    Router::new()
        .route("/products", get(search_products))
        .route("/products", post(create_product))
        .route("/products/:id", get(get_product))
        .with_state(app_state)
        .layer(axum::middleware::from_fn(error_handling_middleware))
}

/// Beispiel für die Integration in eine Haupt-Router-Konfiguration
pub fn configure_example_routes(router: Router) -> Router {
    router.merge(product_routes())
}
