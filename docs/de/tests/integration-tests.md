# Integrationstests-Leitfaden

## Überblick

Integrationstests überprüfen das Zusammenspiel mehrerer Systemkomponenten. Diese Tests stellen sicher, dass verschiedene Teile der Anwendung unter realen Bedingungen korrekt zusammenarbeiten.

## Grundprinzipien

1. **Test-Lokation**
   - Alle Integrationstests befinden sich im `/tests`-Verzeichnis
   - Klare Trennung von Unit-Tests
   - Organisiert nach Feature oder Komponente

2. **Externe Abhängigkeiten**
   - Verwendung von containerisierten Abhängigkeiten via `testcontainers-rs`
   - Kontrollierter Netzwerkzugriff
   - Isolierte Test-Datenbanken
   - Gemockte externe Dienste wo angebracht

3. **Test-Isolation**
   - Jede Testsuite läuft isoliert
   - Saubere Umgebung für jeden Test
   - Keine Interferenz zwischen Tests

## Verzeichnisstruktur

```
/tests
├── src/
│   ├── helpers/       # Gemeinsame Test-Utilities
│   ├── fixtures/      # Testdaten
│   └── mocks/         # Mock-Implementierungen
├── api/               # API-Integrationstests
├── auth/              # Authentifizierungstests
├── database/          # Datenbank-Integrationstests
└── e2e/              # End-to-End-Tests
```

## Test-Setup

### Datenbanktests

```rust
use testcontainers::*;

#[tokio::test]
async fn test_database_integration() {
    let docker = clients::Cli::default();
    let postgres = docker.run(images::postgres::Postgres::default());
    
    let connection_string = format!(
        "postgres://postgres:postgres@localhost:{}/postgres",
        postgres.get_host_port_ipv4(5432)
    );
    
    let db = Database::connect(&connection_string).await?;
    // Testimplementierung
}
```

### API-Tests

```rust
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn test_api_integration() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/api/resource"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;
    
    let client = ApiClient::new(mock_server.uri());
    let response = client.get_resource().await?;
    assert_eq!(response.status(), 200);
}
```

## Testkategorien

### 1. Komponentenintegration

```rust
#[tokio::test]
async fn test_auth_with_database() {
    let db = setup_test_database().await?;
    let auth_service = AuthService::new(db);
    
    let result = auth_service
        .authenticate("user", "password")
        .await?;
    
    assert!(result.is_authenticated());
}
```

### 2. API-Integration

```rust
#[tokio::test]
async fn test_api_workflow() {
    let app = test_app().await?;
    
    // Ressource erstellen
    let create_response = app
        .client
        .post("/api/resources")
        .json(&new_resource)
        .send()
        .await?;
    
    assert_eq!(create_response.status(), 201);
    
    // Ressource verifizieren
    let get_response = app
        .client
        .get("/api/resources")
        .send()
        .await?;
    
    assert_eq!(get_response.status(), 200);
}
```

### 3. Datenbankintegration

```rust
#[tokio::test]
async fn test_database_operations() {
    let db = setup_test_database().await?;
    
    // Datensatz erstellen
    let id = db.create_record(&new_record).await?;
    
    // Datensatz verifizieren
    let record = db.get_record(id).await?;
    assert_eq!(record.field, expected_value);
}
```

## Mock-Strategien

### 1. HTTP-Dienste

```rust
#[tokio::test]
async fn test_external_service() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("POST"))
        .and(path("/external/api"))
        .and(json_body(expected_request))
        .respond_with(json_response(expected_response))
        .mount(&mock_server)
        .await;
    
    // Testimplementierung
}
```

### 2. Datenbank-Mocking

```rust
#[tokio::test]
async fn test_with_mock_database() {
    let mock_db = MockDatabase::new()
        .expect_query()
        .with(eq("SELECT * FROM users"))
        .returning(|_| Ok(mock_users()));
    
    let service = Service::new(mock_db);
    // Testimplementierung
}
```

## Best Practices

1. **Testdaten-Management**
   - Verwendung von Fixtures für konsistente Testdaten
   - Bereinigung der Testdaten nach jedem Test
   - Aussagekräftige Testdatennamen

2. **Fehlerbehandlung**
   - Testen von Fehlerbedingungen
   - Überprüfung von Fehlerantworten
   - Testen von Timeout-Szenarien

3. **Asynchrones Testen**
   - Korrekte Behandlung von asynchronen Operationen
   - Testen von Abbruchszenarien
   - Überprüfung von Timeouts

4. **Container-Management**
   - Effizienter Container-Lebenszyklus
   - Ressourcenbereinigung
   - Parallele Testausführung

## Tests ausführen

1. Alle Integrationstests ausführen:

   ```bash
   cargo test --test '*'
   ```

2. Spezifische Testsuite ausführen:

   ```bash
   cargo test --test api_integration
   ```

3. Mit Logging ausführen:

   ```bash
   RUST_LOG=debug cargo test --test '*'
   ```

## Häufige Muster

### Test-App-Setup

```rust
async fn setup_test_app() -> TestApp {
    let db = setup_test_database().await?;
    let auth = setup_test_auth().await?;
    let api = setup_test_api(db.clone(), auth.clone()).await?;
    
    TestApp {
        db,
        auth,
        api,
        client: reqwest::Client::new(),
    }
}
```

### Aufräumen

```rust
impl Drop for TestApp {
    fn drop(&mut self) {
        // Ressourcen aufräumen
        self.db.cleanup();
        self.auth.cleanup();
    }
}
```

## Weiterführende Literatur

- [Testcontainers-Dokumentation](https://docs.rs/testcontainers)
- [WireMock-Dokumentation](https://docs.rs/wiremock)
- [Unit-Testing-Leitfaden](unit-tests.md)
- [Testkonfiguration](test-configuration.md)
