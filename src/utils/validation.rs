use regex::Regex;
use std::collections::HashSet;
use lazy_static::lazy_static;

lazy_static! {
    static ref DOMAIN_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?)*$").unwrap();
    static ref IPV4_REGEX: Regex = Regex::new(r"^(\d{1,3}\.){3}\d{1,3}$").unwrap();
    static ref IPV6_REGEX: Regex = Regex::new(r"^([0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}$").unwrap();
    static ref EMAIL_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    static ref URL_REGEX: Regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn merge(&mut self, other: ValidationResult) {
        self.is_valid = self.is_valid && other.is_valid;
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

pub struct Validator;

impl Validator {
    /// Validate DNS query string
    pub fn validate_dns_query(query: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        if query.is_empty() {
            result.add_error("Query cannot be empty".to_string());
            return result;
        }
        
        if query.len() > 253 {
            result.add_error("Query too long (max 253 characters)".to_string());
        }
        
        if query.len() < 3 {
            result.add_error("Query too short (min 3 characters)".to_string());
        }
        
        // Check for invalid characters
        let invalid_chars: HashSet<char> = ['<', '>', '"', '\'', '&', '{', '}', '[', ']', '\\', '|'].iter().cloned().collect();
        for (i, ch) in query.chars().enumerate() {
            if invalid_chars.contains(&ch) {
                result.add_error(format!("Invalid character '{}' at position {}", ch, i));
            }
        }
        
        // Check for suspicious patterns
        let suspicious_patterns = [
            "script", "javascript", "vbscript", "expression",
            "union", "select", "insert", "update", "delete",
            "eval", "exec", "system", "shell", "cmd",
        ];
        
        let query_lower = query.to_lowercase();
        for pattern in &suspicious_patterns {
            if query_lower.contains(pattern) {
                result.add_warning(format!("Suspicious pattern detected: {}", pattern));
            }
        }
        
        result
    }

    /// Validate domain name
    pub fn validate_domain(domain: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        if domain.is_empty() {
            result.add_error("Domain cannot be empty".to_string());
            return result;
        }
        
        if domain.len() > 253 {
            result.add_error("Domain too long (max 253 characters)".to_string());
        }
        
        if !DOMAIN_REGEX.is_match(domain) {
            result.add_error("Invalid domain format".to_string());
        }
        
        // Check for reserved TLDs
        let reserved_tlds = ["localhost", "test", "invalid", "example"];
        if let Some(tld) = domain.split('.').last() {
            if reserved_tlds.contains(&tld) {
                result.add_warning(format!("Using reserved TLD: {}", tld));
            }
        }
        
        result
    }

    /// Validate IP address
    pub fn validate_ip_address(ip: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        if ip.is_empty() {
            result.add_error("IP address cannot be empty".to_string());
            return result;
        }
        
        if IPV4_REGEX.is_match(ip) {
            // Validate IPv4 octets
            let octets: Vec<&str> = ip.split('.').collect();
            for octet in octets {
                if let Ok(num) = octet.parse::<u8>() {
                    if num > 255 {
                        result.add_error(format!("Invalid IPv4 octet: {}", octet));
                    }
                } else {
                    result.add_error(format!("Invalid IPv4 octet: {}", octet));
                }
            }
        } else if IPV6_REGEX.is_match(ip) {
            // Basic IPv6 validation (simplified)
            if ip.contains("::") && ip.matches("::").count() > 1 {
                result.add_error("Invalid IPv6 format: multiple ::".to_string());
            }
        } else {
            result.add_error("Invalid IP address format".to_string());
        }
        
        result
    }

    /// Validate email address
    pub fn validate_email(email: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        if email.is_empty() {
            result.add_error("Email cannot be empty".to_string());
            return result;
        }
        
        if email.len() > 254 {
            result.add_error("Email too long (max 254 characters)".to_string());
        }
        
        if !EMAIL_REGEX.is_match(email) {
            result.add_error("Invalid email format".to_string());
        }
        
        // Check for disposable email domains
        let disposable_domains = ["10minutemail.com", "tempmail.org", "guerrillamail.com"];
        if let Some(domain) = email.split('@').last() {
            if disposable_domains.contains(&domain) {
                result.add_warning("Using disposable email domain".to_string());
            }
        }
        
        result
    }

    /// Validate URL
    pub fn validate_url(url: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        if url.is_empty() {
            result.add_error("URL cannot be empty".to_string());
            return result;
        }
        
        if url.len() > 2048 {
            result.add_error("URL too long (max 2048 characters)".to_string());
        }
        
        if !URL_REGEX.is_match(url) {
            result.add_error("Invalid URL format".to_string());
        }
        
        // Check for potentially dangerous protocols
        let dangerous_protocols = ["file://", "data:", "javascript:"];
        for protocol in &dangerous_protocols {
            if url.to_lowercase().starts_with(protocol) {
                result.add_error(format!("Dangerous protocol detected: {}", protocol));
            }
        }
        
        result
    }

    /// Validate configuration values
    pub fn validate_config_value(key: &str, value: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        match key {
            "port" => {
                if let Ok(port) = value.parse::<u16>() {
                    if port == 0 {
                        result.add_error("Port cannot be 0".to_string());
                    }
                    if port < 1024 {
                        result.add_warning("Using privileged port (< 1024)".to_string());
                    }
                } else {
                    result.add_error("Invalid port number".to_string());
                }
            }
            "max_connections" => {
                if let Ok(max) = value.parse::<usize>() {
                    if max == 0 {
                        result.add_error("Max connections cannot be 0".to_string());
                    }
                    if max > 100000 {
                        result.add_warning("Very high max connections value".to_string());
                    }
                } else {
                    result.add_error("Invalid max connections value".to_string());
                }
            }
            "timeout" => {
                if let Ok(timeout) = value.parse::<u64>() {
                    if timeout == 0 {
                        result.add_error("Timeout cannot be 0".to_string());
                    }
                    if timeout > 3600 {
                        result.add_warning("Very high timeout value (> 1 hour)".to_string());
                    }
                } else {
                    result.add_error("Invalid timeout value".to_string());
                }
            }
            "api_key" => {
                if value.is_empty() {
                    result.add_error("API key cannot be empty".to_string());
                }
                if value.len() < 10 {
                    result.add_warning("API key seems too short".to_string());
                }
                if value.contains(' ') {
                    result.add_error("API key cannot contain spaces".to_string());
                }
            }
            _ => {
                // Generic validation for unknown keys
                if value.is_empty() {
                    result.add_warning("Empty value for configuration key".to_string());
                }
            }
        }
        
        result
    }

    /// Validate rate limit configuration
    pub fn validate_rate_limit_config(requests_per_minute: usize, burst_size: usize) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        if requests_per_minute == 0 {
            result.add_error("Requests per minute cannot be 0".to_string());
        }
        
        if burst_size == 0 {
            result.add_error("Burst size cannot be 0".to_string());
        }
        
        if burst_size > requests_per_minute {
            result.add_warning("Burst size is larger than requests per minute".to_string());
        }
        
        if requests_per_minute > 10000 {
            result.add_warning("Very high rate limit (> 10k requests/minute)".to_string());
        }
        
        result
    }

    /// Validate cache configuration
    pub fn validate_cache_config(max_size: usize, ttl_seconds: u64) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        if max_size == 0 {
            result.add_error("Cache max size cannot be 0".to_string());
        }
        
        if max_size > 1000000 {
            result.add_warning("Very large cache size (> 1M entries)".to_string());
        }
        
        if ttl_seconds == 0 {
            result.add_error("Cache TTL cannot be 0".to_string());
        }
        
        if ttl_seconds > 86400 {
            result.add_warning("Very long cache TTL (> 24 hours)".to_string());
        }
        
        result
    }

    /// Validate LLM model configuration
    pub fn validate_llm_config(model: &str, max_tokens: usize, temperature: f32) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        if model.is_empty() {
            result.add_error("Model name cannot be empty".to_string());
        }
        
        if max_tokens == 0 {
            result.add_error("Max tokens cannot be 0".to_string());
        }
        
        if max_tokens > 8192 {
            result.add_warning("Very high max tokens (> 8k)".to_string());
        }
        
        if temperature < 0.0 || temperature > 2.0 {
            result.add_error("Temperature must be between 0.0 and 2.0".to_string());
        }
        
        if temperature > 1.5 {
            result.add_warning("High temperature value (> 1.5)".to_string());
        }
        
        result
    }

    /// Comprehensive validation for LLMdig configuration
    pub fn validate_llmdig_config(config: &crate::config::Config) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Validate server config
        let port_validation = Self::validate_config_value("port", &config.server.port.to_string());
        result.merge(port_validation);
        
        let max_conn_validation = Self::validate_config_value("max_connections", &config.server.max_connections.to_string());
        result.merge(max_conn_validation);
        
        let timeout_validation = Self::validate_config_value("timeout", &config.server.timeout_seconds.to_string());
        result.merge(timeout_validation);
        
        // Validate LLM config
        let llm_validation = Self::validate_llm_config(
            &config.llm.model,
            config.llm.max_tokens,
            config.llm.temperature,
        );
        result.merge(llm_validation);
        
        // Validate rate limit config
        let rate_limit_validation = Self::validate_rate_limit_config(
            config.rate_limit.requests_per_minute,
            config.rate_limit.burst_size,
        );
        result.merge(rate_limit_validation);
        
        result
    }

    /// Sanitize and validate user input
    pub fn sanitize_and_validate_input(input: &str) -> (String, ValidationResult) {
        let mut result = ValidationResult::new();
        let mut sanitized = input.to_string();
        
        // Remove null bytes
        sanitized = sanitized.replace('\0', "");
        
        // Trim whitespace
        sanitized = sanitized.trim().to_string();
        
        // Convert to lowercase for consistency
        sanitized = sanitized.to_lowercase();
        
        // Remove control characters
        sanitized = sanitized.chars().filter(|c| !c.is_control()).collect();
        
        // Validate the sanitized input
        let validation = Self::validate_dns_query(&sanitized);
        result.merge(validation);
        
        (sanitized, result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_query_validation() {
        // Valid queries
        let valid_queries = [
            "what is the weather",
            "how many stars are there",
            "hello-world",
            "test123",
        ];
        
        for query in &valid_queries {
            let result = Validator::validate_dns_query(query);
            assert!(result.is_valid, "Query '{}' should be valid", query);
        }
        
        // Invalid queries
        let invalid_queries = [
            "",
            "a",
            "ab",
            "test<script>alert('xss')</script>",
            "union select * from users",
        ];
        
        for query in &invalid_queries {
            let result = Validator::validate_dns_query(query);
            assert!(!result.is_valid, "Query '{}' should be invalid", query);
        }
    }

    #[test]
    fn test_domain_validation() {
        // Valid domains
        let valid_domains = [
            "example.com",
            "test.example.org",
            "sub-domain.test.co.uk",
        ];
        
        for domain in &valid_domains {
            let result = Validator::validate_domain(domain);
            assert!(result.is_valid, "Domain '{}' should be valid", domain);
        }
        
        // Invalid domains
        let invalid_domains = [
            "",
            ".example.com",
            "example..com",
            "example.com.",
            "test@example.com",
        ];
        
        for domain in &invalid_domains {
            let result = Validator::validate_domain(domain);
            assert!(!result.is_valid, "Domain '{}' should be invalid", domain);
        }
    }

    #[test]
    fn test_ip_validation() {
        // Valid IPs
        let valid_ips = [
            "192.168.1.1",
            "10.0.0.1",
            "127.0.0.1",
            "::1",
            "2001:db8::1",
        ];
        
        for ip in &valid_ips {
            let result = Validator::validate_ip_address(ip);
            assert!(result.is_valid, "IP '{}' should be valid", ip);
        }
        
        // Invalid IPs
        let invalid_ips = [
            "",
            "256.1.2.3",
            "1.2.3.256",
            "192.168.1",
            "192.168.1.1.1",
        ];
        
        for ip in &invalid_ips {
            let result = Validator::validate_ip_address(ip);
            assert!(!result.is_valid, "IP '{}' should be invalid", ip);
        }
    }

    #[test]
    fn test_config_validation() {
        // Valid config values
        let valid_configs = [
            ("port", "8080"),
            ("max_connections", "1000"),
            ("timeout", "30"),
        ];
        
        for (key, value) in &valid_configs {
            let result = Validator::validate_config_value(key, value);
            assert!(result.is_valid, "Config {}={} should be valid", key, value);
        }
        
        // Invalid config values
        let invalid_configs = [
            ("port", "0"),
            ("port", "99999"),
            ("max_connections", "0"),
            ("timeout", "0"),
        ];
        
        for (key, value) in &invalid_configs {
            let result = Validator::validate_config_value(key, value);
            assert!(!result.is_valid, "Config {}={} should be invalid", key, value);
        }
    }

    #[test]
    fn test_sanitize_and_validate() {
        let (sanitized, result) = Validator::sanitize_and_validate_input("  What Is The Weather?  ");
        assert!(result.is_valid);
        assert_eq!(sanitized, "what is the weather?");
        
        let (sanitized, result) = Validator::sanitize_and_validate_input("test<script>alert('xss')</script>");
        assert!(!result.is_valid);
        assert!(sanitized.contains("script"));
    }
} 