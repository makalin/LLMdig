use anyhow::Result;
use clap::Parser;
use dotenv::dotenv;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use llmdig::config::Config;
use llmdig::server::DnsServer;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: Level,

    /// Port to bind the DNS server to
    #[arg(short, long)]
    port: Option<u16>,

    /// Host to bind the DNS server to
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Parse command line arguments
    let args = Args::parse();

    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(args.log_level)
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("Starting LLMdig DNS server...");

    // Load configuration
    let mut config = Config::load(&args.config)?;
    
    // Override config with command line arguments
    if let Some(port) = args.port {
        config.server.port = port;
    }
    config.server.host = args.host;

    info!("Configuration loaded: {:?}", config);

    // Create and start DNS server
    let server = DnsServer::new(config)?;
    
    info!("DNS server starting on {}:{}", server.host(), server.port());
    
    // Run the server
    if let Err(e) = server.run().await {
        error!("Server error: {}", e);
        std::process::exit(1);
    }

    Ok(())
} 