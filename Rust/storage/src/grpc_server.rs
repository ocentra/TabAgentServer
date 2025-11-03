//! gRPC server wrapper for DatabaseCoordinator
//! 
//! This module wraps the existing DatabaseCoordinator to expose it via gRPC.
//! Allows storage to run as a separate service while keeping the same implementation.

use std::sync::Arc;
use tonic::{Request, Response, Status};
use crate::coordinator::DatabaseCoordinator;
use crate::traits::{DirectAccessOperations, ConversationOperations, KnowledgeOperations};
use common::grpc::database::{
    database_service_server::DatabaseService,
    Conversation, ConversationRequest, ConversationResponse, StoreConversationRequest,
    Knowledge, KnowledgeRequest, KnowledgeResponse, StoreKnowledgeRequest,
    StoredEmbedding, GetStoredEmbeddingsRequest, StoredEmbeddingsResponse, StoreEmbeddingRequest,
    ToolResult, ToolResultRequest, ToolResultResponse, StoreToolResultRequest,
    StatusResponse,
};
use common::NodeId;
use common::models::Node;

/// gRPC server wrapping DatabaseCoordinator
pub struct DatabaseServer {
    coordinator: Arc<DatabaseCoordinator>,
}

impl DatabaseServer {
    pub fn new(coordinator: DatabaseCoordinator) -> Self {
        Self {
            coordinator: Arc::new(coordinator),
        }
    }
    
    pub fn with_arc(coordinator: Arc<DatabaseCoordinator>) -> Self {
        Self { coordinator }
    }
}

