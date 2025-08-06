# LLMdig Deployment Guide

This guide covers deploying LLMdig in various environments, from local development to production.

## Local Development

### Prerequisites

- Rust 1.75 or later
- Git

### Quick Start

```bash
# Clone the repository
git clone https://github.com/makalin/LLMdig.git
cd LLMdig

# Copy environment file
cp env.example .env

# Edit .env with your configuration
# Set OPENAI_API_KEY or configure other LLM backend

# Build and run
cargo run --release
```

### Testing

```bash
# Run all tests
cargo test

# Run with test script
./scripts/test.sh

# Test with dig
dig @localhost -p 9000 "what.is.the.weather.com" TXT +short
```

## Docker Deployment

### Using Docker Compose

```bash
# Copy environment file
cp env.example .env

# Edit .env with your configuration
nano .env

# Start services
docker-compose up -d

# Test
dig @localhost -p 9000 "what.is.the.weather.com" TXT +short
```

### Using Docker directly

```bash
# Build image
docker build -t llmdig .

# Run container
docker run -d \
  --name llmdig \
  -p 9000:9000/udp \
  --env-file .env \
  llmdig

# Test
dig @localhost -p 9000 "what.is.the.weather.com" TXT +short
```

### With Ollama

```bash
# Start Ollama service
docker-compose up -d ollama

# Pull a model
docker exec -it llmdig_ollama_1 ollama pull llama2

# Start LLMdig with Ollama backend
docker-compose up -d llmdig-ollama

# Test
dig @localhost -p 9001 "what.is.the.weather.com" TXT +short
```

## Production Deployment

### System Requirements

- **CPU**: 2+ cores recommended
- **Memory**: 512MB+ RAM
- **Storage**: 100MB+ disk space
- **Network**: UDP port 9000 accessible

### Security Considerations

1. **Firewall Configuration**
   ```bash
   # Allow UDP port 9000
   sudo ufw allow 9000/udp
   
   # Or with iptables
   sudo iptables -A INPUT -p udp --dport 9000 -j ACCEPT
   ```

2. **Run as Non-Root User**
   ```bash
   # Create user
   sudo useradd -r -s /bin/false llmdig
   
   # Set ownership
   sudo chown -R llmdig:llmdig /opt/llmdig
   ```

3. **Environment Variables**
   - Never commit API keys to version control
   - Use secrets management for production
   - Rotate API keys regularly

### Systemd Service

Create `/etc/systemd/system/llmdig.service`:

```ini
[Unit]
Description=LLMdig DNS Server
After=network.target

[Service]
Type=simple
User=llmdig
Group=llmdig
WorkingDirectory=/opt/llmdig
ExecStart=/opt/llmdig/llmdig --config /opt/llmdig/config.toml
Restart=always
RestartSec=10
Environment=RUST_LOG=info
EnvironmentFile=/opt/llmdig/.env

[Install]
WantedBy=multi-user.target
```

Enable and start the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable llmdig
sudo systemctl start llmdig
sudo systemctl status llmdig
```

### Kubernetes Deployment

Create `k8s/llmdig-deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llmdig
  labels:
    app: llmdig
spec:
  replicas: 2
  selector:
    matchLabels:
      app: llmdig
  template:
    metadata:
      labels:
        app: llmdig
    spec:
      containers:
      - name: llmdig
        image: llmdig:latest
        ports:
        - containerPort: 9000
          protocol: UDP
        env:
        - name: RUST_LOG
          value: "info"
        - name: OPENAI_API_KEY
          valueFrom:
            secretKeyRef:
              name: llmdig-secrets
              key: openai-api-key
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          exec:
            command:
            - dig
            - "@localhost"
            - "-p"
            - "9000"
            - "health.check"
            - "TXT"
            - "+short"
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          exec:
            command:
            - dig
            - "@localhost"
            - "-p"
            - "9000"
            - "health.check"
            - "TXT"
            - "+short"
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: llmdig-service
spec:
  selector:
    app: llmdig
  ports:
  - protocol: UDP
    port: 9000
    targetPort: 9000
  type: LoadBalancer
