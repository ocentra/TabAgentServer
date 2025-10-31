use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use common::logging::{
    LogEntry, LogLevel, LogQuery, LogQueryResult, LogSource, McpResource, McpResourceList,
};
use dashmap::DashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

const PORT: u16 = 3333;
const MAX_LOGS: usize = 1000;

/// Shared application state
#[derive(Clone)]
struct AppState {
    logs: Arc<RwLock<VecDeque<LogEntry>>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            logs: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_LOGS))),
        }
    }

    async fn add_log(&self, entry: LogEntry) {
        let mut logs = self.logs.write().await;
        
        // Keep only the last MAX_LOGS entries
        if logs.len() >= MAX_LOGS {
            logs.pop_front();
        }
        
        logs.push_back(entry);
    }

    async fn get_logs(&self, query: LogQuery) -> LogQueryResult {
        let logs = self.logs.read().await;
        
        let mut filtered: Vec<LogEntry> = logs
            .iter()
            .filter(|log| {
                // Filter by level
                if let Some(ref level_filter) = query.level {
                    if log.level.to_string() != level_filter.to_lowercase() {
                        return false;
                    }
                }
                
                // Filter by context
                if let Some(ref context_filter) = query.context {
                    if !log.context.contains(context_filter) {
                        return false;
                    }
                }
                
                // Filter by source
                if let Some(ref source_filter) = query.source {
                    let source = LogSource::from(source_filter.as_str());
                    if log.source != source {
                        return false;
                    }
                }
                
                // Filter by timestamp
                if let Some(ref since) = query.since {
                    if let Ok(since_time) = DateTime::parse_from_rfc3339(since) {
                        if log.timestamp < since_time {
                            return false;
                        }
                    }
                }
                
                true
            })
            .cloned()
            .collect();
        
        // Apply limit
        let limit = query.limit.unwrap_or(100).min(MAX_LOGS);
        if filtered.len() > limit {
            filtered = filtered.into_iter().rev().take(limit).rev().collect();
        }
        
        LogQueryResult {
            count: filtered.len(),
            logs: filtered,
        }
    }

    async fn clear_logs(&self) -> usize {
        let mut logs = self.logs.write().await;
        let count = logs.len();
        logs.clear();
        count
    }
}

/// POST /log - Receive log entries from extension or Rust components
async fn receive_log(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Parse incoming log entry
    let entry = match serde_json::from_value::<LogEntry>(payload.clone()) {
        Ok(entry) => entry,
        Err(_) => {
            // Fallback: try to parse from TypeScript extension format
            let level = payload
                .get("level")
                .and_then(|v| v.as_str())
                .map(LogLevel::from)
                .unwrap_or_default();
            
            let context = payload
                .get("context")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            
            let message = payload
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            let source = payload
                .get("source")
                .and_then(|v| v.as_str())
                .map(LogSource::from)
                .unwrap_or(LogSource::Extension);
            
            let timestamp = payload
                .get("timestamp")
                .and_then(|v| v.as_str())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now);
            
            LogEntry {
                timestamp,
                level,
                context,
                message,
                source,
                data: payload.get("data").cloned(),
            }
        }
    };
    
    state.add_log(entry).await;
    
    Json(serde_json::json!({
        "success": true,
        "logsStored": state.logs.read().await.len()
    }))
}

/// GET /logs - Query logs with filters
async fn query_logs(
    State(state): State<AppState>,
    Query(query): Query<LogQuery>,
) -> impl IntoResponse {
    let result = state.get_logs(query).await;
    Json(result)
}

/// POST /clear - Clear all logs
async fn clear_logs(State(state): State<AppState>) -> impl IntoResponse {
    let count = state.clear_logs().await;
    Json(serde_json::json!({
        "success": true,
        "cleared": count
    }))
}

/// DELETE /logs - Clear all logs (alternative endpoint)
async fn delete_logs(State(state): State<AppState>) -> impl IntoResponse {
    let count = state.clear_logs().await;
    Json(serde_json::json!({
        "success": true,
        "cleared": count
    }))
}