#[tonic::async_trait]
impl DatabaseService for DatabaseServer {
    async fn get_conversations(
        &self,
        request: Request<ConversationRequest>,
    ) -> Result<Response<ConversationResponse>, Status> {
        let req = request.into_inner();
        let session_prefix = format!("session:{}", req.session_id);
        
        // Scan conversations_active for this session
        let storage = self.coordinator.conversations_active();
        let conversations: Vec<Conversation> = storage
            .scan_prefix_nodes_ref(session_prefix.as_bytes())
            .filter_map(|result| result.ok())
            .filter_map(|(_key, node_ref)| {
                // Use zero-copy accessors for ALL fields
                if let (
                    Some(text),
                    Some(timestamp),
                    Some(sender),
                    Some(id)
                ) = (
                    node_ref.message_text(),
                    node_ref.message_timestamp(),
                    node_ref.message_sender(),
                    node_ref.message_id()
                ) {
                    Some(Conversation {
                        id: id.to_string(),
                        session_id: req.session_id.clone(),
                        content: text.to_string(),
                        timestamp,
                        role: sender.to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();
        
        Ok(Response::new(ConversationResponse { conversations }))
    }
    
    async fn store_conversation(
        &self,
        request: Request<StoreConversationRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let req = request.into_inner();
        let conv = req.conversation.ok_or_else(|| {
            Status::invalid_argument("conversation is required")
        })?;
        
        // Convert gRPC Conversation to internal Message
        let message = common::models::Message {
            id: NodeId::new(conv.id),
            chat_id: NodeId::new(conv.session_id),
            sender: conv.role,
            timestamp: conv.timestamp,
            text_content: conv.content,
            attachment_ids: vec![],
            embedding_id: None,
            metadata: serde_json::json!({}).to_string(),
        };
        
        // Store using conversation operations
        self.coordinator.conversation_manager.insert_message(message)
            .map_err(|e| Status::internal(format!("Storage error: {}", e)))?;
        
        Ok(Response::new(StatusResponse {
            success: true,
            message: "Conversation stored".to_string(),
        }))
    }
    
    async fn get_knowledge(
        &self,
        request: Request<KnowledgeRequest>,
    ) -> Result<Response<KnowledgeResponse>, Status> {
        let req = request.into_inner();
        
        // Try to get entity by ID
        let entity = self.coordinator.knowledge_manager.get_entity(&req.id)
            .map_err(|e| Status::internal(format!("Storage error: {}", e)))?
            .ok_or_else(|| Status::not_found("Knowledge not found"))?;
        
        // Convert internal Entity to gRPC Knowledge
        let knowledge = Knowledge {
            id: entity.id.to_string(),
            content: entity.label,
            source: entity.entity_type,
            timestamp: chrono::Utc::now().timestamp(), // Entity doesn't have timestamp, use current time
        };
        
        Ok(Response::new(KnowledgeResponse {
            knowledge: Some(knowledge),
        }))
    }
    
    async fn store_knowledge(
        &self,
        request: Request<StoreKnowledgeRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let req = request.into_inner();
        let knowledge = req.knowledge.ok_or_else(|| {
            Status::invalid_argument("knowledge is required")
        })?;
        
        // Convert gRPC Knowledge to internal Entity
        let entity = common::models::Entity {
            id: NodeId::new(knowledge.id),
            label: knowledge.content,
            entity_type: knowledge.source,
            embedding_id: None,
            metadata: serde_json::json!({}).to_string(),
        };
        
        // Store using knowledge operations
        self.coordinator.knowledge_manager.insert_entity(entity)
            .map_err(|e| Status::internal(format!("Storage error: {}", e)))?;
        
        Ok(Response::new(StatusResponse {
            success: true,
            message: "Knowledge stored".to_string(),
        }))
    }
    
    async fn get_stored_embeddings(
        &self,
        request: Request<GetStoredEmbeddingsRequest>,
    ) -> Result<Response<StoredEmbeddingsResponse>, Status> {
        let req = request.into_inner();
        
        // Query embeddings based on the request
        // Scan embeddings_active and apply limit
        let storage = self.coordinator.embeddings_active();
        let embeddings: Vec<StoredEmbedding> = storage
            .iter()
            .filter_map(|result| result.ok())
            .take(req.limit as usize)
            .filter_map(|(_key, value)| {
                let _archived = rkyv::access::<rkyv::Archived<Node>, rkyv::rancor::Error>(&value).ok()?;
                // Note: There's no Embedding variant in Node enum currently
                // This would need to be added to common/models.rs if we want to support it
                // For now, return None
                None
            })
            .collect();
        
        Ok(Response::new(StoredEmbeddingsResponse { embeddings }))
    }
    
    async fn store_embedding(
        &self,
        _request: Request<StoreEmbeddingRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        // TODO: Implement once Embedding node variant is added to common/models.rs
        Ok(Response::new(StatusResponse {
            success: true,
            message: "Embedding storage not yet implemented".to_string(),
        }))
    }
    
    async fn get_tool_results(
        &self,
        request: Request<ToolResultRequest>,
    ) -> Result<Response<ToolResultResponse>, Status> {
        let req = request.into_inner();
        
        // Query tool results based on filters
        let storage = self.coordinator.tool_results();
        let results: Vec<ToolResult> = storage
            .iter()
            .filter_map(|result| result.ok())
            .filter_map(|(_key, value)| {
                let node = rkyv::from_bytes::<Node, rkyv::rancor::Error>(&value).ok()?;
                // Filter based on tool_name and time range
                match node {
                    Node::WebSearch(search) => {
                        if !req.tool_name.is_empty() && req.tool_name != "web_search" {
                            return None;
                        }
                        Some(ToolResult {
                            id: search.id.to_string(),
                            tool_name: "web_search".to_string(),
                            result: search.query,
                            timestamp: search.timestamp,
                        })
                    }
                    Node::ScrapedPage(page) => {
                        if !req.tool_name.is_empty() && req.tool_name != "scrape_page" {
                            return None;
                        }
                        Some(ToolResult {
                            id: page.id.to_string(),
                            tool_name: "scrape_page".to_string(),
                            result: page.url,
                            timestamp: page.scraped_at,
                        })
                    }
                    _ => None,
                }
            })
            .collect();
        
        Ok(Response::new(ToolResultResponse { results }))
    }
    
    async fn store_tool_result(
        &self,
        request: Request<StoreToolResultRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let req = request.into_inner();
        let tool_result = req.result.ok_or_else(|| {
            Status::invalid_argument("result is required")
        })?;
        
        // Convert gRPC ToolResult to internal WebSearch (as an example)
        // In a real implementation, you'd dispatch based on tool_name
        let web_search = common::models::WebSearch {
            id: NodeId::new(tool_result.id),
            query: tool_result.result,
            timestamp: tool_result.timestamp,
            results_urls: vec![],
            embedding_id: None,
            metadata: serde_json::json!({}).to_string(),
        };
        
        // Store using tool result operations
        use crate::traits::ToolResultOperations;
        self.coordinator.tool_result_manager.insert_web_search(web_search)
            .map_err(|e| Status::internal(format!("Storage error: {}", e)))?;
        
        Ok(Response::new(StatusResponse {
            success: true,
            message: "Tool result stored".to_string(),
        }))
    }
}

/// Start the gRPC server for storage
pub async fn start_server(
    coordinator: Arc<DatabaseCoordinator>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    use tonic::transport::Server;
    use common::grpc::database::database_service_server::DatabaseServiceServer;
    
    let addr = format!("0.0.0.0:{}", port).parse()?;
    let server = DatabaseServer::with_arc(coordinator);
    
    tracing::info!("💾 Storage gRPC server listening on {}", addr);
    
    Server::builder()
        .add_service(DatabaseServiceServer::new(server))
        .serve(addr)
        .await?;
    
    Ok(())
}
