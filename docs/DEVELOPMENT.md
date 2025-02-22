# Development Setup

This document describes all necessary tools and setup steps for developing the ACCI Framework.

## Required Tools

### Rust and Cargo

The project uses the nightly Rust toolchain. The specific version and components are defined in `rust-toolchain.toml`.

### Code Quality Tools

- **rustfmt**: Code formatting tool that ensures consistent code style across the project
- **clippy**: Rust linter that helps catch common mistakes and enforces best practices
- **rust-analyzer**: Language Server Protocol (LSP) implementation for IDE support

### Security Tools

- **cargo-audit**: Checks dependencies for known security vulnerabilities
- **cargo-deny**: Enforces dependency policies and controls allowed/denied dependencies
- **cargo-cyclonedx**: Generates Software Bill of Materials (SBOM) in CycloneDX format

### Testing Tools

- **cargo-tarpaulin**: Generates code coverage reports for the test suite
- **cargo-mutants**: Performs mutation testing to assess test suite effectiveness
- **cargo-nextest**: Provides a more feature-rich test runner with better reporting
- **criterion**: Framework for writing and running benchmarks
- **proptest**: Property-based testing framework
- **afl**: American Fuzzy Lop integration for fuzzing tests
- **arbitrary**: Structure-aware fuzzing for security testing

### Database Tools

- **sqlx-cli**: CLI for SQLx, used for database migrations and schema management

### Monitoring and Metrics

- **metrics-rs**: Collects application metrics (Rate, Errors, Duration)
- **metrics-exporter-prometheus**: Exports metrics in Prometheus format

### Container Tools

- **Docker**: Required for containerization (must be installed separately)
- **Docker Compose**: Required for local development (must be installed separately)

## Installation

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Install Required Cargo Tools

```bash
# Install development tools
cargo install \
    cargo-audit \
    cargo-deny \
    cargo-cyclonedx \
    cargo-tarpaulin \
    cargo-mutants \
    cargo-nextest \
    sqlx-cli
```

### 3. Add Required Development Dependencies

Add these testing and benchmarking libraries to your project's `Cargo.toml`:

```toml
[dev-dependencies]
criterion = "0.5"
proptest = "1.6"
afl = "0.15"
```

Or use cargo-add to add them:

```bash
cargo add --dev criterion proptest afl
```

### 4. Install Docker and Docker Compose

Please follow the official installation instructions for your operating system:

- [Docker Installation Guide](https://docs.docker.com/get-docker/)
- [Docker Compose Installation Guide](https://docs.docker.com/compose/install/)

### 5. Configure IDE

For the best development experience, we recommend using an IDE with Rust support through rust-analyzer. Popular choices include:

- VS Code with rust-analyzer extension
- IntelliJ IDEA with Rust plugin
- Cursor IDE (recommended)

## Development Workflow

1. Clone the repository
2. Run `cargo build` to ensure everything compiles
3. Run `cargo test` to run the test suite
4. Start coding!

For more detailed information about the project architecture and goals, please refer to:

- `docs/ARCHITECTURE.md`
- `docs/GOALS.md`
- `docs/MILESTONES.md`
