//! TabAgent MCP (Model Context Protocol) Integration
//!
//! Provides MCP stdio transport for AI assistants (Cursor, Claude Desktop) to access:
//! - System logs with persistent storage
//! - Database queries with efficient serialization
//! - Model information and statistics
//! - System monitoring and health

use storage::DatabaseCoordinator;
use common::{DbResult, models::Node, backend::AppStateProvider};
use common::logging::{LogEntry, LogLevel, LogQuery, LogSource};
use std::sync::Arc;

pub mod error;
pub mod transport;

pub use error::{McpError, McpResult};

/// MCP Manager that coordinates all MCP servers
pub struct McpManager {
    state: Arc<dyn AppStateProvider>,
    coordinator: Arc<DatabaseCoordinator>,
}

impl McpManager {
    /// Create a new MCP Manager with storage coordinator and app state
    pub fn new(state: Arc<dyn AppStateProvider>, coordinator: Arc<DatabaseCoordinator>) -> Self {
        Self { state, coordinator }
    }

    // ========== LOG OPERATIONS ==========

    /// Store a log entry in persistent storage
    ///
    /// Persists a log entry to the database for later querying and analysis.
    ///
    /// # Arguments
    ///
    /// * `entry` - The log entry to store
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Database operation fails
    /// - Serialization fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # use common::logging::{LogEntry, LogLevel, LogSource};
    /// # let manager = create_test_manager();
    /// let log = LogEntry::new(
    ///     LogLevel::Error,
    ///     "database".to_string(),
    ///     "Connection failed".to_string(),
    ///     LogSource::Storage
    /// );
    /// manager.store_log(log)?;
    /// # Ok::<(), common::DbError>(())
    /// ```
    pub fn store_log(&self, entry: LogEntry) -> DbResult<()> {
        let node = Node::Log(entry);
        self.coordinator.logs.insert_node(&node)
    }

    /// Query logs from persistent storage
    ///
    /// Scans log entries from persistent storage and filters them based on the provided query.
    /// Results are sorted by timestamp (newest first) and limited according to query parameters.
    ///
    /// # Arguments
    ///
    /// * `query` - Query parameters including filters (level, context, source, since) and limit
    ///
    /// # Returns
    ///
    /// Vector of log entries matching the query criteria, sorted by timestamp descending.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Invalid limit provided (must be between 1 and 10000)
    /// - Invalid RFC3339 timestamp in `since` field
    /// - Storage operation fails
    /// - Serialization/deserialization fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # use common::logging::LogQuery;
    /// # let manager = create_test_manager();
    /// // Query error logs only
    /// let query = LogQuery {
    ///     level: Some("error".to_string()),
    ///     limit: Some(100),
    ///     ..Default::default()
    /// };
    /// let error_logs = manager.query_logs(query)?;
    /// assert!(error_logs.iter().all(|log| log.level == LogLevel::Error));
    /// # Ok::<(), common::DbError>(())
    /// ```
    pub fn query_logs(&self, query: LogQuery) -> DbResult<Vec<LogEntry>> {
        // Input validation
        if let Some(limit) = query.limit {
            if limit == 0 || limit > 10000 {
                return Err(common::DbError::InvalidInput(
                    "limit must be between 1 and 10000".to_string()
                ));
            }
        }

        if let Some(ref since) = query.since {
            if chrono::DateTime::parse_from_rfc3339(since).is_err() {
                return Err(common::DbError::InvalidInput(
                    format!("Invalid RFC3339 timestamp: {}", since)
                ));
            }
        }

        // Optimize capacity based on limit
        let limit = query.limit.unwrap_or(1000);
        let mut results = Vec::with_capacity(limit.min(1000));
        
        // Early termination: stop after finding enough results
        let mut scan_count = 0;
        const MAX_SCAN: usize = 100000; // Prevent infinite loops
        
        for result in self.coordinator.logs.scan_prefix(b"") {
            if scan_count >= MAX_SCAN {
                tracing::warn!("Log query exceeded max scan limit of {}", MAX_SCAN);
                break;
            }
            scan_count += 1;
            
            let (_, bytes) = result?;
            
            // Deserialize node
            let archived = rkyv::check_archived_root::<Node>(&bytes)
                .map_err(|e| common::DbError::Serialization(e.to_string()))?;
            let node = archived.deserialize(&mut rkyv::Infallible)
                .map_err(|e| common::DbError::Serialization(e.to_string()))?;
            
            if let Node::Log(log) = node {
                // Apply filters using helper function
                if !self::query_matches(&query, &log) {
                    continue;
                }
                
                results.push(log);
                
                // Early termination if we have enough results
                if results.len() >= limit {
                    break;
                }
            }
        }
        
        // Sort only what we collected
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        results.truncate(limit);
        
        Ok(results)
    }

