//! TabAgent Server - Rust-native server with native messaging, HTTP API, and WebRTC support
//!
//! This server replaces the Python-based entry points while maintaining compatibility with
//! existing Rust infrastructure (ONNX, GGUF, database, etc.).

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod state;
mod handler;
mod python_bridge;  // Server-specific Python inference (different from python-ml-bridge crate)
mod api;            // TODO: Move to tabagent-api crate
mod native_messaging; // TODO: Move to tabagent-native-messaging crate

use crate::{
    config::{CliArgs, ServerMode},
    state::AppState,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing (RAG: structured logging)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tabagent_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse CLI arguments
    let args = CliArgs::parse();
    
    info!("Starting TabAgent Server v{}", env!("CARGO_PKG_VERSION"));
    info!("Mode: {:?}", args.mode);

    // Initialize shared application state (wrapped in Arc for sharing across tasks)
    let state = Arc::new(AppState::new(&args).await?);
    
    // Run based on mode
    match args.mode {
        ServerMode::Native => {
            info!("Running in native messaging mode (stdin/stdout)");
            native_messaging::run(state).await?;
        }
        ServerMode::Http => {
            info!("Starting HTTP server on port {}", args.port);
            api::run_server(state, args.port).await?;
        }
        ServerMode::Both => {
            info!("Running in dual mode (HTTP on port {} + native messaging)", args.port);
            
            // Run both simultaneously (RAG: proper async concurrency)
            let http_handle = {
                let state = state.clone();
                let port = args.port;
                tokio::spawn(async move {
                    if let Err(e) = api::run_server(state, port).await {
                        error!("HTTP server error: {}", e);
                    }
                })
            };
            
            let native_handle = tokio::spawn(async move {
                if let Err(e) = native_messaging::run(state).await {
                    error!("Native messaging error: {}", e);
                }
            });
            
            // Wait for either to complete (shouldn't happen unless error)
            tokio::select! {
                _ = http_handle => {
                    error!("HTTP server terminated unexpectedly");
                }
                _ = native_handle => {
                    error!("Native messaging terminated unexpectedly");
                }
            }
        }
    }

    Ok(())
}

