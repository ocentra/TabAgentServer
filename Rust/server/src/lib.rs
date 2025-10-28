//! TabAgent Server Library
//!
//! This library provides the core server functionality for TabAgent,
//! including request handling, state management, and backend coordination.
//!
//! The primary entry point is `AppState`, which implements `common::backend::AppStateProvider`
//! and can be used by all three entry points (API, native messaging, WebRTC).

pub mod config;
pub mod error;
pub mod state;
pub mod handler;
pub mod python_bridge;
pub mod hf_auth;

// Re-export the main types for external use
pub use state::AppState;
pub use config::{CliArgs, ServerMode};
pub use error::{ServerError, ServerResult};

