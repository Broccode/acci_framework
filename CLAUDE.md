# ACCI Framework Development Guide

## Project Description

The ACCI Framework is a modular, enterprise-grade application development platform built in Rust, designed following Domain-Driven Design principles. It features a layered architecture with four main components:

- **core**: Provides foundational infrastructure for database connectivity, error handling, and telemetry
- **auth**: Implements comprehensive authentication and authorization with JWT, session management, and password security
- **api**: Offers a flexible RESTful API layer using Axum with standardized error handling, validation, and monitoring
- **web**: Delivers server-side rendered web interfaces using Leptos SSR

The framework emphasizes security, performance, and scalability through multi-tenancy support, event sourcing, and CQRS patterns. It's designed for enterprise applications requiring strong compliance (GDPR, ISO 27001), internationalization, and seamless integration with external systems.

### Core Crate (acci_core)

The core crate serves as the foundational layer of the ACCI Framework, providing essential infrastructure components that other crates depend on. It implements database connectivity through a PostgreSQL abstraction using SQLx, with built-in migration capabilities to ensure schema consistency. The crate offers standardized error handling with a comprehensive Error enum using thiserror for type-safe errors and anyhow for propagation. Its telemetry module establishes robust observability through structured logging with tracing and metrics collection via prometheus, enabling comprehensive system monitoring. Configuration management is implemented through environment variables using dotenvy, with serialization/deserialization via serde. The crate follows a clean architecture approach, exposing well-defined interfaces while hiding implementation details, making it the stable foundation upon which higher-level crates build their functionality.

### Auth Crate (acci_auth)

The auth crate implements a comprehensive authentication and authorization system built around user identity, credentials, and session management. It provides secure password handling with argon2 for hashing and zxcvbn for strength validation, along with JWT-based token management for authentication via the jsonwebtoken library. The crate implements a complete user lifecycle including registration, login, session validation, and account management through a domain-driven design with clear separation between models, repositories, and services. Session management includes support for multiple devices, fingerprinting, and automatic invalidation, with PostgreSQL persistence via SQLx. The crate integrates with acci_core for database connectivity and error handling while exposing a clean API that higher-level crates like acci_api and acci_web can consume, maintaining security best practices throughout with proper rate limiting via governor.

### API Crate (acci_api)

The API crate provides a comprehensive REST API infrastructure built on Axum, implementing routing, middleware, and request/response handling. It offers a layered architecture with handler functions decoupled from routing logic, enabling clean separation of concerns and testability. The middleware stack includes sophisticated logging using tracing, error handling that converts domain errors to appropriate HTTP responses, and input validation with the validator crate. Responses follow a standardized format with ApiResponse and ApiError types, ensuring consistent error reporting across endpoints. Authentication integration with acci_auth enables secure login, registration, and token validation. The crate implements Prometheus metrics for performance monitoring and includes documentation generation capabilities. Through its modular design and extensive middleware system, it provides a complete foundation for building RESTful services that other parts of the application can utilize.

### Web Crate (acci_web)

The web crate implements a server-side rendered frontend using Leptos for component-based UI development without WebAssembly. It provides a complete user interface with pages for authentication (login/registration) and application features, structured with reusable components for forms, navigation, and layout elements. The application uses Axum for HTTP routing and request handling, integrating seamlessly with static file serving through tower-http. Authentication flows connect to acci_auth services for credential validation and session management. The architecture follows a clear separation between UI components, page handlers, and service interfaces, with a prelude module for commonly used imports. Designed with server-side rendering, the crate eliminates client-side JavaScript dependencies while maintaining a component-based development model. The modular structure allows for easy expansion as new features are added, with internationalization considerations built into the design.

## Build/Test/Lint Commands

- Build: `cargo build --workspace --all-features`
- Lint: `make clippy`
- Format: `make fmt`
- All tests: `make test`
- Unit tests: `make test-unit`
- Integration tests: `make test-integration`
- E2E tests: `make test-e2e`
- Single test: `cargo nextest run -p <package> --test <test_file> <test_name>`
- Coverage: `make coverage` or `make coverage-html`
- Database: `make db-up`, `make db-down`, `make db-reset`
- Prepare commit: `make prepare-commit`

## Code Style Guidelines

