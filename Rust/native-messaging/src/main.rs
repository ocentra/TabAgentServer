//! Native messaging host binary entry point.
//!
//! This binary serves as the Chrome native messaging host that communicates
//! with Chrome extensions via stdin/stdout using Chrome's native messaging protocol.

use clap::Parser;
use std::sync::Arc;
use std::path::PathBuf;
use tabagent_native_messaging::{run_host_with_state, NativeMessagingConfig};
use tabagent_server::{AppState, CliArgs as ServerCliArgs, ServerMode};
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
    
    /// Database path
    #[arg(long, default_value = "./data/db")]
    db_path: String,
    
    /// Model cache path
    #[arg(long, default_value = "./data/models")]
    model_cache_path: String,
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
    
    // Create real TabAgent server state (uses actual backend)
    tracing::info!("Initializing TabAgent server state...");
    let server_args = ServerCliArgs {
        mode: ServerMode::Native,
        port: 8001, // Not used in native mode
        config: PathBuf::from("server.toml"),
        db_path: PathBuf::from(&args.db_path),
        model_cache_path: PathBuf::from(&args.model_cache_path),
        log_level: args.log_level.clone(),
        webrtc_enabled: false,
        webrtc_port: 8002,
    };
    
    let state = AppState::new(&server_args).await?;
    tracing::info!("TabAgent server state initialized successfully");
    
    // Wrap in Arc<dyn AppStateProvider> for trait object
    let state: Arc<dyn common::backend::AppStateProvider> = Arc::new(state);
    
    // Run the native messaging host
    tracing::info!("Starting native messaging host");
    run_host_with_state(state, config).await?;
    
    Ok(())
}