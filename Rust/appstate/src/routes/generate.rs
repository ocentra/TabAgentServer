//! Text generation handler.

use anyhow::Result;
use tabagent_values::ResponseValue;

use crate::AppState;

/// Handle text generation request (converts to chat).
pub async fn handle(
    state: &AppState,
    model: &str,
    prompt: &str,
    temperature: Option<f32>,
) -> Result<ResponseValue> {
    tracing::info!("Generate request for model: {}", model);

    // Convert to single-message chat
    let messages = vec![tabagent_values::Message {
        role: tabagent_values::MessageRole::User,
        content: prompt.to_string(),
        name: None,
    }];

    super::chat::handle(state, model, &messages, temperature).await
}