- **Memory Management**: No unsafe blocks without docs; Clear ownership patterns; Implement Drop for resource management
- **Error Handling**: Return Result for fallible operations; Use anyhow for app errors; Use thiserror for library errors; No unwrap/expect in production
- **Types**: Use newtype pattern for constraints; Implement From/Into for conversions; Prefer associated types over generics
- **Documentation**: Document all public APIs with examples; Include runnable examples; Use English only; Follow rustdoc conventions
- **Naming**: Use English identifiers; Follow Rust naming conventions (snake_case functions, CamelCase types)

## Architecture

- DDD (Domain-Driven Design) approach for business domains
- REST and GraphQL API interfaces with versioning
- Multi-tenancy with isolated tenant data
- Event Sourcing and CQRS for state management

## Async Programming

- Use tokio for async runtime
- Prefer Stream over future vectors
- Keep async boundaries at edges
- Use async-trait for async traits
- Never block the async executor
- Handle cancellation gracefully with proper cleanup

## Test Organization

- Unit tests: In source files under `#[cfg(test)]` module
- Integration tests: In `/tests` directory
- Test fixtures: In `/tests/src/fixtures`
- Test helpers: In `/tests/src/helpers`
- Test mocks: In `/tests/src/mocks`
- Unit tests MUST NOT use mocks or external dependencies
- Integration tests SHOULD use testcontainers for external dependencies
- NEVER mix unit and integration tests

## Important Documentation References

### Architecture Documentation

- `/docs/ARCHITECTURE.md` - Comprehensive overview of the system architecture based on the arc42 template, covering introduction, constraints, scope, solution strategy, building blocks, runtime view, deployment, cross-cutting concerns, and architectural decisions.
- `/docs/arc42/ARCH_Section4_SolutionStrategy.md` - Details on architectural principles (modularity, scalability, security), key methodologies (DDD, Event Sourcing, CQRS), and strategic focus areas.
- `/docs/arc42/ARCH_Section5_BuildingBlockView.md` - Describes the primary containers (API, Business Logic, Database, Integration) and key components (User Management, License Management, etc.).
- `/docs/GOALS.md` - Outlines the framework's main goals: flexibility, security, scalability, integration, and user experience; valuable for understanding design priorities.

### Implementation Details

- `/docs/implementation/authentication-workflow.md` - Detailed implementation of the authentication system including registration, login, session management with code examples and security measures.
- `/docs/implementation/api-infrastructure-implementation.md` - Explains API infrastructure with middleware architecture, error handling, request validation, and response formats with practical code examples.
- `/docs/implementation/leptos-frontend-implementation.md` - Details the server-side rendering approach using Leptos framework without WebAssembly for the frontend components.
- `/docs/implementation/auth-repository.md` - Implementation details for authentication data persistence layer with database schema and repository patterns.

### Development Processes

- `/docs/DEVELOPMENT.md` - Lists required tools, installation instructions, and development workflow; essential for setting up the development environment.
- `/docs/CICD_PIPELINE.md` - Continuous integration and deployment processes, including build, test, and deployment stages.
- `/docs/MILESTONES.md` - Project roadmap with incremental development goals and current progress tracking.
- `/docs/templates/task-plan-template.md` - Standard template for planning development tasks, ensuring consistent approach to implementation.

### Testing Strategies

- `/docs/TESTS.md` - Master document on testing strategy with links to detailed documentation for each testing aspect and directory structure.
- `/docs/tests/unit-tests.md` - Guidelines for writing unit tests, including structure, naming conventions, categories, and best practices.
- `/docs/tests/integration-tests.md` - Approach to integration testing with container-based testing of component interactions.
- `/docs/tests/security-tests.md` - Security testing practices including fuzzing, boundary testing, and vulnerability scanning focused on authentication flows.

### Feature Specifications

- `/docs/templates/feature-documentation-template.md` - Template for documenting new features with consistent structure.
- `/docs/milestones/milestone1-foundation-and-auth.md` - Specific implementation details for the foundation and authentication milestone.
- `/docs/API_SPECIFICATION.md` - API design guidelines, endpoints, authentication, and versioning approach.
- `/docs/templates/feature/architecture.md` - Template for documenting feature-specific architectural decisions and patterns.
