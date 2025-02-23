.PHONY: dev test test-unit test-integration sqlx-prepare db-up db-down db-reset help

# Development Environment Variables
export DATABASE_URL=postgres://acci:acci@localhost:15432/acci_test

help:
	@echo "Available commands:"
	@echo "  make dev          - Start development environment"
	@echo "  make test         - Run all tests"
	@echo "  make test-unit    - Run unit tests only"
	@echo "  make test-integration - Run integration tests only"
	@echo "  make sqlx-prepare - Prepare SQLx offline mode"
	@echo "  make db-up        - Start database container"
	@echo "  make db-down      - Stop database container"
	@echo "  make db-reset     - Reset database (drop and recreate)"

dev: db-up sqlx-prepare

db-up:
	docker run --name acci-test-db -e POSTGRES_USER=acci -e POSTGRES_PASSWORD=acci -e POSTGRES_DB=acci_test -p 15432:5432 -d postgres:16-alpine
	sleep 3
	cd crates/core && sqlx migrate run --database-url ${DATABASE_URL}

db-down:
	docker stop acci-test-db || true
	docker rm acci-test-db || true

db-reset: db-down db-up

sqlx-prepare:
	cargo sqlx prepare --workspace --database-url ${DATABASE_URL}

test: test-unit test-integration

test-unit:
	SQLX_OFFLINE=true cargo test --lib --bins --all-features --workspace --exclude acci_tests

test-integration:
	SQLX_OFFLINE=true cargo test -p acci_tests --lib --all-features
