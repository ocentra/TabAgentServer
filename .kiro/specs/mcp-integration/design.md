# MCP Integration Design Document

## Overview

This design document outlines the integration of Model Context Protocol (MCP) as a fourth transport layer into the existing TabAgent server architecture. The design leverages existing infrastructure (AppState, Storage Engine, logging) while providing AI assistants with comprehensive access to system data and operations.

**Design Philosophy:**
- **Unified Architecture**: MCP as a peer transport layer, not a separate service
- **Leverage Existing Infrastructure**: Use AppState, Storage Engine, and logging systems
- **Zero-Copy Performance**: Benefit from rkyv serialization in storage engine migration
- **Extensible Tool System**: Support multiple MCP tool categories with room for growth

## Architecture

### Current Server Architecture

```
TabAgent Server (main.rs)
├── AppState (unified business logic)
├── HTTP Transport (tabagent-api)
├── WebRTC Transport (tabagent-webrtc)  
├── Native Messaging Transport (tabagent-native-messaging)
└── Storage Engine (sled → libmdbx migration)
```

### Target Architecture with MCP Integration

```
TabAgent Server (main.rs)
├── AppState (unified business logic)
├── HTTP Transport (tabagent-api)
├── WebRTC Transport (tabagent-webrtc)
├── Native Messaging Transport (tabagent-native-messaging)
├── MCP Transport (tabagent-mcp) ← NEW
│   ├── MCP Manager
│   ├── Log Server (logs tools)
│   ├── Database Server (database tools)
│   ├── Model Server (model tools)
│   └── System Server (monitoring tools)
└── Storage Engine (with persistent logs)
```

## Components and Interfaces

### Phase 1: MCP Transport Layer

#### MCP Transport Integration

```rust
// In server/src/main.rs - add MCP mode support
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
pub enum ServerMode {
    Native,
    Http,
    WebRtc,
    Mcp,      // ← NEW: MCP only
    Web,      // HTTP + WebRTC
    Both,     // HTTP + Native
    All,      // HTTP + WebRTC + Native + MCP ← UPDATED
}

// MCP transport startup
ServerMode::Mcp => {
    info!("Running in MCP mode (stdio)");
    tabagent_mcp::run_mcp_transport(state).await?;
}
ServerMode::All => {
    // ... existing transports
    
    // Add MCP transport
    let mcp_handle = {
        let state = state.clone();
        tokio::spawn(async move {
            info!("Starting MCP transport (stdio)");
            if let Err(e) = tabagent_mcp::run_mcp_transport(state).await {
                error!("MCP transport error: {}", e);
            }
        })
    };
    
    // Wait for any transport to fail
    tokio::select! {
        _ = http_handle => error!("HTTP server terminated"),
        _ = webrtc_handle => error!("WebRTC server terminated"),
        _ = mcp_handle => error!("MCP transport terminated"),
    }
}
```

#### New MCP Crate Structure

```
Rust/mcp/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API for server integration
│   ├── transport.rs        # MCP stdio transport implementation
│   ├── manager.rs          # MCP server manager and routing
│   ├── servers/
│   │   ├── mod.rs
│   │   ├── logs.rs         # Log-related MCP tools
│   │   ├── database.rs     # Database query MCP tools
│   │   ├── models.rs       # Model information MCP tools
│   │   └── system.rs       # System monitoring MCP tools
│   ├── tools/
│   │   ├── mod.rs
│   │   └── registry.rs     # Tool registration and discovery
│   └── error.rs            # MCP-specific error types
```

### Phase 2: MCP Manager and Tool System

#### MCP Manager Design

