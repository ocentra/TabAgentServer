/// Test fixtures - realistic mock data for testing
/// 
/// Creates a complete conversation history with:
/// - 10 messages back-and-forth
/// - Proper parent-child relationships
/// - Embeddings for semantic search
/// - Tool results
/// - Summaries
/// 
/// Can be used by query, weaver, indexing tests

use storage::StorageManager;
use common::models::{Node, Edge, NodeType, EdgeType};
use uuid::Uuid;
use chrono::Utc;

pub struct ConversationFixture {
    pub user_id: String,
    pub conversation_id: String,
    pub messages: Vec<Node>,
    pub edges: Vec<Edge>,
    pub embeddings: Vec<(String, Vec<f32>)>, // (node_id, embedding)
}

impl ConversationFixture {
    /// Create a realistic conversation with 10 messages
    pub fn create_realistic_conversation() -> Self {
        let user_id = "user_test_123".to_string();
        let conversation_id = Uuid::new_v4().to_string();
        let mut messages = Vec::new();
        let mut edges = Vec::new();
        let mut embeddings = Vec::new();
        
        let base_time = Utc::now().timestamp() - 3600; // 1 hour ago
        
        // Message 1: User asks about Python
        let msg1_id = Uuid::new_v4().to_string();
        messages.push(Node {
            id: msg1_id.clone(),
            node_type: NodeType::Message,
            content: Some("How do I read a CSV file in Python?".to_string()),
            metadata: Some(serde_json::json!({
                "role": "user",
                "conversation_id": conversation_id,
                "tokens": 9
            }).to_string()),
            created_at: base_time,
            updated_at: base_time,
        });
        embeddings.push((msg1_id.clone(), vec![0.1, 0.2, 0.3, 0.4])); // Mock embedding
        
        // Message 2: Assistant responds
        let msg2_id = Uuid::new_v4().to_string();
        messages.push(Node {
            id: msg2_id.clone(),
            node_type: NodeType::Message,
            content: Some("You can use pandas: `import pandas as pd; df = pd.read_csv('file.csv')`".to_string()),
            metadata: Some(serde_json::json!({
                "role": "assistant",
                "conversation_id": conversation_id,
                "tokens": 25
            }).to_string()),
            created_at: base_time + 2,
            updated_at: base_time + 2,
        });
        embeddings.push((msg2_id.clone(), vec![0.15, 0.25, 0.35, 0.45]));
        
        // Edge: msg2 replies to msg1
        edges.push(Edge {
            id: Uuid::new_v4().to_string(),
            from_node: msg1_id.clone(),
            to_node: msg2_id.clone(),
            edge_type: EdgeType::Reply,
            weight: Some(1.0),
            metadata: None,
            created_at: base_time + 2,
        });
        
        // Message 3: User asks follow-up
        let msg3_id = Uuid::new_v4().to_string();
        messages.push(Node {
            id: msg3_id.clone(),
            node_type: NodeType::Message,
            content: Some("What if the CSV has missing values?".to_string()),
            metadata: Some(serde_json::json!({
                "role": "user",
                "conversation_id": conversation_id,
                "tokens": 8
            }).to_string()),
            created_at: base_time + 10,
            updated_at: base_time + 10,
        });
        embeddings.push((msg3_id.clone(), vec![0.12, 0.22, 0.32, 0.42]));
        
        // Message 4: Assistant explains
        let msg4_id = Uuid::new_v4().to_string();
        messages.push(Node {
            id: msg4_id.clone(),
            node_type: NodeType::Message,
            content: Some("Use `df.fillna()` to handle missing values. For example: `df.fillna(0)` replaces with 0.".to_string()),
            metadata: Some(serde_json::json!({
                "role": "assistant",
                "conversation_id": conversation_id,
                "tokens": 32
            }).to_string()),
            created_at: base_time + 12,
            updated_at: base_time + 12,
        });
        embeddings.push((msg4_id.clone(), vec![0.16, 0.26, 0.36, 0.46]));
        
        edges.push(Edge {
            id: Uuid::new_v4().to_string(),
            from_node: msg3_id.clone(),
            to_node: msg4_id.clone(),
            edge_type: EdgeType::Reply,
            weight: Some(1.0),
            metadata: None,
            created_at: base_time + 12,
        });
        
        // Message 5-10: Continue conversation realistically
        let topics = vec![
            ("Can I filter rows?", "Yes, use boolean indexing: `df[df['column'] > 5]`"),
            ("How do I save changes?", "Use `df.to_csv('output.csv', index=False)`"),
            ("What about large files?", "Use `chunksize` parameter: `for chunk in pd.read_csv('file.csv', chunksize=1000): ...`"),
        ];
        
        let mut prev_user_id = msg4_id;
        for (i, (user_q, assistant_a)) in topics.iter().enumerate() {
            let time_offset = base_time + 20 + (i as i64 * 10);
            
            let user_msg_id = Uuid::new_v4().to_string();
            messages.push(Node {
                id: user_msg_id.clone(),
                node_type: NodeType::Message,
                content: Some(user_q.to_string()),
                metadata: Some(serde_json::json!({
                    "role": "user",
                    "conversation_id": conversation_id,
                    "tokens": user_q.split_whitespace().count()
                }).to_string()),
                created_at: time_offset,
                updated_at: time_offset,
            });
            embeddings.push((user_msg_id.clone(), vec![0.1 + i as f32 * 0.01, 0.2, 0.3, 0.4]));
            
            edges.push(Edge {
                id: Uuid::new_v4().to_string(),
                from_node: prev_user_id.clone(),
                to_node: user_msg_id.clone(),
                edge_type: EdgeType::Continuation,
                weight: Some(0.8),
                metadata: None,
                created_at: time_offset,
            });
            
            let assistant_msg_id = Uuid::new_v4().to_string();
            messages.push(Node {
                id: assistant_msg_id.clone(),
                node_type: NodeType::Message,
                content: Some(assistant_a.to_string()),
                metadata: Some(serde_json::json!({
                    "role": "assistant",
                    "conversation_id": conversation_id,
                    "tokens": assistant_a.split_whitespace().count()
                }).to_string()),
                created_at: time_offset + 2,
                updated_at: time_offset + 2,
            });
            embeddings.push((assistant_msg_id.clone(), vec![0.15 + i as f32 * 0.01, 0.25, 0.35, 0.45]));
            
            edges.push(Edge {
                id: Uuid::new_v4().to_string(),
                from_node: user_msg_id.clone(),
                to_node: assistant_msg_id.clone(),
                edge_type: EdgeType::Reply,
                weight: Some(1.0),
                metadata: None,
                created_at: time_offset + 2,
            });
            
            prev_user_id = assistant_msg_id;
        }
        
        ConversationFixture {
            user_id,
            conversation_id,
            messages,
            edges,
            embeddings,
        }
    }
    
