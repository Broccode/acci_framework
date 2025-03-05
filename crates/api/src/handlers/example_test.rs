#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
    };
    use serde_json::{Value, json};
    use std::sync::Arc;
    use tower::ServiceExt;

    use crate::handlers::example::{ProductAppState, ProductService, create_product, get_product};
    use crate::middleware::error_handling::error_handling_middleware;

    // Hilfsfunktion zum Erstellen eines Test-Routers
    fn create_test_router() -> Router {
        let product_service = Arc::new(ProductService::new());
        let app_state = ProductAppState { product_service };

        Router::new()
            .route("/products/:id", axum::routing::get(get_product))
            .route("/products", axum::routing::post(create_product))
            .with_state(app_state)
            .layer(axum::middleware::from_fn(error_handling_middleware))
    }

    #[tokio::test]
    async fn test_get_product_success() {
        // Router erstellen
        let app = create_test_router();

        // Anfrage erstellen
        let request = Request::builder()
            .uri("/products/123")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        // Anfrage ausführen
        let response = app.oneshot(request).await.unwrap();

        // Statuscode prüfen
        assert_eq!(response.status(), StatusCode::OK);

        // Antwort-Body extrahieren und prüfen
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();

        // Prüfen, ob die Antwort die erwartete Struktur hat
        assert!(body.get("success").unwrap().as_bool().unwrap());
        assert!(body.get("request_id").is_some());

        let data = body.get("data").unwrap();
        assert_eq!(data.get("id").unwrap().as_str().unwrap(), "123");
        assert_eq!(
            data.get("name").unwrap().as_str().unwrap(),
            "Beispielprodukt"
        );
    }

    #[tokio::test]
    async fn test_get_product_not_found() {
        // Router erstellen
        let app = create_test_router();

        // Anfrage erstellen
        let request = Request::builder()
            .uri("/products/not-found")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        // Anfrage ausführen
        let response = app.oneshot(request).await.unwrap();

        // Statuscode prüfen
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Antwort-Body extrahieren und prüfen
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();

        // Prüfen, ob die Antwort die erwartete Struktur hat
        assert!(!body.get("success").unwrap().as_bool().unwrap());
        assert!(body.get("request_id").is_some());
        assert_eq!(
            body.get("error")
                .unwrap()
                .get("code")
                .unwrap()
                .as_str()
                .unwrap(),
            "NOT_FOUND"
        );
    }

    #[tokio::test]
    async fn test_get_product_server_error() {
        // Router erstellen
        let app = create_test_router();

        // Anfrage erstellen
        let request = Request::builder()
            .uri("/products/error")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        // Anfrage ausführen
        let response = app.oneshot(request).await.unwrap();

        // Statuscode prüfen
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        // Antwort-Body extrahieren und prüfen
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();

        // Prüfen, ob die Antwort die erwartete Struktur hat
        assert!(!body.get("success").unwrap().as_bool().unwrap());
        assert!(body.get("request_id").is_some());
        assert_eq!(
            body.get("error")
                .unwrap()
                .get("code")
                .unwrap()
                .as_str()
                .unwrap(),
            "DATABASE_ERROR"
        );
    }

    #[tokio::test]
    async fn test_create_product_success() {
        // Router erstellen
        let app = create_test_router();

        // Anfrage-Body erstellen
        let json_body = json!({
            "name": "Testprodukt",
            "description": "Ein Testprodukt",
            "price": 29.99,
            "stock": 10
        });

        // Anfrage erstellen
        let request = Request::builder()
            .uri("/products")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&json_body).unwrap()))
            .unwrap();

        // Anfrage ausführen
        let response = app.oneshot(request).await.unwrap();

        // Statuscode prüfen
        assert_eq!(response.status(), StatusCode::CREATED);

        // Antwort-Body extrahieren und prüfen
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();

        // Prüfen, ob die Antwort die erwartete Struktur hat
        assert!(body.get("success").unwrap().as_bool().unwrap());
        assert!(body.get("request_id").is_some());

        let data = body.get("data").unwrap();
        assert!(data.get("id").is_some());
        assert_eq!(data.get("name").unwrap().as_str().unwrap(), "Testprodukt");
        assert_eq!(data.get("price").unwrap().as_f64().unwrap(), 29.99);
    }

    #[tokio::test]
    async fn test_create_product_validation_error() {
        // Router erstellen
        let app = create_test_router();

        // Anfrage-Body mit ungültigen Daten erstellen
        let json_body = json!({
            "name": "Te", // Zu kurz (min = 3)
            "description": "Ein Testprodukt",
            "price": 0.0, // Zu niedrig (min = 0.01)
            "stock": 10
        });

        // Anfrage erstellen
        let request = Request::builder()
            .uri("/products")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&json_body).unwrap()))
            .unwrap();

        // Anfrage ausführen
        let response = app.oneshot(request).await.unwrap();

        // Statuscode prüfen
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Antwort-Body extrahieren und prüfen
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();

        // Prüfen, ob die Antwort die erwartete Struktur hat
        assert!(!body.get("success").unwrap().as_bool().unwrap());
        assert!(body.get("request_id").is_some());
        assert_eq!(
            body.get("error")
                .unwrap()
                .get("code")
                .unwrap()
                .as_str()
                .unwrap(),
            "VALIDATION_ERROR"
        );

        // Prüfen, ob die Validierungsfehler korrekt zurückgegeben werden
        let validation_errors = body
            .get("error")
            .unwrap()
            .get("details")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(validation_errors.len(), 2); // Zwei Fehler: name und price
    }

    #[tokio::test]
    async fn test_create_product_server_error() {
        // Router erstellen
        let app = create_test_router();

        // Anfrage-Body erstellen, der einen Serverfehler auslöst
        let json_body = json!({
            "name": "Fehler", // Löst einen Serverfehler aus
            "description": "Ein Testprodukt",
            "price": 29.99,
            "stock": 10
        });

        // Anfrage erstellen
        let request = Request::builder()
            .uri("/products")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&json_body).unwrap()))
            .unwrap();

        // Anfrage ausführen
        let response = app.oneshot(request).await.unwrap();

        // Statuscode prüfen
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        // Antwort-Body extrahieren und prüfen
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body: Value = serde_json::from_slice(&body).unwrap();

        // Prüfen, ob die Antwort die erwartete Struktur hat
        assert!(!body.get("success").unwrap().as_bool().unwrap());
        assert!(body.get("request_id").is_some());
        assert_eq!(
            body.get("error")
                .unwrap()
                .get("code")
                .unwrap()
                .as_str()
                .unwrap(),
            "PRODUCT_CREATION_ERROR"
        );
    }
}
