//! API integration tests
//!
//! This module contains tests for the API endpoints.

use crate::response::{ApiError, ApiResponse};
use crate::validation::{generate_request_id, validate_json_payload};
use axum::{
    Json, Router,
    body::Body,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use http::{Method, Request, StatusCode, header};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower::ServiceExt;
use tracing::{debug, info, warn};
use uuid::Uuid;
use validator::Validate;

/// Example of a product handler with error handling
/// and tests
/// Product data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub stock: i32,
}

/// Produkt-Service (Beispiel)
#[derive(Clone)]
pub struct ProductService {
    // In a real application, there would be a repository or database connection here
}

impl Default for ProductService {
    fn default() -> Self {
        Self::new()
    }
}

impl ProductService {
    pub fn new() -> Self {
        Self {}
    }

    /// Find product by ID
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Product>, String> {
        // Simuliere Datenbankabfrage
        if id == "not-found" {
            return Ok(None);
        }

        if id == "error" {
            return Err("Database error".to_string());
        }

        Ok(Some(Product {
            id: id.to_string(),
            name: "Example product".to_string(),
            description: Some("An example product for the API".to_string()),
            price: 19.99,
            stock: 100,
        }))
    }

    /// Create product
    pub async fn create(&self, product: CreateProductRequest) -> Result<Product, String> {
        // Simuliere Produkterstellung
        if product.name.to_lowercase() == "error" {
            return Err("Product could not be created".to_string());
        }

        Ok(Product {
            id: Uuid::new_v4().to_string(),
            name: product.name,
            description: product.description,
            price: product.price,
            stock: product.stock.unwrap_or(0),
        })
    }
}

/// Product app state
#[derive(Clone)]
pub struct ProductAppState {
    pub product_service: Arc<ProductService>,
}

/// Request to create a product
#[derive(Debug, Deserialize, Validate)]
pub struct CreateProductRequest {
    #[validate(length(
        min = 3,
        max = 100,
        message = "Product name must be between 3 and 100 characters long"
    ))]
    pub name: String,

    pub description: Option<String>,

    #[validate(range(min = 0.01, message = "Price must be greater than 0"))]
    pub price: f64,

    pub stock: Option<i32>,
}

/// Handler for retrieving a product by ID
#[axum::debug_handler]
pub async fn get_product(State(state): State<ProductAppState>, Path(id): Path<String>) -> Response {
    debug!("Retrieving product with ID {}", id);

    // Generate request ID
    let request_id = generate_request_id();

    // Retrieve product
    match state.product_service.find_by_id(&id).await {
        Ok(Some(product)) => {
            info!(
                request_id = %request_id,
                product_id = %product.id,
                "Product successfully retrieved"
            );

            let api_response = ApiResponse::success(product, request_id);
            (StatusCode::OK, Json(api_response)).into_response()
        },
        Ok(None) => {
            warn!(
                request_id = %request_id,
                product_id = %id,
                "Product not found"
            );

            ApiError::not_found_error("Product", request_id).into_response()
        },
        Err(err) => {
            warn!(
                request_id = %request_id,
                error = %err,
                product_id = %id,
                "Error retrieving product"
            );

            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error retrieving product",
                "DATABASE_ERROR",
                request_id,
            )
            .into_response()
        },
    }
}

/// Handler for creating a new product
#[axum::debug_handler]
pub async fn create_product(
    State(state): State<ProductAppState>,
    Json(product_data): Json<CreateProductRequest>,
) -> Response {
    debug!("Processing create product request");

    // Generate a unique request ID
    let request_id = generate_request_id();

    // Validate the request
    let product_data = match validate_json_payload(Json(product_data)).await {
        Ok(data) => data,
        Err(validation_error) => {
            return validation_error.into_response();
        },
    };

    // Create the product
    match state.product_service.create(product_data).await {
        Ok(product) => {
            info!(
                request_id = %request_id,
                product_id = %product.id,
                "Product created successfully"
            );

            let api_response = ApiResponse::success(product, request_id);
            (StatusCode::CREATED, Json(api_response)).into_response()
        },
        Err(err) => {
            warn!(
                request_id = %request_id,
                error = %err,
                "Failed to create product"
            );

            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create product",
                "PRODUCT_CREATION_ERROR",
                request_id,
            )
            .into_response()
        },
    }
}

