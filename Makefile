.PHONY: dev test test-unit test-integration sqlx-prepare db-up db-down db-reset clippy fmt coverage coverage-html fix prepare-commit benchmark help

# Development Environment Variables
export DATABASE_URL=postgres://acci:acci@localhost:15432/acci_test
export SQLX_OFFLINE=true

help:
	@echo "Available commands:"
	@echo "  make dev          - Start development environment"
	@echo "  make test         - Run all tests"
	@echo "  make test-unit    - Run unit tests only"
	@echo "  make test-integration - Run integration tests only"
	@echo "  make test-e2e     - Run end-to-end tests only"
	@echo "  make benchmark    - Run performance benchmarks"
	@echo "  make sqlx-prepare - Prepare SQLx offline mode"
	@echo "  make db-up        - Start database container"
	@echo "  make db-down      - Stop database container"
	@echo "  make db-reset     - Reset database (drop and recreate)"
	@echo "  make clippy       - Run clippy with all targets"
	@echo "  make fmt          - Format all code"
	@echo "  make coverage     - Generate LCOV coverage report"
	@echo "  make coverage-html - Generate HTML coverage report"
	@echo "  make fix          - Run cargo fix"
	@echo "  make prepare-commit - Run all checks before commit"

dev: db-up sqlx-prepare

db-up:
	docker run --name acci-test-db -e POSTGRES_USER=acci -e POSTGRES_PASSWORD=acci -e POSTGRES_DB=acci_test -p 15432:5432 -d postgres:16-alpine
	sleep 3
	sqlx migrate run --source migrations --database-url ${DATABASE_URL}

db-down:
	docker stop acci-test-db || true
	docker rm acci-test-db || true

db-reset: db-down db-up

sqlx-prepare: db-reset
	@for pkg in auth; do \
		echo "Preparing SQLx queries for package $$pkg"; \
		SQLX_OFFLINE=false cargo sqlx prepare --workspace --database-url ${DATABASE_URL} -- --manifest-path crates/$$pkg/Cargo.toml --all-targets --tests || exit $$?; \
	done
	@echo "SQLx preparation complete!"

test: test-unit test-integration

test-unit:
	SQLX_OFFLINE=true cargo nextest run --workspace --all-features --lib --bins --exclude acci_tests

test-integration:
	SQLX_OFFLINE=true cargo nextest run -p acci_tests --lib --all-features

test-e2e:
	SQLX_OFFLINE=true cargo nextest run --test '*' --features e2e

clippy:
	SQLX_OFFLINE=true cargo clippy --workspace --lib --bins --fix --allow-dirty --allow-staged --all-features --exclude acci_tests -- -D warnings

coverage:
	SQLX_OFFLINE=true cargo llvm-cov --workspace --all-features --profile coverage nextest --lcov --output-path lcov.info
	@echo "Coverage info written to lcov.info"

coverage-html:
	SQLX_OFFLINE=true cargo llvm-cov --workspace --all-features --profile coverage nextest --html
	@echo "HTML coverage report generated in target/llvm-cov/html/index.html"

fmt:
	@echo "Running cargo fmt..."
	cargo fmt --all
	@echo "Formatting individual Rust files with edition 2024..."
	@find . -name "*.rs" -not -path "*/target/*" -type f -exec sh -c 'rustfmt --edition 2024 --check "{}" >/dev/null 2>&1 || rustfmt --edition 2024 "{}"' \;
	@echo "Code formatting complete."

fix:
	SQLX_OFFLINE=true cargo fix --broken-code --allow-dirty --allow-staged --workspace --all-targets --all-features --exclude acci_tests
	@echo "Code fixing complete."

benchmark:
	@echo "Running performance benchmarks..."
	SQLX_OFFLINE=true cargo bench
	@echo "Performance benchmarks complete."
	@echo "Benchmark reports are available in target/criterion/"

prepare-commit:
	$(MAKE) fmt
	$(MAKE) fix
	$(MAKE) clippy
	$(MAKE) test-unit