```

Create secrets:

```bash
kubectl create secret generic llmdig-secrets \
  --from-literal=openai-api-key=your-api-key-here
```

Deploy:

```bash
kubectl apply -f k8s/llmdig-deployment.yaml
```

## Monitoring and Logging

### Log Management

Configure log rotation in `/etc/logrotate.d/llmdig`:

```
/var/log/llmdig/*.log {
    daily
    missingok
    rotate 7
    compress
    delaycompress
    notifempty
    create 644 llmdig llmdig
    postrotate
        systemctl reload llmdig
    endscript
}
```

### Metrics Collection

LLMdig logs structured events that can be parsed by monitoring tools:

```bash
# Monitor logs
journalctl -u llmdig -f

# Parse structured logs
journalctl -u llmdig -o json | jq '.MESSAGE'
```

### Health Checks

```bash
# Basic health check
dig @localhost -p 9000 "health.check" TXT +short

# Response time check
time dig @localhost -p 9000 "test.query.com" TXT +short
```

## Troubleshooting

### Common Issues

1. **Port Already in Use**
   ```bash
   # Check what's using port 9000
   sudo netstat -tulpn | grep :9000
   
   # Kill process or change port
   sudo kill -9 <PID>
   ```

2. **Permission Denied**
   ```bash
   # Check file permissions
   ls -la /opt/llmdig/
   
   # Fix ownership
   sudo chown -R llmdig:llmdig /opt/llmdig/
   ```

3. **API Key Issues**
   ```bash
   # Test API key
   curl -H "Authorization: Bearer $OPENAI_API_KEY" \
        https://api.openai.com/v1/models
   ```

4. **DNS Resolution Issues**
   ```bash
   # Test DNS server
   dig @localhost -p 9000 "test.com" TXT +short
   
   # Check server logs
   journalctl -u llmdig -f
   ```

### Performance Tuning

1. **Increase File Descriptors**
   ```bash
   # Edit /etc/security/limits.conf
   llmdig soft nofile 65536
   llmdig hard nofile 65536
   ```

2. **Optimize Network Settings**
   ```bash
   # Increase UDP buffer sizes
   echo 'net.core.rmem_max = 16777216' >> /etc/sysctl.conf
   echo 'net.core.wmem_max = 16777216' >> /etc/sysctl.conf
   sysctl -p
   ```

3. **Memory Optimization**
   ```bash
   # Monitor memory usage
   watch -n 1 'ps aux | grep llmdig'
   
   # Adjust cache size in config.toml
   ```

## Backup and Recovery

### Configuration Backup

```bash
# Backup configuration
sudo tar -czf llmdig-config-$(date +%Y%m%d).tar.gz \
  /opt/llmdig/config.toml \
  /opt/llmdig/.env \
  /etc/systemd/system/llmdig.service
```

### Data Recovery

```bash
# Restore configuration
sudo tar -xzf llmdig-config-20231201.tar.gz -C /

# Reload systemd
sudo systemctl daemon-reload
sudo systemctl restart llmdig
```

## Scaling

### Horizontal Scaling

1. **Load Balancer Configuration**
   ```bash
   # Nginx UDP load balancer
   stream {
       upstream llmdig_backend {
           server 192.168.1.10:9000;
           server 192.168.1.11:9000;
           server 192.168.1.12:9000;
       }
       
       server {
           listen 9000 udp;
           proxy_pass llmdig_backend;
       }
   }
   ```

2. **DNS Round Robin**
   ```bash
   # Configure multiple A records
   llmdig1.example.com. IN A 192.168.1.10
   llmdig2.example.com. IN A 192.168.1.11
   llmdig3.example.com. IN A 192.168.1.12
   ```

### Vertical Scaling

1. **Resource Limits**
   ```bash
   # Increase memory limits
   sudo systemctl set-property llmdig MemoryMax=1G
   
   # Increase CPU limits
   sudo systemctl set-property llmdig CPUQuota=200%
   ```

2. **Performance Monitoring**
   ```bash
   # Monitor resource usage
   htop
   
   # Profile performance
   cargo install flamegraph
   cargo flamegraph --bin llmdig
   ``` 