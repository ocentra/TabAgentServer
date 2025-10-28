//! API route handlers.

use crate::route_trait::RouteHandler;

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
pub mod discovery;

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

/// Get all available HTTP API route metadata for service discovery.
///
/// This allows clients to query available endpoints dynamically:
/// - OpenAPI documentation generation
/// - Client SDK auto-configuration
/// - Admin dashboards
/// - Route validation
///
/// Note: This is for HTTP API routes only. Not all routes exist in all transports.
/// For example, WebRTC signaling routes are HTTP-only, while some management
/// commands are Native Messaging-only.
pub fn list_available_routes() -> Vec<crate::route_trait::RouteMetadata> {
    vec![
        health::HealthRoute::metadata(),
        chat::ChatRoute::metadata(),
        chat::ResponsesRoute::metadata(),
        generate::GenerateRoute::metadata(),
        embeddings::EmbeddingsRoute::metadata(),
        models::ListModelsRoute::metadata(),
        models::LoadModelRoute::metadata(),
        models::UnloadModelRoute::metadata(),
        models::ModelInfoRoute::metadata(),
        sessions::GetHistoryRoute::metadata(),
        sessions::SaveMessageRoute::metadata(),
        rag::RagRoute::metadata(),
        rerank::RerankRoute::metadata(),
        system::SystemRoute::metadata(),
        generation::StopGenerationRoute::metadata(),
        generation::GetHaltStatusRoute::metadata(),
        params::GetParamsRoute::metadata(),
        params::SetParamsRoute::metadata(),
        stats::GetStatsRoute::metadata(),
        resources::GetResourcesRoute::metadata(),
        resources::EstimateMemoryRoute::metadata(),
        resources::CompatibilityRoute::metadata(),
        management::PullModelRoute::metadata(),
        management::DeleteModelRoute::metadata(),
        management::GetLoadedModelsRoute::metadata(),
        management::SelectModelRoute::metadata(),
        management::GetEmbeddingModelsRoute::metadata(),
        management::GetRecipesRoute::metadata(),
        management::GetRegisteredModelsRoute::metadata(),
        rag_extended::SemanticSearchRoute::metadata(),
        rag_extended::SimilarityRoute::metadata(),
        rag_extended::EvaluateEmbeddingsRoute::metadata(),
        rag_extended::ClusterRoute::metadata(),
        rag_extended::RecommendRoute::metadata(),
        webrtc::CreateOfferRoute::metadata(),
        webrtc::SubmitAnswerRoute::metadata(),
        webrtc::AddIceCandidateRoute::metadata(),
        // Note: GetWebRtcSessionRoute uses Path parameter, handled separately
        discovery::DiscoveryRoute::metadata(),
    ]
}

/// Get route count for HTTP API routes.
///
/// Note: Different transports may have different route counts.
#[allow(dead_code)] // Used by discovery route
pub fn get_route_count() -> usize {
    list_available_routes().len()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_route_registry_completeness() {
        let routes = list_available_routes();
        
        // Ensure we have at least the core routes
        assert!(routes.len() >= 30, "Expected at least 30 routes, got {}", routes.len());
        
        // Check for key route IDs
        let paths: Vec<&str> = routes.iter().map(|r| r.path).collect();
        assert!(paths.contains(&"/health"), "Missing health endpoint");
        assert!(paths.contains(&"/v1/chat/completions"), "Missing chat endpoint");
        assert!(paths.contains(&"/v1/embeddings"), "Missing embeddings endpoint");
    }
    
    #[test]
    fn test_route_id_uniqueness() {
        let routes = list_available_routes();
        let mut paths = std::collections::HashSet::new();
        
        for route in routes {
            let key = format!("{} {}", route.method, route.path);
            assert!(!paths.contains(&key), "Duplicate route: {}", key);
            paths.insert(key);
        }
    }
}

