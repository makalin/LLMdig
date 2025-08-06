pub mod config;
pub mod dns;
pub mod error;
pub mod llm;
pub mod server;
pub mod utils;

pub use config::Config;
pub use dns::DnsHandler;
pub use error::Error;
pub use llm::{LlmBackend, LlmClient};
pub use server::DnsServer;

// Re-export common types
pub use anyhow::Result; 