    /// Helper function to check if a log entry matches query criteria
    fn query_matches(query: &LogQuery, log: &LogEntry) -> bool {
        if let Some(ref level_filter) = query.level {
            if log.level.to_string() != level_filter.to_lowercase() {
                return false;
            }
        }
        
        if let Some(ref context_filter) = query.context {
            if !log.context.contains(context_filter) {
                return false;
            }
        }
        
        if let Some(ref source_filter) = query.source {
            let source = LogSource::from(source_filter.as_str());
            if log.source != source {
                return false;
            }
        }
        
        if let Some(ref since) = query.since {
            if let Ok(since_time) = chrono::DateTime::parse_from_rfc3339(since) {
                let since_ms = since_time.timestamp_millis();
                if log.timestamp < since_ms {
                    return false;
                }
            }
        }
        
        true
    }

    /// Clear logs from persistent storage
    ///
    /// Removes all log entries from the database.
    ///
    /// # Returns
    ///
    /// Number of log entries deleted.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Database operation fails
    /// - Deletion encounters errors
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # let manager = create_test_manager();
    /// let count = manager.clear_logs()?;
    /// println!("Cleared {} log entries", count);
    /// # Ok::<(), common::DbError>(())
    /// ```
    pub fn clear_logs(&self) -> DbResult<usize> {
        let mut count = 0;
        
        // Iterate and delete all logs
        for result in self.coordinator.logs.iter() {
            let (key, _) = result?;
            
            // We need to read the node to check if it's a log, but we can just try to delete
            // For now, let's just delete all nodes in the logs database
            self.coordinator.logs.remove_node(&String::from_utf8_lossy(&key).as_ref())?;
            count += 1;
        }
        
        Ok(count)
    }

    /// Get log statistics and aggregations
    ///
    /// Analyzes stored logs and returns statistics grouped by level,
    /// source, and context.
    ///
    /// # Returns
    ///
    /// `LogStats` struct with aggregated log data.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Database query fails
    /// - Log deserialization fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # let manager = create_test_manager();
    /// let stats = manager.get_log_stats()?;
    /// println!("Total logs: {}", stats.total_logs);
    /// println!("Error count: {}", stats.by_level.get("error").unwrap_or(&0));
    /// # Ok::<(), common::DbError>(())
    /// ```
    pub fn get_log_stats(&self) -> DbResult<LogStats> {
        let logs = self.query_logs(LogQuery::default())?;
        
        let mut by_level = std::collections::HashMap::new();
        let mut by_source = std::collections::HashMap::new();
        let mut by_context: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        
        let mut oldest_log = None;
        let mut newest_log = None;
        
        for log in &logs {
            // Count by level
            let level_str = log.level.to_string();
            *by_level.entry(level_str).or_insert(0) += 1;
            
            // Count by source
            let source_str = format!("{:?}", log.source);
            *by_source.entry(source_str).or_insert(0) += 1;
            
            // Count by context
            *by_context.entry(&log.context).or_insert(0) += 1;
            
            // Track oldest/newest
            match oldest_log {
                None => oldest_log = Some(log.timestamp),
                Some(old) if log.timestamp < old => oldest_log = Some(log.timestamp),
                _ => {}
            }
            
            match newest_log {
                None => newest_log = Some(log.timestamp),
                Some(new) if log.timestamp > new => newest_log = Some(log.timestamp),
                _ => {}
            }
        }
        
        Ok(LogStats {
            total_logs: logs.len(),
            by_level,
            by_source,
            by_context: by_context.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
            oldest_timestamp: oldest_log,
            newest_timestamp: newest_log,
        })
    }

