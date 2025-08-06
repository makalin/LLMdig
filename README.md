# LLMdig — LLM over DNS 🔍🧠

LLMdig is a **high-performance DNS TXT server** that lets you query large language models (LLMs) through standard DNS commands. It transforms DNS queries into LLM requests and returns responses via DNS TXT records.

```bash
dig @localhost -p 9000 "what is the meaning of life" TXT +short
```

Perfect for environments where only DNS traffic is allowed, air-gapped networks, or simply for the novelty of querying AI through DNS!

---

## 🚀 Features

### Core Functionality
* 📡 **LLM-over-DNS** communication with real DNS server
* 🧠 **Multiple LLM Backends**: OpenAI, Ollama, and custom endpoints
* ⚡ **Async Rust** stack for high performance
* 🔄 **On-demand generation** with intelligent caching
* 🔐 **Built-in security**: Rate limiting, input sanitization, and validation

### Advanced Capabilities
* 📊 **Real-time metrics** and performance monitoring
* 🗄️ **Advanced caching** with TTL and LRU eviction
* 🔒 **Encryption utilities** for secure API key management
* 🌐 **Network diagnostics** and connectivity testing
* 📈 **Comprehensive benchmarking** and load testing tools
* 🛡️ **Input validation** and security pattern detection

### Production Ready
* 🐳 **Docker support** with multi-stage builds
* ☸️ **Kubernetes manifests** for orchestration
* 🔧 **Systemd service** configuration
* 📋 **Health checks** and monitoring
* 🔄 **Auto-scaling** and load balancing support

---

## 📦 Installation

### Prerequisites

