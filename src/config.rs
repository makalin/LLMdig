use anyhow::Result;
use config::{Config as ConfigFile, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub llm: LlmConfig,
    pub rate_limit: RateLimitConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub backend: LlmBackendType,
    pub api_key: Option<String>,
    pub model: String,
    pub max_tokens: usize,
    pub temperature: f32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmBackendType {
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "ollama")]
    Ollama,
    #[serde(rename = "custom")]
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: usize,
    pub burst_size: usize,
    pub enabled: bool,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = ConfigFile::builder()
            // Start with default values
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 9000)?
            .set_default("server.max_connections", 1000)?
            .set_default("server.timeout_seconds", 30)?
            .set_default("llm.backend", "openai")?
            .set_default("llm.model", "gpt-3.5-turbo")?
            .set_default("llm.max_tokens", 256)?
            .set_default("llm.temperature", 0.7)?
            .set_default("llm.timeout_seconds", 30)?
            .set_default("rate_limit.requests_per_minute", 60)?
            .set_default("rate_limit.burst_size", 10)?
            .set_default("rate_limit.enabled", true)?
            // Load config file if it exists
            .add_source(File::from(path.as_ref()).required(false))
            // Override with environment variables
            .add_source(Environment::with_prefix("LLMDIG").separator("_"))
            .build()?;

        let config: Config = config.try_deserialize()?;
        
        // Override with environment variables for sensitive data
        let mut config = config;
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            config.llm.api_key = Some(api_key);
        }
        
        if let Ok(port) = std::env::var("PORT") {
            if let Ok(port) = port.parse() {
                config.server.port = port;
            }
        }

        Ok(config)
    }

    pub fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 9000,
                max_connections: 1000,
                timeout_seconds: 30,
            },
            llm: LlmConfig {
                backend: LlmBackendType::OpenAI,
                api_key: None,
                model: "gpt-3.5-turbo".to_string(),
                max_tokens: 256,
                temperature: 0.7,
                timeout_seconds: 30,
            },
            rate_limit: RateLimitConfig {
                requests_per_minute: 60,
                burst_size: 10,
                enabled: true,
            },
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::default()
    }
} 