    // ========== MODEL OPERATIONS (Forward to AppState) ==========

    /// List all available models
    ///
    /// Returns a JSON object containing all models available in the system,
    /// including their IDs, types, and current status.
    ///
    /// # Returns
    ///
    /// JSON value containing array of model objects with metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - AppState communication fails
    /// - Serialization of response fails
    /// - Internal model registry is unavailable
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # use std::sync::Arc;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// # let manager = create_test_manager();
    /// let models = manager.list_models().await?;
    /// 
    /// // Parse response
    /// let models_array = models["models"].as_array()
    ///     .expect("models should be an array");
    /// println!("Found {} models", models_array.len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # See Also
    ///
    /// - [`load_model`] - Load a specific model
    /// - [`get_model_info`] - Get details about a loaded model
    pub async fn list_models(&self) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::list_models();
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    /// Load a model into memory
    ///
    /// Loads a specific model by ID with an optional variant (quantization level).
    ///
    /// # Arguments
    ///
    /// * `model_id` - The identifier of the model to load
    /// * `variant` - Optional variant/quantization (e.g., "q4_0", "q8_0")
    ///
    /// # Returns
    ///
    /// JSON value containing load status and model metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Model ID is not found
    /// - Insufficient memory to load model
    /// - AppState communication fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let result = manager.load_model(
    ///     "llama-7b".to_string(),
    ///     Some("q4_0".to_string())
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_model(&self, model_id: String, variant: Option<String>) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::load_model(model_id, variant);
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    /// Unload a model from memory
    ///
    /// Removes a loaded model from memory to free up resources.
    ///
    /// # Arguments
    ///
    /// * `model_id` - The identifier of the model to unload
    ///
    /// # Returns
    ///
    /// JSON value confirming the model was unloaded.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Model ID is not found
    /// - Model is not currently loaded
    /// - AppState communication fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let result = manager.unload_model("llama-7b".to_string()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn unload_model(&self, model_id: String) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::unload_model(model_id);
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    /// Get detailed information about a model
    ///
    /// Returns comprehensive metadata about a specific model including size,
    /// architecture, capabilities, and current status.
    ///
    /// # Arguments
    ///
    /// * `model_id` - The identifier of the model
    ///
    /// # Returns
    ///
    /// JSON value containing model details (size, type, parameters, etc.).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Model ID is not found
    /// - AppState communication fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let info = manager.get_model_info("llama-7b".to_string()).await?;
    /// println!("Model params: {}", info["parameters"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_model_info(&self, model_id: String) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::model_info(model_id);
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    /// Pull/download a model from HuggingFace
    ///
    /// Downloads a model from the HuggingFace Hub to local cache.
    ///
    /// # Arguments
    ///
    /// * `model` - HuggingFace model identifier (e.g., "meta-llama/Llama-2-7b")
    /// * `quantization` - Optional quantization level to download
    ///
    /// # Returns
    ///
    /// JSON value with download progress and final status.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Model not found on HuggingFace
    /// - Network connection fails
    /// - Insufficient disk space
    /// - AppState communication fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let result = manager.pull_model(
    ///     "meta-llama/Llama-2-7b".to_string(),
    ///     Some("q4_0".to_string())
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn pull_model(&self, model: String, quantization: Option<String>) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::pull_model(model, quantization);
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    /// Delete a model from local cache
    ///
    /// Permanently removes a model from disk storage to free up space.
    ///
    /// # Arguments
    ///
    /// * `model_id` - The identifier of the model to delete
    ///
    /// # Returns
    ///
    /// JSON value confirming deletion and space freed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Model ID is not found
    /// - Model is currently loaded (unload first)
    /// - File system operation fails
    /// - AppState communication fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let result = manager.delete_model("old-model".to_string()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_model(&self, model_id: String) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::delete_model(model_id);
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    /// Get all currently loaded models
    ///
    /// Returns a list of models that are currently loaded in memory and
    /// ready for inference.
    ///
    /// # Returns
    ///
    /// JSON array of loaded model objects with their status.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - AppState communication fails
    /// - Serialization fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let loaded = manager.get_loaded_models().await?;
    /// let count = loaded["models"].as_array().unwrap().len();
    /// println!("Loaded models: {}", count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_loaded_models(&self) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::get_loaded_models();
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    /// Get all available embedding models
    ///
    /// Returns a list of models specifically designed for generating embeddings
    /// for semantic search and similarity tasks.
    ///
    /// # Returns
    ///
    /// JSON array of embedding model objects.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - AppState communication fails
    /// - Serialization fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let embedders = manager.get_embedding_models().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_embedding_models(&self) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::get_embedding_models();
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    /// Select a model as the active default
    ///
    /// Sets a model as the default for inference operations when no model
    /// is explicitly specified.
    ///
    /// # Arguments
    ///
    /// * `model_id` - The identifier of the model to set as active
    ///
    /// # Returns
    ///
    /// JSON value confirming the active model selection.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Model ID is not found
    /// - Model is not loaded
    /// - AppState communication fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// manager.select_model("llama-7b".to_string()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn select_model(&self, model_id: String) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::select_model(model_id);
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    // ========== SYSTEM OPERATIONS ==========

