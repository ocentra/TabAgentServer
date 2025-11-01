//! Main entry point for MCP transport
//!
//! This can be used for standalone MCP server mode or integrated into the main server

use storage::DatabaseCoordinator;
use appstate::{AppState, AppStateConfig};
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    info!("Initializing MCP transport server");
    
    // Initialize AppState (for model/system operations)
    let config = AppStateConfig::default();
    let state = Arc::new(AppState::new(config).await?);
    
    // Initialize storage coordinator (for logs)
    let coordinator = Arc::new(DatabaseCoordinator::new()?);
    
    // Create MCP manager
    let manager = tabagent_mcp::McpManager::new(state, coordinator);
    
    info!("Starting MCP stdio transport");
    
    // Run stdio transport
    tabagent_mcp::transport::run_stdio_transport(manager).await?;
    
    Ok(())
}
