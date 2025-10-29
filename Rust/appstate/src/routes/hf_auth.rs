//! HuggingFace authentication handlers.

use anyhow::{Context, Result};
use tabagent_values::ResponseValue;

use crate::AppState;

/// Handle set HuggingFace token request.
pub async fn handle_set_token(state: &AppState, token: &str) -> Result<ResponseValue> {
    tracing::info!("Setting HuggingFace auth token");
    
    state.hf_auth.set_token(token)
        .context("Failed to store HF token")?;
    
    Ok(ResponseValue::success("HuggingFace token stored securely"))
}

/// Handle get HuggingFace token status request.
pub async fn handle_get_token_status(state: &AppState) -> Result<ResponseValue> {
    tracing::debug!("Checking HuggingFace token status");
    
    let has_token = state.hf_auth.has_token();
    
    Ok(ResponseValue::generic(serde_json::json!({
        "hasToken": has_token,
        "message": if has_token { "Token is stored" } else { "No token stored" }
    })))
}

/// Handle clear HuggingFace token request.
pub async fn handle_clear_token(state: &AppState) -> Result<ResponseValue> {
    tracing::info!("Clearing HuggingFace auth token");
    
    state.hf_auth.clear_token()
        .context("Failed to clear HF token")?;
    
    Ok(ResponseValue::success("HuggingFace token removed"))
}

