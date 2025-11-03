//! TabAgent Server - Rust-native server with native messaging, HTTP API, and WebRTC support
//!
//! This server replaces the Python-based entry points while maintaining compatibility with
//! existing Rust infrastructure (ONNX, GGUF, database, etc.).

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tracing::{info, error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;

use appstate::{AppState, AppStateConfig};
use common::{AppStateProvider, PythonProcessManager};
use crate::config::{CliArgs, ServerMode};
use std::path::PathBuf;

/// Kill any process using the specified port (Windows-specific for now).
#[cfg(target_os = "windows")]
async fn kill_port_process(port: u16) -> Result<()> {
    use std::process::Command;
    
    // Find process using the port
    let output = Command::new("netstat")
        .args(&["-ano"])
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse netstat output to find PID
    for line in stdout.lines() {
        if line.contains(&format!(":{}", port)) && line.contains("LISTENING") {
            // Extract PID (last column)
            if let Some(pid_str) = line.split_whitespace().last() {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    // Don't kill ourselves
                    if pid != std::process::id() {
                        warn!("Found existing server on port {} (PID: {}), killing it...", port, pid);
                        let _ = Command::new("taskkill")
                            .args(&["/F", "/PID", &pid.to_string()])
                            .output();
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        info!("Killed old server process (PID: {})", pid);
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// Kill any process using the specified port (Unix-specific).
#[cfg(not(target_os = "windows"))]
async fn kill_port_process(port: u16) -> Result<()> {
    use std::process::Command;
    
    // Use lsof to find process using the port
    let output = Command::new("lsof")
        .args(&["-ti", &format!(":{}", port)])
        .output()?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let pids: Vec<u32> = stdout
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect();
    
    for pid in pids {
        // Don't kill ourselves
        if pid != std::process::id() {
            warn!("Found existing server on port {} (PID: {}), killing it...", port, pid);
            let _ = Command::new("kill")
                .args(&["-9", &pid.to_string()])
                .output();
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            info!("Killed old server process (PID: {})", pid);
        }
    }
    
    Ok(())
}

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
    let mut args = CliArgs::parse();
    
    // Override port from environment if set by port manager
    if let Ok(port_str) = std::env::var("TABAGENT_RUST_PORT") {
        if let Ok(port) = port_str.parse::<u16>() {
            info!("Using port {} from TABAGENT_RUST_PORT environment variable", port);
            args.port = port;
        }
    }
    
    info!("Starting TabAgent Server v{}", env!("CARGO_PKG_VERSION"));
    info!("Mode: {:?}", args.mode);
    info!("Ports: HTTP={}, WebRTC={}", args.port, args.webrtc_port);

    // Kill any existing processes on ports BEFORE initializing state (to avoid DB lock conflicts)
    match args.mode {
        ServerMode::Http | ServerMode::Both => {
            kill_port_process(args.port).await?;
        }
        ServerMode::WebRtc => {
            kill_port_process(args.webrtc_port).await?;
        }
        ServerMode::Web | ServerMode::All => {
            kill_port_process(args.port).await?;
            kill_port_process(args.webrtc_port).await?;
        }
        ServerMode::Native | ServerMode::Mcp => {
            // Native messaging and MCP don't use ports, skip
        }
        ServerMode::Everything => {
            kill_port_process(args.port).await?;
            kill_port_process(args.webrtc_port).await?;
        }
    }

    // Initialize Python ML service (if PythonML directory exists)
    let python_ml_dir = std::env::current_dir()?
        .parent()
        .map(|p| p.join("PythonML"))
        .unwrap_or_else(|| PathBuf::from("../PythonML"));
    
    let python_ml_port = 50051;
    let python_manager = if python_ml_dir.exists() {
        info!("Detected PythonML directory, starting Python ML service...");
        let manager = PythonProcessManager::new(python_ml_dir.clone(), python_ml_port);
        
        match manager.start().await {
            Ok(_) => {
                info!("Python ML service started successfully on port {}", python_ml_port);
                Some(Arc::new(manager))
            }
            Err(e) => {
                warn!("Failed to start Python ML service: {}. Continuing without ML features.", e);
                warn!("To enable ML features, ensure PythonML directory is set up with:");
                warn!("  1. requirements.txt installed: pip install -r requirements.txt");
                warn!("  2. Proto files generated: cd PythonML && ./generate_protos.bat (or .sh)");
                None
            }
        }
    } else {
        info!("PythonML directory not found at {}, skipping ML service", python_ml_dir.display());
        None
    };
    
    // Initialize shared application state (wrapped in Arc for sharing across tasks)
    // Use platform-specific AppData paths if not explicitly provided
    let db_path = args.db_path.clone().unwrap_or_else(|| {
        common::platform::get_default_db_path().join("tabagent_db")
    });
    
    let model_cache_path = args.model_cache_path.clone().unwrap_or_else(|| {
        common::platform::get_default_db_path().join("models")
    });
    
    info!("Database location: {}", db_path.display());
    info!("Model cache location: {}", model_cache_path.display());
    
    let config = AppStateConfig {
        db_path,
        model_cache_path,
    };
    let state = Arc::new(AppState::new(config).await?) as Arc<dyn AppStateProvider>;
    
    // Set up cleanup handler for Python ML service
    let python_manager_for_cleanup = python_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        if let Some(manager) = python_manager_for_cleanup {
            info!("Shutting down Python ML service...");
            let _ = manager.stop().await;
        }
    });
    
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
            info!("Starting WebRTC signaling server (HTTP API) on port {}", args.webrtc_port);
            // Create WebRTC manager for signaling
            let webrtc_config = tabagent_webrtc::WebRtcConfig {
                max_sessions: 1000,
                max_sessions_per_client: 10,
                session_timeout: std::time::Duration::from_secs(3600),
                ..Default::default()
            };
            
            // Create app_state_handler closure for WebRTC
            let state_for_webrtc = state.clone();
            let app_state_handler: Arc<dyn Fn(tabagent_values::RequestValue) -> futures::future::BoxFuture<'static, anyhow::Result<tabagent_values::ResponseValue>> + Send + Sync> = 
                Arc::new(move |request| {
                    let state = state_for_webrtc.clone();
                    Box::pin(async move {
                        state.handle_request(request).await
                    })
                });
            
            let webrtc_manager = Arc::new(tabagent_webrtc::WebRtcManager::new(webrtc_config, app_state_handler));
            
            // Run API server with WebRTC support
            let config = tabagent_api::ApiConfig { port: args.webrtc_port, ..Default::default() };
            tabagent_api::run_server_with_config(state, Some(webrtc_manager), config).await?;
        }
        ServerMode::Web => {
            info!("Running in WEB mode: HTTP ({}) + WebRTC ({})", args.port, args.webrtc_port);
            
            // Create WebRTC manager
            let webrtc_config = tabagent_webrtc::WebRtcConfig {
                max_sessions: 1000,
                max_sessions_per_client: 10,
                session_timeout: std::time::Duration::from_secs(3600),
                ..Default::default()
            };
            
            let state_for_webrtc = state.clone();
            let app_state_handler: Arc<dyn Fn(tabagent_values::RequestValue) -> futures::future::BoxFuture<'static, anyhow::Result<tabagent_values::ResponseValue>> + Send + Sync> = 
                Arc::new(move |request| {
                    let state = state_for_webrtc.clone();
                    Box::pin(async move {
                        state.handle_request(request).await
                    })
                });
            
            let webrtc_manager = Arc::new(tabagent_webrtc::WebRtcManager::new(webrtc_config, app_state_handler));
            
            // HTTP API with WebRTC on main port
            let http_handle = {
                let state = state.clone();
                let port = args.port;
                let webrtc = webrtc_manager.clone();
                tokio::spawn(async move {
                    let config = tabagent_api::ApiConfig { port, ..Default::default() };
                    if let Err(e) = tabagent_api::run_server_with_config(state, Some(webrtc), config).await {
                        error!("HTTP+WebRTC server error: {}", e);
                    }
                })
            };
            
            // WebRTC signaling on separate port
            let webrtc_handle = {
                let state = state.clone();
                let port = args.webrtc_port;
                tokio::spawn(async move {
                    let config = tabagent_api::ApiConfig { port, ..Default::default() };
                    if let Err(e) = tabagent_api::run_server_with_config(state, Some(webrtc_manager), config).await {
                        error!("WebRTC signaling server error: {}", e);
                    }
                })
            };
            
            // Wait for either to complete
            tokio::select! {
                _ = http_handle => {
                    error!("HTTP server terminated unexpectedly");
                }
                _ = webrtc_handle => {
                    error!("WebRTC server terminated unexpectedly");
                }
            }
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
            
            // Native Messaging (runs in background, may exit if no Chrome)
            let state_for_native = state.clone();
            tokio::spawn(async move {
                info!("Starting Native Messaging (stdin/stdout)");
                // Native messaging will exit gracefully if stdin closes (no Chrome)
                // This is expected behavior when running from terminal
                match tabagent_native_messaging::run_host_with_state(
                    state_for_native,
                    tabagent_native_messaging::NativeMessagingConfig::default()
                ).await {
                    Ok(_) => {
                        info!("Native messaging exited gracefully (stdin closed)");
                    }
                    Err(e) => {
                        error!("Native messaging error: {}", e);
                    }
                }
            });
            
            // WebRTC (HTTP API on different port with WebRTC manager)
            let webrtc_handle = {
                let state = state.clone();
                let port = args.webrtc_port;
                tokio::spawn(async move {
                    info!("Starting WebRTC signaling (HTTP API) on port {}", port);
                    
                    // Create WebRTC manager for signaling
                    let webrtc_config = tabagent_webrtc::WebRtcConfig {
                        max_sessions: 1000,
                        max_sessions_per_client: 10,
                        session_timeout: std::time::Duration::from_secs(3600),
                        ..Default::default()
                    };
                    
                    // Create app_state_handler closure for WebRTC
                    let state_for_webrtc = state.clone();
                    let app_state_handler: Arc<dyn Fn(tabagent_values::RequestValue) -> futures::future::BoxFuture<'static, anyhow::Result<tabagent_values::ResponseValue>> + Send + Sync> = 
                        Arc::new(move |request| {
                            let state = state_for_webrtc.clone();
                            Box::pin(async move {
                                state.handle_request(request).await
                            })
                        });
                    
                    let webrtc_manager = Arc::new(tabagent_webrtc::WebRtcManager::new(webrtc_config, app_state_handler));
                    
                    // Run API with WebRTC support
                    let config = tabagent_api::ApiConfig { port, ..Default::default() };
                    if let Err(e) = tabagent_api::run_server_with_config(state, Some(webrtc_manager), config).await {
                        error!("WebRTC signaling server error: {}", e);
                    }
                })
            };
            
            // Wait for HTTP or WebRTC to fail (ignore Native Messaging exit)
            // Native Messaging may exit gracefully if stdin closes (terminal use)
            tokio::select! {
                _ = http_handle => {
                    error!("HTTP server terminated unexpectedly");
                }
                _ = webrtc_handle => {
                    error!("WebRTC server terminated unexpectedly");
                }
            }
        }
        ServerMode::Mcp => {
            info!("Starting MCP transport (stdio for AI assistants)");
            
            // Create coordinator for MCP logs
            let coordinator = Arc::new(storage::DatabaseCoordinator::new()?);
            let mcp_manager = tabagent_mcp::McpManager::new(state.clone(), coordinator);
            
            // Run MCP stdio transport
            tabagent_mcp::transport::run_stdio_transport(mcp_manager).await?;
        }
        ServerMode::Everything => {
            info!("Running ALL transports: HTTP ({}), WebRTC ({}), Native Messaging, MCP", args.port, args.webrtc_port);
            
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
            
            // Native Messaging (runs in background, may exit if no Chrome)
            let state_for_native = state.clone();
            tokio::spawn(async move {
                info!("Starting Native Messaging (stdin/stdout)");
                match tabagent_native_messaging::run_host_with_state(
                    state_for_native,
                    tabagent_native_messaging::NativeMessagingConfig::default()
                ).await {
                    Ok(_) => {
                        info!("Native messaging exited gracefully (stdin closed)");
                    }
                    Err(e) => {
                        error!("Native messaging error: {}", e);
                    }
                }
            });
            
            // WebRTC (HTTP API on different port with WebRTC manager)
            let webrtc_handle = {
                let state = state.clone();
                let port = args.webrtc_port;
                tokio::spawn(async move {
                    info!("Starting WebRTC signaling (HTTP API) on port {}", port);
                    
                    let webrtc_config = tabagent_webrtc::WebRtcConfig {
                        max_sessions: 1000,
                        max_sessions_per_client: 10,
                        session_timeout: std::time::Duration::from_secs(3600),
                        ..Default::default()
                    };
                    
                    let state_for_webrtc = state.clone();
                    let app_state_handler: Arc<dyn Fn(tabagent_values::RequestValue) -> futures::future::BoxFuture<'static, anyhow::Result<tabagent_values::ResponseValue>> + Send + Sync> = 
                        Arc::new(move |request| {
                            let state = state_for_webrtc.clone();
                            Box::pin(async move {
                                state.handle_request(request).await
                            })
                        });
                    
                    let webrtc_manager = Arc::new(tabagent_webrtc::WebRtcManager::new(webrtc_config, app_state_handler));
                    
                    let config = tabagent_api::ApiConfig { port, ..Default::default() };
                    if let Err(e) = tabagent_api::run_server_with_config(state, Some(webrtc_manager), config).await {
                        error!("WebRTC signaling server error: {}", e);
                    }
                })
            };
            
            // MCP stdio transport
            let state_for_mcp = state.clone();
            let coordinator = Arc::new(storage::DatabaseCoordinator::new()?);
            let mcp_manager = tabagent_mcp::McpManager::new(state_for_mcp, coordinator);
            let mcp_handle = tokio::spawn(async move {
                info!("Starting MCP transport (stdio)");
                if let Err(e) = tabagent_mcp::transport::run_stdio_transport(mcp_manager).await {
                    error!("MCP transport error: {}", e);
                }
            });
            
            // Wait for any transport to fail
            tokio::select! {
                _ = http_handle => {
                    error!("HTTP server terminated unexpectedly");
                }
                _ = webrtc_handle => {
                    error!("WebRTC server terminated unexpectedly");
                }
                _ = mcp_handle => {
                    error!("MCP transport terminated unexpectedly");
                }
            }
        }
    }

    Ok(())
}

