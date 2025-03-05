#!/bin/bash
# Security check script for ACCI Framework
# This script performs various security checks on the codebase

set -e

echo "=== ACCI Framework Security Check ==="
echo "Running security checks..."

# Create directory for reports
mkdir -p reports/security

# Check for cargo-audit
if ! command -v cargo-audit &> /dev/null; then
    echo "cargo-audit not found, installing..."
    cargo install cargo-audit
fi

# Check for cargo-deny
if ! command -v cargo-deny &> /dev/null; then
    echo "cargo-deny not found, installing..."
    cargo install cargo-deny
fi

# Check for cargo-geiger
if ! command -v cargo-geiger &> /dev/null; then
    echo "cargo-geiger not found, installing..."
    cargo install cargo-geiger
fi

echo ""
echo "=== Running dependency vulnerability scan ==="
cargo audit --json > reports/security/audit.json
cargo audit

echo ""
echo "=== Running license compliance check ==="
cargo deny check licenses

echo ""
echo "=== Checking for unsafe code ==="
cargo geiger --output-format json > reports/security/unsafe.json
cargo geiger --all-features

echo ""
echo "=== Checking cryptographic parameters ==="
# Check for consistent cryptographic parameters
grep -r "Argon2" --include="*.rs" . > reports/security/crypto_params.txt
grep -r "hash_password" --include="*.rs" . >> reports/security/crypto_params.txt
grep -r "verify_password" --include="*.rs" . >> reports/security/crypto_params.txt

echo ""
echo "=== Checking for hardcoded secrets ==="
# Simple check for potential hardcoded secrets
grep -r "password" --include="*.rs" . | grep -v "fn" | grep -v "struct" | grep -v "enum" > reports/security/potential_secrets.txt
grep -r "secret" --include="*.rs" . | grep -v "fn" | grep -v "struct" | grep -v "enum" >> reports/security/potential_secrets.txt
grep -r "key" --include="*.rs" . | grep -v "fn" | grep -v "struct" | grep -v "enum" >> reports/security/potential_secrets.txt

echo ""
echo "=== Checking for CSRF protection ==="
grep -r "csrf" --include="*.rs" . > reports/security/csrf.txt

echo ""
echo "=== Checking for secure headers ==="
grep -r "headers" --include="*.rs" . | grep -E "security|content-security|x-frame|strict-transport" > reports/security/secure_headers.txt

echo ""
echo "=== Security check complete ==="
echo "Reports saved to reports/security/"
