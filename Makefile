.PHONY: help build test run clean release install uninstall

# Default target
help:
	@echo "LLMdig - LLM over DNS"
	@echo ""
	@echo "Available targets:"
	@echo "  build     - Build the project in debug mode"
	@echo "  release   - Build the project in release mode"
	@echo "  test      - Run all tests"
	@echo "  run       - Run the server in debug mode"
	@echo "  run-release - Run the server in release mode"
	@echo "  clean     - Clean build artifacts"
	@echo "  install   - Install the binary to ~/.cargo/bin"
	@echo "  uninstall - Remove the binary from ~/.cargo/bin"
	@echo "  check     - Run cargo check"
	@echo "  clippy    - Run cargo clippy"
	@echo "  fmt       - Format code with rustfmt"

# Build targets
build:
	cargo build

release:
	cargo build --release

# Test targets
test:
	cargo test --all-features

test-unit:
	cargo test --lib

test-integration:
	cargo test --test "*"

# Run targets
run:
	cargo run

run-release:
	cargo run --release

# Development targets
check:
	cargo check

clippy:
	cargo clippy --all-features -- -D warnings

fmt:
	cargo fmt

fmt-check:
	cargo fmt -- --check

# Tools targets
tools:
	cd tools && cargo build --release

tools-test:
	cd tools && cargo test

tools-clean:
	cd tools && cargo clean

# Clean targets
clean:
	cargo clean

# Install targets
install: release
	cargo install --path .

uninstall:
	cargo uninstall llmdig

# Docker targets
docker-build:
	docker build -t llmdig .

docker-run:
	docker run -p 9000:9000 --env-file .env llmdig

# Documentation
docs:
	cargo doc --open

# Linting and formatting
lint: clippy fmt-check

# Pre-commit checks
pre-commit: fmt clippy test

# Development setup
setup:
	rustup component add rustfmt clippy
	cargo install cargo-watch

# Watch for changes and run tests
watch:
	cargo watch -x check -x test -x run

# Monitoring and benchmarking
monitor:
	./scripts/monitor.sh

benchmark:
	./scripts/benchmark.sh

test-examples:
	./examples/query_examples.sh 