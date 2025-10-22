//! Semantic indexer module - generates vector embeddings for nodes.
//!
//! This module is triggered when new nodes are created or updated.
//! It extracts text content from nodes and generates embeddings using the ML bridge.

use crate::{WeaverContext, WeaverResult};
use common::models::{Embedding, Node};

/// Processes a newly created node for semantic indexing.
///
/// Extracts text content, generates embedding via ML bridge, and stores it.
pub async fn on_node_created(
    context: &WeaverContext,
    node_id: &str,
    node_type: &str,
) -> WeaverResult<()> {
    log::debug!("Semantic indexer: Processing {} ({})", node_id, node_type);
    
    // Only index text-bearing node types
    if !should_index_node_type(node_type) {
        log::debug!("Skipping semantic indexing for node type: {}", node_type);
        return Ok(());
    }
    
    // Load the node
    let node = match context.storage.get_node(node_id)? {
        Some(n) => n,
        None => {
            log::warn!("Node {} not found for semantic indexing", node_id);
            return Ok(());
        }
    };
    
    // Check if already has embedding
    if has_embedding(&node) {
        log::debug!("Node {} already has embedding, skipping", node_id);
        return Ok(());
    }
    
    // Extract text content
    let text = match extract_text_content(&node) {
        Some(t) if !t.trim().is_empty() => t,
        _ => {
            log::debug!("No text content found for node {}", node_id);
            return Ok(());
        }
    };
    
    log::debug!("Generating embedding for {} chars of text", text.len());
    
    // Generate embedding via ML bridge
    let vector = context.ml_bridge.generate_embedding(&text).await
        .map_err(|e| crate::WeaverError::MlInference(e.to_string()))?;
    
    // Create Embedding object
    let embedding_id = format!("emb_{}", uuid::Uuid::new_v4());
    let embedding = Embedding {
        id: embedding_id.clone(),
        vector,
        model: "default".to_string(), // TODO: Get model name from ML bridge
    };
    
    // Store embedding (this will also update the vector index via storage's auto-indexing)
    context.storage.insert_embedding(&embedding)?;
    
    log::info!("Generated embedding {} for node {}", embedding_id, node_id);
    
    // TODO: Update the node to include embedding_id
    // This requires updating the node's embedding_id field
    // For now, the node will need to be updated separately
    
    Ok(())
}

/// Processes an updated node for re-indexing.
pub async fn on_node_updated(
    context: &WeaverContext,
    node_id: &str,
    node_type: &str,
) -> WeaverResult<()> {
    log::debug!("Semantic indexer: Re-indexing {} ({})", node_id, node_type);
    
    // For now, treat updates the same as creation
    // TODO: Check if content actually changed before re-indexing
    on_node_created(context, node_id, node_type).await
}

/// Determines if a node type should be semantically indexed.
fn should_index_node_type(node_type: &str) -> bool {
    matches!(
        node_type,
        "Message" | "Summary" | "Entity" | "ScrapedPage" | "WebSearch" | "AudioTranscript"
    )
}

/// Checks if a node already has an embedding.
fn has_embedding(node: &Node) -> bool {
    match node {
        Node::Message(m) => m.embedding_id.is_some(),
        Node::Summary(s) => s.embedding_id.is_some(),
        Node::Entity(e) => e.embedding_id.is_some(),
        _ => false,
    }
}

/// Extracts text content from a node.
fn extract_text_content(node: &Node) -> Option<String> {
    match node {
        Node::Message(m) => Some(m.text_content.clone()),
        Node::Summary(s) => Some(s.content.clone()),
        Node::Entity(e) => Some(e.label.clone()),
        Node::Chat(c) => Some(format!("{} - {}", c.title, c.topic)),
        Node::ScrapedPage(p) => {
            let title = p.title.as_deref().unwrap_or("");
            Some(format!("{} {}", title, p.text_content))
        }
        Node::WebSearch(w) => Some(w.query.clone()),
        Node::AudioTranscript(a) => Some(a.transcript.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_index_node_type() {
        assert!(should_index_node_type("Message"));
        assert!(should_index_node_type("Summary"));
        assert!(should_index_node_type("Entity"));
        assert!(!should_index_node_type("Attachment"));
        assert!(!should_index_node_type("Edge"));
    }
}

