# LLMdig Tools

This directory contains various tools for testing, monitoring, and interacting with the LLMdig DNS server.

## DNS Client Tool

A custom DNS client for testing LLMdig server functionality.

### Building

```bash
cd tools
cargo build --release
```

### Usage

#### Basic Query

```bash
# Query a single domain
./target/release/dns-client query "what.is.the.weather.com"

# Query with custom record type
./target/release/dns-client query "example.com" --record-type A

# Query remote server
./target/release/dns-client --host 192.168.1.100 --port 9000 query "test.com"
```

#### Batch Queries

```bash
# Create a file with domains
echo "what.is.the.weather.com
how.many.stars.are.there.com
what.is.the.capital.of.france.com" > domains.txt

# Run batch query
./target/release/dns-client batch domains.txt --concurrent 5
```

#### Health Check

```bash
# Check server health
./target/release/dns-client health

# Health check with custom timeout
./target/release/dns-client --timeout 30 health
```

#### Performance Test

```bash
# Run performance test
./target/release/dns-client perf --requests 1000 --concurrent 20

# Quick performance test
./target/release/dns-client perf --requests 100 --concurrent 10
```

### Command Reference

```bash
dns-client [OPTIONS] <COMMAND>

Commands:
  query   Query a domain
  batch   Batch query multiple domains
  health  Health check
  perf    Performance test

Options:
  -h, --host <HOST>        DNS server host [default: 127.0.0.1]
  -p, --port <PORT>        DNS server port [default: 9000]
  -t, --timeout <TIMEOUT>  Timeout in seconds [default: 10]
  -h, --help               Print help
```

## Scripts

### Test Script

```bash
# Run comprehensive tests
./scripts/test.sh

# Run tests with verbose output
RUST_LOG=debug ./scripts/test.sh
```

### Benchmark Script

```bash
# Run benchmark with defaults
./scripts/benchmark.sh

# Custom benchmark configuration
./scripts/benchmark.sh \
  --host localhost \
  --port 9000 \
  --duration 120 \
  --concurrent 20 \
  --requests 5000
```

### Monitor Script

```bash
# Start real-time monitoring
./scripts/monitor.sh

# Monitor remote server
./scripts/monitor.sh --host 192.168.1.100 --port 9000

# Custom monitoring interval
./scripts/monitor.sh --interval 10

# Custom log files
./scripts/monitor.sh \
  --log-file ./monitor.log \
  --metrics-file ./metrics.json
```

## Example Usage Scenarios

### Development Testing

```bash
# 1. Start LLMdig server
cargo run --release

# 2. In another terminal, run health check
./tools/target/release/dns-client health

# 3. Test basic functionality
./tools/target/release/dns-client query "what.is.the.weather.com"

# 4. Run performance test
./tools/target/release/dns-client perf --requests 100 --concurrent 5
```

### Production Monitoring

```bash
# 1. Start monitoring
./scripts/monitor.sh --host production-server.com --port 9000

# 2. Run periodic health checks
while true; do
  ./tools/target/release/dns-client health --host production-server.com
  sleep 60
done

# 3. Run benchmark tests
./scripts/benchmark.sh --host production-server.com --duration 300
```

### Load Testing

```bash
# 1. Create test domains file
cat > test_domains.txt << EOF
what.is.the.weather.com
how.many.stars.are.there.com
what.is.the.capital.of.france.com
hello.world.com
test.query.com
EOF

# 2. Run batch test
./tools/target/release/dns-client batch test_domains.txt --concurrent 50

# 3. Run performance test
./tools/target/release/dns-client perf --requests 10000 --concurrent 100
```

### Troubleshooting

```bash
# Check if server is running
./tools/target/release/dns-client health

# Test with different record types
./tools/target/release/dns-client query "test.com" --record-type A
./tools/target/release/dns-client query "test.com" --record-type AAAA
./tools/target/release/dns-client query "test.com" --record-type MX

# Monitor server performance
./scripts/monitor.sh --interval 1

# Run comprehensive tests
./scripts/test.sh
```

## Output Examples

### Health Check Output

```
Performing health check on 127.0.0.1:9000
✓ Health check passed
Response time: 15.2ms
Response: Message { header: Header { id: 12345, message_type: Response, ... } }
```

### Performance Test Output

```
Performance test: 1000 requests, 20 concurrent
Performance test completed
Total time: 45.2s
Total requests: 1000
Successful requests: 998
Failed requests: 2
Success rate: 99.80%
Requests per second: 22.12
Average response time: 45.1ms
Min response time: 12.3ms
Max response time: 234.7ms
```

### Monitor Output

```
📊 LLMdig Server Monitor
================================
Server: localhost:9000
Interval: 5s
Press Ctrl+C to stop monitoring

Server Status
----------------------------------------
✓ Server is responding
Response Time: 23ms

Process Information
----------------------------------------
PID: 12345
Memory Usage: 45MB
CPU Usage: 2.3%
Uptime: 2h 15m

System Resources
----------------------------------------
System CPU: 15.2%
System Memory: 67.8%
Disk Usage: 23.1%

Network Statistics
----------------------------------------
Interface: eth0
RX Bytes: 1234567
TX Bytes: 9876543

Last Update: 2023-12-01 14:30:25
```

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: LLMdig Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Build and test
        run: |
          cargo build --release
          ./scripts/test.sh
      
      - name: Run benchmarks
        run: |
          cargo run --release &
          sleep 5
          ./scripts/benchmark.sh --duration 30
```

### Docker Integration

```dockerfile
# Build tools
FROM rust:1.75 as tools
WORKDIR /app
COPY tools/ .
RUN cargo build --release

# Use tools in main image
FROM debian:bookworm-slim
COPY --from=tools /app/target/release/dns-client /usr/local/bin/
COPY scripts/ /usr/local/bin/scripts/
RUN chmod +x /usr/local/bin/scripts/*.sh

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD dns-client health || exit 1
``` 