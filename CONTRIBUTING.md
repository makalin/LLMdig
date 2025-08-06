# Contributing to LLMdig

Thank you for your interest in contributing to LLMdig! This document provides guidelines and information for contributors.

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Git
- Basic knowledge of DNS protocols
- Familiarity with async Rust

### Development Setup

1. **Fork and Clone**
   ```bash
   git clone https://github.com/your-username/LLMdig.git
   cd LLMdig
   ```

2. **Install Dependencies**
   ```bash
   # Install Rust components
   rustup component add rustfmt clippy
   
   # Install development tools
   cargo install cargo-watch
   ```

3. **Setup Environment**
   ```bash
   cp env.example .env
   # Edit .env with your configuration
   ```

4. **Run Tests**
   ```bash
   cargo test
   ./scripts/test.sh
   ```

## Development Workflow

### Code Style

- Follow Rust coding conventions
- Use `cargo fmt` to format code
- Run `cargo clippy` to check for issues
- Write meaningful commit messages

### Testing

- Write unit tests for new functionality
- Add integration tests for complex features
- Ensure all tests pass before submitting PR

### Branching Strategy

- Create feature branches from `main`
- Use descriptive branch names: `feature/add-ollama-support`
- Keep branches focused and small

## Project Structure

```
LLMdig/
├── src/
│   ├── main.rs          # Application entry point
│   ├── lib.rs           # Library root
│   ├── config.rs        # Configuration management
│   ├── dns.rs           # DNS server implementation
│   ├── llm.rs           # LLM backend implementations
│   ├── server.rs        # Server setup and management
│   ├── error.rs         # Error types
│   └── utils/           # Utility modules
│       ├── rate_limiter.rs
│       └── sanitizer.rs
├── tests/               # Integration tests
├── docs/               # Documentation
├── scripts/            # Build and test scripts
└── config.toml         # Configuration file
```

## Adding New Features

### 1. LLM Backend Support

To add support for a new LLM backend:

1. **Add Backend Type**
   ```rust
   // In src/config.rs
   pub enum LlmBackendType {
       OpenAI,
       Ollama,
       Custom(String),
       NewBackend, // Add here
   }
   ```

2. **Implement Backend**
   ```rust
   // In src/llm.rs
   pub struct NewBackendBackend {
       client: Client,
       config: Config,
   }
   
   #[async_trait]
   impl LlmBackend for NewBackendBackend {
       async fn generate_response(&self, prompt: &str) -> Result<String> {
           // Implementation here
       }
   }
   ```

3. **Add to Client Factory**
   ```rust
   // In src/llm.rs, LlmClient::new()
   match &config.llm.backend {
       LlmBackendType::NewBackend => {
           Box::new(NewBackendBackend::new(config.clone())?)
       }
       // ... other cases
   }
   ```

4. **Add Tests**
   ```rust
   #[test]
   fn test_new_backend_serialization() {
       let backend = LlmBackendType::NewBackend;
       assert_eq!(serde_json::to_string(&backend).unwrap(), "\"new-backend\"");
   }
   ```

### 2. DNS Features

To add new DNS features:

1. **Extend DNS Handler**
   ```rust
   // In src/dns.rs
   impl DnsHandler {
       pub fn new_feature(&self, query: &str) -> Result<String> {
           // Implementation
       }
   }
   ```

2. **Add Query Type Support**
   ```rust
   // Support for new record types
   match query.query_type() {
       RecordType::TXT => self.handle_txt_query(request, response_handle).await,
       RecordType::A => self.handle_a_query(request, response_handle).await,
       // Add new types here
   }
   ```

### 3. Configuration Options

To add new configuration options:

1. **Extend Config Struct**
   ```rust
   // In src/config.rs
   pub struct Config {
       pub server: ServerConfig,
       pub llm: LlmConfig,
       pub rate_limit: RateLimitConfig,
       pub new_feature: NewFeatureConfig, // Add here
   }
   ```

2. **Add Default Values**
   ```rust
   // In Config::load()
   .set_default("new_feature.enabled", true)?
   .set_default("new_feature.timeout", 30)?
   ```

3. **Update Documentation**
   - Update `config.toml`
   - Update `env.example`
   - Update `docs/API.md`

## Testing Guidelines

### Unit Tests

- Test individual functions and methods
- Mock external dependencies
- Test error conditions
- Aim for >90% code coverage

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Arrange
        let input = "test";
        
        // Act
        let result = function_name(input);
        
        // Assert
        assert_eq!(result, "expected");
    }
}
```

### Integration Tests

- Test complete workflows
- Test with real DNS queries
- Test configuration loading
- Test error handling

```rust
#[tokio::test]
async fn test_dns_query_workflow() {
    // Setup
    let config = Config::default();
    let handler = DnsHandler::new(config).unwrap();
    
    // Test
    // ... test implementation
}
```

### Performance Tests

- Test response times
- Test memory usage
- Test concurrent requests
- Test rate limiting

```rust
#[tokio::test]
async fn test_concurrent_requests() {
    let config = Config::default();
    let handler = DnsHandler::new(config).unwrap();
    
    let mut handles = vec![];
    for _ in 0..100 {
        let handler = handler.clone();
        handles.push(tokio::spawn(async move {
            // Test request
        }));
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
}
```

## Documentation

### Code Documentation

- Document all public APIs
- Use Rust doc comments
- Include examples
- Explain complex algorithms

```rust
/// Generates a response for the given query.
///
/// # Arguments
///
/// * `query` - The DNS query string
///
/// # Returns
///
/// A Result containing the generated response or an error.
///
/// # Examples
///
/// ```
/// let response = handler.generate_response("what is the weather").await?;
/// assert!(!response.is_empty());
/// ```
pub async fn generate_response(&self, query: &str) -> Result<String> {
    // Implementation
}
```

### User Documentation

- Update README.md for new features
- Add examples to docs/
- Update configuration examples
- Document breaking changes

## Pull Request Process

### Before Submitting

1. **Run Tests**
   ```bash
   cargo test
   cargo clippy --all-features -- -D warnings
   cargo fmt -- --check
   ```

2. **Update Documentation**
   - Update relevant docs
   - Add examples
   - Update configuration files

3. **Check Performance**
   - Run benchmarks if applicable
   - Check memory usage
   - Test with realistic load

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests added/updated
```

### Review Process

1. **Automated Checks**
   - CI/CD pipeline runs tests
   - Code coverage is checked
   - Security scans are performed

2. **Manual Review**
   - Code review by maintainers
   - Testing by maintainers
   - Documentation review

3. **Merge**
   - Squash commits if needed
   - Update version if required
   - Create release notes

## Issue Reporting

### Bug Reports

When reporting bugs, include:

- LLMdig version
- Operating system
- Configuration (sanitized)
- Steps to reproduce
- Expected vs actual behavior
- Logs (if applicable)

### Feature Requests

When requesting features, include:

- Use case description
- Proposed implementation
- Benefits
- Potential drawbacks
- Examples

## Code of Conduct

- Be respectful and inclusive
- Help others learn
- Give constructive feedback
- Follow project guidelines

## Getting Help

- Check existing issues and PRs
- Read documentation
- Ask questions in discussions
- Join community channels

## License

By contributing to LLMdig, you agree that your contributions will be licensed under the MIT License. 