    /// Perform system health check
    ///
    /// Verifies that the system is operational and all components are responsive.
    ///
    /// # Returns
    ///
    /// JSON value with health status and component states.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - AppState communication fails
    /// - Critical system components are unresponsive
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let health = manager.health_check().await?;
    /// assert_eq!(health["status"], "healthy");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn health_check(&self) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::health();
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    /// Get comprehensive system information
    ///
    /// Returns detailed information about the system including OS, CPU,
    /// memory, GPU, and TabAgent version.
    ///
    /// # Returns
    ///
    /// JSON value with system configuration and capabilities.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - AppState communication fails
    /// - System query fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let info = manager.get_system_info().await?;
    /// println!("OS: {}, RAM: {}", info["os"], info["memory"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_system_info(&self) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::system_info();
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    /// Get performance statistics
    ///
    /// Returns runtime performance metrics including request counts,
    /// latencies, throughput, and resource utilization.
    ///
    /// # Returns
    ///
    /// JSON value with performance statistics.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - AppState communication fails
    /// - Statistics collection fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let stats = manager.get_stats().await?;
    /// println!("Requests: {}", stats["total_requests"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_stats(&self) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::get_stats();
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    /// Get current resource utilization
    ///
    /// Returns real-time resource usage including CPU, memory, GPU utilization,
    /// and available capacity.
    ///
    /// # Returns
    ///
    /// JSON value with current resource usage metrics.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - AppState communication fails
    /// - Resource monitoring fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let resources = manager.get_resources().await?;
    /// println!("GPU usage: {}%", resources["gpu_utilization"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_resources(&self) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::get_resources();
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    /// Estimate memory requirements for a model
    ///
    /// Calculates the approximate memory needed to load a specific model
    /// with the given quantization.
    ///
    /// # Arguments
    ///
    /// * `model` - Model identifier or path
    /// * `quantization` - Optional quantization level
    ///
    /// # Returns
    ///
    /// JSON value with estimated memory requirements in MB.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Model information unavailable
    /// - AppState communication fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let est = manager.estimate_memory(
    ///     "llama-13b".to_string(),
    ///     Some("q4_0".to_string())
    /// ).await?;
    /// println!("Needs {} MB", est["estimated_mb"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn estimate_memory(&self, model: String, quantization: Option<String>) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::estimate_memory(model, quantization);
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    /// Get detailed hardware information
    ///
    /// Returns comprehensive hardware specifications including CPU model,
    /// core count, memory size, GPU details, and capabilities.
    ///
    /// # Returns
    ///
    /// JSON value with hardware specifications.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - AppState communication fails
    /// - Hardware detection fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let hw = manager.get_hardware_info().await?;
    /// println!("CPU: {}, Cores: {}", hw["cpu"], hw["cores"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_hardware_info(&self) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::get_hardware_info();
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    /// Check if a model can be loaded on current hardware
    ///
    /// Verifies whether the system has sufficient resources to load
    /// a model of the specified size.
    ///
    /// # Arguments
    ///
    /// * `model_size_mb` - Model size in megabytes
    ///
    /// # Returns
    ///
    /// JSON value with feasibility assessment and recommendations.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - AppState communication fails
    /// - Resource check fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let check = manager.check_model_feasibility(7000).await?;
    /// if check["feasible"].as_bool().unwrap() {
    ///     println!("Model can be loaded!");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn check_model_feasibility(&self, model_size_mb: u64) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::check_model_feasibility(model_size_mb);
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    /// Get recommended models for current hardware
    ///
    /// Returns a list of model sizes and quantizations that are optimal
    /// for the current hardware configuration.
    ///
    /// # Returns
    ///
    /// JSON array of recommended model configurations.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - AppState communication fails
    /// - Hardware analysis fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let recs = manager.get_recommended_models().await?;
    /// for model in recs["models"].as_array().unwrap() {
    ///     println!("Recommended: {}", model["name"]);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_recommended_models(&self) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::get_recommended_models();
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    // ========== GENERATION OPERATIONS ==========

