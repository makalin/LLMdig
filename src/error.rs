use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("LLM API error: {0}")]
    LlmApi(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("DNS error: {0}")]
    Dns(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Sanitization error: {0}")]
    Sanitization(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("DNS protocol error: {0}")]
    DnsProto(#[from] trust_dns_proto::error::ProtoError),

    #[error("DNS server error: {0}")]
    DnsServer(#[from] trust_dns_server::error::ServerError),
} 