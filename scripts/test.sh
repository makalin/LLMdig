#!/bin/bash

set -e

echo "🧪 Running LLMdig tests..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Cargo.toml not found. Please run this script from the project root."
    exit 1
fi

# Run cargo check
echo "🔍 Checking code..."
if cargo check; then
    print_status "Code check passed"
else
    print_error "Code check failed"
    exit 1
fi

# Run clippy
echo "🔧 Running clippy..."
if cargo clippy --all-features -- -D warnings; then
    print_status "Clippy passed"
else
    print_error "Clippy failed"
    exit 1
fi

# Run formatting check
echo "📝 Checking code formatting..."
if cargo fmt -- --check; then
    print_status "Code formatting is correct"
else
    print_warning "Code formatting issues found. Run 'cargo fmt' to fix."
fi

# Run unit tests
echo "🧪 Running unit tests..."
if cargo test --lib; then
    print_status "Unit tests passed"
else
    print_error "Unit tests failed"
    exit 1
fi

# Run integration tests
echo "🔗 Running integration tests..."
if cargo test --test "*"; then
    print_status "Integration tests passed"
else
    print_error "Integration tests failed"
    exit 1
fi

# Run all tests with features
echo "🚀 Running all tests with features..."
if cargo test --all-features; then
    print_status "All tests passed"
else
    print_error "Some tests failed"
    exit 1
fi

# Check if .env file exists
if [ ! -f ".env" ]; then
    print_warning ".env file not found. Copy env.example to .env and configure your settings."
fi

# Check if config.toml exists
if [ ! -f "config.toml" ]; then
    print_warning "config.toml not found. Using default configuration."
fi

# Build release version
echo "🏗️ Building release version..."
if cargo build --release; then
    print_status "Release build successful"
else
    print_error "Release build failed"
    exit 1
fi

# Check binary size
BINARY_SIZE=$(stat -f%z target/release/llmdig 2>/dev/null || stat -c%s target/release/llmdig 2>/dev/null || echo "unknown")
echo "📦 Binary size: ${BINARY_SIZE} bytes"

echo ""
echo "🎉 All tests passed! LLMdig is ready to run."
echo ""
echo "To start the server:"
echo "  cargo run --release"
echo ""
echo "To test with dig:"
echo "  dig @localhost -p 9000 'what.is.the.weather.com' TXT +short" 