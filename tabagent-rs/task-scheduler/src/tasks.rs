//! Task definitions and execution.
//!
//! This module defines the types of background tasks that can be scheduled
//! and how they are executed.

use crate::queue::TaskPriority;
use common::NodeId;
use thiserror::Error;

/// A background task that can be scheduled for execution.
#[derive(Debug, Clone)]
pub enum Task {
    /// Generate a vector embedding for text content.
    ///
    /// This is the most common background task. Embeddings enable semantic search.
    GenerateEmbedding {
        node_id: NodeId,
        text: String,
        priority: TaskPriority,
    },
    
    /// Extract entities (people, places, concepts) from text.
    ///
    /// Uses NER (Named Entity Recognition) to identify and create Entity nodes.
    ExtractEntities {
        node_id: NodeId,
        text: String,
        priority: TaskPriority,
    },
    
    /// Link entities across the knowledge graph.
    ///
    /// Creates MENTIONS edges between messages and entities.
    LinkEntities {
        node_id: NodeId,
        entity_ids: Vec<NodeId>,
        priority: TaskPriority,
    },
    
    /// Generate a summary of a conversation or section.
    ///
    /// Uses LLM to create hierarchical memory summaries.
    GenerateSummary {
        chat_id: NodeId,
        message_ids: Vec<NodeId>,
        priority: TaskPriority,
    },
    
    /// Find and create associative links between semantically similar content.
    ///
    /// Creates IS_SEMANTICALLY_SIMILAR_TO edges, enabling "intuitive leaps".
    CreateAssociativeLinks {
        node_id: NodeId,
        similarity_threshold: f32,
        priority: TaskPriority,
    },
    
    /// Index a newly created node in the structural/graph indexes.
    ///
    /// This is typically URGENT priority for instant recall.
    IndexNode {
        node_id: NodeId,
        priority: TaskPriority,
    },
    
    /// Update the HNSW vector index with a new embedding.
    ///
    /// Usually NORMAL priority as it's needed for search but not instant.
    UpdateVectorIndex {
        embedding_id: String,
        vector: Vec<f32>,
        priority: TaskPriority,
    },
    
    /// Rotate memory layers (hot → warm → cold).
    ///
    /// Runs periodically to manage the hybrid vector index.
    RotateMemoryLayers {
        priority: TaskPriority,
    },
    
    /// Encrypt and backup recent data.
    ///
    /// Runs during sleep mode to ensure data safety.
    BackupData {
        since_timestamp: i64,
        priority: TaskPriority,
    },
}

impl Task {
    /// Returns the priority of this task.
    pub fn priority(&self) -> TaskPriority {
        match self {
            Task::GenerateEmbedding { priority, .. } => *priority,
            Task::ExtractEntities { priority, .. } => *priority,
            Task::LinkEntities { priority, .. } => *priority,
            Task::GenerateSummary { priority, .. } => *priority,
            Task::CreateAssociativeLinks { priority, .. } => *priority,
            Task::IndexNode { priority, .. } => *priority,
            Task::UpdateVectorIndex { priority, .. } => *priority,
            Task::RotateMemoryLayers { priority, .. } => *priority,
            Task::BackupData { priority, .. } => *priority,
        }
    }
    
    /// Returns a human-readable name for this task type.
    pub fn name(&self) -> &str {
        match self {
            Task::GenerateEmbedding { .. } => "GenerateEmbedding",
            Task::ExtractEntities { .. } => "ExtractEntities",
            Task::LinkEntities { .. } => "LinkEntities",
            Task::GenerateSummary { .. } => "GenerateSummary",
            Task::CreateAssociativeLinks { .. } => "CreateAssociativeLinks",
            Task::IndexNode { .. } => "IndexNode",
            Task::UpdateVectorIndex { .. } => "UpdateVectorIndex",
            Task::RotateMemoryLayers { .. } => "RotateMemoryLayers",
            Task::BackupData { .. } => "BackupData",
        }
    }
    
