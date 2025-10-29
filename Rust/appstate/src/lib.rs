//! Application state for TabAgent.
//!
//! This crate provides the central `AppState` struct that holds all shared resources
//! and implements the `AppStateProvider` trait from `common`.
//!
//! # Architecture
//!
//! `appstate` sits between the infrastructure crates and the transport layers:
//!
//! ```text
//! Transport Layer (api, native-messaging, webrtc)
//!          ↓
//!     appstate (this crate)
//!          ↓
//! Infrastructure (model-cache, db, hardware, onnx-loader, gguf-loader)
//! ```
//!
//! # Dependency Flow
//!
//! - `appstate` depends on: common, values, model-cache, db, hardware, loaders
//! - Transport crates depend on: appstate, common, values
//! - Infrastructure crates depend on: common, values (NO dependency on appstate)
//! - Server binary depends on: appstate, transport crates
//!
//! This creates a clean one-way dependency flow with no cycles.

pub mod routes;
pub mod state;
pub mod hf_auth;
pub mod python_bridge;

pub use state::{AppState, AppStateConfig};
pub use hf_auth::HfAuthManager;
pub use python_bridge::PythonMlBridge;

