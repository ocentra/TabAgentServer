//! Parameter management handlers.

use anyhow::Result;
use tabagent_values::{ResponseValue, TokenUsage};

use crate::AppState;

/// Handle get params request.
///
/// Returns default generation parameters.
/// Note: Per-session parameter storage is not yet implemented.
pub async fn handle_get_params(_state: &AppState) -> Result<ResponseValue> {
    Ok(ResponseValue::chat(
        "params",
        "system",
        serde_json::json!({
            "temperature": 0.7,
            "max_tokens": 512,
            "top_p": 0.9,
            "top_k": 50
        }).to_string(),
        TokenUsage::zero(),
    ))
}

/// Handle set params request.
///
/// Note: Parameter persistence is not yet implemented.
/// Parameters are acknowledged but not stored.
pub async fn handle_set_params(_state: &AppState, params: &serde_json::Value) -> Result<ResponseValue> {
    tracing::info!("Setting params: {:?}", params);
    Ok(ResponseValue::chat(
        "params_set",
        "system",
        format!("Parameters acknowledged: {}", params),
        TokenUsage::zero(),
    ))
}

