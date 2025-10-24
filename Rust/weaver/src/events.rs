//! Event definitions for the Knowledge Weaver.
//!
//! Events represent changes in the database that trigger autonomous enrichment.

use common::NodeId;
use serde::{Deserialize, Serialize};

/// Events that the Knowledge Weaver responds to.
///
/// These events are emitted by the storage layer when data changes occur.
/// The Weaver listens for these events and triggers appropriate enrichment tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeaverEvent {
    /// A new node was created in the database.
    ///
    /// Triggers:
    /// - Semantic indexing (generate embeddings)
    /// - Entity extraction and linking
    /// - Associative linking (find similar nodes)
    NodeCreated {
        /// ID of the newly created node
        node_id: NodeId,
        
        /// Type of node (e.g., "Message", "Chat", "Entity")
        node_type: String,
    },

    /// A node was updated.
    ///
    /// May trigger re-indexing if content changed.
    NodeUpdated {
        /// ID of the updated node
        node_id: NodeId,
        
        /// Type of node
        node_type: String,
    },

    /// A chat was updated (e.g., new messages added).
    ///
    /// Triggers:
    /// - Topic modeling
    /// - Summarization (if enough new messages)
    ChatUpdated {
        /// ID of the chat
        chat_id: NodeId,
        
        /// Number of messages since last summary
        messages_since_summary: usize,
    },

    /// Multiple messages were added in a batch.
    ///
    /// Allows for batch processing optimizations.
    BatchMessagesAdded {
        /// ID of the chat
        chat_id: NodeId,
        
        /// IDs of new messages
        message_ids: Vec<NodeId>,
    },

    /// An edge was created.
    ///
    /// May trigger graph analysis updates.
    EdgeCreated {
        /// ID of the edge
        edge_id: String,
        
        /// Type of edge (e.g., "MENTIONS", "CONTAINS")
        edge_type: String,
    },
}

impl WeaverEvent {
    /// Returns a human-readable description of the event.
    pub fn description(&self) -> String {
        match self {
            Self::NodeCreated { node_type, .. } => format!("Node created: {}", node_type),
            Self::NodeUpdated { node_type, .. } => format!("Node updated: {}", node_type),
            Self::ChatUpdated { messages_since_summary, .. } => {
                format!("Chat updated ({} new messages)", messages_since_summary)
            }
            Self::BatchMessagesAdded { message_ids, .. } => {
                format!("Batch of {} messages added", message_ids.len())
            }
            Self::EdgeCreated { edge_type, .. } => format!("Edge created: {}", edge_type),
        }
    }

    /// Returns the primary node ID associated with this event.
    pub fn primary_node_id(&self) -> Option<&NodeId> {
        match self {
            Self::NodeCreated { node_id, .. } => Some(node_id),
            Self::NodeUpdated { node_id, .. } => Some(node_id),
            Self::ChatUpdated { chat_id, .. } => Some(chat_id),
            Self::BatchMessagesAdded { chat_id, .. } => Some(chat_id),
            Self::EdgeCreated { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_description() {
        let event = WeaverEvent::NodeCreated {
            node_id: "node_123".to_string().into(),
            node_type: "Message".to_string(),
        };
        
        assert_eq!(event.description(), "Node created: Message");
    }

    #[test]
    fn test_primary_node_id() {
        let chat_id: NodeId = "chat_456".to_string().into();
        let event = WeaverEvent::ChatUpdated {
            chat_id: chat_id.clone(),
            messages_since_summary: 5,
        };
        
        assert_eq!(event.primary_node_id(), Some(&chat_id));
    }
}

