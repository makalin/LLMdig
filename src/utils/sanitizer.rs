use regex::Regex;
use std::collections::HashSet;
use lazy_static::lazy_static;

lazy_static! {
    static ref DANGEROUS_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"(?i)(script|javascript|vbscript|expression|onload|onerror|onclick)").unwrap(),
        Regex::new(r"(?i)(union|select|insert|update|delete|drop|create|alter)").unwrap(),
        Regex::new(r"(?i)(eval|exec|system|shell|cmd|powershell)").unwrap(),
        Regex::new(r"[<>\"'&]").unwrap(),
    ];
    
    static ref ALLOWED_CHARS: HashSet<char> = {
        let mut set = HashSet::new();
        // Allow letters, numbers, spaces, and common punctuation
        for c in 'a'..='z' { set.insert(c); }
        for c in 'A'..='Z' { set.insert(c); }
        for c in '0'..='9' { set.insert(c); }
        set.insert(' ');
        set.insert('.');
        set.insert(',');
        set.insert('!');
        set.insert('?');
        set.insert('-');
        set.insert('_');
        set.insert('\'');
        set.insert('"');
        set.insert('(');
        set.insert(')');
        set.insert(':');
        set.insert(';');
        set
    };
}

pub struct Sanitizer;

impl Sanitizer {
    /// Sanitize a DNS query string to prevent injection attacks
    pub fn sanitize_query(query: &str) -> String {
        let mut sanitized = query.to_string();
        
        // Convert to lowercase for consistency
        sanitized = sanitized.to_lowercase();
        
        // Remove dangerous patterns
        for pattern in DANGEROUS_PATTERNS.iter() {
            sanitized = pattern.replace_all(&sanitized, "").to_string();
        }
        
        // Remove non-allowed characters
        sanitized = sanitized
            .chars()
            .filter(|c| ALLOWED_CHARS.contains(c))
            .collect();
        
        // Normalize whitespace
        sanitized = sanitized
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        
        // Truncate if too long
        if sanitized.len() > 200 {
            sanitized = sanitized[..200].to_string();
        }
        
        sanitized
    }
    
    /// Validate if a query is safe to process
    pub fn is_safe(query: &str) -> bool {
        let sanitized = Self::sanitize_query(query);
        
        // Check if sanitization significantly changed the query
        if sanitized.len() < query.len() * 3 / 4 {
            return false;
        }
        
        // Check for dangerous patterns
        for pattern in DANGEROUS_PATTERNS.iter() {
            if pattern.is_match(query) {
                return false;
            }
        }
        
        // Check if query is too short or too long
        if sanitized.len() < 3 || sanitized.len() > 200 {
            return false;
        }
        
        true
    }
    
    /// Extract and validate a question from a domain name
    pub fn extract_question_from_domain(domain: &str) -> Option<String> {
        let domain = domain.trim_end_matches('.');
        let parts: Vec<&str> = domain.split('.').collect();
        
        if parts.len() < 2 {
            return None;
        }
        
        // The question is everything except the last part (TLD)
        let question_parts = &parts[..parts.len() - 1];
        let question = question_parts.join(" ");
        
        // Clean up the question
        let question = question.replace('-', " ").replace('_', " ");
        
        if Self::is_safe(&question) {
            Some(question)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_query_basic() {
        let query = "What is the weather like today?";
        let sanitized = Sanitizer::sanitize_query(query);
        assert_eq!(sanitized, "what is the weather like today?");
    }

    #[test]
    fn test_sanitize_query_dangerous_patterns() {
        let query = "What is <script>alert('xss')</script> the weather?";
        let sanitized = Sanitizer::sanitize_query(query);
        assert!(!sanitized.contains("script"));
        assert!(!sanitized.contains("alert"));
    }

    #[test]
    fn test_sanitize_query_sql_injection() {
        let query = "What is the weather UNION SELECT * FROM users?";
        let sanitized = Sanitizer::sanitize_query(query);
        assert!(!sanitized.contains("union"));
        assert!(!sanitized.contains("select"));
    }

    #[test]
    fn test_is_safe() {
        assert!(Sanitizer::is_safe("What is the weather?"));
        assert!(!Sanitizer::is_safe("<script>alert('xss')</script>"));
        assert!(!Sanitizer::is_safe(""));
        assert!(!Sanitizer::is_safe("a")); // too short
    }

    #[test]
    fn test_extract_question_from_domain() {
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
    }
} 