    /// Test text generation with a prompt
    ///
    /// Generates text completion for the given prompt using the specified model.
    ///
    /// # Arguments
    ///
    /// * `model` - Model identifier to use for generation
    /// * `prompt` - Input text prompt
    ///
    /// # Returns
    ///
    /// JSON value with generated text and metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Model not found or not loaded
    /// - Generation fails
    /// - AppState communication fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let result = manager.test_generation(
    ///     "llama-7b".to_string(),
    ///     "Once upon a time".to_string()
    /// ).await?;
    /// println!("Generated: {}", result["text"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn test_generation(&self, model: String, prompt: String) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::generate(model, prompt, None);
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    /// Test chat completion with messages
    ///
    /// Generates a chat response based on a conversation history.
    ///
    /// # Arguments
    ///
    /// * `model` - Model identifier to use for chat
    /// * `messages` - Conversation history as a vector of messages
    ///
    /// # Returns
    ///
    /// JSON value with assistant's response and metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Model not found or not loaded
    /// - Invalid message format
    /// - Generation fails
    /// - AppState communication fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # use tabagent_values::Message;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let messages = vec![
    ///     Message::user("Hello, how are you?"),
    /// ];
    /// let result = manager.test_chat("llama-7b".to_string(), messages).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn test_chat(&self, model: String, messages: Vec<tabagent_values::Message>) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::chat(model, messages, None);
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    /// Generate embeddings for text input
    ///
    /// Creates vector embeddings for the given text using an embedding model.
    ///
    /// # Arguments
    ///
    /// * `model` - Embedding model identifier
    /// * `input` - Text input (string or array of strings)
    ///
    /// # Returns
    ///
    /// JSON value with embedding vectors and metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Model not found or not loaded
    /// - Invalid input format
    /// - Embedding generation fails
    /// - AppState communication fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # use tabagent_values::EmbeddingInput;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let input = EmbeddingInput::String("Hello world".to_string());
    /// let result = manager.generate_embeddings("bge-small".to_string(), input).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn generate_embeddings(&self, model: String, input: tabagent_values::EmbeddingInput) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::embeddings(model, input);
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    /// Stop an ongoing generation request
    ///
    /// Cancels an in-progress text generation or chat completion.
    ///
    /// # Arguments
    ///
    /// * `request_id` - The ID of the generation request to stop
    ///
    /// # Returns
    ///
    /// JSON value confirming the request was stopped.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Request ID not found
    /// - Request already completed
    /// - AppState communication fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// manager.stop_generation("req-12345".to_string()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn stop_generation(&self, request_id: String) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::stop_generation(request_id);
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    // ========== RAG OPERATIONS ==========