* Rust 1.75+ ([https://rustup.rs](https://rustup.rs))
* LLM API key (OpenAI, Ollama, or custom endpoint)
* `dig` command (for testing)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/makalin/LLMdig.git
cd LLMdig

# Build and run
cargo run --release

# Or use Docker
docker-compose up -d
```

The server runs on `0.0.0.0:9000` by default.

---

## ⚙️ Configuration

### Environment Variables

```bash
# Server configuration
PORT=9000
HOST=0.0.0.0
MAX_CONNECTIONS=1000
TIMEOUT_SECONDS=30

# LLM configuration
LLM_BACKEND=openai  # openai, ollama, custom
OPENAI_API_KEY=sk-xxxxx
LLM_MODEL=gpt-3.5-turbo
LLM_MAX_TOKENS=1000
LLM_TEMPERATURE=0.7

# Rate limiting
RATE_LIMIT_ENABLED=true
RATE_LIMIT_REQUESTS_PER_MINUTE=60
RATE_LIMIT_BURST_SIZE=10

# Logging
RUST_LOG=info
```

### Configuration File (`config.toml`)

```toml
[server]
host = "0.0.0.0"
port = 9000
max_connections = 1000
timeout_seconds = 30

[llm]
backend = "openai"
model = "gpt-3.5-turbo"
max_tokens = 1000
temperature = 0.7
timeout_seconds = 30

[rate_limit]
enabled = true
requests_per_minute = 60
burst_size = 10
```

---

## 🧪 Usage Examples

### Basic Queries

```bash
# Simple question
dig @localhost -p 9000 "what is the weather today" TXT +short

# Complex query
dig @localhost -p 9000 "explain quantum computing in simple terms" TXT +short

# Multi-word queries (use dots or hyphens)
dig @localhost -p 9000 "how.many.stars.are.there.in.the.universe" TXT +short
```

### Advanced Queries

```bash
# Health check
dig @localhost -p 9000 "health.check" TXT +short

# System information
dig @localhost -p 9000 "system.status" TXT +short

# Custom prompts
dig @localhost -p 9000 "write.a.haiku.about.rust" TXT +short
```

---

## 🛠️ Tools & Utilities

### Built-in Tools

#### DNS Client Tool
```bash
# Build tools
cd tools && cargo build --release

# Health check
./target/release/dns-client health

# Single query
./target/release/dns-client query "what is the weather"

# Performance test
./target/release/dns-client perf --requests 1000 --concurrent 20

# Batch queries
./target/release/dns-client batch domains.txt --concurrent 10
```

#### Monitoring Scripts
```bash
# Real-time monitoring
./scripts/monitor.sh

# Performance benchmarking
./scripts/benchmark.sh

# Run examples
./examples/query_examples.sh
```

### Makefile Commands

```bash
# Build and run
make run

# Run tests
make test

# Build tools
make tools

# Start monitoring
make monitor

# Run benchmarks
make benchmark

# Docker operations
make docker-build
make docker-run
```

---

## 🔧 Development

### Project Structure

```
LLMdig/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library root
│   ├── config.rs            # Configuration management
│   ├── dns.rs               # DNS request handling
│   ├── llm.rs               # LLM backend integration
│   ├── server.rs            # DNS server implementation
│   ├── error.rs             # Error handling
│   └── utils/               # Utility modules
│       ├── metrics.rs       # Performance metrics
│       ├── cache.rs         # Advanced caching
│       ├── network.rs       # Network utilities
│       ├── validation.rs    # Input validation
│       ├── encryption.rs    # Security utilities
│       ├── rate_limiter.rs  # Rate limiting
│       └── sanitizer.rs     # Input sanitization
├── tests/                   # Test suites
├── tools/                   # Custom tools
├── scripts/                 # Utility scripts
├── docs/                    # Documentation
├── examples/                # Usage examples
└── k8s/                     # Kubernetes manifests
```

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test "*"

# All tests with features
cargo test --all-features

# Run test script
./scripts/test.sh
```

---

## 🐳 Deployment

### Docker

```bash
# Build image
docker build -t llmdig .

# Run container
docker run -d -p 9000:9000/udp \
  -e OPENAI_API_KEY=sk-xxxxx \
  llmdig

# Or use Docker Compose
docker-compose up -d
```

### Kubernetes

```bash
# Deploy to cluster
kubectl apply -f k8s/

# Check status
kubectl get pods -l app=llmdig
```

### Systemd Service

```bash
# Install service
sudo cp systemd/llmdig.service /etc/systemd/system/
sudo systemctl enable llmdig
sudo systemctl start llmdig
```

---

## 📊 Monitoring & Observability

### Metrics Dashboard

LLMdig provides comprehensive metrics including:
- Request/response statistics
- Cache hit/miss rates
- LLM backend performance
- Rate limiting statistics
- Error rates and types

### Health Checks

```bash
# Basic health check
dig @localhost -p 9000 "health.check" TXT +short

# Detailed status
dig @localhost -p 9000 "system.status" TXT +short
```

### Logging

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Structured logging with JSON
RUST_LOG=info cargo run
```

---

## 🔒 Security Features

### Input Validation
- DNS query sanitization
- Malicious pattern detection
- Length and character validation
- SQL injection prevention

### Rate Limiting
- Per-client token bucket algorithm
- Configurable limits and burst sizes
- Automatic cleanup of expired tokens

### Encryption
- Secure API key storage
- Encrypted configuration values
- Certificate management utilities

---

## 🚀 Performance

### Benchmarks

LLMdig is designed for high performance:
- **10,000+ requests/second** on modern hardware
- **Sub-10ms latency** for cached responses
- **Efficient memory usage** with LRU caching
- **Async I/O** for concurrent request handling

### Optimization Tips

```bash
# Use release build for production
cargo run --release

# Enable optimizations
export RUSTFLAGS="-C target-cpu=native"

# Monitor performance
./scripts/benchmark.sh --duration 300 --concurrent 50
```

---

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Fork and clone
git clone https://github.com/your-username/LLMdig.git
cd LLMdig

# Install dependencies
cargo build

# Run tests
cargo test

# Check code quality
cargo clippy
cargo fmt --check
```

---

## 📚 Documentation

- [API Documentation](docs/API.md) - Complete API reference
- [Deployment Guide](docs/DEPLOYMENT.md) - Production deployment
- [Tools Guide](tools/README.md) - Custom tools and utilities
- [Contributing Guide](CONTRIBUTING.md) - Development guidelines

---

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

## 🙏 Acknowledgments

- Built with [trust-dns](https://github.com/bluejekyll/trust-dns)
- Inspired by DNS-based communication protocols
- Community contributors and feedback

---

**Happy DNS querying! 🎉**
