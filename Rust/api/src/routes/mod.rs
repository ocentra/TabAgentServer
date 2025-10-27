//! API route handlers.

pub mod health;
pub mod chat;
pub mod generate;
pub mod embeddings;
pub mod models;
pub mod sessions;
pub mod rag;
pub mod rerank;
pub mod system;
pub mod generation;
pub mod params;
pub mod stats;
pub mod resources;
pub mod management;
pub mod rag_extended;
pub mod webrtc;

use utoipa::OpenApi;

/// OpenAPI documentation for all routes.
/// 
/// NOTE: Using trait-based routes now. OpenAPI generation will be added
/// in a future iteration using a different approach compatible with the
/// RouteHandler trait system.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "TabAgent API",
        version = "1.0.0",
        description = "Enterprise-grade AI/ML API for TabAgent Server",
        license(name = "MIT"),
        contact(
            name = "TabAgent Team",
            url = "https://github.com/TabAgent/TabAgent"
        )
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development"),
        (url = "https://api.tabagent.dev", description = "Production")
    ),
    // Paths removed - will be regenerated using trait metadata
    paths(),
    // Schemas kept for backwards compatibility
    components(schemas(
        health::HealthResponse,
        chat::ChatCompletionRequest,
        chat::ChatCompletionResponse,
        chat::ChatMessage,
        chat::ChatChoice,
        chat::Usage,
        generate::CompletionRequest,
        embeddings::EmbeddingRequest,
        models::LoadModelRequest,
        models::ModelListResponse,
        models::Model,
        rag::RagQueryRequest,
        rerank::RerankRequest,
        params::GenerationParams,
        stats::PerformanceStats,
        resources::ResourceInfo,
        resources::RamInfo,
        resources::GpuResourceInfo,
        resources::LoadedModelInfo,
        resources::EstimateRequest,
        resources::EstimateResponse,
        webrtc::CreateOfferRequest,
        webrtc::CreateOfferResponse,
        webrtc::SubmitAnswerRequest,
        webrtc::SubmitAnswerResponse,
        webrtc::AddIceCandidateRequest,
        webrtc::AddIceCandidateResponse,
        webrtc::WebRtcSessionResponse,
    ))
)]
pub struct ApiDoc;

