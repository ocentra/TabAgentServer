//! Database client for gRPC-based storage access
//! 
//! This module provides a unified interface for database operations that works
//! transparently with either in-process storage or remote gRPC storage.

use std::sync::Arc;
use anyhow::Result;
use tonic::transport::Channel;
use common::grpc::database::{
    database_service_client::DatabaseServiceClient,
    Conversation, ConversationRequest, StoreConversationRequest,
    Knowledge, KnowledgeRequest, StoreKnowledgeRequest,
};
use storage::coordinator::DatabaseCoordinator;
use storage::traits::{ConversationOperations, KnowledgeOperations, DirectAccessOperations};

/// Database client - either in-process or remote
#[derive(Clone)]
pub enum DatabaseClient {
    InProcess(Arc<DatabaseCoordinator>),
    Remote(DatabaseServiceClient<Channel>),
}

impl DatabaseClient {
    /// Create a new in-process database client
    pub async fn new_in_process() -> Result<Self> {
        let coordinator = DatabaseCoordinator::new()?;
        Ok(Self::InProcess(Arc::new(coordinator)))
    }
    
    /// Create a new remote database client
    pub async fn new_remote(endpoint: &str) -> Result<Self> {
        let client = DatabaseServiceClient::connect(endpoint.to_string()).await?;
        Ok(Self::Remote(client))
    }
    
    /// Get conversations for a session (unified interface)
    pub async fn get_conversations(&self, session_id: String) -> Result<Vec<Conversation>> {
        match self {
            Self::InProcess(coordinator) => {
                // Direct in-process access
                use common::models::Node;
                
                let storage = coordinator.conversations_active();
                let session_prefix = format!("session:{}", session_id);
                
                let conversations: Vec<Conversation> = storage
                    .scan_prefix(session_prefix.as_bytes())
                    .filter_map(|result| result.ok())
                    .filter_map(|(_key, value)| {
                        let node = rkyv::from_bytes::<Node, rkyv::rancor::Error>(&value).ok()?;
                        
                        if let Node::Message(msg) = node {
                            Some(Conversation {
                                id: msg.id.to_string(),
                                session_id: session_id.clone(),
                                content: msg.text_content,
                                timestamp: msg.timestamp,
                                role: msg.sender,
                            })
                        } else {
                            None
                        }
                    })
                    .collect();
                
                Ok(conversations)
            }
            Self::Remote(client) => {
                // Remote gRPC call
                let mut client = client.clone();
                let response = client.get_conversations(ConversationRequest { session_id }).await?;
                Ok(response.into_inner().conversations)
            }
        }
    }
    
    /// Store a conversation (unified interface)
    pub async fn store_conversation(&self, conversation: Conversation) -> Result<()> {
        match self {
            Self::InProcess(coordinator) => {
                // Direct in-process storage
                use common::NodeId;
                
                let message = common::models::Message {
                    id: NodeId::new(conversation.id),
                    chat_id: NodeId::new(conversation.session_id),
                    sender: conversation.role,
                    timestamp: conversation.timestamp,
                    text_content: conversation.content,
                    attachment_ids: vec![],
                    embedding_id: None,
                    metadata: serde_json::json!({}).to_string(),
                };
                
                coordinator.conversation_manager.insert_message(message)?;
                Ok(())
            }
            Self::Remote(client) => {
                // Remote gRPC call
                let mut client = client.clone();
                client.store_conversation(StoreConversationRequest {
                    conversation: Some(conversation),
                }).await?;
                Ok(())
            }
        }
    }
    
    /// Get knowledge by ID (unified interface)
    pub async fn get_knowledge(&self, id: String) -> Result<Option<Knowledge>> {
        match self {
            Self::InProcess(coordinator) => {
                // Direct in-process access
                let entity = coordinator.knowledge_manager.get_entity(&id)?;
                Ok(entity.map(|e| Knowledge {
                    id: e.id.to_string(),
                    content: e.label,
                    source: e.entity_type,
                    timestamp: chrono::Utc::now().timestamp(),
                }))
            }
            Self::Remote(client) => {
                // Remote gRPC call
                let mut client = client.clone();
                let response = client.get_knowledge(KnowledgeRequest { id }).await?;
                Ok(response.into_inner().knowledge)
            }
        }
    }
    
    /// Store knowledge (unified interface)
    pub async fn store_knowledge(&self, knowledge: Knowledge) -> Result<()> {
        match self {
            Self::InProcess(coordinator) => {
                // Direct in-process storage
                use common::NodeId;
                
                let entity = common::models::Entity {
                    id: NodeId::new(knowledge.id),
                    label: knowledge.content,
                    entity_type: knowledge.source,
                    embedding_id: None,
                    metadata: serde_json::json!({}).to_string(),
                };
                
                coordinator.knowledge_manager.insert_entity(entity)?;
                Ok(())
            }
            Self::Remote(client) => {
                // Remote gRPC call
                let mut client = client.clone();
                client.store_knowledge(StoreKnowledgeRequest {
                    knowledge: Some(knowledge),
                }).await?;
                Ok(())
            }
        }
    }
    
    /// Get the underlying coordinator (only available for in-process)
    pub fn coordinator(&self) -> Option<Arc<DatabaseCoordinator>> {
        match self {
            Self::InProcess(coord) => Some(coord.clone()),
            Self::Remote(_) => None,
        }
    }
}

