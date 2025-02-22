# Entwicklungsumgebung einrichten

Dieses Dokument beschreibt alle notwendigen Werkzeuge und Einrichtungsschritte für die Entwicklung des ACCI Frameworks.

## Erforderliche Werkzeuge

### Rust und Cargo

Das Projekt verwendet die Rust Nightly Toolchain. Die spezifische Version und Komponenten sind in `rust-toolchain.toml` definiert.

### Code-Qualitätswerkzeuge

- **rustfmt**: Werkzeug zur Code-Formatierung, das einen einheitlichen Codestil im Projekt sicherstellt
- **clippy**: Rust-Linter, der häufige Fehler erkennt und Best Practices durchsetzt
- **rust-analyzer**: Language Server Protocol (LSP) Implementierung für IDE-Unterstützung

### Sicherheitswerkzeuge

- **cargo-audit**: Überprüft Abhängigkeiten auf bekannte Sicherheitslücken
- **cargo-deny**: Setzt Abhängigkeitsrichtlinien durch und kontrolliert erlaubte/verbotene Abhängigkeiten
- **cargo-cyclonedx**: Generiert Software Bill of Materials (SBOM) im CycloneDX-Format

### Testwerkzeuge

- **cargo-tarpaulin**: Erstellt Code-Coverage-Berichte für die Testsuite
- **cargo-mutants**: Führt Mutationstests durch, um die Effektivität der Testsuite zu bewerten
- **cargo-nextest**: Bietet einen funktionsreicheren Testrunner mit besserer Berichterstattung
- **criterion**: Framework zum Schreiben und Ausführen von Benchmarks
- **proptest**: Framework für eigenschaftsbasiertes Testen
- **afl**: American Fuzzy Lop Integration für Fuzzing-Tests
- **arbitrary**: Strukturbewusstes Fuzzing für Sicherheitstests

### Datenbank-Werkzeuge

- **sqlx-cli**: CLI für SQLx, verwendet für Datenbank-Migrationen und Schema-Management

### Überwachung und Metriken

- **metrics-rs**: Sammelt Anwendungsmetriken (Rate, Errors, Duration)
- **metrics-exporter-prometheus**: Exportiert Metriken im Prometheus-Format

### Container-Werkzeuge

- **Docker**: Erforderlich für Containerisierung (muss separat installiert werden)
- **Docker Compose**: Erforderlich für lokale Entwicklung (muss separat installiert werden)

## Installation

### 1. Rust installieren

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Erforderliche Cargo-Werkzeuge installieren

```bash
# Installation aller erforderlichen Cargo-Werkzeuge in einem Befehl
cargo install \
    cargo-audit \
    cargo-deny \
    cargo-cyclonedx \
    cargo-tarpaulin \
    cargo-mutants \
    cargo-nextest \
    criterion \
    proptest \
    afl \
    sqlx-cli
```

### 3. Docker und Docker Compose installieren

Bitte folgen Sie den offiziellen Installationsanleitungen für Ihr Betriebssystem:

- [Docker Installationsanleitung](https://docs.docker.com/get-docker/)
- [Docker Compose Installationsanleitung](https://docs.docker.com/compose/install/)

### 4. IDE konfigurieren

Für die beste Entwicklungserfahrung empfehlen wir die Verwendung einer IDE mit Rust-Unterstützung durch rust-analyzer. Beliebte Optionen sind:

- VS Code mit rust-analyzer Extension
- IntelliJ IDEA mit Rust Plugin
- Cursor IDE (empfohlen)

## Entwicklungs-Workflow

1. Repository klonen
2. `cargo build` ausführen, um sicherzustellen, dass alles kompiliert
3. `cargo test` ausführen, um die Testsuite zu starten
4. Mit dem Coding beginnen!

Für detailliertere Informationen über die Projektarchitektur und -ziele, siehe:

- `docs/ARCHITECTURE.md`
- `docs/GOALS.md`
- `docs/MILESTONES.md`