```rust
use rmcp::{Server, ServerConfig, ServerHandler};
use std::sync::Arc;
use common::AppStateProvider;

pub struct McpManager {
    servers: Vec<Box<dyn McpServerTrait>>,
    state: Arc<dyn AppStateProvider>,
}

#[async_trait]
pub trait McpServerTrait: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn tools(&self) -> Vec<rmcp::ToolSpec>;
    async fn handle_tool(&self, name: &str, params: serde_json::Value) -> anyhow::Result<serde_json::Value>;
}

impl McpManager {
    pub fn new(state: Arc<dyn AppStateProvider>) -> Self {
        let servers: Vec<Box<dyn McpServerTrait>> = vec![
            Box::new(LogsServer::new(state.clone())),
            Box::new(DatabaseServer::new(state.clone())),
            Box::new(ModelsServer::new(state.clone())),
            Box::new(SystemServer::new(state.clone())),
        ];
        
        Self { servers, state }
    }
    
    pub async fn run_stdio_transport(&self) -> anyhow::Result<()> {
        // Combine all tools from all servers
        let mut all_tools = Vec::new();
        for server in &self.servers {
            all_tools.extend(server.tools());
        }
        
        let config = ServerConfig::new()
            .with_name("TabAgent MCP Server")
            .with_version(env!("CARGO_PKG_VERSION"));
            
        let mut mcp_server = Server::new(config);
        
        // Register all tools
        for tool in all_tools {
            let tool_name = tool.name.clone();
            let servers = &self.servers;
            
            mcp_server.register_tool_handler(&tool_name, move |params| {
                // Route to appropriate server
                for server in servers {
                    if server.tools().iter().any(|t| t.name == tool_name) {
                        return server.handle_tool(&tool_name, params);
                    }
                }
                Err(anyhow::anyhow!("Tool not found: {}", tool_name))
            })?;
        }
        
        // Start stdio transport
        let transport = rmcp::transport::stdio::StdioTransport::new();
        mcp_server.start(transport).await?;
        
        Ok(())
    }
}
```

### Phase 3: Persistent Log Storage

#### Storage Engine Integration for Logs

```rust
// Extend AppState to include log storage
impl AppState {
    pub async fn store_log(&self, entry: LogEntry) -> DbResult<()> {
        let storage = self.get_storage_manager(DatabaseType::Logs)?;
        
        // Serialize log entry using current serialization (bincode/sled or rkyv/libmdbx)
        let key = format!("{}_{}", entry.timestamp.timestamp_nanos(), entry.source);
        let node = Node::LogEntry(entry); // Add LogEntry variant to Node enum
        
        storage.insert_node(&node).await?;
        Ok(())
    }
    
    pub async fn query_logs(&self, query: LogQuery) -> DbResult<LogQueryResult> {
        let storage = self.get_storage_manager(DatabaseType::Logs)?;
        
        // Use storage engine's scan capabilities for efficient filtering
        let mut results = Vec::new();
        let prefix = match &query.source {
            Some(source) => format!("logs_{}", source),
            None => "logs_".to_string(),
        };
        
        for result in storage.scan_prefix(&prefix)? {
            let (_, node) = result?;
            if let Node::LogEntry(entry) = node {
                // Apply filters
                if query.matches(&entry) {
                    results.push(entry);
                }
            }
        }
        
        // Apply limit and sorting
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }
        
        Ok(LogQueryResult {
            count: results.len(),
            logs: results,
        })
    }
}

// Add LogEntry to Node enum in common/src/models.rs
#[derive(Serialize, Deserialize, Archive, rkyv::Deserialize, rkyv::Serialize, Debug, Clone)]
pub enum Node {
    // ... existing variants
    LogEntry(LogEntry),
}
```

#### Logs MCP Server Implementation

