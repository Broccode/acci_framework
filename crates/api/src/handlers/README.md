# API Handler

Dieses Verzeichnis enthält die Handler für die API-Endpunkte des ACCI Frameworks.

## Beispiel-Handler

Im Modul `example` finden Sie Beispiel-Handler für eine Produkt-API, die die Fehlerbehandlung und Validierung demonstrieren. Diese Beispiele zeigen, wie Sie die API-Funktionen des ACCI Frameworks in Ihren eigenen Anwendungen verwenden können.

### Produkt-Handler

Die Produkt-Handler demonstrieren folgende Funktionen:

- Standardisierte API-Antworten mit `ApiResponse`
- Fehlerbehandlung mit `ApiError`
- Validierung von Anfragen mit `validator`
- Generierung von Request-IDs für Tracing
- Logging mit strukturierten Metadaten
- Metriken für Monitoring

### Beispiel-Endpunkte

Die folgenden Endpunkte sind im Beispiel implementiert:

- `GET /products/:id` - Abrufen eines Produkts nach ID
- `POST /products` - Erstellen eines neuen Produkts
- `GET /products` - Suchen von Produkten mit Filterparametern

### Verwendung

Um die Beispiel-Handler in Ihrer Anwendung zu verwenden, importieren Sie die Router-Konfiguration:

```rust
use crate::handlers::example_router::configure_example_routes;

// In Ihrer Router-Konfiguration:
let app = Router::new()
    // ... andere Router ...
    .merge(configure_example_routes(Router::new()));
```

### Tests

Die Beispiel-Handler sind vollständig getestet und demonstrieren, wie Sie API-Endpunkte mit verschiedenen Szenarien testen können:

- Erfolgreiche Anfragen
- Nicht gefundene Ressourcen
- Validierungsfehler
- Serverfehler

Die Tests zeigen, wie Sie die Antworten auf korrekte Statuscodes und JSON-Strukturen prüfen können.

## Eigene Handler erstellen

Wenn Sie eigene Handler erstellen, empfehlen wir, dem Muster der Beispiel-Handler zu folgen:

1. Verwenden Sie `ApiResponse` für erfolgreiche Antworten
2. Verwenden Sie `ApiError` für Fehlerantworten
3. Validieren Sie Eingaben mit `validator` und `validate_json_payload`
4. Generieren Sie Request-IDs für jede Anfrage
5. Fügen Sie strukturierte Logs mit `tracing` hinzu
6. Verwenden Sie die Fehlerbehandlungs-Middleware

Beispiel für einen einfachen Handler:

```rust
async fn my_handler() -> Response {
    let request_id = generate_request_id();
    
    // Ihre Logik hier
    
    let data = MyData { /* ... */ };
    let api_response = ApiResponse::success(data, request_id);
    (StatusCode::OK, Json(api_response)).into_response()
}
