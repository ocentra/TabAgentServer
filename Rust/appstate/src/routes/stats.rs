//! Statistics and metrics handlers.

use anyhow::Result;
use tabagent_values::ResponseValue;

use crate::AppState;

/// Handle get stats request.
pub async fn handle_get_stats(state: &AppState) -> Result<ResponseValue> {
    let stats = serde_json::json!({
        "models_loaded": state.list_loaded_models().len(),
        "uptime_seconds": 0, // Uptime tracking not implemented
        "total_requests": 0, // Request counting not implemented
        "memory_used_mb": 0, // Memory tracking not implemented
    });
    
    Ok(ResponseValue::chat(
        "stats",
        "system",
        stats.to_string(),
        tabagent_values::TokenUsage::zero(),
    ))
}