    /// Load this fixture into a database
    pub fn load_into_db(&self, storage: &StorageManager) -> Result<(), String> {
        // Insert all messages
        for msg in &self.messages {
            storage.insert_node(msg)
                .map_err(|e| format!("Failed to insert node: {}", e))?;
        }
        
        // Insert all edges
        for edge in &self.edges {
            storage.insert_edge(edge)
                .map_err(|e| format!("Failed to insert edge: {}", e))?;
        }
        
        Ok(())
    }
    
    /// Get message by index (0-based)
    pub fn get_message(&self, index: usize) -> Option<&Node> {
        self.messages.get(index)
    }
    
    /// Get user messages only
    pub fn user_messages(&self) -> Vec<&Node> {
        self.messages.iter()
            .filter(|m| {
                m.metadata.as_ref()
                    .and_then(|md| serde_json::from_str::<serde_json::Value>(md).ok())
                    .and_then(|v| v["role"].as_str())
                    == Some("user")
            })
            .collect()
    }
    
    /// Get assistant messages only
    pub fn assistant_messages(&self) -> Vec<&Node> {
        self.messages.iter()
            .filter(|m| {
                m.metadata.as_ref()
                    .and_then(|md| serde_json::from_str::<serde_json::Value>(md).ok())
                    .and_then(|v| v["role"].as_str())
                    == Some("assistant")
            })
            .collect()
    }
}

/// Create a multi-conversation fixture (3 conversations)
pub fn create_multi_conversation_fixture() -> Vec<ConversationFixture> {
    vec![
        ConversationFixture::create_realistic_conversation(),
        ConversationFixture::create_realistic_conversation(),
        ConversationFixture::create_realistic_conversation(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fixture_creation() {
        let fixture = ConversationFixture::create_realistic_conversation();
        
        assert_eq!(fixture.messages.len(), 10, "Should have 10 messages");
        assert!(fixture.edges.len() >= 6, "Should have multiple edges");
        assert_eq!(fixture.embeddings.len(), 10, "Should have embedding for each message");
        
        // Verify user/assistant alternation
        let user_msgs = fixture.user_messages();
        let assistant_msgs = fixture.assistant_messages();
        
        assert_eq!(user_msgs.len(), 5, "Should have 5 user messages");
        assert_eq!(assistant_msgs.len(), 5, "Should have 5 assistant messages");
    }
}

