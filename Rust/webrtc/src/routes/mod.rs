//! WebRTC data channel route handlers.
//!
//! This module provides route handlers for WebRTC data channels with
//! compile-time enforcement of architectural standards.
//!
//! # CRITICAL: Route Standards Enforcement
//!
//! Every route MUST implement the `DataChannelRoute` trait from `route_trait.rs`,
//! which enforces at compile-time:
//! - Documentation (description cannot be empty)
//! - Validation (real validation, not just Ok(()))
//! - Testing (at least one test case)
//! - Tracing (request_id logging)
//! - Error handling (proper WebRtcError usage)
//! - Media validation (for video/audio routes)
//!
//! See `route_trait.rs` for full enforcement rules.
//!
//! # Adding New Routes
//!
//! 1. Create a new module with your route struct
//! 2. Implement `DataChannelRoute` trait with ALL methods
//! 3. Use `enforce_data_channel_route!()` macro to verify compliance
//! 4. Add to route registry below
//!
//! Example:
//! ```ignore
//! use crate::route_trait::{DataChannelRoute, RouteMetadata};
//!
//! pub struct MyRoute;
//!
//! impl DataChannelRoute for MyRoute {
//!     // ... implementation with metadata, validation, tests, handle ...
//! }
//!
//! enforce_data_channel_route!(MyRoute);
//! ```

pub mod chat;
pub mod embeddings;
pub mod generate;
pub mod generation;
pub mod health;
pub mod management;
pub mod models;
pub mod params;
pub mod rag;
pub mod rag_extended;
pub mod rerank;
pub mod resources;
pub mod sessions;
pub mod stats;
pub mod system;

// Media routes (reference implementations showing enforcement)
pub mod video_stream;
pub mod audio_stream;

// HuggingFace Auth and Hardware routes (TIER1)
pub mod hf_auth;
pub mod hardware;

// Re-export the trait (defined in route_trait.rs for full enforcement)
#[allow(unused_imports)] // Re-exported for external use
pub use crate::route_trait::DataChannelRoute;

// Route registry - all routes must be verified at compile time
// When adding new routes, use register_data_channel_routes! to enforce compliance
//
// Example:
// register_data_channel_routes!([
//     chat::ChatRoute,
//     embeddings::EmbeddingsRoute,
//     // ... add your route here
// ]);

/// Get all available WebRTC data channel route metadata for service discovery.
///
/// This allows WebRTC clients to query available routes dynamically:
/// - Route validation before sending messages
/// - Auto-generating UI for available commands
/// - Capability detection
/// - Error handling (know which routes exist)
///
/// Note: This is for WebRTC data channel routes only. Not all routes exist
/// in all transports (HTTP API, Native Messaging, WebRTC).
/// 
/// WebRTC has unique media routes (video_stream, audio_stream) not present in other transports.
pub fn list_available_routes() -> Vec<crate::route_trait::RouteMetadata> {
    vec![
        // Core routes
        health::HealthRoute::metadata(),
        system::SystemRoute::metadata(),
        stats::StatsRoute::metadata(),
        chat::ChatRoute::metadata(),
        chat::ResponsesRoute::metadata(),
        
        // Model routes (split pattern)
        models::ListModelsRoute::metadata(),
        models::LoadModelRoute::metadata(),
        models::UnloadModelRoute::metadata(),
        models::ModelInfoRoute::metadata(),
        
        // Session routes (split pattern)
        sessions::GetHistoryRoute::metadata(),
        sessions::SaveMessageRoute::metadata(),
        
        // Generation routes
        embeddings::EmbeddingsRoute::metadata(),
        generate::GenerateRoute::metadata(),
        generation::StopGenerationRoute::metadata(),
        generation::GetHaltStatusRoute::metadata(),
        
        // Parameter routes (split pattern)
        params::GetParamsRoute::metadata(),
        params::SetParamsRoute::metadata(),
        
        // Resource routes (split pattern)
        resources::GetResourcesRoute::metadata(),
        resources::EstimateMemoryRoute::metadata(),
        resources::CompatibilityRoute::metadata(),
        
        // Management routes (split pattern)
        management::PullModelRoute::metadata(),
        management::DeleteModelRoute::metadata(),
        management::GetLoadedModelsRoute::metadata(),
        // TIER 2: management::SelectModelRoute, GetEmbeddingModelsRoute, GetRecipesRoute, GetRegisteredModelsRoute
        
        // RAG routes
        rag::RagRoute::metadata(),
        rerank::RerankRoute::metadata(),
        
        // RAG Extended routes (split pattern)
        rag_extended::SemanticSearchRoute::metadata(),
        rag_extended::SimilarityRoute::metadata(),
        rag_extended::EvaluateEmbeddingsRoute::metadata(),
        rag_extended::ClusterRoute::metadata(),
        rag_extended::RecommendRoute::metadata(),
        
        // Media routes (WebRTC-specific)
        video_stream::VideoStreamRoute::metadata(),
        audio_stream::AudioStreamRoute::metadata(),
        
        // HuggingFace Auth routes (TIER1)
        hf_auth::SetHfTokenRoute::metadata(),
        hf_auth::GetHfTokenStatusRoute::metadata(),
        hf_auth::ClearHfTokenRoute::metadata(),
        
        // Hardware routes (TIER1)
        hardware::GetHardwareInfoRoute::metadata(),
        hardware::CheckModelFeasibilityRoute::metadata(),
        hardware::GetRecommendedModelsRoute::metadata(),
    ]
}

/// Get route count for WebRTC data channel routes.
///
/// Note: Different transports may have different route counts.
pub fn get_route_count() -> usize {
    list_available_routes().len()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_route_registry_completeness() {
        let routes = list_available_routes();
        
        // WebRTC has 35+ routes after split pattern migration
        assert!(routes.len() >= 35, "Expected at least 35 routes, got {}", routes.len());
        
        // Ensure all routes have valid metadata
        for route in &routes {
            assert!(!route.route_id.is_empty(), "Route ID cannot be empty");
            assert!(!route.description.is_empty(), "Route description cannot be empty");
            assert!(!route.tags.is_empty(), "Route tags cannot be empty");
        }
    }
    
    #[test]
    fn test_route_id_uniqueness() {
        let routes = list_available_routes();
        let mut route_ids = std::collections::HashSet::new();
        
        for route in routes {
            assert!(
                route_ids.insert(route.route_id),
                "Duplicate route ID: {}",
                route.route_id
            );
        }
    }
    
    #[test]
    fn test_webrtc_specific_routes() {
        let routes = list_available_routes();
        let route_ids: Vec<&str> = routes.iter().map(|r| r.route_id).collect();
        
        // Verify WebRTC-specific media routes exist
        assert!(route_ids.contains(&"video_stream"), "Missing video_stream route");
        assert!(route_ids.contains(&"audio_stream"), "Missing audio_stream route");
        
        // Verify split pattern routes exist
        assert!(route_ids.contains(&"models"), "Missing models (list) route");
        assert!(route_ids.contains(&"load_model"), "Missing load_model route");
        assert!(route_ids.contains(&"get_history"), "Missing get_history route");
        assert!(route_ids.contains(&"get_params"), "Missing get_params route");
    }
}

