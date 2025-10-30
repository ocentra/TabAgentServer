//! Generated gRPC code and re-exports
//! 
//! This module includes the generated protobuf code from tonic-build
//! and re-exports commonly used types for convenience.

// Include generated code from build.rs
pub mod database {
    tonic::include_proto!("database");
}

pub mod ml {
    tonic::include_proto!("ml");
}

// Re-export database types for convenience
pub use database::{
    database_service_server, database_service_client,
    Conversation, ConversationRequest, ConversationResponse, StoreConversationRequest,
    Knowledge, KnowledgeRequest, KnowledgeResponse, StoreKnowledgeRequest,
    StoredEmbedding, GetStoredEmbeddingsRequest, StoredEmbeddingsResponse, StoreEmbeddingRequest,
    ToolResult, ToolResultRequest, ToolResultResponse, StoreToolResultRequest,
    StatusResponse,
};

// Re-export ML types for convenience
pub use ml::{
    transformers_service_server, transformers_service_client,
    mediapipe_service_server, mediapipe_service_client,
    TextRequest, TextResponse,
    GenerateEmbeddingsRequest, GeneratedEmbeddingsResponse, GeneratedEmbedding,
    ChatRequest, ChatResponse, ChatMessage,
    ImageRequest, FaceDetectionResponse, FaceDetection,
    HandDetectionResponse, HandDetection, Landmark,
    PoseDetectionResponse,
};
