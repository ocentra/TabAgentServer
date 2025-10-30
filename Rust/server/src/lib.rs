//! TabAgent Server Binary
//!
//! This crate provides the main server binary that orchestrates all three transport layers:
//! - HTTP API (using `tabagent-api`)
//! - Native Messaging (using `tabagent-native-messaging`)
//! - WebRTC (using `tabagent-webrtc`)
//!
//! The server creates an `appstate::AppState` and passes it to all transport layers.
//! Business logic lives in `appstate`, not here!

pub mod config;
pub mod error;
pub mod grpc_config;
pub mod database_client;
pub mod ml_client;

// Re-export server configuration types
pub use config::{CliArgs, ServerMode};
pub use error::{ServerError, ServerResult};
pub use grpc_config::GrpcConfig;
pub use database_client::DatabaseClient;
pub use ml_client::MlClient;