```rust
use rmcp::tool;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

pub struct LogsServer {
    state: Arc<dyn AppStateProvider>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
struct QueryLogsParams {
    level: Option<String>,
    context: Option<String>,
    source: Option<String>,
    since: Option<String>,
    limit: Option<usize>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
struct ClearLogsParams {
    before: Option<String>,
    level: Option<String>,
}

impl McpServerTrait for LogsServer {
    fn name(&self) -> &str { "logs" }
    fn version(&self) -> &str { "1.0.0" }
    
    fn tools(&self) -> Vec<rmcp::ToolSpec> {
        vec![
            rmcp::ToolSpec {
                name: "query_logs".to_string(),
                description: "Query and filter log entries".to_string(),
                input_schema: schemars::schema_for!(QueryLogsParams),
            },
            rmcp::ToolSpec {
                name: "get_log_stats".to_string(),
                description: "Get log statistics and summaries".to_string(),
                input_schema: schemars::schema_for!(()),
            },
            rmcp::ToolSpec {
                name: "clear_logs".to_string(),
                description: "Clear log entries with optional filters".to_string(),
                input_schema: schemars::schema_for!(ClearLogsParams),
            },
        ]
    }
    
    async fn handle_tool(&self, name: &str, params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        match name {
            "query_logs" => {
                let params: QueryLogsParams = serde_json::from_value(params)?;
                let query = LogQuery {
                    level: params.level,
                    context: params.context,
                    source: params.source,
                    since: params.since,
                    limit: params.limit,
                };
                
                let result = self.state.query_logs(query).await?;
                Ok(serde_json::to_value(result)?)
            }
            "get_log_stats" => {
                // Implement log statistics
                let stats = self.calculate_log_stats().await?;
                Ok(serde_json::to_value(stats)?)
            }
            "clear_logs" => {
                let params: ClearLogsParams = serde_json::from_value(params)?;
                let count = self.clear_logs_with_filter(params).await?;
                Ok(serde_json::json!({ "cleared": count }))
            }
            _ => Err(anyhow::anyhow!("Unknown tool: {}", name))
        }
    }
}
```

### Phase 4: Database and Model MCP Servers

#### Database Server for Node/Edge Queries

```rust
pub struct DatabaseServer {
    state: Arc<dyn AppStateProvider>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
struct SearchDatabaseParams {
    node_type: Option<String>,
    query: Option<String>,
    limit: Option<usize>,
}

impl McpServerTrait for DatabaseServer {
    fn tools(&self) -> Vec<rmcp::ToolSpec> {
        vec![
            rmcp::ToolSpec {
                name: "search_nodes".to_string(),
                description: "Search nodes by type and content".to_string(),
                input_schema: schemars::schema_for!(SearchDatabaseParams),
            },
            rmcp::ToolSpec {
                name: "get_node_details".to_string(),
                description: "Get detailed information about a specific node".to_string(),
                input_schema: schemars::schema_for!(String), // node_id
            },
            rmcp::ToolSpec {
                name: "get_database_stats".to_string(),
                description: "Get database statistics and health info".to_string(),
                input_schema: schemars::schema_for!(()),
            },
        ]
    }
    
    async fn handle_tool(&self, name: &str, params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        match name {
            "search_nodes" => {
                let params: SearchDatabaseParams = serde_json::from_value(params)?;
                let results = self.search_nodes(params).await?;
                Ok(serde_json::to_value(results)?)
            }
            "get_node_details" => {
                let node_id: String = serde_json::from_value(params)?;
                let node = self.state.get_node(&node_id).await?;
                Ok(serde_json::to_value(node)?)
            }
            "get_database_stats" => {
                let stats = self.get_database_statistics().await?;
                Ok(serde_json::to_value(stats)?)
            }
            _ => Err(anyhow::anyhow!("Unknown tool: {}", name))
        }
    }
}
```

#### Models Server for Model Information