    /// Perform semantic search in vector database
    ///
    /// Searches for semantically similar documents using embeddings.
    ///
    /// # Arguments
    ///
    /// * `query` - Search query text
    /// * `k` - Number of results to return
    /// * `filters` - Optional metadata filters
    ///
    /// # Returns
    ///
    /// JSON array of search results with similarity scores.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Embedding model not available
    /// - Database query fails
    /// - AppState communication fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let results = manager.semantic_search(
    ///     "machine learning".to_string(),
    ///     10,
    ///     None
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn semantic_search(&self, query: String, k: usize, filters: Option<serde_json::Value>) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::semantic_search(query, k, filters);
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    /// Calculate similarity score between two texts
    ///
    /// Computes semantic similarity using embeddings and cosine similarity.
    ///
    /// # Arguments
    ///
    /// * `text1` - First text to compare
    /// * `text2` - Second text to compare
    /// * `model` - Optional embedding model (uses default if not specified)
    ///
    /// # Returns
    ///
    /// JSON value with similarity score (0.0 to 1.0).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Embedding model not available
    /// - Embedding generation fails
    /// - AppState communication fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let sim = manager.calculate_similarity(
    ///     "cat".to_string(),
    ///     "dog".to_string(),
    ///     None
    /// ).await?;
    /// println!("Similarity: {}", sim["score"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn calculate_similarity(&self, text1: String, text2: String, model: Option<String>) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::calculate_similarity(text1, text2, model);
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    /// Rerank documents by relevance to query
    ///
    /// Uses a reranking model to sort documents by relevance to the query.
    ///
    /// # Arguments
    ///
    /// * `model` - Reranking model identifier
    /// * `query` - Query text for relevance scoring
    /// * `documents` - List of documents to rerank
    /// * `top_n` - Optional number of top results to return
    ///
    /// # Returns
    ///
    /// JSON array of reranked documents with relevance scores.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Model not found or not loaded
    /// - Reranking fails
    /// - AppState communication fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let docs = vec!["doc1".to_string(), "doc2".to_string()];
    /// let ranked = manager.rerank(
    ///     "bge-reranker".to_string(),
    ///     "query".to_string(),
    ///     docs,
    ///     Some(5)
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn rerank(&self, model: String, query: String, documents: Vec<String>, top_n: Option<usize>) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::rerank(model, query, documents, top_n);
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }

    // ========== SESSION OPERATIONS ==========

    /// Get chat conversation history
    ///
    /// Retrieves message history for a chat session.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Optional session identifier (uses current session if not specified)
    /// * `limit` - Optional maximum number of messages to return
    ///
    /// # Returns
    ///
    /// JSON array of chat messages in chronological order.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Session not found
    /// - Database query fails
    /// - AppState communication fails
    ///
    /// # Examples
    ///
    /// ```
    /// # use tabagent_mcp::McpManager;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let manager = create_test_manager();
    /// let history = manager.get_chat_history(
    ///     Some("session-123".to_string()),
    ///     Some(50)
    /// ).await?;
    /// println!("Messages: {}", history["messages"].as_array().unwrap().len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_chat_history(&self, session_id: Option<String>, limit: Option<usize>) -> McpResult<serde_json::Value> {
        let request = tabagent_values::RequestValue::chat_history(session_id, limit);
        let response = self.state.handle_request(request).await
            .map_err(|e| McpError::AppState(e.to_string()))?;
        Ok(serde_json::to_value(&response)?)
    }
}

/// Log statistics response
#[derive(Debug, serde::Serialize)]
pub struct LogStats {
    pub total_logs: usize,
    pub by_level: std::collections::HashMap<String, usize>,
    pub by_source: std::collections::HashMap<String, usize>,
    pub by_context: std::collections::HashMap<String, usize>,
    pub oldest_timestamp: Option<i64>,
    pub newest_timestamp: Option<i64>,
}
