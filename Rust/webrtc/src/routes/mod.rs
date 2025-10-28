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

// TODO: TIER 0 - Migrate WebRTC routes to match API/Native Messaging parity
// 
// Currently WebRTC uses unified route pattern (e.g., ModelsRoute with action enum)
// while API/Native Messaging use split routes (ListModelsRoute, LoadModelRoute, etc.)
//
// This breaks parity. Need to refactor WebRTC routes to match:
// - Split ModelsRoute → ListModelsRoute, LoadModelRoute, UnloadModelRoute, ModelInfoRoute
// - Split SessionsRoute → GetHistoryRoute, SaveMessageRoute
// - Split ParamsRoute → GetParamsRoute, SetParamsRoute
// - Split ResourcesRoute → GetResourcesRoute, EstimateMemoryRoute, CompatibilityRoute
// - Split GenerationRoute → StopGenerationRoute, GetHaltStatusRoute
// - Split ManagementRoute → PullModelRoute, DeleteModelRoute, GetLoadedModelsRoute
// - Split RagExtendedRoute → SemanticSearchRoute, SimilarityRoute, etc.
//
// Once migrated, add get_all_routes() and get_route_count() functions like
// API and Native Messaging have for route registry validation.