    /// Executes the task.
    ///
    /// This is currently a placeholder. Actual implementations will be added
    /// in later phases when the corresponding systems (embeddings, NER, etc.) are built.
    pub async fn execute(&self) -> Result<Option<TaskResult>, TaskError> {
        match self {
            Task::GenerateEmbedding { node_id, text, .. } => {
                // TODO: Call actual embedding generation
                // For now, simulate work
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                println!("[Task] Generated embedding for node {}: \"{}...\"", 
                    node_id, 
                    text.chars().take(30).collect::<String>()
                );
                Ok(None)
            }
            
            Task::ExtractEntities { node_id, .. } => {
                // TODO: Call actual NER
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                println!("[Task] Extracted entities from node {}", node_id);
                Ok(None)
            }
            
            Task::LinkEntities { node_id, entity_ids, .. } => {
                // TODO: Create edges in graph
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                println!("[Task] Linked {} entities to node {}", entity_ids.len(), node_id);
                Ok(None)
            }
            
            Task::GenerateSummary { chat_id, message_ids, .. } => {
                // TODO: Call LLM for summarization
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                println!("[Task] Generated summary for chat {} ({} messages)", 
                    chat_id, message_ids.len()
                );
                Ok(None)
            }
            
            Task::CreateAssociativeLinks { node_id, similarity_threshold, .. } => {
                // TODO: Vector search + edge creation
                tokio::time::sleep(std::time::Duration::from_millis(30)).await;
                println!("[Task] Created associative links for node {} (threshold: {})", 
                    node_id, similarity_threshold
                );
                Ok(None)
            }
            
            Task::IndexNode { node_id, .. } => {
                // TODO: Update structural/graph indexes
                tokio::time::sleep(std::time::Duration::from_millis(2)).await;
                println!("[Task] Indexed node {}", node_id);
                Ok(None)
            }
            
            Task::UpdateVectorIndex { embedding_id, .. } => {
                // TODO: Update HNSW index
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
                println!("[Task] Updated vector index for embedding {}", embedding_id);
                Ok(None)
            }
            
            Task::RotateMemoryLayers { .. } => {
                // TODO: Move vectors between hot/warm/cold layers
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                println!("[Task] Rotated memory layers");
                Ok(None)
            }
            
            Task::BackupData { since_timestamp, .. } => {
                // TODO: Encrypt and backup
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                println!("[Task] Backed up data since timestamp {}", since_timestamp);
                Ok(None)
            }
        }
    }
}

/// Result of a task execution (if any).
#[derive(Debug, Clone)]
pub enum TaskResult {
    /// Embedding was generated successfully.
    EmbeddingGenerated {
        embedding_id: String,
        vector_size: usize,
    },
    
    /// Entities were extracted.
    EntitiesExtracted {
        entity_ids: Vec<NodeId>,
    },
    
    /// Summary was generated.
    SummaryGenerated {
        summary_id: NodeId,
    },
    
    /// Generic success message.
    Success(String),
}

/// Error that can occur during task execution.
#[derive(Debug, Error)]
pub enum TaskError {
    #[error("Task scheduler has shut down")]
    SchedulerShutdown,
    
    #[error("Failed to generate embedding: {0}")]
    EmbeddingFailed(String),
    
    #[error("Failed to extract entities: {0}")]
    EntityExtractionFailed(String),
    
    #[error("Failed to generate summary: {0}")]
    SummarizationFailed(String),
    
    #[error("Failed to update index: {0}")]
    IndexUpdateFailed(String),
    
    #[error("Task execution error: {0}")]
    ExecutionError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_task_execution() {
        let task = Task::GenerateEmbedding {
            node_id: "msg_123".to_string(),
            text: "Hello world".to_string(),
            priority: TaskPriority::Normal,
        };
        
        let result = task.execute().await;
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_task_priority() {
        let urgent_task = Task::IndexNode {
            node_id: "node_1".to_string(),
            priority: TaskPriority::Urgent,
        };
        assert_eq!(urgent_task.priority(), TaskPriority::Urgent);
        
        let batch_task = Task::GenerateSummary {
            chat_id: "chat_1".to_string(),
            message_ids: vec![],
            priority: TaskPriority::Batch,
        };
        assert_eq!(batch_task.priority(), TaskPriority::Batch);
    }
}

