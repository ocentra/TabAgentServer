//! MCP stdio transport implementation
//!
//! Handles JSON-RPC over stdin/stdout for MCP protocol communication

use crate::McpManager;
use common::logging::{LogEntry, LogLevel, LogSource};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{error, info, warn};

/// MCP JSON-RPC request
#[derive(Debug, Clone, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

/// MCP JSON-RPC response
#[derive(Debug, Clone, Serialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<McpError>,
    pub id: Option<Value>,
}

/// MCP error response
#[derive(Debug, Clone, Serialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Run the MCP stdio transport server
pub async fn run_stdio_transport(manager: McpManager) -> anyhow::Result<()> {
    info!("Starting MCP stdio transport server");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    
    let mut reader = BufReader::new(stdin);
    let mut writer = stdout;
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        
        if bytes_read == 0 {
            // EOF - client disconnected
            info!("MCP client disconnected");
            break;
        }

        // Parse JSON-RPC request
        let request: McpRequest = match serde_json::from_str(&line.trim()) {
            Ok(req) => req,
            Err(e) => {
                warn!("Invalid JSON-RPC request: {}", e);
                let response = McpResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(McpError {
                        code: -32700,
                        message: "Parse error".to_string(),
                        data: Some(Value::String(e.to_string())),
                    }),
                    id: None,
                };
                let response_str = serde_json::to_string(&response)?;
                writer.write_all(response_str.as_bytes()).await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;
                continue;
            }
        };

        // Handle request
        let result = handle_request(&manager, &request.method, &request.params).await;
        
        // Send response
        let response = McpResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result?),
            error: None,
            id: request.id,
        };
        
        let response_str = serde_json::to_string(&response)?;
        writer.write_all(response_str.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
    }

    Ok(())
}