```rust
pub struct ModelsServer {
    state: Arc<dyn AppStateProvider>,
}

impl McpServerTrait for ModelsServer {
    fn tools(&self) -> Vec<rmcp::ToolSpec> {
        vec![
            rmcp::ToolSpec {
                name: "list_models".to_string(),
                description: "List all loaded models and their status".to_string(),
                input_schema: schemars::schema_for!(()),
            },
            rmcp::ToolSpec {
                name: "get_model_info".to_string(),
                description: "Get detailed information about a specific model".to_string(),
                input_schema: schemars::schema_for!(String), // model_id
            },
            rmcp::ToolSpec {
                name: "get_inference_stats".to_string(),
                description: "Get inference performance statistics".to_string(),
                input_schema: schemars::schema_for!(()),
            },
        ]
    }
    
    async fn handle_tool(&self, name: &str, params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        match name {
            "list_models" => {
                let models = self.state.list_loaded_models().await?;
                Ok(serde_json::to_value(models)?)
            }
            "get_model_info" => {
                let model_id: String = serde_json::from_value(params)?;
                let info = self.state.get_model_info(&model_id).await?;
                Ok(serde_json::to_value(info)?)
            }
            "get_inference_stats" => {
                let stats = self.state.get_inference_statistics().await?;
                Ok(serde_json::to_value(stats)?)
            }
            _ => Err(anyhow::anyhow!("Unknown tool: {}", name))
        }
    }
}
```

## Data Models

### Enhanced Node Enum for Log Storage

```rust
// In common/src/models.rs - add LogEntry variant
#[derive(Serialize, Deserialize, Archive, rkyv::Deserialize, rkyv::Serialize, Debug, Clone)]
pub enum Node {
    Chat(Chat),
    Message(Message),
    Summary(Summary),
    Document(Document),
    User(User),
    LogEntry(LogEntry), // ← NEW: Store logs as nodes
}

// Extend DatabaseType for log storage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DatabaseType {
    Conversations,
    Documents,
    Users,
    Logs, // ← NEW: Dedicated log database
}
```

### MCP Tool Response Types

```rust
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct LogStats {
    pub total_logs: usize,
    pub by_level: HashMap<String, usize>,
    pub by_source: HashMap<String, usize>,
    pub by_context: HashMap<String, usize>,
    pub oldest_log: Option<DateTime<Utc>>,
    pub newest_log: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct DatabaseStats {
    pub total_nodes: usize,
    pub by_type: HashMap<String, usize>,
    pub total_edges: usize,
    pub total_embeddings: usize,
    pub database_size_bytes: u64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub model_type: String,
    pub loaded_at: DateTime<Utc>,
    pub memory_usage_bytes: u64,
    pub inference_count: u64,
    pub average_latency_ms: f64,
}
```

## Error Handling

### MCP-Specific Error Types

```rust
#[derive(thiserror::Error, Debug)]
pub enum McpError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("Invalid tool parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Storage error: {0}")]
    Storage(#[from] common::DbError),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("MCP protocol error: {0}")]
    Protocol(String),
}

// Convert to MCP protocol errors
impl From<McpError> for rmcp::Error {
    fn from(err: McpError) -> Self {
        match err {
            McpError::ToolNotFound(name) => rmcp::Error::MethodNotFound(name),
            McpError::InvalidParameters(msg) => rmcp::Error::InvalidParams(msg),
            _ => rmcp::Error::InternalError(err.to_string()),
        }
    }
}
```

## Testing Strategy

