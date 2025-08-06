use llmdig::config::{Config, LlmBackendType};
use llmdig::utils::sanitizer::Sanitizer;
use llmdig::utils::rate_limiter::RateLimiter;
use std::net::IpAddr;
use std::str::FromStr;

#[test]
fn test_config_default() {
    let config = Config::default();
    assert_eq!(config.server.port, 9000);
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.llm.model, "gpt-3.5-turbo");
    assert_eq!(config.llm.max_tokens, 256);
    assert_eq!(config.llm.temperature, 0.7);
    assert!(config.rate_limit.enabled);
    assert_eq!(config.rate_limit.requests_per_minute, 60);
    assert_eq!(config.rate_limit.burst_size, 10);
}

#[test]
fn test_llm_backend_type_serialization() {
    use serde_json;
    
    let openai = LlmBackendType::OpenAI;
    let ollama = LlmBackendType::Ollama;
    let custom = LlmBackendType::Custom("http://localhost:8080".to_string());
    
    assert_eq!(serde_json::to_string(&openai).unwrap(), "\"openai\"");
    assert_eq!(serde_json::to_string(&ollama).unwrap(), "\"ollama\"");
    assert_eq!(serde_json::to_string(&custom).unwrap(), "\"http://localhost:8080\"");
}

#[test]
fn test_llm_backend_type_deserialization() {
    use serde_json;
    
    let openai: LlmBackendType = serde_json::from_str("\"openai\"").unwrap();
    let ollama: LlmBackendType = serde_json::from_str("\"ollama\"").unwrap();
    let custom: LlmBackendType = serde_json::from_str("\"http://localhost:8080\"").unwrap();
    
    assert!(matches!(openai, LlmBackendType::OpenAI));
    assert!(matches!(ollama, LlmBackendType::Ollama));
    assert!(matches!(custom, LlmBackendType::Custom(url) if url == "http://localhost:8080"));
}

#[test]
fn test_sanitizer_basic() {
    let query = "What is the weather like today?";
    let sanitized = Sanitizer::sanitize_query(query);
    assert_eq!(sanitized, "what is the weather like today?");
}

#[test]
fn test_sanitizer_remove_dangerous_patterns() {
    let query = "What is <script>alert('xss')</script> the weather?";
    let sanitized = Sanitizer::sanitize_query(query);
    assert!(!sanitized.contains("script"));
    assert!(!sanitized.contains("alert"));
}

#[test]
fn test_sanitizer_remove_sql_injection() {
    let query = "What is the weather UNION SELECT * FROM users?";
    let sanitized = Sanitizer::sanitize_query(query);
    assert!(!sanitized.contains("union"));
    assert!(!sanitized.contains("select"));
}

#[test]
fn test_sanitizer_remove_special_chars() {
    let query = "What is the weather? <>&\"'";
    let sanitized = Sanitizer::sanitize_query(query);
    assert!(!sanitized.contains('<'));
    assert!(!sanitized.contains('>'));
    assert!(!sanitized.contains('&'));
    assert!(!sanitized.contains('"'));
    assert!(!sanitized.contains('\''));
}

#[test]
fn test_sanitizer_truncate_long_queries() {
    let long_query = "a".repeat(300);
    let sanitized = Sanitizer::sanitize_query(&long_query);
    assert_eq!(sanitized.len(), 200);
}

#[test]
fn test_sanitizer_is_safe() {
    assert!(Sanitizer::is_safe("What is the weather?"));
    assert!(!Sanitizer::is_safe("<script>alert('xss')</script>"));
    assert!(!Sanitizer::is_safe(""));
    assert!(!Sanitizer::is_safe("a")); // too short
    assert!(!Sanitizer::is_safe(&"a".repeat(300))); // too long
}

#[test]
fn test_sanitizer_extract_question_from_domain() {
    assert_eq!(
        Sanitizer::extract_question_from_domain("what.is.the.weather.com"),
        Some("what is the weather".to_string())
    );
    
    assert_eq!(
        Sanitizer::extract_question_from_domain("hello-world.example.com"),
        Some("hello world example".to_string())
    );
    
    assert_eq!(
        Sanitizer::extract_question_from_domain("single.com"),
        Some("single".to_string())
    );
    
    assert_eq!(
        Sanitizer::extract_question_from_domain("domain"),
        None
    );
    
    assert_eq!(
        Sanitizer::extract_question_from_domain(""),
        None
    );
}

#[tokio::test]
async fn test_rate_limiter_basic() {
    let limiter = RateLimiter::new(60, 10); // 60 requests per minute, burst of 10
    let addr = std::net::SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 12345);
    
    // Should allow first 10 requests immediately
    for _ in 0..10 {
        assert!(limiter.allow_request(addr).await);
    }
    
    // 11th request should be rate limited
    assert!(!limiter.allow_request(addr).await);
}

#[tokio::test]
async fn test_rate_limiter_refill() {
    let limiter = RateLimiter::new(60, 1); // 60 requests per minute, burst of 1
    let addr = std::net::SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 12345);
    
    // First request should succeed
    assert!(limiter.allow_request(addr).await);
    
    // Second request should fail
    assert!(!limiter.allow_request(addr).await);
    
    // Wait for refill (1 second should add 1 token)
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    // Should succeed again
    assert!(limiter.allow_request(addr).await);
}

#[tokio::test]
async fn test_rate_limiter_multiple_clients() {
    let limiter = RateLimiter::new(60, 5); // 60 requests per minute, burst of 5
    let addr1 = std::net::SocketAddr::new(IpAddr::from_str("127.0.0.1").unwrap(), 12345);
    let addr2 = std::net::SocketAddr::new(IpAddr::from_str("127.0.0.2").unwrap(), 12345);
    
    // Both clients should be able to use their full burst
    for _ in 0..5 {
        assert!(limiter.allow_request(addr1).await);
        assert!(limiter.allow_request(addr2).await);
    }
    
    // Both should be rate limited after burst
    assert!(!limiter.allow_request(addr1).await);
    assert!(!limiter.allow_request(addr2).await);
} 