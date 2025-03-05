#!/bin/bash
# Performance test script for ACCI Framework
# This script runs performance tests on the authentication endpoints

set -e

echo "=== ACCI Framework Performance Test ==="
echo "Running performance tests..."

# Create directory for reports
mkdir -p reports/performance

# Check for hey
if ! command -v hey &> /dev/null; then
    echo "hey not found, installing..."
    go install github.com/rakyll/hey@latest
fi

# Check if the server is running
if ! curl -s http://localhost:8080/health > /dev/null; then
    echo "Server is not running. Please start the server before running performance tests."
    exit 1
fi

echo ""
echo "=== Running login endpoint performance test ==="
hey -n 100 -c 10 -m POST -H "Content-Type: application/json" \
    -d '{"email":"test@example.com","password":"password123"}' \
    http://localhost:8080/api/auth/login > reports/performance/login.txt

echo ""
echo "=== Running registration endpoint performance test ==="
# Generate random emails for registration
for i in {1..100}; do
    email="test$i@example.com"
    hey -n 1 -c 1 -m POST -H "Content-Type: application/json" \
        -d "{\"email\":\"$email\",\"password\":\"password123\",\"password_confirmation\":\"password123\"}" \
        http://localhost:8080/api/auth/register >> reports/performance/registration.txt
done

echo ""
echo "=== Running session validation performance test ==="
# Get a session token first
TOKEN=$(curl -s -X POST -H "Content-Type: application/json" \
    -d '{"email":"test@example.com","password":"password123"}' \
    http://localhost:8080/api/auth/login | jq -r '.data.token')

hey -n 100 -c 10 -H "Authorization: Bearer $TOKEN" \
    http://localhost:8080/api/auth/validate > reports/performance/validation.txt

echo ""
echo "=== Performance test complete ==="
echo "Reports saved to reports/performance/"

# Analyze results
echo ""
echo "=== Performance Test Results ==="
echo ""
echo "Login Endpoint:"
grep "Requests/sec" reports/performance/login.txt
grep "Average" reports/performance/login.txt | head -n 1

echo ""
echo "Registration Endpoint:"
grep "Requests/sec" reports/performance/registration.txt | tail -n 1
grep "Average" reports/performance/registration.txt | tail -n 2 | head -n 1

echo ""
echo "Session Validation Endpoint:"
grep "Requests/sec" reports/performance/validation.txt
grep "Average" reports/performance/validation.txt | head -n 1