### Integration Testing with Real AI Assistants

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mcp_logs_integration() {
        // Set up test environment
        let state = create_test_app_state().await;
        let mcp_manager = McpManager::new(state.clone());
        
        // Store test logs
        let test_log = LogEntry::new(
            LogLevel::Info,
            "test_context".to_string(),
            "Test message".to_string(),
            LogSource::Extension,
        );
        state.store_log(test_log.clone()).await.unwrap();
        
        // Test MCP tool
        let params = serde_json::json!({
            "context": "test_context",
            "limit": 10
        });
        
        let result = mcp_manager
            .handle_tool("query_logs", params)
            .await
            .unwrap();
        
        let query_result: LogQueryResult = serde_json::from_value(result).unwrap();
        assert_eq!(query_result.count, 1);
        assert_eq!(query_result.logs[0].message, "Test message");
    }
    
    #[tokio::test]
    async fn test_mcp_database_integration() {
        let state = create_test_app_state().await;
        let mcp_manager = McpManager::new(state.clone());
        
        // Create test data
        let chat = create_test_chat();
        state.insert_node(&Node::Chat(chat.clone())).await.unwrap();
        
        // Test database search
        let params = serde_json::json!({
            "node_type": "chat",
            "limit": 10
        });
        
        let result = mcp_manager
            .handle_tool("search_nodes", params)
            .await
            .unwrap();
        
        // Verify results contain our test chat
        let search_results: Vec<Node> = serde_json::from_value(result).unwrap();
        assert!(!search_results.is_empty());
    }
}
```

### Performance Testing

```rust
#[cfg(test)]
mod performance_tests {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn bench_mcp_log_query(c: &mut Criterion) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let state = rt.block_on(create_test_app_state_with_logs(10000));
        let mcp_manager = McpManager::new(state);
        
        c.bench_function("mcp_query_logs_10k", |b| {
            b.to_async(&rt).iter(|| async {
                let params = serde_json::json!({
                    "level": "error",
                    "limit": 100
                });
                
                black_box(
                    mcp_manager
                        .handle_tool("query_logs", params)
                        .await
                        .unwrap()
                );
            });
        });
    }
    
    criterion_group!(benches, bench_mcp_log_query);
    criterion_main!(benches);
}
```

## Performance Considerations

### Zero-Copy Benefits with Storage Engine Migration

1. **Log Queries**: Direct memory access to rkyv-serialized log entries
2. **Database Searches**: Zero-copy node deserialization for large result sets
3. **Model Information**: Efficient access to cached model metadata

### Concurrent Access Patterns

1. **Read-Heavy Workload**: MCP tools primarily read data (logs, database, models)
2. **MVCC Benefits**: Multiple AI assistants can query simultaneously without blocking
3. **Async Tool Execution**: Non-blocking tool execution using tokio

### Memory Efficiency

```rust
// Zero-copy log query implementation
impl LogsServer {
    async fn query_logs_zero_copy(&self, query: LogQuery) -> anyhow::Result<LogQueryResult> {
        let storage = self.state.get_storage_manager(DatabaseType::Logs)?;
        
        // Use storage engine's zero-copy scan
        let mut results = Vec::new();
        for archived_entry in storage.scan_archived_logs(&query)? {
            // Direct access to archived data without deserialization
            if query.matches_archived(archived_entry) {
                // Only deserialize matching entries
                let entry = archived_entry.deserialize(&mut rkyv::Infallible)?;
                results.push(entry);
            }
        }
        
        Ok(LogQueryResult {
            count: results.len(),
            logs: results,
        })
    }
}
```

## Migration Path

### Phase 1: Basic MCP Integration (Week 1)
1. Create `tabagent-mcp` crate with rmcp dependencies
2. Implement MCP transport in server main.rs
3. Create basic MCP manager and logs server
4. Add MCP mode to CLI arguments and configuration

### Phase 2: Persistent Log Storage (Week 2)
1. Add LogEntry variant to Node enum
2. Extend AppState with log storage methods
3. Implement log persistence in storage engine
4. Create comprehensive logs MCP tools

### Phase 3: Database and Model Servers (Week 3)
1. Implement DatabaseServer with node/edge query tools
2. Implement ModelsServer with model information tools
3. Implement SystemServer with monitoring tools
4. Add comprehensive error handling and validation

### Phase 4: Integration and Testing (Week 4)
1. Integration testing with real AI assistants
2. Performance benchmarking and optimization
3. Documentation and examples
4. Production readiness validation

### Phase 5: Storage Engine Alignment (Week 5)
1. Ensure compatibility with libmdbx migration
2. Implement zero-copy optimizations
3. Performance validation with rkyv serialization
4. Final integration testing