//! API Monitoring
//!
//! This module provides monitoring and metrics functionality for the API.

use std::net::SocketAddr;
use std::sync::OnceLock;
use std::time::Instant;

use axum::Router;
use axum::extract::MatchedPath;
use axum::http::Request;
use axum::response::IntoResponse;
use axum::routing::get;
use metrics::counter;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use tokio::net::TcpListener;
use tracing::info;

use crate::config::ApiConfig;

// Global metrics handle
static METRICS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

/// Get a reference to the Prometheus handle
pub fn get_metrics_handle() -> Option<&'static PrometheusHandle> {
    METRICS_HANDLE.get()
}

/// Initialize the metrics system
///
/// # Errors
///
/// Returns an error if the initialization fails.
pub fn init_metrics() -> Result<(), String> {
    if METRICS_HANDLE.get().is_some() {
        return Ok(());
    }

    let builder = PrometheusBuilder::new();
    let handle = builder
        .install_recorder()
        .map_err(|err| format!("Failed to install metrics recorder: {}", err))?;

    METRICS_HANDLE
        .set(handle)
        .map_err(|_| "Failed to store metrics handle".to_string())?;

    info!("Metrics system initialized");
    Ok(())
}

/// Start the metrics server
///
/// # Errors
///
/// Returns an error if the server cannot be started.
pub async fn start_metrics_server(config: &ApiConfig) -> Result<(), String> {
    let metrics_handle =
        get_metrics_handle().ok_or_else(|| "Metrics not initialized".to_string())?;

    // Klonen des Handles für den Closure
    let handle_clone = metrics_handle.clone();

    let app = Router::new().route(
        "/metrics",
        get(move || async move { handle_clone.render() }),
    );

    let addr: SocketAddr = config
        .metrics_addr
        .parse()
        .map_err(|err| format!("Failed to parse metrics address: {}", err))?;

    info!("Starting metrics server on {}", addr);

    let listener = TcpListener::bind(addr)
        .await
        .map_err(|err| format!("Failed to bind to address: {}", err))?;

    axum::serve(listener, app)
        .await
        .map_err(|err| format!("Metrics server error: {}", err))?;

    Ok(())
}

/// Zeichnet eine Authentifizierungsoperation auf.
#[allow(unused_must_use)]
pub fn record_auth_operation(operation: &str, result: &str) {
    let metric_name = format!("auth.operations.{}.{}", operation, result);
    counter!(metric_name);
}

/// Zeichnet die Anzahl der aktiven Sitzungen auf.
#[allow(unused_must_use)]
pub fn record_active_sessions(_count: u64) {
    counter!("auth.active_sessions");
}

/// Zeichnet eine API-Anfrage auf.
#[allow(unused_must_use)]
pub fn record_api_request(method: &str, path: &str) {
    let metric_name = format!("api.requests.{}.{}", method, path);
    counter!(metric_name);
}

/// Zeichnet eine API-Antwort auf.
#[allow(unused_must_use)]
pub fn record_api_response(status: u16) {
    let metric_name = format!("api.responses.{}", status);
    counter!(metric_name);
}

/// Zeichnet die Dauer einer API-Anfrage auf.
pub fn record_request_duration(duration_secs: f64, method: &str, path: &str) {
    let metric_name = format!("api.request.duration.{}.{}", method, path);
    metrics::histogram!(metric_name).record(duration_secs);
}

/// Handler for Metrics endpoint.
pub fn metrics_handler() -> Result<impl IntoResponse, String> {
    let handle = get_metrics_handle().ok_or_else(|| "Metrics not initialized".to_string())?;
    Ok(handle.render())
}

/// Middleware for recording metrics for requests.
pub fn track_metrics<B>(req: Request<B>, _start: Instant) -> Request<B> {
    let method = req.method().to_string();
    let uri = req.uri().to_string();

    if let Some(path) = req.extensions().get::<MatchedPath>() {
        let path = path.as_str().to_owned();
        record_api_request(&method, &path);
    } else {
        record_api_request(&method, &uri);
    }

    req
}

