# API-Modul des ACCI Frameworks

Das API-Modul implementiert die grundlegende Infrastruktur und Routing-Logik für RESTful APIs im ACCI Framework.

## Fehlerbehandlung im API-Modul

### Standardisierte API-Fehler

Das API-Modul verwendet ein standardisiertes Fehlerformat für alle Antworten:

```json
{
  "status": "error",
  "message": "Die Fehlerbeschreibung",
  "code": "FEHLER_CODE",
  "request_id": "req-12345"
}
```

Bei Validierungsfehlern werden zusätzliche Details bereitgestellt:

```json
{
  "status": "error",
  "message": "Validation error: email: Invalid email format, password: Password must be at least 8 characters long",
  "code": "VALIDATION_ERROR",
  "request_id": "req-12345",
  "errors": {
    "email": ["Invalid email format"],
    "password": ["Password must be at least 8 characters long"]
  }
}
```

### Fehlertypen

| HTTP Status | Code | Beschreibung |
|-------------|------|--------------|
| 400 | INVALID_REQUEST | Allgemeiner Fehler für ungültige Anfragen |
| 400 | INVALID_JSON | Ungültiges JSON-Format in der Anfrage |
| 401 | AUTHENTICATION_REQUIRED | Authentifizierung erforderlich |
| 403 | AUTHORIZATION_ERROR | Keine Berechtigung für die angeforderte Ressource |
| 404 | RESOURCE_NOT_FOUND | Die angeforderte Ressource wurde nicht gefunden |
| 422 | VALIDATION_ERROR | Die Anfrage enthält ungültige Daten |
| 429 | RATE_LIMIT_EXCEEDED | Zu viele Anfragen wurden gesendet |
| 500 | INTERNAL_SERVER_ERROR | Ein interner Serverfehler ist aufgetreten |

### Verwendung der Fehlerbehandlung

Um die API-Fehlerbehandlung zu nutzen, gibt es mehrere Möglichkeiten:

1. **ApiError-Struktur verwenden**:

```rust
use acci_api::response::ApiError;
use axum::http::StatusCode;

// In einem Handler:
fn error_example() -> impl IntoResponse {
    let request_id = generate_request_id();
    ApiError::new(
        StatusCode::BAD_REQUEST,
        "Ungültige Parameter",
        "INVALID_PARAMETERS",
        request_id
    )
}
```

2. **API-Fehlerhelfer verwenden**:

```rust
use acci_api::response::ApiError;

// Vordefinierte Fehlertypen
fn not_found_example() -> impl IntoResponse {
    let request_id = generate_request_id();
    ApiError::not_found_error("User", request_id)
}
```

3. **Fehlerbehandlungsmiddleware**:

Das API-Modul enthält eine Fehlerbehandlungsmiddleware, die automatisch Fehlerantworten standardisiert.

```rust
use axum::{Router, middleware::from_fn};
use acci_api::middleware::error_handling::error_handling_middleware;

let app = Router::new()
    // ... Routen hinzufügen ...
    .layer(from_fn(error_handling_middleware));
```

4. **Validierung mit error_handling**:

```rust
use acci_api::validation::validate_json_payload;
use axum::Json;

async fn create_item(
    Json(payload): Json<CreateItemRequest>
) -> impl IntoResponse {
    // Validierung mit detaillierten Fehlerberichten
    let validated = validate_json_payload(Json(payload)).await?;
    
    // Mit validated.0 auf validierte Daten zugreifen
    let item = create_item_in_db(validated.0).await?;
    
    // Erfolgsantwort zurückgeben
    // ...
}
```

### Metriken und Protokollierung

Die API-Fehlerbehandlung zeichnet automatisch Fehlermetriken und erstellt Log-Einträge:

1. **Fehlermetriken**: `api.errors.client`, `api.errors.server`, `api.validation.errors`
2. **Protokollierung**: Je nach Fehlerart werden Fehler als ERROR, WARN oder INFO protokolliert
3. **Request-IDs**: Jeder Fehler erhält eine eindeutige Request-ID für Nachverfolgbarkeit

### Erweiterungen

Mit dem Feature-Flag `extended_errors` können zusätzliche Fehlerdetails in der Antwort eingeschlossen werden:

```toml
[dependencies]
acci_api = { version = "0.1.0", features = ["extended_errors"] }
```
