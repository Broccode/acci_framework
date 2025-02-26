# 🏢 ACCI Framework

> A robust, flexible, and secure Enterprise Application Framework built with modern technologies for scalable business applications.

[![Rust](https://img.shields.io/badge/rust-nightly-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.1-green.svg)](CHANGELOG.md)

## 🌐 Language / Sprache / Gjuha

- [English](#english)
- [Deutsch](#deutsch)
- [Shqip](#shqip)

---

<a name="english"></a>

# 🇬🇧 English

## 📋 Table of Contents

- [Overview](#-overview)
- [Project Goals](#-project-goals)
- [Technical Stack](#-technical-stack)
- [Project Timeline](#-project-timeline)
- [Development Setup](#-development-setup)
- [Documentation](#-documentation)
- [License](#-license)
- [Contact](#-contact)

---

## 🔍 Overview

The ACCI Framework provides a solid foundation for various business applications. It is designed for organizations seeking a flexible, secure, and scalable platform that streamlines software development and operation.

---

## 🎯 Project Goals

### Core Objectives

| Goal | Description |
|------|-------------|
| 🔄 **Flexibility and Reusability** | Adaptable framework that grows with diverse products and evolving requirements through a modular architecture |
| 🔒 **Security and Compliance** | Enterprise-grade security with MFA, encryption, and compliance with regulations like GDPR |
| 📈 **Scalability and Availability** | Support for growing user numbers and data volumes with high availability features |
| 🔌 **Integration and Extensibility** | Seamless integration with existing systems and extensibility through a plugin architecture |
| 👥 **User-Centric Experience** | Intuitive interfaces with multi-language support and efficient automated workflows |

---

## 💻 Technical Stack

The ACCI Platform is built on modern technologies that provide optimal performance, security, and scalability:

### 🔧 Backend

- 🦀 **Rust**: Core backend language, providing memory safety and high performance
- 🌐 **Axum**: Web framework for API development
- 🏗️ **Domain-Driven Design (DDD)**: Clear modeling of business domains
- 📊 **Event Sourcing & CQRS**: Storage of state changes as events for traceability

### 🖥️ Frontend

- ⚡ **Leptos**: Modern Rust-based web framework
- 🧩 **WebAssembly**: For high-performance client-side processing

### 💾 Data Storage

- 🐘 **PostgreSQL**: Primary database for persistent storage
- ⚡ **Redis**: For caching and session management

### 🏛️ Architecture Patterns

- 🏢 **Multi-Tenancy**: Shared platform with isolated data per tenant
- 🧩 **Plugin Architecture**: Extensible business logic via modular plugins
- 🔄 **Dual API Exposure**: Both REST and GraphQL interfaces

### 🚢 Deployment & Infrastructure

- 🐳 **Docker & Docker Compose**: For containerization and orchestration
- 🔄 **Zero-Downtime Deployment**: With rollbacks and health checks
- 📦 **SBOM Management**: Software Bill of Materials for security tracking

---

## 📅 Project Timeline

The development is planned in several key phases:

| Phase | Timeframe | Description |
|-------|-----------|-------------|
| 1️⃣ **Foundation and Basic Authentication** | Q1 2025 | Core framework, authentication, and session management |
| 2️⃣ **Multi-Tenancy and Enhanced Security** | Q1 2025 | Tenant isolation and security features |
| 3️⃣ **Core Business Logic and DDD Implementation** | Q2 2025 | Event Sourcing, CQRS, and plugin architecture |
| 4️⃣ **Integration and Extensibility** | Q2 2025 | External system integrations and GraphQL API |

---

## ⚙️ Development Setup

### 🛠️ Required Tools

- 🦀 **Rust Nightly**: The specific version is defined in `rust-toolchain.toml`
- 🐳 **Docker and Docker Compose**: For containerization and local development
- 🧰 **Code Quality Tools**: rustfmt, clippy, rust-analyzer
- 🔒 **Security Tools**: cargo-audit, cargo-deny, cargo-cyclonedx
- 🧪 **Testing Tools**: cargo-llvm-cov, cargo-mutants, cargo-nextest
- 🗃️ **Database Tools**: sqlx-cli for migrations and schema management

### 📝 Installation Steps

<details>
<summary>1. Install Rust</summary>

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

</details>

<details>
<summary>2. Install Required Cargo Tools</summary>

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
<summary>3. Install Docker and Docker Compose</summary>

Follow the official installation guides:

- [Docker Installation Guide](https://docs.docker.com/get-docker/)
- [Docker Compose Installation Guide](https://docs.docker.com/compose/install/)

</details>

<details>
<summary>4. Setup Development Environment</summary>

```bash
# Clone the repository
git clone https://github.com/your-org/acci-framework.git
cd acci-framework

# Build the project
make dev

# Run tests
make test
```

</details>

### 💻 IDE Configuration

For the best development experience, we recommend:

- 🟣 VS Code with rust-analyzer extension
- 🟠 Rust Rover from JetBrains
- 🔵 Cursor IDE (recommended)

---

## 📚 Documentation

For more detailed information, please refer to:

| Document | Description |
|----------|-------------|
| [🏛️ Architecture Overview](docs/ARCHITECTURE.md) | Detailed architecture information |
| [🎯 Project Goals](docs/GOALS.md) | Comprehensive project goals and vision |
| [💻 Development Guidelines](docs/DEVELOPMENT.md) | Guidelines and best practices for developers |
| [📅 Milestones and Roadmap](docs/MILESTONES.md) | Project planning and progress tracking |
| [🧪 Testing Guidelines](docs/TESTS.md) | Comprehensive testing strategies and guidelines |

---

## 📜 License

[Apache License 2.0](LICENSE)

---

## 📬 Contact

For questions and support, please contact [Michael Walloschke](mailto:michael.walloschke@axians.de)

---

<div align="center">
<b>ACCI Framework</b> - Developed with 💙 and 🦀
</div>

---

<a name="deutsch"></a>

# 🇩🇪 Deutsch

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

---

<a name="shqip"></a>

# 🇦🇱 Shqip

## 📋 Përmbajtja

- [Përmbledhje](#-përmbledhje)
- [Objektivat e Projektit](#-objektivat-e-projektit)
- [Teknologjitë e Përdorura](#-teknologjitë-e-përdorura)
- [Plani Kohor i Projektit](#-plani-kohor-i-projektit)
- [Konfigurimi i Mjedisit të Zhvillimit](#-konfigurimi-i-mjedisit-të-zhvillimit)
- [Dokumentacioni](#-dokumentacioni)
- [Licenca](#-licenca)
- [Kontakti](#-kontakti)

---

## 🔍 Përmbledhje

Korniza ACCI ofron një bazë të fortë për aplikacione të ndryshme biznesi. Është dizajnuar për organizata që kërkojnë një platformë fleksibile, të sigurt dhe të shkallëzueshme që optimizon zhvillimin dhe funksionimin e softuerit.

---

## 🎯 Objektivat e Projektit

### Objektivat Kryesore

| Objektivi | Përshkrimi |
|-----------|------------|
| 🔄 **Fleksibiliteti dhe Ripërdorimi** | Kornizë e përshtatshme që rritet me produkte të ndryshme dhe kërkesa në zhvillim përmes një arkitekture modulare |
| 🔒 **Siguria dhe Përputhshmëria** | Siguri e nivelit të ndërmarrjes me MFA, enkriptim dhe përputhje me rregulloret si GDPR |
| 📈 **Shkallëzueshmëria dhe Disponueshmëria** | Mbështetje për numra në rritje të përdoruesve dhe vëllime të të dhënave me funksione të disponueshmërisë së lartë |
| 🔌 **Integrimi dhe Zgjerueshmëria** | Integrim i lehtë me sistemet ekzistuese dhe zgjerueshmëri përmes një arkitekture plugin |
| 👥 **Përvoja e Përqendruar tek Përdoruesi** | Ndërfaqe intuitive me mbështetje për shumë gjuhë dhe flukse të automatizuara efikase |

---

## 💻 Teknologjitë e Përdorura

Platforma ACCI bazohet në teknologji moderne që ofrojnë performancë, siguri dhe shkallëzueshmëri optimale:

### 🔧 Backend

- 🦀 **Rust**: Gjuha kryesore e backend-it, që ofron siguri të memories dhe performancë të lartë
- 🌐 **Axum**: Kuadër web për zhvillimin e API-ve
- 🏗️ **Domain-Driven Design (DDD)**: Modelim i qartë i domeneve të biznesit
- 📊 **Event Sourcing & CQRS**: Ruajtja e ndryshimeve të gjendjes si ngjarje për gjurmueshmëri

### 🖥️ Frontend

- ⚡ **Leptos**: Kuadër modern web i bazuar në Rust
- 🧩 **WebAssembly**: Për përpunim të performancës së lartë në anën e klientit

### 💾 Ruajtja e të Dhënave

- 🐘 **PostgreSQL**: Baza kryesore e të dhënave për ruajtje të qëndrueshme
- ⚡ **Redis**: Për caching dhe menaxhim të sesionit

### 🏛️ Modelet e Arkitekturës

- 🏢 **Multi-Tenancy**: Platformë e përbashkët me të dhëna të izoluara për çdo qiramarrës
- 🧩 **Arkitektura Plugin**: Logjikë biznesi e zgjerueshme përmes plugineve modulare
- 🔄 **Ekspozimi i Dyfishtë i API-ve**: Ndërfaqe si REST ashtu edhe GraphQL

### 🚢 Dërgimi & Infrastruktura

- 🐳 **Docker & Docker Compose**: Për kontenjerizim dhe orkestrimin
- 🔄 **Dërgimi pa Ndërprerje**: Me kthime pas dhe kontrolle të shëndetit
- 📦 **Menaxhimi SBOM**: Listë Materiale Softuerike për gjurmimin e sigurisë

---

## 📅 Plani Kohor i Projektit

Zhvillimi është planifikuar në disa faza kyçe:

| Faza | Periudha | Përshkrimi |
|------|----------|------------|
| 1️⃣ **Bazat dhe Autentikimi Bazë** | Q1 2025 | Korniza bazë, autentikimi dhe menaxhimi i sesionit |
| 2️⃣ **Multi-Tenancy dhe Siguri e Përmirësuar** | Q1 2025 | Izolimi i qiramarrësve dhe funksione sigurie |
| 3️⃣ **Logjika e Biznesit Bazë dhe Implementimi DDD** | Q2 2025 | Event Sourcing, CQRS dhe arkitektura plugin |
| 4️⃣ **Integrimi dhe Zgjerueshmëria** | Q2 2025 | Integrime me sisteme të jashtme dhe API GraphQL |

---

## ⚙️ Konfigurimi i Mjedisit të Zhvillimit

### 🛠️ Mjetet e Nevojshme

- 🦀 **Rust Nightly**: Versioni specifik është përcaktuar në `rust-toolchain.toml`
- 🐳 **Docker dhe Docker Compose**: Për kontenjerizim dhe zhvillim lokal
- 🧰 **Mjetet e Cilësisë së Kodit**: rustfmt, clippy, rust-analyzer
- 🔒 **Mjetet e Sigurisë**: cargo-audit, cargo-deny, cargo-cyclonedx
- 🧪 **Mjetet e Testimit**: cargo-llvm-cov, cargo-mutants, cargo-nextest
- 🗃️ **Mjetet e Bazës së të Dhënave**: sqlx-cli për migrimet dhe menaxhimin e skemës

### 📝 Hapat e Instalimit

<details>
<summary>1. Instaloni Rust</summary>

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

</details>

<details>
<summary>2. Instaloni Mjetet e Nevojshme Cargo</summary>

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
<summary>3. Instaloni Docker dhe Docker Compose</summary>

Ndiqni udhëzimet zyrtare të instalimit:

- [Udhëzuesi i Instalimit të Docker](https://docs.docker.com/get-docker/)
- [Udhëzuesi i Instalimit të Docker Compose](https://docs.docker.com/compose/install/)

</details>

<details>
<summary>4. Konfiguroni Mjedisin e Zhvillimit</summary>

```bash
# Klononi repozitorinë
git clone https://github.com/your-org/acci-framework.git
cd acci-framework

# Ndërtoni projektin
make dev

# Ekzekutoni testet
make test
```

</details>

### 💻 Konfigurimi IDE

Për përvojën më të mirë të zhvillimit, ne rekomandojmë:

- 🟣 VS Code me zgjatimin rust-analyzer
- 🟠 Rust Rover nga JetBrains
- 🔵 Cursor IDE (rekomanduar)

---

## 📚 Dokumentacioni

Për informacion më të detajuar, ju lutemi referojuni:

| Dokumenti | Përshkrimi |
|-----------|------------|
| [🏛️ Përmbledhja e Arkitekturës](docs/ARCHITECTURE.md) | Informacion i detajuar i arkitekturës |
| [🎯 Objektivat e Projektit](docs/GOALS.md) | Objektivat gjithëpërfshirëse të projektit dhe vizioni |
| [💻 Udhëzimet e Zhvillimit](docs/DEVELOPMENT.md) | Udhëzime dhe praktikat më të mira për zhvilluesit |
| [📅 Gurët Kilometrikë dhe Plani](docs/MILESTONES.md) | Planifikimi i projektit dhe ndjekja e progresit |
| [🧪 Udhëzimet e Testimit](docs/TESTS.md) | Strategji dhe udhëzime gjithëpërfshirëse të testimit |

---

## 📜 Licenca

[Apache License 2.0](LICENSE)

---

## 📬 Kontakti

Për pyetje dhe mbështetje, ju lutemi kontaktoni [Michael Walloschke](mailto:michael.walloschke@axians.de)

---

<div align="center">
<b>ACCI Framework</b> - Zhvilluar me 💙 dhe 🦀
</div>