/// Search parameters for products
#[derive(Debug, Deserialize, Validate)]
pub struct ProductSearchParams {
    #[validate(length(min = 3, message = "Search term must be at least 3 characters long"))]
    pub query: Option<String>,

    #[validate(range(
        min = 0.0,
        max = 1000.0,
        message = "Price range must be between 0 and 1000"
    ))]
    pub min_price: Option<f64>,

    #[validate(range(
        min = 0.0,
        max = 10000.0,
        message = "Price range must be between 0 and 10000"
    ))]
    pub max_price: Option<f64>,
}

/// Handler for searching products
#[axum::debug_handler]
pub async fn search_products(
    State(_state): State<ProductAppState>,
    query_params: Query<ProductSearchParams>,
) -> Response {
    debug!("Performing product search");

    // Generate request ID
    let request_id = generate_request_id();

    // Validate search parameters
    if let Err(validation_errors) = query_params.validate() {
        warn!(
            request_id = %request_id,
            validation_errors = %validation_errors,
            "Invalid search parameters"
        );

        // Convert validation errors to a structured response
        let error_response = ApiError::validation_error(validation_errors.to_string(), request_id);

        return error_response.into_response();
    }

    // Example response (in a real application, there would be a database query here)
    let products = vec![
        Product {
            id: Uuid::new_v4().to_string(),
            name: "Example product 1".to_string(),
            description: Some("An example product for the API".to_string()),
            price: 19.99,
            stock: 100,
        },
        Product {
            id: Uuid::new_v4().to_string(),
            name: "Example product 2".to_string(),
            description: Some("Another example product".to_string()),
            price: 29.99,
            stock: 50,
        },
    ];

    info!(
        request_id = %request_id,
        count = products.len(),
        "Products successfully searched"
    );

    let api_response = ApiResponse::success(products, request_id);
    (StatusCode::OK, Json(api_response)).into_response()
}

