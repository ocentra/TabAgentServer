//! Native messaging route handlers.
//!
//! This module provides route handlers for native messaging with
//! compile-time enforcement of architectural standards, identical
//! to the patterns used in API and WebRTC crates.
//!
//! # CRITICAL: Route Standards Enforcement
//!
//! Every route MUST implement the `NativeMessagingRoute` trait from `route_trait.rs`,
//! which enforces at compile-time:
//! - Documentation (description cannot be empty)
//! - Validation (real validation, not just Ok(()))
//! - Testing (at least one test case)
//! - Tracing (request_id logging)
//! - Error handling (proper NativeMessagingError usage)
//! - Values integration (tabagent-values RequestValue/ResponseValue)
//!
//! See `route_trait.rs` for full enforcement rules.
//!
//! # Adding New Routes
//!
//! 1. Create a new module with your route struct
//! 2. Implement `NativeMessagingRoute` trait with ALL methods
//! 3. Use `enforce_native_messaging_route!()` macro to verify compliance
//! 4. Add to route registry below
//!
//! Example:
//! ```ignore
//! use crate::route_trait::{NativeMessagingRoute, RouteMetadata};
//!
//! pub struct MyRoute;
//!
//! impl NativeMessagingRoute for MyRoute {
//!     // ... implementation with metadata, validation, tests, handle ...
//! }
//!
//! enforce_native_messaging_route!(MyRoute);
//! ```

pub mod health;
pub mod chat;
pub mod embeddings;
pub mod generate;
pub mod generation;
pub mod models;
pub mod sessions;
pub mod rag;
pub mod rag_extended;
pub mod rerank;
pub mod system;
pub mod params;
pub mod stats;
pub mod resources;
pub mod management;

// Re-export the trait (defined in route_trait.rs for full enforcement)
pub use crate::route_trait::NativeMessagingRoute;

// Route registry - all routes must be verified at compile time
// When adding new routes, use register_native_messaging_routes! to enforce compliance
//
// Example:
// register_native_messaging_routes!([
//     health::HealthRoute,
//     chat::ChatRoute,
//     // ... add your route here
// ]);

/// Get all available route metadata for documentation and discovery.
pub fn get_all_routes() -> Vec<crate::route_trait::RouteMetadata> {
    vec![
        health::HealthRoute::metadata(),
        system::SystemRoute::metadata(),
        stats::GetStatsRoute::metadata(),
        chat::ChatRoute::metadata(),
        chat::ResponsesRoute::metadata(),
        embeddings::EmbeddingsRoute::metadata(),
        generate::GenerateRoute::metadata(),
        models::ListModelsRoute::metadata(),
        models::LoadModelRoute::metadata(),
        models::UnloadModelRoute::metadata(),
        models::ModelInfoRoute::metadata(),
        rag::RagRoute::metadata(),
        rerank::RerankRoute::metadata(),
        sessions::GetHistoryRoute::metadata(),
        sessions::SaveMessageRoute::metadata(),
        params::GetParamsRoute::metadata(),
        params::SetParamsRoute::metadata(),
        generation::StopGenerationRoute::metadata(),
        generation::GetHaltStatusRoute::metadata(),
        resources::GetResourcesRoute::metadata(),
        resources::EstimateMemoryRoute::metadata(),
        resources::CompatibilityRoute::metadata(),
        management::PullModelRoute::metadata(),
        management::DeleteModelRoute::metadata(),
        management::GetLoadedModelsRoute::metadata(),
    ]
}

/// Get route count for validation.
pub fn get_route_count() -> usize {
    get_all_routes().len()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_route_registry_completeness() {
        let routes = get_all_routes();
        
        // Ensure we have at least the implemented routes
        assert!(routes.len() >= 5, "Expected at least 5 routes, got {}", routes.len());
        
        // Ensure all routes have valid metadata
        for route in routes {
            assert!(!route.route_id.is_empty(), "Route ID cannot be empty");
            assert!(!route.description.is_empty(), "Route description cannot be empty");
            assert!(!route.tags.is_empty(), "Route tags cannot be empty");
        }
    }
    
    #[test]
    fn test_route_id_uniqueness() {
        let routes = get_all_routes();
        let mut route_ids = std::collections::HashSet::new();
        
        for route in routes {
            assert!(
                route_ids.insert(route.route_id),
                "Duplicate route ID found: {}",
                route.route_id
            );
        }
    }
}