/// Handle an MCP request
async fn handle_request(
    manager: &McpManager,
    method: &str,
    params: &Option<Value>,
) -> anyhow::Result<Value> {
    match method {
        // === LOGS TOOLS ===
        "query_logs" => {
            let query = match params {
                Some(p) => serde_json::from_value::<common::logging::LogQuery>(p.clone())?,
                None => common::logging::LogQuery::default(),
            };
            
            let logs = manager.query_logs(query)?;
            Ok(serde_json::json!({
                "count": logs.len(),
                "logs": logs
            }))
        }
        
        "get_log_stats" => {
            let stats = manager.get_log_stats()?;
            Ok(serde_json::to_value(&stats)?)
        }
        
        "clear_logs" => {
            let count = manager.clear_logs()?;
            Ok(serde_json::json!({ "cleared": count }))
        }
        
        "store_log" => {
            let entry = match params {
                Some(p) => serde_json::from_value::<LogEntry>(p.clone())?,
                None => return Err(anyhow::anyhow!("Missing log entry")),
            };
            
            manager.store_log(entry)?;
            Ok(serde_json::json!({ "success": true }))
        }
        
        // === MODEL TOOLS ===
        "list_models" => {
            manager.list_models().await
        }
        
        "load_model" => {
            let model_id = params
                .as_ref()
                .and_then(|p| p.get("model_id")?.as_str().map(|s| s.to_string()))
                .ok_or_else(|| anyhow::anyhow!("Missing model_id"))?;
            let variant = params
                .as_ref()
                .and_then(|p| p.get("variant"))
                .and_then(|v| v.as_str().map(|s| s.to_string()));
            
            manager.load_model(model_id, variant).await
        }
        
        "unload_model" => {
            let model_id = params
                .as_ref()
                .and_then(|p| p.get("model_id")?.as_str().map(|s| s.to_string()))
                .ok_or_else(|| anyhow::anyhow!("Missing model_id"))?;
            
            manager.unload_model(model_id).await
        }
        
        "get_model_info" => {
            let model_id = params
                .as_ref()
                .and_then(|p| p.get("model_id")?.as_str().map(|s| s.to_string()))
                .ok_or_else(|| anyhow::anyhow!("Missing model_id"))?;
            
            manager.get_model_info(model_id).await
        }
        
        "pull_model" => {
            let model = params
                .as_ref()
                .and_then(|p| p.get("model")?.as_str().map(|s| s.to_string()))
                .ok_or_else(|| anyhow::anyhow!("Missing model"))?;
            let quantization = params
                .as_ref()
                .and_then(|p| p.get("quantization"))
                .and_then(|v| v.as_str().map(|s| s.to_string()));
            
            manager.pull_model(model, quantization).await
        }
        
        "delete_model" => {
            let model_id = params
                .as_ref()
                .and_then(|p| p.get("model_id")?.as_str().map(|s| s.to_string()))
                .ok_or_else(|| anyhow::anyhow!("Missing model_id"))?;
            
            manager.delete_model(model_id).await
        }
        
        "get_loaded_models" => {
            manager.get_loaded_models().await
        }
        
        "get_embedding_models" => {
            manager.get_embedding_models().await
        }
        
        "select_model" => {
            let model_id = params
                .as_ref()
                .and_then(|p| p.get("model_id")?.as_str().map(|s| s.to_string()))
                .ok_or_else(|| anyhow::anyhow!("Missing model_id"))?;
            
            manager.select_model(model_id).await
        }
        
        // === SYSTEM TOOLS ===
        "health_check" => {
            manager.health_check().await
        }
        
        "get_system_info" => {
            manager.get_system_info().await
        }
        
        "get_stats" => {
            manager.get_stats().await
        }
        
        "get_resources" => {
            manager.get_resources().await
        }
        
        "estimate_memory" => {
            let model = params
                .as_ref()
                .and_then(|p| p.get("model")?.as_str().map(|s| s.to_string()))
                .ok_or_else(|| anyhow::anyhow!("Missing model"))?;
            let quantization = params
                .as_ref()
                .and_then(|p| p.get("quantization"))
                .and_then(|v| v.as_str().map(|s| s.to_string()));
            
            manager.estimate_memory(model, quantization).await
        }
        
        "get_hardware_info" => {
            manager.get_hardware_info().await
        }
        
        "check_model_feasibility" => {
            let model_size_mb = params
                .as_ref()
                .and_then(|p| p.get("model_size_mb")?.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Missing model_size_mb"))?;
            
            manager.check_model_feasibility(model_size_mb).await
        }
        
        "get_recommended_models" => {
            manager.get_recommended_models().await
        }
        
        // === GENERATION TOOLS ===
        "test_generation" => {
            let model = params
                .as_ref()
                .and_then(|p| p.get("model")?.as_str().map(|s| s.to_string()))
                .ok_or_else(|| anyhow::anyhow!("Missing model"))?;
            let prompt = params
                .as_ref()
                .and_then(|p| p.get("prompt")?.as_str().map(|s| s.to_string()))
                .ok_or_else(|| anyhow::anyhow!("Missing prompt"))?;
            
            manager.test_generation(model, prompt).await
        }
        
        "test_chat" => {
            let model = params
                .as_ref()
                .and_then(|p| p.get("model")?.as_str().map(|s| s.to_string()))
                .ok_or_else(|| anyhow::anyhow!("Missing model"))?;
            let messages = params
                .as_ref()
                .and_then(|p| p.get("messages")?.as_array().cloned())
                .and_then(|msgs| serde_json::from_value::<Vec<tabagent_values::Message>>(serde_json::Value::Array(msgs)).ok())
                .ok_or_else(|| anyhow::anyhow!("Missing messages"))?;
            
            manager.test_chat(model, messages).await
        }
        
        "generate_embeddings" => {
            let model = params
                .as_ref()
                .and_then(|p| p.get("model")?.as_str().map(|s| s.to_string()))
                .ok_or_else(|| anyhow::anyhow!("Missing model"))?;
            let input = params
                .as_ref()
                .and_then(|p| p.get("input"))
                .and_then(|v| serde_json::from_value::<tabagent_values::EmbeddingInput>(v.clone()).ok())
                .ok_or_else(|| anyhow::anyhow!("Missing input"))?;
            
            manager.generate_embeddings(model, input).await
        }
        
        "stop_generation" => {
            let request_id = params
                .as_ref()
                .and_then(|p| p.get("request_id")?.as_str().map(|s| s.to_string()))
                .ok_or_else(|| anyhow::anyhow!("Missing request_id"))?;
            
            manager.stop_generation(request_id).await
        }
        
        // === RAG TOOLS ===
        "semantic_search" => {
            let query = params
                .as_ref()
                .and_then(|p| p.get("query")?.as_str().map(|s| s.to_string()))
                .ok_or_else(|| anyhow::anyhow!("Missing query"))?;
            let k = params
                .as_ref()
                .and_then(|p| p.get("k")?.as_u64().map(|u| u as usize))
                .unwrap_or(10);
            let filters = params
                .as_ref()
                .and_then(|p| p.get("filters").cloned());
            
            manager.semantic_search(query, k, filters).await
        }
        
        "calculate_similarity" => {
            let text1 = params
                .as_ref()
                .and_then(|p| p.get("text1")?.as_str().map(|s| s.to_string()))
                .ok_or_else(|| anyhow::anyhow!("Missing text1"))?;
            let text2 = params
                .as_ref()
                .and_then(|p| p.get("text2")?.as_str().map(|s| s.to_string()))
                .ok_or_else(|| anyhow::anyhow!("Missing text2"))?;
            let model = params
                .as_ref()
                .and_then(|p| p.get("model"))
                .and_then(|v| v.as_str().map(|s| s.to_string()));
            
            manager.calculate_similarity(text1, text2, model).await
        }
        
        "rerank" => {
            let model = params
                .as_ref()
                .and_then(|p| p.get("model")?.as_str().map(|s| s.to_string()))
                .ok_or_else(|| anyhow::anyhow!("Missing model"))?;
            let query = params
                .as_ref()
                .and_then(|p| p.get("query")?.as_str().map(|s| s.to_string()))
                .ok_or_else(|| anyhow::anyhow!("Missing query"))?;
            let documents = params
                .as_ref()
                .and_then(|p| p.get("documents")?.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .ok_or_else(|| anyhow::anyhow!("Missing documents"))?;
            let top_n = params
                .as_ref()
                .and_then(|p| p.get("top_n"))
                .and_then(|v| v.as_u64().map(|u| u as usize));
            
            manager.rerank(model, query, documents, top_n).await
        }
        
        // === SESSION TOOLS ===
        "get_chat_history" => {
            let session_id = params
                .as_ref()
                .and_then(|p| p.get("session_id"))
                .and_then(|v| v.as_str().map(|s| s.to_string()));
            let limit = params
                .as_ref()
                .and_then(|p| p.get("limit"))
                .and_then(|v| v.as_u64().map(|u| u as usize));
            
            manager.get_chat_history(session_id, limit).await
        }
        
        // Unknown method
        _ => Err(anyhow::anyhow!("Unknown method: {}", method)),
    }
}
