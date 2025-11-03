//! Entity linker module - extracts and links named entities.
//!
//! This module performs Named Entity Recognition (NER) on text nodes
//! and creates Entity nodes and MENTIONS edges.

use crate::{WeaverContext, WeaverResult, WeaverError};
use common::{NodeId, EdgeId, models::{Edge, Entity as EntityNode, Node}};
use storage::traits::DirectAccessOperations;

/// Processes a newly created node for entity extraction and linking.
pub async fn on_node_created(
    context: &WeaverContext,
    node_id: &str,
    node_type: &str,
) -> WeaverResult<()> {
    log::debug!("Entity linker: Processing {} ({})", node_id, node_type);
    
    // Only extract entities from text-bearing nodes
    if !should_extract_entities(node_type) {
        return Ok(());
    }
    
    // Load the node
    let node = if let Some(node_ref) = context.coordinator.conversations_active().get_node_ref(node_id)? {
        node_ref.deserialize()
            .map_err(|e| WeaverError::Database(e))?
    } else {
        log::warn!("Node {} not found for entity linking", node_id);
        return Ok(());
    };
    
    // Extract text content
    let text = match extract_text_content(&node) {
        Some(t) if !t.trim().is_empty() => t,
        _ => {
            log::debug!("No text content for entity extraction in {}", node_id);
            return Ok(());
        }
    };
    
    // Extract entities via ML bridge
    let entities = context.ml_bridge.extract_entities(&text).await
        .map_err(|e| crate::WeaverError::MlInference(e.to_string()))?;
    
    if entities.is_empty() {
        log::debug!("No entities found in node {}", node_id);
        return Ok(());
    }
    
    log::info!("Found {} entities in node {}", entities.len(), node_id);
    
    // Process each extracted entity
    for entity in entities {
        // Create or find the Entity node
        let entity_id = create_or_find_entity(context, &entity.text, &entity.label).await?;
        
        // Create MENTIONS edge from the source node to the entity
        create_mentions_edge(context, node_id, &entity_id).await?;
    }
    
    Ok(())
}

/// Determines if entity extraction should be performed for this node type.
fn should_extract_entities(node_type: &str) -> bool {
    matches!(node_type, "Message" | "Summary" | "ScrapedPage")
}

/// Extracts text content from a node.
fn extract_text_content(node: &Node) -> Option<String> {
    match node {
        Node::Message(m) => Some(m.text_content.clone()),
        Node::Summary(s) => Some(s.content.clone()),
        Node::ScrapedPage(p) => {
            let title = p.title.as_deref().unwrap_or("");
            Some(format!("{} {}", title, p.text_content))
        }
        _ => None,
    }
}

/// Creates a new Entity node or returns existing one with same label+type.
async fn create_or_find_entity(
    context: &WeaverContext,
    label: &str,
    entity_type: &str,
) -> WeaverResult<String> {
    // Try to find existing entity with same label and type using structural index
    // Query by entity.label property first
    if let Ok(matching_ids) = context.knowledge_index.get_nodes_by_property("entity_label", label) {
        // Check each candidate to see if it matches both label and entity_type
        for candidate_id in matching_ids {
            // Load the entity node to verify entity_type matches
            if let Some(node_ref) = context.coordinator.knowledge_active().get_node_ref(candidate_id.as_str())? {
                let node = node_ref.deserialize()
                    .map_err(|e| WeaverError::Database(e))?;
                
                if let Node::Entity(entity) = node {
                    // Check if entity_type matches
                    if entity.entity_type == entity_type {
                        log::debug!("Found existing entity: {} ({})", label, entity_type);
                        return Ok(entity.id.as_str().to_string());
                    }
                }
            }
            
            // Also check stable knowledge tier
            if let Some(node_ref) = context.coordinator.knowledge_stable().get_node_ref(candidate_id.as_str())? {
                let node = node_ref.deserialize()
                    .map_err(|e| WeaverError::Database(e))?;
                
                if let Node::Entity(entity) = node {
                    if entity.entity_type == entity_type {
                        log::debug!("Found existing entity in stable: {} ({})", label, entity_type);
                        return Ok(entity.id.as_str().to_string());
                    }
                }
            }
        }
    }
    
    // No matching entity found, create a new one
    let entity_id = format!("ent_{}", uuid::Uuid::new_v4());
    let entity = EntityNode {
        id: NodeId::from(entity_id.as_str()),
        label: label.to_string(),
        entity_type: entity_type.to_string(),
        embedding_id: None,
        metadata: serde_json::json!({}).to_string(),
    };
    
    context.coordinator.knowledge_active().insert_node(&Node::Entity(entity))?;
    
    log::debug!("Created entity node: {} ({})", label, entity_type);
    
    Ok(entity_id)
}

/// Creates a MENTIONS edge from source node to entity.
async fn create_mentions_edge(
    context: &WeaverContext,
    from_node_id: &str,
    to_entity_id: &str,
) -> WeaverResult<()> {
    let edge_id = format!("edge_{}", uuid::Uuid::new_v4());
    let edge = Edge {
        id: EdgeId::from(edge_id.as_str()),
        from_node: NodeId::from(from_node_id),
        to_node: NodeId::from(to_entity_id),
        edge_type: "MENTIONS".to_string(),
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| WeaverError::Database(common::DbError::InvalidOperation(
                format!("System time error: {}", e)
            )))?
            .as_millis() as i64,
        metadata: serde_json::json!({}).to_string(),
    };
    
    context.coordinator.knowledge_active().insert_edge(&edge)?;
    
    log::debug!("Created MENTIONS edge: {} -> {}", from_node_id, to_entity_id);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_extract_entities() {
        assert!(should_extract_entities("Message"));
        assert!(should_extract_entities("Summary"));
        assert!(!should_extract_entities("Attachment"));
        assert!(!should_extract_entities("Entity"));
    }
}

