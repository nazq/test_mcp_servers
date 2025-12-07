#!/bin/bash
# Verification script for CI/CD setup
# Run this locally to verify all CI checks will pass

set -e

echo "🔍 Verifying CI/CD setup..."
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track failures
FAILED=0

# Function to run check
run_check() {
    local name="$1"
    local command="$2"

    echo -e "${YELLOW}Running: $name${NC}"
    if eval "$command"; then
        echo -e "${GREEN}✓ $name passed${NC}"
        echo ""
        return 0
    else
        echo -e "${RED}✗ $name failed${NC}"
        echo ""
        FAILED=$((FAILED + 1))
        return 1
    fi
}

# 1. Format check
run_check "Format Check" "cargo fmt --all -- --check"

# 2. Clippy
run_check "Clippy Lints" "cargo clippy --all-targets --all-features -- -D warnings"

# 3. Tests
run_check "Test Suite" "cargo test --all-features --verbose"

# 4. Build check
run_check "Build Check" "cargo build --release"

# 5. Docker build (if docker is available)
if command -v docker &> /dev/null; then
    run_check "Docker Build" "docker build -t mcp-test-server:test ."
else
    echo -e "${YELLOW}⚠ Docker not found, skipping Docker build check${NC}"
    echo ""
fi

# 6. Coverage check (if cargo-llvm-cov is installed)
if command -v cargo-llvm-cov &> /dev/null; then
    echo -e "${YELLOW}Running: Coverage Check${NC}"
    cargo llvm-cov --all-features --workspace --json --summary-only > /tmp/coverage.json
    coverage=$(jq -r '.data[0].totals.lines.percent' /tmp/coverage.json)
    echo "Current coverage: ${coverage}%"

    if (( $(echo "$coverage < 90" | bc -l) )); then
        echo -e "${RED}✗ Coverage ${coverage}% is below threshold of 90%${NC}"
        FAILED=$((FAILED + 1))
    else
        echo -e "${GREEN}✓ Coverage check passed (${coverage}%)${NC}"
    fi
    echo ""
else
    echo -e "${YELLOW}⚠ cargo-llvm-cov not found, skipping coverage check${NC}"
    echo "Install with: cargo install cargo-llvm-cov"
    echo ""
fi

# Summary
echo "=========================================="
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed!${NC}"
    echo "Your changes are ready for CI/CD pipeline"
    exit 0
else
    echo -e "${RED}✗ $FAILED check(s) failed${NC}"
    echo "Please fix the issues before pushing"
    exit 1
fi
