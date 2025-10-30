//! Session and chat history handlers.

use anyhow::{Context, Result};
use tabagent_values::{ResponseValue, TokenUsage};
use storage::Message as DbMessage;

use crate::AppState;

/// Handle chat history request.
pub async fn handle_chat_history(
    _state: &AppState,
    session_id: Option<&str>,
) -> Result<ResponseValue> {
    tracing::info!("Chat history request for session: {:?}", session_id);
    
    let _session = session_id.unwrap_or("default");
    
    // Note: Chat history retrieval requires storage crate to expose message queries
    anyhow::bail!("Chat history retrieval not yet implemented in storage crate")
}

/// Handle save message request.
pub async fn handle_save_message(
    state: &AppState,
    session_id: &str,
    message: &tabagent_values::Message,
) -> Result<ResponseValue> {
    tracing::info!("Save message to session: {}", session_id);
    
    // Convert tabagent_values::Message to storage Message
    let sender = match message.role {
        tabagent_values::MessageRole::User => "user".to_string(),
        tabagent_values::MessageRole::Assistant => "assistant".to_string(),
        tabagent_values::MessageRole::System => "system".to_string(),
        tabagent_values::MessageRole::Function => "function".to_string(),
    };

    let db_message = DbMessage {
        id: common::NodeId::new(uuid::Uuid::new_v4().to_string()),
        chat_id: common::NodeId::new(session_id),
        sender,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0) as i64,
        text_content: message.content.clone(),
        attachment_ids: vec![],
        embedding_id: None,
        metadata: serde_json::json!({}),
    };

    // Save to database using the new DatabaseClient
    // Get the underlying coordinator for direct access (in-process mode)
    let coordinator = state.db_client.coordinator()
        .ok_or_else(|| anyhow::anyhow!("Database client is not in in-process mode"))?;
    
    use storage::traits::ConversationOperations;
    coordinator.conversation_manager.insert_message(db_message)
        .context("Failed to save message to database")?;
    
    Ok(ResponseValue::chat(
        "saved",
        "system",
        format!("Message saved to session '{}'", session_id),
        TokenUsage::zero(),
    ))
}

