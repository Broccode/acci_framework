# ACCI Framework

A robust, flexible and secure Enterprise Application Framework built with modern technologies for scalable business applications.

## Project Goals

The ACCI Framework is designed to serve as a robust foundation for various business applications. It is tailored for organizations seeking a flexible, secure, and scalable platform that streamlines software development and operation.

### Core Objectives

- **Flexibility and Reusability**: Adaptable framework that grows with diverse products and evolving requirements through a modular architecture
- **Security and Compliance**: Enterprise-grade security with MFA, encryption, and compliance with regulations like GDPR
- **Scalability and Availability**: Support for growing user numbers and data volumes with high availability features
- **Integration and Extensibility**: Seamless integration with existing systems and extensibility through a plugin architecture
- **User-Centric Experience**: Intuitive interfaces with multi-language support and efficient automated workflows

## Technical Stack

The ACCI Framework is built on modern technologies for optimal performance, security, and scalability:

### Backend

- **Rust**: Core backend language, providing memory safety and high performance
- **Axum**: Web framework for API development
- **Domain-Driven Design (DDD)**: Clear modeling of business domains
- **Event Sourcing & CQRS**: Storage of state changes as events for traceability and separation of read/write operations

### Frontend

- **Leptos**: Modern Rust-based web framework
- **WebAssembly**: For high-performance client-side processing

### Data Storage

- **PostgreSQL**: Primary database for persistent storage
- **Redis**: For caching and session management

### Architecture Patterns

- **Multi-Tenancy**: Shared platform with isolated data per tenant
- **Plugin Architecture**: Extensible business logic via modular plugins
- **Dual API Exposure**: Both REST and GraphQL interfaces

### Deployment & Infrastructure

- **Docker & Docker Compose**: For containerization and orchestration
- **Zero-Downtime Deployment**: With rollbacks and health checks
- **SBOM Management**: Software Bill of Materials for security tracking

## Project Timeline

The development is planned in several key milestones:

1. **Foundation and Basic Authentication** (Q1 2025): Core framework, authentication, and session management
2. **Multi-Tenancy and Enhanced Security** (Q1 2025): Tenant isolation and security features
3. **Core Business Logic and DDD Implementation** (Q2 2025): Event Sourcing, CQRS, and plugin architecture
4. **Integration and Extensibility** (Q2 2025): External system integrations and GraphQL API

## Development Setup

### Required Tools

- **Rust Nightly**: The specific version is defined in `rust-toolchain.toml`
- **Docker and Docker Compose**: For containerization and local development
- **Code Quality Tools**: rustfmt, clippy, rust-analyzer
- **Security Tools**: cargo-audit, cargo-deny, cargo-cyclonedx
- **Testing Tools**: cargo-llvm-cov, cargo-mutants, cargo-nextest
- **Database Tools**: sqlx-cli for migrations and schema management

### Installation Steps

1. **Install Rust**:

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install Required Cargo Tools**:

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

3. **Install Docker and Docker Compose** following the official installation guides:
   - [Docker Installation Guide](https://docs.docker.com/get-docker/)
   - [Docker Compose Installation Guide](https://docs.docker.com/compose/install/)

4. **Setup Development Environment**:

   ```bash
   # Clone the repository
   git clone https://github.com/your-org/acci-framework.git
   cd acci-framework

   # Build the project
   make dev

   # Run tests
   make test
   ```

### IDE Configuration

For the best development experience, we recommend:

- VS Code with rust-analyzer extension
- Rust Rover from JetBrains
- Cursor IDE (recommended)

## Documentation

For more detailed information, please refer to:

- [Architecture Overview](docs/ARCHITECTURE.md)
- [Project Goals](docs/GOALS.md)
- [Development Guidelines](docs/DEVELOPMENT.md)
- [Milestones and Roadmap](docs/MILESTONES.md)
- [Testing Guidelines](docs/TESTS.md)

## License

[Apache License 2.0](LICENSE)

## Contact

For questions and support, please contact [Michael Walloschke](mailto:michael.walloschke@axians.de)
