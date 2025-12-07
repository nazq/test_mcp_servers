# MCP Test Server - Development Tasks

# List available recipes
default:
    @just --list

# Run all tests
test:
    cargo test --all-features

# Run tests with output
test-verbose:
    cargo test --all-features -- --nocapture

# Format code
fmt:
    cargo fmt --all

# Check formatting without modifying
fmt-check:
    cargo fmt --all -- --check

# Run clippy lints (after formatting)
clippy: fmt
    cargo clippy --all-targets --all-features -- -D warnings

# Run all checks (fmt + clippy + test)
check: fmt-check clippy test
    @echo "All checks passed!"

# Generate coverage report
coverage:
    cargo llvm-cov --all-features --html --open

# Build release
build:
    cargo build --release

# Build Docker image
docker:
    docker build -t mcp-test-server .

# Run the server
run:
    cargo run --release

# Clean build artifacts
clean:
    cargo clean
