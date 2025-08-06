# Multi-stage build for LLMdig
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release

# Remove dummy main.rs and copy real source code
RUN rm src/main.rs
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false llmdig

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/llmdig /usr/local/bin/

# Copy configuration files
COPY config.toml ./
COPY env.example ./

# Create data directory
RUN mkdir -p /app/data && chown -R llmdig:llmdig /app

# Switch to non-root user
USER llmdig

# Expose DNS port
EXPOSE 9000/udp

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD dig @localhost -p 9000 "health.check" TXT +short || exit 1

# Default command
CMD ["llmdig", "--config", "config.toml"] 