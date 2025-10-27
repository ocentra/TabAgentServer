//! Native messaging module (placeholder - will be moved to separate crate)
//!
//! TODO: This will become tabagent-native-messaging crate

use std::sync::Arc;
use crate::state::AppState;

/// Run the native messaging protocol (stdin/stdout)
pub async fn run(_state: Arc<AppState>) -> anyhow::Result<()> {
    tracing::info!("Native messaging protocol started (PLACEHOLDER)");
    
    // TODO: Implement Chrome native messaging protocol
    // For now, just keep the process alive
    tokio::signal::ctrl_c().await?;
    
    Ok(())
}

