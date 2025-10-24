//! Associative linker module - creates semantic similarity connections.
//!
//! This module finds semantically similar nodes and creates
//! IS_SEMANTICALLY_SIMILAR_TO edges between them.

use crate::{WeaverContext, WeaverResult};
use common::{NodeId, EdgeId, models::Edge};

/// Processes a newly created node to find and link similar content.
///
/// This runs AFTER semantic indexing has generated an embedding.
pub async fn on_node_created(
    context: &WeaverContext,
    node_id: &str,
    node_type: &str,
) -> WeaverResult<()> {
    log::debug!("Associative linker: Processing {} ({})", node_id, node_type);
    
    // Only link certain node types
    if !should_create_associations(node_type) {
        return Ok(());
    }
    
    // Get the node's embedding
    let embedding = match context.coordinator.embeddings_active().get_embedding_by_node(node_id)? {
        Some(emb) => emb,
        None => {
            log::debug!("No embedding yet for {}, skipping associative linking", node_id);
            return Ok(());
        }
    };
    
    // Search for similar nodes (top 5, within last 30 days)
    let similar_nodes = context.conversations_index.search_vectors(&embedding.vector, 6)?; // +1 for self
    
    let similarity_threshold = 0.85; // High threshold for associative links
    let mut links_created = 0;
    
    for result in similar_nodes {
        // Skip self and low-similarity nodes
        if result.id.as_str() == node_id || result.distance < similarity_threshold {
            continue;
        }
        
        // Check if edge already exists
        // TODO: Implement proper check for existing edge
        
        // Create associative edge
        create_similarity_edge(context, node_id, result.id.as_str(), result.distance).await?;
        links_created += 1;
        
        // Limit number of links per node
        if links_created >= 3 {
            break;
        }
    }
    
    if links_created > 0 {
        log::info!(
            "Created {} associative links for node {}",
            links_created,
            node_id
        );
    }
    
    Ok(())
}

/// Determines if associative links should be created for this node type.
fn should_create_associations(node_type: &str) -> bool {
    matches!(node_type, "Message" | "Summary" | "ScrapedPage")
}

/// Creates an IS_SEMANTICALLY_SIMILAR_TO edge between two nodes.
async fn create_similarity_edge(
    context: &WeaverContext,
    from_node_id: &str,
    to_node_id: &str,
    similarity_score: f32,
) -> WeaverResult<()> {
    let edge_id = format!("edge_{}", uuid::Uuid::new_v4());
    let edge = Edge {
        id: EdgeId::from(edge_id.as_str()),
        from_node: NodeId::from(from_node_id),
        to_node: NodeId::from(to_node_id),
        edge_type: "IS_SEMANTICALLY_SIMILAR_TO".to_string(),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64,
        metadata: serde_json::json!({
            "similarity_score": similarity_score,
        }),
    };
    
    context.coordinator.knowledge_active().insert_edge(&edge)?;
    
    log::debug!(
        "Created similarity edge: {} -> {} (score: {:.2})",
        from_node_id,
        to_node_id,
        similarity_score
    );
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_create_associations() {
        assert!(should_create_associations("Message"));
        assert!(should_create_associations("Summary"));
        assert!(!should_create_associations("Entity"));
        assert!(!should_create_associations("Attachment"));
    }
}

