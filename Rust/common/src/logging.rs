use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Source of a log entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum LogSource {
    Extension,        // Browser extension (JS/TS)
    NativeMessaging,  // Chrome native messaging bridge
    GrpcServer,       // gRPC API server
    WebRtc,           // WebRTC transport
    ModelExecution,   // LLM inference
    Storage,          // Database operations
    Pipeline,         // Processing pipeline
    Indexing,         // Document indexing
    Query,            // Query engine
    Unknown,          // Fallback
}

impl Default for LogSource {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<&str> for LogSource {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "extension" => Self::Extension,
            "nativemessaging" | "native" => Self::NativeMessaging,
            "grpcserver" | "grpc" => Self::GrpcServer,
            "webrtc" => Self::WebRtc,
            "modelexecution" | "model" => Self::ModelExecution,
            "storage" | "db" => Self::Storage,
            "pipeline" => Self::Pipeline,
            "indexing" => Self::Indexing,
            "query" => Self::Query,
            _ => Self::Unknown,
        }
    }
}

/// Log level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Log,
    Info,
    Warn,
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl From<&str> for LogLevel {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "debug" => Self::Debug,
            "log" => Self::Log,
            "info" => Self::Info,
            "warn" | "warning" => Self::Warn,
            "error" => Self::Error,
            _ => Self::Log,
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Debug => write!(f, "debug"),
            Self::Log => write!(f, "log"),
            Self::Info => write!(f, "info"),
            Self::Warn => write!(f, "warn"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// A single log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub context: String,
    pub message: String,
    pub source: LogSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(level: LogLevel, context: String, message: String, source: LogSource) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            context,
            message,
            source,
            data: None,
        }
    }

    /// Create with additional data
    pub fn with_data(
        level: LogLevel,
        context: String,
        message: String,
        source: LogSource,
        data: serde_json::Value,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            context,
            message,
            source,
            data: Some(data),
        }
    }
}

/// Query filters for logs
#[derive(Debug, Clone, Default, Deserialize)]
pub struct LogQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// Response containing filtered logs
#[derive(Debug, Serialize)]
pub struct LogQueryResult {
    pub count: usize,
    pub logs: Vec<LogEntry>,
}

impl LogQueryResult {
    pub fn empty() -> Self {
        Self {
            count: 0,
            logs: Vec::new(),
        }
    }
}

/// MCP Resource definition
#[derive(Debug, Clone, Serialize)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
}

/// MCP Resource list response
#[derive(Debug, Serialize)]
pub struct McpResourceList {
    pub resources: Vec<McpResource>,
}