/// GET / - Serve terminal-style web UI
async fn serve_ui() -> Html<&'static str> {
    Html(include_str!("ui.html"))
}

/// GET /mcp/resources - List available MCP resources
async fn mcp_list_resources() -> impl IntoResponse {
    let resources = McpResourceList {
        resources: vec![
            McpResource {
                uri: "tabagent://logs/all".to_string(),
                name: "All Logs".to_string(),
                description: "All logs from extension and Rust server".to_string(),
                mime_type: "application/json".to_string(),
            },
            McpResource {
                uri: "tabagent://logs/extension".to_string(),
                name: "Extension Logs".to_string(),
                description: "Logs from browser extension only".to_string(),
                mime_type: "application/json".to_string(),
            },
            McpResource {
                uri: "tabagent://logs/rust".to_string(),
                name: "Rust Server Logs".to_string(),
                description: "Logs from Rust backend services".to_string(),
                mime_type: "application/json".to_string(),
            },
            McpResource {
                uri: "tabagent://logs/errors".to_string(),
                name: "Error Logs".to_string(),
                description: "All error-level logs".to_string(),
                mime_type: "application/json".to_string(),
            },
            McpResource {
                uri: "tabagent://logs/generation".to_string(),
                name: "Model Generation Logs".to_string(),
                description: "Logs related to LLM generation and inference".to_string(),
                mime_type: "application/json".to_string(),
            },
        ],
    };
    
    Json(resources)
}

/// GET /mcp/resource - Read a specific MCP resource
async fn mcp_read_resource(
    State(state): State<AppState>,
    Query(params): Query<serde_json::Value>,
) -> impl IntoResponse {
    let uri = params
        .get("uri")
        .and_then(|v| v.as_str())
        .unwrap_or("tabagent://logs/all");
    
    let result = match uri {
        "tabagent://logs/all" => {
            state.get_logs(LogQuery::default()).await
        }
        "tabagent://logs/extension" => {
            state
                .get_logs(LogQuery {
                    source: Some("extension".to_string()),
                    ..Default::default()
                })
                .await
        }
        "tabagent://logs/rust" => {
            // Get all Rust-sourced logs (not extension)
            let logs = state.logs.read().await;
            let filtered: Vec<LogEntry> = logs
                .iter()
                .filter(|log| log.source != LogSource::Extension)
                .cloned()
                .collect();
            
            LogQueryResult {
                count: filtered.len(),
                logs: filtered,
            }
        }
        "tabagent://logs/errors" => {
            state
                .get_logs(LogQuery {
                    level: Some("error".to_string()),
                    ..Default::default()
                })
                .await
        }
        "tabagent://logs/generation" => {
            let logs = state.logs.read().await;
            let filtered: Vec<LogEntry> = logs
                .iter()
                .filter(|log| {
                    log.context.to_lowercase().contains("model")
                        || log.context.to_lowercase().contains("generation")
                        || log.message.to_lowercase().contains("ttft")
                        || log.message.to_lowercase().contains("token")
                })
                .cloned()
                .collect();
            
            LogQueryResult {
                count: filtered.len(),
                logs: filtered,
            }
        }
        _ => LogQueryResult::empty(),
    };
    
    Json(result)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    let state = AppState::new();

    // Build router
    let app = Router::new()
        .route("/", get(serve_ui))
        .route("/log", post(receive_log))
        .route("/logs", get(query_logs).delete(delete_logs))
        .route("/clear", post(clear_logs))
        .route("/mcp/resources", get(mcp_list_resources))
        .route("/mcp/resource", get(mcp_read_resource))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("127.0.0.1:{}", PORT);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("📝 TabAgent Log Server running on http://localhost:{}", PORT);
    info!("   POST /log         - Extension sends logs here");
    info!("   GET  /logs        - Fetch logs (supports ?level=error&limit=50)");
    info!("   POST /clear       - Clear all logs");
    info!("   DELETE /logs      - Clear all logs");
    info!("   GET  /mcp/resources - List MCP resources");
    info!("   GET  /mcp/resource  - Read MCP resource");

    axum::serve(listener, app).await?;

    Ok(())
}
