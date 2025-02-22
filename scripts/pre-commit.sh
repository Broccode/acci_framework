#!/usr/bin/env bash

# Exit on error
set -e

# Get the root directory of the git repository
ROOT_DIR=$(git rev-parse --show-toplevel)

# Change to the root directory
cd "$ROOT_DIR"

# Run the prepare-commit target
echo "Running pre-commit checks..."
make prepare-commit
echo "Pre-commit checks completed."

# If we got here, the checks passed
exit 0