/// Zeichnet eine Validierungsfehler-Operation auf.
#[allow(unused_must_use)]
pub fn record_validation_error(field: &str, error_type: &str) {
    let metric_name = "api.validation.errors.detail".to_string();
    counter!(metric_name, "field" => field.to_string(), "error_type" => error_type.to_string())
        .increment(1);
}

/// Zeichnet einen API-Fehler mit Typ und Code auf.
#[allow(unused_must_use)]
pub fn record_api_error(error_type: &str, error_code: &str, status_code: u16) {
    let metric_name = "api.errors.detail".to_string();
    counter!(
        metric_name,
        "type" => error_type.to_string(),
        "code" => error_code.to_string(),
        "status" => status_code.to_string()
    )
    .increment(1);
}

/// Zeichnet detaillierte Validierungsstatistiken auf
pub fn record_validation_stats(
    req_type: &str,
    field_count: u32,
    error_count: u32,
    duration_ms: u64,
) {
    metrics::gauge!("api.validation.stats", "req_type" => req_type.to_string())
        .set(error_count as f64 / field_count.max(1) as f64);

    metrics::histogram!("api.validation.duration.detail", "req_type" => req_type.to_string())
        .record(duration_ms as f64 / 1000.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Method, Uri};

    #[test]
    fn test_init_metrics() {
        // Simple test to verify if initialization works
        assert!(init_metrics().is_ok());

        // Verify we can get the handle after initialization
        assert!(get_metrics_handle().is_some());
    }

    #[test]
    fn test_record_auth_operation() {
        // Initialize metrics
        init_metrics().unwrap();

        // Record auth operations
        record_auth_operation("login", "success");
        record_auth_operation("login", "failure");

        // We can't easily verify the counter values directly in tests,
        // but we can verify that the code executes without error
    }

    #[test]
    fn test_record_api_request() {
        // Initialize metrics
        init_metrics().unwrap();

        // Record API requests
        record_api_request("GET", "/api/users");
        record_api_request("POST", "/api/products");
    }

    #[test]
    fn test_record_api_response() {
        // Initialize metrics
        init_metrics().unwrap();

        // Record API responses
        record_api_response(200);
        record_api_response(404);
        record_api_response(500);
    }

    #[test]
    fn test_record_request_duration() {
        // Initialize metrics
        init_metrics().unwrap();

        // Record request durations
        record_request_duration(0.1, "GET", "/api/users");
        record_request_duration(1.5, "POST", "/api/products");
    }

    #[test]
    fn test_track_metrics() {
        // Initialize metrics
        init_metrics().unwrap();

        // Create a simple request
        let uri = Uri::from_static("/api/products");
        let method = Method::GET;

        let req = Request::builder()
            .uri(uri.clone())
            .method(method.clone())
            .body(())
            .unwrap();

        // Track metrics - ohne MatchedPath wird der URI direkt verwendet
        let start = Instant::now();
        let _req = track_metrics(req, start);

        // Dies testet den Fall, wenn kein MatchedPath vorhanden ist
        // und wir stattdessen den URI verwenden
    }

    #[test]
    fn test_record_validation_error() {
        // Initialize metrics
        init_metrics().unwrap();

        // Record validation errors
        record_validation_error("email", "format");
        record_validation_error("password", "length");
    }

    #[test]
    fn test_record_api_error() {
        // Initialize metrics
        init_metrics().unwrap();

        // Record API errors
        record_api_error("client", "BAD_REQUEST", 400);
        record_api_error("server", "INTERNAL_SERVER_ERROR", 500);
    }

    #[test]
    fn test_record_validation_stats() {
        // Initialize metrics
        init_metrics().unwrap();

        // Record validation stats
        record_validation_stats("user_registration", 10, 2, 50);
        record_validation_stats("product_creation", 5, 0, 20);
    }
}
