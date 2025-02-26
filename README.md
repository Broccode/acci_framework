# 🏢 ACCI Framework

> 🚀 Eine robuste, flexible und sichere Enterprise-Anwendungsplattform, entwickelt mit modernen Technologien für skalierbare Geschäftsanwendungen.

[![Rust](https://img.shields.io/badge/rust-nightly-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.1-green.svg)](CHANGELOG.md)

---

## 📋 Inhaltsverzeichnis

- [Überblick](#-überblick)
- [Projektziele](#-projektziele)
- [Technischer Stack](#-technischer-stack)
- [Projektzeitplan](#-projektzeitplan)
- [Entwicklungsumgebung](#-entwicklungsumgebung)
- [Dokumentation](#-dokumentation)
- [Lizenz](#-lizenz)
- [Kontakt](#-kontakt)

---

## 🔍 Überblick

Das ACCI Framework bietet eine solide Grundlage für verschiedene Geschäftsanwendungen. Es ist für Unternehmen konzipiert, die eine flexible, sichere und skalierbare Plattform suchen, die die Softwareentwicklung und den Betrieb optimiert.

---

## 🎯 Projektziele

### Kernziele

| Ziel | Beschreibung |
|------|-------------|
| 🔄 **Flexibilität und Wiederverwendbarkeit** | Anpassungsfähiges Framework, das mit verschiedenen Produkten und sich entwickelnden Anforderungen durch eine modulare Architektur wächst |
| 🔒 **Sicherheit und Compliance** | Unternehmenstaugliche Sicherheit mit MFA, Verschlüsselung und Einhaltung von Vorschriften wie GDPR |
| 📈 **Skalierbarkeit und Verfügbarkeit** | Unterstützung für wachsende Benutzerzahlen und Datenmengen mit Hochverfügbarkeitsfunktionen |
| 🔌 **Integration und Erweiterbarkeit** | Nahtlose Integration mit bestehenden Systemen und Erweiterbarkeit durch eine Plugin-Architektur |
| 👥 **Benutzerorientierte Erfahrung** | Intuitive Benutzeroberflächen mit Mehrsprachenunterstützung und effizienten automatisierten Workflows |

---

## 💻 Technischer Stack

Die ACCI-Plattform basiert auf modernen Technologien, die optimale Leistung, Sicherheit und Skalierbarkeit bieten:

### 🔧 Backend

- 🦀 **Rust**: Kernsprache für das Backend, bietet Speichersicherheit und hohe Leistung
- 🌐 **Axum**: Web-Framework für API-Entwicklung
- 🏗️ **Domain-Driven Design (DDD)**: Klare Modellierung von Geschäftsdomänen
- 📊 **Event Sourcing & CQRS**: Speicherung von Zustandsänderungen als Ereignisse für Nachverfolgbarkeit

### 🖥️ Frontend

- ⚡ **Leptos**: Modernes Rust-basiertes Web-Framework
- 🧩 **WebAssembly**: Für hochleistungsfähige clientseitige Verarbeitung

### 💾 Datenspeicherung

- 🐘 **PostgreSQL**: Primäre Datenbank für persistente Speicherung
- ⚡ **Redis**: Für Caching und Session-Management

### 🏛️ Architekturmuster

- 🏢 **Multi-Tenancy**: Gemeinsame Plattform mit isolierten Daten pro Mandant
- 🧩 **Plugin-Architektur**: Erweiterbare Geschäftslogik über modulare Plugins
- 🔄 **Dual-API-Bereitstellung**: Sowohl REST- als auch GraphQL-Schnittstellen

### 🚢 Deployment & Infrastruktur

- 🐳 **Docker & Docker Compose**: Für Containerisierung und Orchestrierung
- 🔄 **Zero-Downtime Deployment**: Mit Rollbacks und Gesundheitschecks
- 📦 **SBOM-Management**: Software Bill of Materials für Sicherheitsverfolgung

---

## 📅 Projektzeitplan

Die Entwicklung ist in mehreren Schlüsselphasen geplant:

| Phase | Zeitrahmen | Beschreibung |
|-------|------------|--------------|
| 1️⃣ **Grundlagen und Basisauthentifizierung** | Q1 2025 | Core-Framework, Authentifizierung und Session-Management |
| 2️⃣ **Multi-Tenancy und verbesserte Sicherheit** | Q1 2025 | Mandantenisolierung und Sicherheitsfunktionen |
| 3️⃣ **Kern-Geschäftslogik und DDD-Implementierung** | Q2 2025 | Event Sourcing, CQRS und Plugin-Architektur |
| 4️⃣ **Integration und Erweiterbarkeit** | Q2 2025 | Integration externer Systeme und GraphQL-API |

---

## ⚙️ Entwicklungsumgebung

### 🛠️ Erforderliche Tools

- 🦀 **Rust Nightly**: Die spezifische Version ist in `rust-toolchain.toml` definiert
- 🐳 **Docker und Docker Compose**: Für Containerisierung und lokale Entwicklung
- 🧰 **Code-Qualitätstools**: rustfmt, clippy, rust-analyzer
- 🔒 **Sicherheitstools**: cargo-audit, cargo-deny, cargo-cyclonedx
- 🧪 **Testtools**: cargo-llvm-cov, cargo-mutants, cargo-nextest
- 🗃️ **Datenbanktools**: sqlx-cli für Migrationen und Schema-Management

### 📝 Installationsschritte

<details>
<summary>1. Rust installieren</summary>

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

</details>

<details>
<summary>2. Erforderliche Cargo-Tools installieren</summary>

```bash
cargo install \
    cargo-audit \
    cargo-deny \
    cargo-cyclonedx \
    cargo-llvm-cov \
    cargo-mutants \
    cargo-nextest \
    sqlx-cli

rustup component add llvm-tools-preview
```

</details>

<details>
<summary>3. Docker und Docker Compose installieren</summary>

Folgen Sie den offiziellen Installationsanleitungen:

- [Docker-Installationsanleitung](https://docs.docker.com/get-docker/)
- [Docker Compose-Installationsanleitung](https://docs.docker.com/compose/install/)

</details>

<details>
<summary>4. Entwicklungsumgebung einrichten</summary>

```bash
# Repository klonen
git clone https://github.com/your-org/acci-framework.git
cd acci-framework

# Projekt bauen
make dev

# Tests ausführen
make test
```

</details>

### 💻 IDE-Konfiguration

Für die beste Entwicklungserfahrung empfehlen wir:

- 🟣 VS Code mit rust-analyzer-Erweiterung
- 🟠 Rust Rover von JetBrains
- 🔵 Cursor IDE (empfohlen)

---

## 📚 Dokumentation

Für detailliertere Informationen beziehen Sie sich bitte auf:

| Dokument | Beschreibung |
|----------|--------------|
| [🏛️ Architekturübersicht](docs/ARCHITECTURE.md) | Detaillierte Architekturinformationen |
| [🎯 Projektziele](docs/GOALS.md) | Ausführliche Projektziele und Vision |
| [💻 Entwicklungsrichtlinien](docs/DEVELOPMENT.md) | Anleitungen und Best Practices für Entwickler |
| [📅 Meilensteine und Roadmap](docs/MILESTONES.md) | Projektplanung und Fortschrittsverfolgung |
| [🧪 Testrichtlinien](docs/TESTS.md) | Umfassende Teststrategien und -richtlinien |

---

## 📜 Lizenz

[Apache License 2.0](LICENSE)

---

## 📬 Kontakt

Für Fragen und Support kontaktieren Sie bitte [Michael Walloschke](mailto:michael.walloschke@axians.de)

---

<div align="center">
<b>ACCI Framework</b> - Entwickelt mit 💙 und 🦀
</div>
