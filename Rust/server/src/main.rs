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
            tabagent_native_messaging::run_host_with_state(
                state,
                tabagent_native_messaging::NativeMessagingConfig::default()
            ).await?;
        }
        ServerMode::Http => {
            info!("Starting HTTP server on port {}", args.port);
            tabagent_api::run_server(state, args.port).await?;
        }
        ServerMode::WebRtc => {
            info!("Starting WebRTC signaling server on port {}", args.webrtc_port);
            // WebRTC runs its own signaling HTTP server + data channel handler
            let webrtc_config = tabagent_webrtc::WebRtcConfig {
                signaling_port: args.webrtc_port,
                max_sessions: 100,
                session_timeout_secs: 300,
                ..Default::default()
            };
            let manager = std::sync::Arc::new(tabagent_webrtc::WebRtcManager::new(webrtc_config));
            
            // TODO: Wire up data channel handler to use AppState
            info!("WebRTC signaling server running (data channels not yet wired to backend)");
            
            // Keep server running
            tokio::signal::ctrl_c().await?;
        }
        ServerMode::Both => {
            info!("Running in dual mode (HTTP on port {} + native messaging)", args.port);
            
            // Run both simultaneously (RAG: proper async concurrency)
            let http_handle = {
                let state = state.clone();
                let port = args.port;
                tokio::spawn(async move {
                    if let Err(e) = tabagent_api::run_server(state, port).await {
                        error!("HTTP server error: {}", e);
                    }
                })
            };
            
            let native_handle = tokio::spawn(async move {
                if let Err(e) = tabagent_native_messaging::run_host_with_state(
                    state,
                    tabagent_native_messaging::NativeMessagingConfig::default()
                ).await {
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
        ServerMode::All => {
            info!("Running ALL transports: HTTP ({}), WebRTC ({}), Native Messaging", args.port, args.webrtc_port);
            
            // HTTP API
            let http_handle = {
                let state = state.clone();
                let port = args.port;
                tokio::spawn(async move {
                    info!("Starting HTTP API on port {}", port);
                    if let Err(e) = tabagent_api::run_server(state, port).await {
                        error!("HTTP server error: {}", e);
                    }
                })
            };
            
            // Native Messaging
            let native_handle = {
                let state = state.clone();
                tokio::spawn(async move {
                    info!("Starting Native Messaging (stdin/stdout)");
                    if let Err(e) = tabagent_native_messaging::run_host_with_state(
                        state,
                        tabagent_native_messaging::NativeMessagingConfig::default()
                    ).await {
                        error!("Native messaging error: {}", e);
                    }
                })
            };
            
            // WebRTC
            let webrtc_handle = {
                let _state = state.clone(); // TODO: Wire to data channel handler
                let port = args.webrtc_port;
                tokio::spawn(async move {
                    info!("Starting WebRTC signaling on port {}", port);
                    let webrtc_config = tabagent_webrtc::WebRtcConfig {
                        signaling_port: port,
                        max_sessions: 100,
                        session_timeout_secs: 300,
                        ..Default::default()
                    };
                    let _manager = std::sync::Arc::new(tabagent_webrtc::WebRtcManager::new(webrtc_config));
                    
                    // TODO: Wire up data channel handler to use AppState
                    info!("WebRTC signaling server running (data channels not yet wired)");
                    
                    // Keep running
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                    }
                })
            };
            
            // Wait for any to fail
            tokio::select! {
                _ = http_handle => {
                    error!("HTTP server terminated unexpectedly");
                }
                _ = native_handle => {
                    error!("Native messaging terminated unexpectedly");
                }
                _ = webrtc_handle => {
                    error!("WebRTC server terminated unexpectedly");
                }
            }
        }
    }

    Ok(())
}

