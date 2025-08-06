# LLMdig API Documentation

## Overview

LLMdig provides a DNS server that responds to TXT queries with LLM-generated responses. This document describes the API and configuration options.

## Configuration

### Server Configuration

```toml
[server]
host = "0.0.0.0"           # Server host to bind to
port = 9000               # Server port
max_connections = 1000    # Maximum concurrent connections
timeout_seconds = 30      # Request timeout
```

### LLM Configuration

```toml
[llm]
backend = "openai"        # LLM backend: openai, ollama, or custom URL
model = "gpt-3.5-turbo"   # Model name
max_tokens = 256          # Maximum response tokens
temperature = 0.7         # Response randomness (0.0-1.0)
timeout_seconds = 30      # LLM API timeout
```

### Rate Limiting

```toml
[rate_limit]
enabled = true                    # Enable rate limiting
requests_per_minute = 60         # Requests per minute per client
burst_size = 10                  # Burst allowance
```

## Environment Variables

All configuration can be overridden with environment variables:

```bash
# Server
PORT=9000
LLMDIG_SERVER_HOST=0.0.0.0
LLMDIG_SERVER_MAX_CONNECTIONS=1000
LLMDIG_SERVER_TIMEOUT_SECONDS=30

# LLM
LLMDIG_LLM_BACKEND=openai
LLMDIG_LLM_MODEL=gpt-3.5-turbo
LLMDIG_LLM_MAX_TOKENS=256
LLMDIG_LLM_TEMPERATURE=0.7
LLMDIG_LLM_TIMEOUT_SECONDS=30

# API Keys
OPENAI_API_KEY=sk-your-key-here

# Rate Limiting
LLMDIG_RATE_LIMIT_ENABLED=true
LLMDIG_RATE_LIMIT_REQUESTS_PER_MINUTE=60
LLMDIG_RATE_LIMIT_BURST_SIZE=10
```

## LLM Backends

### OpenAI

Uses OpenAI's API for generating responses.

```toml
[llm]
backend = "openai"
model = "gpt-3.5-turbo"
```

Required environment variable: `OPENAI_API_KEY`

### Ollama

Uses local Ollama instance for generating responses.

```toml
[llm]
backend = "ollama"
model = "llama2"
```

Requires Ollama running on `http://localhost:11434`

### Custom Backend

Uses a custom HTTP API endpoint.

```toml
[llm]
backend = "http://localhost:8080/api/generate"
```

The custom endpoint should accept POST requests with JSON:

```json
{
  "prompt": "user question",
  "model": "model-name",
  "max_tokens": 256,
  "temperature": 0.7
}
```

And return JSON:

```json
{
  "response": "generated answer"
}
```

## DNS Query Format

LLMdig expects DNS TXT queries where the domain name contains the question:

```
what.is.the.weather.com
```

The question is extracted by:
1. Removing the TLD (`.com`)
2. Replacing dots with spaces
3. Replacing hyphens and underscores with spaces

Examples:
- `what.is.the.weather.com` → "what is the weather"
- `hello-world.example.com` → "hello world example"
- `how.many.stars.are.there.com` → "how many stars are there"

## Response Format

Responses are returned as DNS TXT records, split into chunks of 255 bytes or less to comply with DNS standards.

## Error Handling

### DNS Response Codes

- `NOERROR` - Successful response
- `SERVFAIL` - Server error (LLM failure, rate limit exceeded)
- `NOTIMP` - Unsupported query type (non-TXT queries)
- `FORMERR` - Malformed query

### Common Errors

1. **Rate Limit Exceeded**: Client has exceeded the rate limit
2. **LLM API Error**: The LLM backend returned an error
3. **Invalid Query**: The domain name couldn't be parsed into a valid question
4. **Configuration Error**: Missing API keys or invalid configuration

## Security Considerations

### Input Sanitization

All queries are sanitized to prevent:
- XSS attacks
- SQL injection
- Command injection
- Special character injection

### Rate Limiting

Per-client rate limiting prevents abuse and controls costs.

### Caching

Responses are cached for 5 minutes to reduce LLM API calls and improve performance.

## Monitoring and Logging

LLMdig uses structured logging with the following levels:
- `ERROR` - Errors that prevent operation
- `WARN` - Warnings about potential issues
- `INFO` - General operational information
- `DEBUG` - Detailed debugging information

Set log level with `RUST_LOG` environment variable:

```bash
RUST_LOG=debug cargo run
```

## Performance Tuning

### Memory Usage

- Response cache: ~1MB per 1000 cached responses
- Rate limiter: ~1KB per unique client
- DNS handler: ~10MB base usage

### Throughput

Typical performance on modern hardware:
- 1000+ requests/second for cached responses
- 10-50 requests/second for LLM-generated responses (depends on LLM backend)

### Optimization Tips

1. Use caching for repeated queries
2. Adjust rate limits based on your LLM API limits
3. Use local models (Ollama) for better latency
4. Monitor memory usage and adjust cache size if needed 