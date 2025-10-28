//! Native messaging host binary entry point.
//!
//! This binary serves as the Chrome native messaging host that communicates
//! with Chrome extensions via stdin/stdout using Chrome's native messaging protocol.

use clap::Parser;
use std::sync::Arc;
use tabagent_native_messaging::{run_host_with_state, AppStateProvider, NativeMessagingConfig};
use tabagent_values::{RequestValue, ResponseValue};
use tracing_subscriber;

/// Command line arguments for the native messaging host
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,
    
    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,
    
    /// Enable development mode with additional logging
    #[arg(short, long)]
    dev: bool,
}

/// Mock application state for testing
/// TODO: Replace with actual TabAgent server state integration
struct MockAppState;

#[async_trait::async_trait]
impl AppStateProvider for MockAppState {
    async fn handle_request(&self, request: RequestValue) -> anyhow::Result<ResponseValue> {
        // TODO: Integrate with actual TabAgent server handler
        // For now, return mock responses based on request type
        
        match request.request_type() {
            tabagent_values::RequestType::Health => {
                Ok(ResponseValue::health(tabagent_values::HealthStatus::Healthy))
            }
            tabagent_values::RequestType::Chat { .. } => {
                // Mock chat response
                Ok(ResponseValue::chat(
                    "chat-123".to_string(),
                    "mock-model".to_string(),
                    "Hello! This is a mock response from the native messaging host.".to_string(),
                    tabagent_values::TokenUsage {
                        prompt_tokens: 10,
                        completion_tokens: 15,
                        total_tokens: 25,
                    },
                ))
            }
            _ => {
                // Return a generic success response for other request types
                Ok(ResponseValue::health(tabagent_values::HealthStatus::Healthy))
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    let log_level = match args.log_level.to_lowercase().as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };
    
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();
    
    tracing::info!("TabAgent Native Messaging Host starting");
    tracing::info!("Log level: {}", args.log_level);
    tracing::info!("Development mode: {}", args.dev);
    
    // Load configuration
    let config = if let Some(config_path) = args.config {
        tracing::info!("Loading configuration from: {}", config_path);
        NativeMessagingConfig::from_file(&config_path)?
    } else {
        tracing::info!("Using default configuration");
        NativeMessagingConfig::default()
    };
    
    // Create application state
    // TODO: Replace MockAppState with actual TabAgent server state
    let state: Arc<dyn AppStateProvider> = Arc::new(MockAppState);
    
    // Run the native messaging host
    tracing::info!("Starting message processing loop");
    run_host_with_state(state, config).await?;
    
    Ok(())
}