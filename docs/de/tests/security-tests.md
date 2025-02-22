# Leitfaden für Sicherheitstests

## Überblick

Sicherheitstests sind essentiell für die Identifizierung von Schwachstellen und die Gewährleistung, dass unsere Anwendung die Sicherheitsanforderungen erfüllt. Dieser Leitfaden behandelt verschiedene Sicherheitstestansätze, Werkzeuge und Best Practices für die Aufrechterhaltung der Anwendungssicherheit.

## Grundkonzepte

1. **Statische Analyse**
   - Code-Scanning
   - Abhängigkeitsprüfung
   - SAST (Statische Anwendungssicherheitstests)

2. **Dynamische Analyse**
   - Penetrationstests
   - Fuzzing
   - DAST (Dynamische Anwendungssicherheitstests)

3. **Compliance-Tests**
   - Überprüfung von Sicherheitsstandards
   - Regulatorische Compliance-Prüfungen
   - Durchsetzung von Sicherheitsrichtlinien

## Werkzeuge für statische Analyse

### Cargo Audit

```bash
# Bekannte Schwachstellen in Abhängigkeiten prüfen
cargo audit

# Advisory-Datenbank aktualisieren
cargo audit update

# Bericht generieren
cargo audit --json > security-audit.json
```

### Clippy Sicherheits-Lints

```rust
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]

// Sicherheitsspezifische Lints
#![deny(clippy::missing_safety_doc)]
#![deny(clippy::undocumented_unsafe_blocks)]
```

## Testkategorien

### 1. Eingabevalidierungstests

```rust
#[test]
fn test_sql_injection_prevention() {
    let input = "'; DROP TABLE users; --";
    let query = sanitize_sql_input(input);
    assert!(!query.contains(';'));
    assert!(!query.contains("DROP"));
}
```

### 2. Authentifizierungstests

```rust
#[tokio::test]
async fn test_password_hashing() {
    let password = "secure_password123";
    let hash = hash_password(password).await?;
    
    // Überprüfen, dass Hash nicht im Klartext ist
    assert_ne!(password, hash);
    
    // Hash-Validierung überprüfen
    assert!(verify_password(password, &hash).await?);
}
```

### 3. Autorisierungstests

```rust
#[test]
fn test_rbac_permissions() {
    let user = User::new_with_role(Role::Standard);
    
    assert!(user.can_read());
    assert!(!user.can_admin());
    
    let admin = User::new_with_role(Role::Admin);
    assert!(admin.can_admin());
}
```

## Fuzzing-Tests

### 1. Grundlegendes Fuzzing

```rust
use afl::fuzz;

#[cfg(fuzzing)]
fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(s) = std::str::from_utf8(data) {
            let _ = parse_user_input(s);
        }
    });
}
```

### 2. Strukturbewusstes Fuzzing

```rust
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct RequestData {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
}

#[test]
fn fuzz_http_request() {
    let mut fuzzer = arbitrary::Unstructured::new(&SEED);
    let request = RequestData::arbitrary(&mut fuzzer)?;
    process_request(&request);
}
```

## Penetrationstests

### 1. API-Sicherheitstests

```rust
#[tokio::test]
async fn test_api_security() {
    let client = reqwest::Client::new();
    
    // CORS testen
    let response = client
        .get("/api/resource")
        .header("Origin", "https://malicious.com")
        .send()
        .await?;
    
    assert!(!response.headers().contains_key("Access-Control-Allow-Origin"));
}
```

### 2. Sitzungssicherheit

```rust
#[test]
fn test_session_security() {
    let session = create_session();
    
    // Sitzungstoken-Stärke testen
    assert!(session.token().len() >= 32);
    assert!(is_cryptographically_secure(session.token()));
    
    // Sitzungsablauf testen
    assert!(session.expires_in() <= Duration::from_hours(24));
}
```

## Best Practices

1. **Sichere Konfiguration**
   - Sichere Standardeinstellungen verwenden
   - Sicherheitseinstellungen validieren
   - Regelmäßige Sicherheitsaudits

2. **Datenschutz**
   - Verschlüsselung im Ruhezustand
   - Sichere Datenübertragung
   - Angemessenes Schlüsselmanagement

3. **Fehlerbehandlung**
   - Sichere Fehlermeldungen
   - Angemessenes Logging
   - Keine Preisgabe sensibler Daten

4. **Abhängigkeitsmanagement**
   - Regelmäßige Aktualisierungen
   - Schwachstellenscanning
   - Minimale Abhängigkeitsnutzung

## Tests ausführen

1. Sicherheitsprüfungen ausführen:

   ```bash
   cargo audit && cargo clippy -- -D warnings
   ```

2. Fuzzing-Tests ausführen:

   ```bash
   cargo afl build
   cargo afl fuzz -i input -o output target/debug/fuzz_target
   ```

3. Sicherheitstest-Suite ausführen:

   ```bash
   cargo test --test security_tests
   ```

## Häufige Muster

### Sichere Passwortverarbeitung

```rust
use argon2::{self, Config};

fn hash_password(password: &[u8]) -> Result<String> {
    let salt = generate_salt();
    let config = Config::default();
    argon2::hash_encoded(password, &salt, &config)
}
```

### Eingabebereinigung

```rust
fn sanitize_input(input: &str) -> String {
    let mut sanitized = input.to_owned();
    sanitized.retain(|c| !c.is_control());
    html_escape::encode_text(&sanitized).to_string()
}
```

## Sicherheitsüberwachung

### 1. Audit-Logging

```rust
use tracing::{info, warn, error};

fn log_security_event(event: SecurityEvent) {
    info!(
        event_type = %event.type_str(),
        user = %event.user_id,
        "Sicherheitsereignis aufgetreten"
    );
}
```

### 2. Einbruchserkennung

```rust
fn detect_suspicious_activity(request: &Request) -> bool {
    let rules = load_security_rules();
    rules.iter().any(|rule| rule.matches(request))
}
```

## Weiterführende Literatur

- [OWASP Rust Sicherheitsrichtlinien](https://owasp.org/rust-security-framework/)
- [Rust Security Working Group](https://github.com/rust-secure-code/wg)
- [Häufige Sicherheitstests](../security/COMMON_TESTS.md)
- [Sicherheitsrichtlinien](../security/POLICIES.md)