pub async fn make_json_request<T: Serialize + Send>(
    app: Router<ProductAppState>,
    method: &str,
    uri: &str,
    json: Option<T>,
) -> Response {
    let method = Method::from_bytes(method.as_bytes()).expect("Invalid HTTP method string");
    let mut req = Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .expect("Failed to build HTTP request");

    if let Some(json) = json {
        *req.body_mut() =
            Body::from(serde_json::to_vec(&json).expect("Failed to serialize JSON to bytes"));
        req.headers_mut().insert(
            header::CONTENT_TYPE,
            "application/json"
                .parse()
                .expect("Failed to parse content-type header"),
        );
    }

    let app = app.with_state(ProductAppState {
        product_service: Arc::new(ProductService::new()),
    });

    app.into_service()
        .oneshot(req)
        .await
        .expect("Failed to process HTTP request")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::error_handling::error_handling_middleware;
    use axum::{
        Router,
        body::Body,
        http::StatusCode,
        routing::{get, post},
    };
    use http_body_util::BodyExt;
    use serde_json::{Value, json};
    use tower::ServiceExt;

    // Helper function to setup test app with error_handling_middleware
    fn setup_test_app() -> Router<ProductAppState> {
        let app_state = ProductAppState {
            product_service: Arc::new(ProductService::new()),
        };

        Router::new()
            .route("/products", post(create_product))
            .route("/products", get(search_products))
            .route("/products/{id}", get(get_product))
            .with_state(app_state)
            .layer(axum::middleware::from_fn(error_handling_middleware))
    }

    // Test creating a valid product
    #[tokio::test]
    async fn test_create_valid_product() {
        let app = setup_test_app();

        let product_data = json!({
            "name": "Test Product",
            "description": "A valid product description",
            "price": 19.99,
            "stock": 10
        });

        let response = make_json_request(app, "POST", "/products", Some(product_data)).await;

        // Assert success
        assert_eq!(response.status(), StatusCode::CREATED);

        // Get response body
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8_lossy(&body);

        // Verify response body
        let response: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(response["status"], "success");
        assert!(response["data"]["id"].is_string());
        assert_eq!(response["data"]["name"], "Test Product");
    }

    // Test creating an invalid product (missing required fields)
    #[tokio::test]
    async fn test_create_product_missing_fields() {
        let app = setup_test_app();

        let product_data = json!({
            "name": "Test Product",
            // Missing price field which is required
        });

        let response = make_json_request(app, "POST", "/products", Some(product_data)).await;

        // Assert validation error
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        // Get response body
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8_lossy(&body);

        // Verify error response
        let error: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(error["status"], "error");
        assert_eq!(error["code"], "VALIDATION_ERROR");
        assert!(error["message"].is_string());
        assert!(error["request_id"].is_string());
    }

    // Test creating a product with invalid data (name too short)
    #[tokio::test]
    async fn test_create_product_invalid_name() {
        let app = setup_test_app();

        let product_data = json!({
            "name": "T", // Too short
            "description": "A valid product description",
            "price": 19.99,
            "stock": 10
        });

        let response = make_json_request(app, "POST", "/products", Some(product_data)).await;

        // Assert validation error
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Get response body
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8_lossy(&body);

        // Verify error response
        let error: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(error["status"], "error");
        assert_eq!(error["code"], "BAD_REQUEST");
    }

    // Test creating a product with invalid price (negative)
    #[tokio::test]
    async fn test_create_product_invalid_price() {
        let app = setup_test_app();

        let product_data = json!({
            "name": "Test Product",
            "description": "A valid product description",
            "price": -10.0, // Negative price
            "stock": 10
        });

        let response = make_json_request(app, "POST", "/products", Some(product_data)).await;

        // Assert validation error
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Get response body
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8_lossy(&body);

        // Verify error response
        let error: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(error["status"], "error");
        assert_eq!(error["code"], "BAD_REQUEST");
    }

    // Test searching products with valid parameters
    #[tokio::test]
    async fn test_search_products_valid() {
        let app = setup_test_app();

        // Valid query params
        let response = make_json_request(app, "GET", "/products", None::<Value>).await;

        // Assert success
        assert_eq!(response.status(), StatusCode::OK);

        // Get response body
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8_lossy(&body);

        // Verify response body
        let response: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(response["status"], "success");
        assert!(response["data"].is_array());
    }

    // Test searching products with invalid limit parameter
    #[tokio::test]
    async fn test_search_products_invalid_limit() {
        let app = setup_test_app();

        // Min price outside valid range
        let response =
            make_json_request(app, "GET", "/products?min_price=-10.0", None::<Value>).await;

        // Assert validation error
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Get response body
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8_lossy(&body);

        // Verify error response
        let error: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(error["status"], "error");
        assert_eq!(error["code"], "VALIDATION_ERROR");
    }

    // Test getting a product by valid ID
    #[tokio::test]
    async fn test_get_product_valid_id() {
        let app = setup_test_app();

        let response = make_json_request(app, "GET", "/products/not-found", None::<Value>).await;

        // Assert not found (product doesn't exist, but ID format is valid)
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Get response body
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8_lossy(&body);

        // Verify error response
        let error: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(error["status"], "error");
        assert_eq!(error["code"], "RESOURCE_NOT_FOUND");
    }

    // Test getting a product with invalid ID format
    #[tokio::test]
    async fn test_get_product_invalid_id() {
        let app = setup_test_app();

        let response = make_json_request(app, "GET", "/products/error", None::<Value>).await;

        // Assert server error
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        // Get response body
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8_lossy(&body);

        // Verify error response
        let error: serde_json::Value = serde_json::from_str(&body_str).unwrap();
        assert_eq!(error["status"], "error");
        assert!(error["message"].is_string());
    }

    // Test malformed JSON
    #[tokio::test]
    async fn test_malformed_json() {
        let app = setup_test_app();

        // Direkter Aufruf mit InvalidJSON statt make_json_request
        let method = "POST";
        let uri = "/products";

        // Manuell einen ungültigen JSON-Request erstellen
        let req = Request::builder()
            .method(method)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from("{invalid_json}"))
            .unwrap();

        // App-State vorbereiten
        let app = app.with_state(ProductAppState {
            product_service: Arc::new(ProductService::new()),
        });

        // Request senden
        let response = app.into_service().oneshot(req).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Get response body
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(body["status"], "error");
        assert_eq!(body["code"], "BAD_REQUEST");
    }

    // Angepasste Version der bestehenden make_json_request Funktion für invalid JSON
    #[allow(dead_code)]
    pub async fn make_invalid_json_request(app: Router<ProductAppState>) -> Response {
        let request = Request::builder()
            .uri("/products")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from("{invalid_json}"))
            .unwrap();

        // Entspricht dem Muster in make_json_request
        let app = app.with_state(ProductAppState {
            product_service: Arc::new(ProductService::new()),
        });

        app.into_service().oneshot(request).await.unwrap()
    }
}
