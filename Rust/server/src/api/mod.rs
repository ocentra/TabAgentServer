//! HTTP API module (placeholder - will be moved to separate crate)
//!
//! TODO: This will become tabagent-api crate

use axum::{
    Router,
    routing::get,
    response::Json,
};
use serde_json::json;
use std::sync::Arc;
use crate::state::AppState;

/// Run the HTTP API server
pub async fn run_server(state: Arc<AppState>, port: u16) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/health", get(health_check))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("HTTP API listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "tabagent-server"
    }))
}

