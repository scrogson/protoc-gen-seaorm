# Justfile for protoc-gen-seaorm

# Default recipe - show available commands
default:
    @just --list

# Format all Rust code
fmt:
    cargo fmt
    cd examples/database && cargo fmt

# Check formatting without modifying files
fmt-check:
    cargo fmt --check
    cd examples/database && cargo fmt --check

# Run clippy linter
lint:
    cargo clippy -- -D warnings
    cd examples/database && cargo clippy -- -D warnings

# Run all tests
test: build
    cargo test
    cd examples/database && buf generate && cargo test

# Build the project
build:
    cargo build --release

# Build and run the example
example: build
    cd examples/database && buf generate && cargo run

# Run all CI checks (fmt, lint, test)
ci: fmt-check lint test
    @echo "All CI checks passed!"

# Clean build artifacts
clean:
    cargo clean
    cd examples/database && cargo clean
    rm -rf gen/

# Generate entities from proto files
generate: build
    buf generate
    cd examples/database && buf generate
