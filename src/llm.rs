use crate::config::{Config, LlmBackendType};
use crate::Error;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info};

#[async_trait]
pub trait LlmBackend: Send + Sync {
    async fn generate_response(&self, prompt: &str) -> Result<String>;
}

pub struct LlmClient {
    backend: Box<dyn LlmBackend>,
    config: Config,
}

impl LlmClient {
    pub fn new(config: Config) -> Result<Self> {
        let backend: Box<dyn LlmBackend> = match &config.llm.backend {
            LlmBackendType::OpenAI => {
                Box::new(OpenAiBackend::new(config.clone())?)
            }
            LlmBackendType::Ollama => {
                Box::new(OllamaBackend::new(config.clone())?)
            }
            LlmBackendType::Custom(url) => {
                Box::new(CustomBackend::new(config.clone(), url.clone())?)
            }
        };

        Ok(Self { backend, config })
    }

    pub async fn query(&self, question: &str) -> Result<String> {
        info!("Processing LLM query: {}", question);
        
        let response = self.backend.generate_response(question).await?;
        
        // Truncate response to fit in DNS TXT record (255 bytes per string, max 16 strings)
        let max_length = 255 * 16;
        let truncated = if response.len() > max_length {
            let truncated = &response[..max_length];
            format!("{}...", truncated)
        } else {
            response
        };

        debug!("LLM response ({} chars): {}", truncated.len(), truncated);
        Ok(truncated)
    }
}

pub struct OpenAiBackend {
    client: Client,
    config: Config,
}

impl OpenAiBackend {
    pub fn new(config: Config) -> Result<Self> {
        let api_key = config
            .llm
            .api_key
            .as_ref()
            .ok_or_else(|| Error::Configuration("OpenAI API key not found".to_string()))?;

        let client = Client::builder()
            .timeout(Duration::from_secs(config.llm.timeout_seconds))
            .build()?;

        Ok(Self { client, config })
    }
}

#[async_trait]
impl LlmBackend for OpenAiBackend {
    async fn generate_response(&self, prompt: &str) -> Result<String> {
        let request = OpenAiRequest {
            model: self.config.llm.model.clone(),
            messages: vec![OpenAiMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: self.config.llm.max_tokens,
            temperature: self.config.llm.temperature,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.config.llm.api_key.as_ref().unwrap()))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("OpenAI API error: {}", error_text);
            return Err(Error::LlmApi(error_text).into());
        }

        let response: OpenAiResponse = response.json().await?;
        
        Ok(response
            .choices
            .first()
            .and_then(|choice| choice.message.content.clone())
            .unwrap_or_else(|| "No response generated".to_string()))
    }
}

pub struct OllamaBackend {
    client: Client,
    config: Config,
}

impl OllamaBackend {
    pub fn new(config: Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.llm.timeout_seconds))
            .build()?;

        Ok(Self { client, config })
    }
}

#[async_trait]
impl LlmBackend for OllamaBackend {
    async fn generate_response(&self, prompt: &str) -> Result<String> {
        let request = OllamaRequest {
            model: self.config.llm.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let response = self
            .client
            .post("http://localhost:11434/api/generate")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("Ollama API error: {}", error_text);
            return Err(Error::LlmApi(error_text).into());
        }

        let response: OllamaResponse = response.json().await?;
        Ok(response.response)
    }
}

pub struct CustomBackend {
    client: Client,
    config: Config,
    url: String,
}

impl CustomBackend {
    pub fn new(config: Config, url: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.llm.timeout_seconds))
            .build()?;

        Ok(Self { client, config, url })
    }
}

#[async_trait]
impl LlmBackend for CustomBackend {
    async fn generate_response(&self, prompt: &str) -> Result<String> {
        let request = CustomRequest {
            prompt: prompt.to_string(),
            model: self.config.llm.model.clone(),
            max_tokens: self.config.llm.max_tokens,
            temperature: self.config.llm.temperature,
        };

        let response = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("Custom LLM API error: {}", error_text);
            return Err(Error::LlmApi(error_text).into());
        }

        let response: CustomResponse = response.json().await?;
        Ok(response.response)
    }
}

// Request/Response structures for different backends

#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    max_tokens: usize,
    temperature: f32,
}

#[derive(Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Serialize)]
struct CustomRequest {
    prompt: String,
    model: String,
    max_tokens: usize,
    temperature: f32,
}

#[derive(Deserialize)]
struct CustomResponse {
    response: String,
} 