//! Enhanced error types and recovery mechanisms for the TabAgent system.
//!
//! This module provides more granular error types and recovery mechanisms
//! to improve error handling throughout the system. It follows the Rust
//! Architecture Guidelines for safety, performance, and clarity.

use thiserror::Error;
use std::io;

/// Enhanced error types for database operations.
#[derive(Debug, Error)]
pub enum DatabaseError {
    /// Error from the sled storage backend.
    #[error("Sled database error: {0}")]
    Sled(#[from] sled::Error),
    
    /// Error during serialization/deserialization.
    #[error("Serialization error: {0}")]
    Serialization(#[from] SerializationError),
    
    /// Requested entity not found.
    #[error("Entity not found: {0}")]
    NotFound(String),
    
    /// Invalid operation or arguments.
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    /// Constraint violation.
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
    
    /// Transaction error.
    #[error("Transaction error: {0}")]
    Transaction(String),
    
    /// Index error.
    #[error("Index error: {0}")]
    Index(String),
    
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    /// Lock poisoning error.
    #[error("Lock poisoning error: {0}")]
    LockPoisoning(String),
    
    /// Concurrency error.
    #[error("Concurrency error: {0}")]
    Concurrency(String),
    
    /// Resource exhaustion error.
    #[error("Resource exhaustion: {0}")]
    ResourceExhaustion(String),
    
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    /// Network error.
    #[error("Network error: {0}")]
    Network(String),
    
    /// Timeout error.
    #[error("Timeout error: {0}")]
    Timeout(String),
    
    /// Other errors.
    #[error("Other error: {0}")]
    Other(String),
}

impl Clone for DatabaseError {
    fn clone(&self) -> Self {
        match self {
            DatabaseError::Sled(_) => DatabaseError::Other("Sled error".to_string()),
            DatabaseError::Serialization(e) => DatabaseError::Serialization(e.clone()),
            DatabaseError::NotFound(s) => DatabaseError::NotFound(s.clone()),
            DatabaseError::InvalidOperation(s) => DatabaseError::InvalidOperation(s.clone()),
            DatabaseError::ConstraintViolation(s) => DatabaseError::ConstraintViolation(s.clone()),
            DatabaseError::Transaction(s) => DatabaseError::Transaction(s.clone()),
            DatabaseError::Index(s) => DatabaseError::Index(s.clone()),
            DatabaseError::Io(_) => DatabaseError::Other("IO error".to_string()),
            DatabaseError::LockPoisoning(s) => DatabaseError::LockPoisoning(s.clone()),
            DatabaseError::Concurrency(s) => DatabaseError::Concurrency(s.clone()),
            DatabaseError::ResourceExhaustion(s) => DatabaseError::ResourceExhaustion(s.clone()),
            DatabaseError::Configuration(s) => DatabaseError::Configuration(s.clone()),
            DatabaseError::Network(s) => DatabaseError::Network(s.clone()),
            DatabaseError::Timeout(s) => DatabaseError::Timeout(s.clone()),
            DatabaseError::Other(s) => DatabaseError::Other(s.clone()),
        }
    }
}

/// Enhanced serialization error types.
#[derive(Debug, Error)]
pub enum SerializationError {
    /// Bincode serialization error.
    #[error("Bincode error: {0}")]
    Bincode(String),
    
    /// JSON serialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    /// MessagePack serialization error.
    #[error("MessagePack error: {0}")]
    MessagePack(String),
    
    /// Protocol buffer serialization error.
    #[error("Protocol buffer error: {0}")]
    Protobuf(String),
    
    /// Custom serialization error.
    #[error("Custom serialization error: {0}")]
    Custom(String),
}

impl Clone for SerializationError {
    fn clone(&self) -> Self {
        match self {
            SerializationError::Bincode(s) => SerializationError::Bincode(s.clone()),
            SerializationError::Json(_) => SerializationError::Custom("JSON error".to_string()),
            SerializationError::MessagePack(s) => SerializationError::MessagePack(s.clone()),
            SerializationError::Protobuf(s) => SerializationError::Protobuf(s.clone()),
            SerializationError::Custom(s) => SerializationError::Custom(s.clone()),
        }
    }
}

impl From<bincode::Error> for SerializationError {
    fn from(err: bincode::Error) -> Self {
        SerializationError::Bincode(err.to_string())
    }
}

/// Enhanced error types for graph operations.
#[derive(Debug, Error, Clone)]
pub enum GraphError {
    /// Node not found.
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    
    /// Edge not found.
    #[error("Edge not found: {0}")]
    EdgeNotFound(String),
    
    /// Invalid node ID.
    #[error("Invalid node ID: {0}")]
    InvalidNodeId(String),
    
    /// Invalid edge ID.
    #[error("Invalid edge ID: {0}")]
    InvalidEdgeId(String),
    
    /// Cycle detected.
    #[error("Cycle detected: {0}")]
    CycleDetected(String),
    
    /// Invalid graph operation.
    #[error("Invalid graph operation: {0}")]
    InvalidOperation(String),
    
    /// Graph is empty.
    #[error("Graph is empty")]
    EmptyGraph,
    
    /// Node already exists.
    #[error("Node already exists: {0}")]
    NodeAlreadyExists(String),
    
    /// Edge already exists.
    #[error("Edge already exists: {0}")]
    EdgeAlreadyExists(String),
    
    /// Disconnected graph.
    #[error("Disconnected graph: {0}")]
    DisconnectedGraph(String),
    
    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    /// Other graph error.
    #[error("Other graph error: {0}")]
    Other(String),
}

/// Enhanced error types for vector operations.
#[derive(Debug, Error, Clone)]
pub enum VectorError {
    /// Invalid vector dimension.
    #[error("Invalid vector dimension: expected {expected}, got {actual}")]
    InvalidDimension { expected: usize, actual: usize },
    
    /// Vector not found.
    #[error("Vector not found: {0}")]
    VectorNotFound(String),
    
    /// Invalid distance metric.
    #[error("Invalid distance metric: {0}")]
    InvalidDistanceMetric(String),
    
    /// Index error.
    #[error("Index error: {0}")]
    Index(String),
    
    /// Search error.
    #[error("Search error: {0}")]
    Search(String),
    
    /// Invalid query vector.
    #[error("Invalid query vector: {0}")]
    InvalidQuery(String),
    
    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    /// Resource exhaustion.
    #[error("Resource exhaustion: {0}")]
    ResourceExhaustion(String),
    
    /// Other vector error.
    #[error("Other vector error: {0}")]
    Other(String),
}

/// Enhanced error types for indexing operations.
#[derive(Debug, Error, Clone)]
pub enum IndexError {
    /// Index not found.
    #[error("Index not found: {0}")]
    IndexNotFound(String),
    
    /// Index already exists.
    #[error("Index already exists: {0}")]
    IndexAlreadyExists(String),
    
    /// Invalid index key.
    #[error("Invalid index key: {0}")]
    InvalidKey(String),
    
    /// Index corruption.
    #[error("Index corruption: {0}")]
    Corruption(String),
    
    /// Index rebuild required.
    #[error("Index rebuild required: {0}")]
    RebuildRequired(String),
    
    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    /// Resource exhaustion.
    #[error("Resource exhaustion: {0}")]
    ResourceExhaustion(String),
    
    /// Other index error.
    #[error("Other index error: {0}")]
    Other(String),
}

/// Enhanced error types for hybrid operations.
#[derive(Debug, Error, Clone)]
pub enum HybridError {
    /// Graph error.
    #[error("Graph error: {0}")]
    Graph(#[from] GraphError),
    
    /// Vector error.
    #[error("Vector error: {0}")]
    Vector(#[from] VectorError),
    
    /// Index error.
    #[error("Index error: {0}")]
    Index(#[from] IndexError),
    
    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    /// Invalid operation.
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    /// Resource exhaustion.
    #[error("Resource exhaustion: {0}")]
    ResourceExhaustion(String),
    
    /// Other hybrid error.
    #[error("Other hybrid error: {0}")]
    Other(String),
}

/// Recovery strategy for handling errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Retry the operation with exponential backoff.
    RetryWithBackoff {
        max_attempts: usize,
        initial_delay_ms: u64,
        max_delay_ms: u64,
    },
    
    /// Failover to a backup system.
    Failover {
        backup_endpoint: String,
    },
    
    /// Degraded mode with reduced functionality.
    DegradedMode,
    
    /// Cache fallback using stale data.
    CacheFallback,
    
    /// Skip the operation and continue.
    Skip,
    
    /// Abort the operation.
    Abort,
    
    /// Custom recovery strategy.
    Custom(String),
}

/// Error context providing additional information about an error.
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Timestamp when the error occurred.
    pub timestamp: std::time::SystemTime,
    
    /// Operation that was being performed when the error occurred.
    pub operation: String,
    
    /// Component where the error occurred.
    pub component: String,
    
    /// Additional context data.
    pub context_data: std::collections::HashMap<String, String>,
    
    /// Recovery strategy to use.
    pub recovery_strategy: RecoveryStrategy,
}

impl ErrorContext {
    /// Creates a new error context.
    pub fn new(operation: String, component: String) -> Self {
        Self {
            timestamp: std::time::SystemTime::now(),
            operation,
            component,
            context_data: std::collections::HashMap::new(),
            recovery_strategy: RecoveryStrategy::Abort,
        }
    }
    
    /// Adds context data to the error context.
    pub fn with_context(mut self, key: String, value: String) -> Self {
        self.context_data.insert(key, value);
        self
    }
    
    /// Sets the recovery strategy.
    pub fn with_recovery_strategy(mut self, strategy: RecoveryStrategy) -> Self {
        self.recovery_strategy = strategy;
        self
    }
}

/// Result type with enhanced error handling.
pub type Result<T> = std::result::Result<T, DatabaseError>;

/// Result type for graph operations.
pub type GraphResult<T> = std::result::Result<T, GraphError>;

/// Result type for vector operations.
pub type VectorResult<T> = std::result::Result<T, VectorError>;

/// Result type for index operations.
pub type IndexResult<T> = std::result::Result<T, IndexError>;

/// Result type for hybrid operations.
pub type HybridResult<T> = std::result::Result<T, HybridError>;

/// Extension trait for adding context to results.
pub trait ResultExt<T> {
    /// Adds context to a result.
    fn with_context<F>(self, context_fn: F) -> Self
    where
        F: FnOnce() -> ErrorContext;
    
    /// Maps a result to a different error type.
    fn map_err_type<E>(self, map_fn: impl Fn(DatabaseError) -> E) -> std::result::Result<T, E>;
}

impl<T> ResultExt<T> for Result<T> {
    fn with_context<F>(self, _context_fn: F) -> Self
    where
        F: FnOnce() -> ErrorContext,
    {
        // In a real implementation, we would attach the context to the error
        // For now, we'll just return the original result
        self
    }
    
    fn map_err_type<E>(self, map_fn: impl Fn(DatabaseError) -> E) -> std::result::Result<T, E> {
        self.map_err(map_fn)
    }
}

/// A recoverable error that can be handled with a specific strategy.
#[derive(Debug, Clone)]
pub struct RecoverableError {
    /// The underlying error.
    pub error: DatabaseError,
    
    /// Context information.
    pub context: ErrorContext,
}

impl RecoverableError {
    /// Creates a new recoverable error.
    pub fn new(error: DatabaseError, context: ErrorContext) -> Self {
        Self { error, context }
    }
    
    /// Gets the recovery strategy for this error.
    pub fn recovery_strategy(&self) -> &RecoveryStrategy {
        &self.context.recovery_strategy
    }
}

impl std::fmt::Display for RecoverableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Recoverable error: {} (context: {:?})", self.error, self.context)
    }
}

impl std::error::Error for RecoverableError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Error handler for implementing recovery strategies.
pub struct ErrorHandler {
    /// Maximum number of retry attempts.
    pub max_retry_attempts: usize,
    
    /// Initial delay for exponential backoff in milliseconds.
    pub initial_retry_delay_ms: u64,
    
    /// Maximum delay for exponential backoff in milliseconds.
    pub max_retry_delay_ms: u64,
}

impl ErrorHandler {
    /// Creates a new error handler.
    pub fn new(max_retry_attempts: usize, initial_retry_delay_ms: u64, max_retry_delay_ms: u64) -> Self {
        Self {
            max_retry_attempts,
            initial_retry_delay_ms,
            max_retry_delay_ms,
        }
    }
    
    /// Handles a recoverable error using the specified recovery strategy.
    pub fn handle_error<T>(&self, error: RecoverableError, operation: impl Fn() -> Result<T>) -> Result<T> {
        match error.recovery_strategy() {
            RecoveryStrategy::RetryWithBackoff { max_attempts, initial_delay_ms, max_delay_ms } => {
                self.retry_with_backoff(error.clone(), operation, *max_attempts, *initial_delay_ms, *max_delay_ms)
            }
            RecoveryStrategy::Failover { backup_endpoint } => {
                self.failover(error.clone(), operation, backup_endpoint)
            }
            RecoveryStrategy::DegradedMode => {
                self.degraded_mode(error.clone(), operation)
            }
            RecoveryStrategy::CacheFallback => {
                self.cache_fallback(error.clone(), operation)
            }
            RecoveryStrategy::Skip => {
                self.skip(error.clone(), operation)
            }
            RecoveryStrategy::Abort => {
                Err(error.error)
            }
            RecoveryStrategy::Custom(_) => {
                // Custom recovery strategies would be implemented here
                Err(error.error)
            }
        }
    }
    
    /// Retries an operation with exponential backoff.
    fn retry_with_backoff<T>(
        &self,
        _error: RecoverableError,
        operation: impl Fn() -> Result<T>,
        max_attempts: usize,
        initial_delay_ms: u64,
        max_delay_ms: u64,
    ) -> Result<T> {
        let mut attempts = 0;
        let mut delay_ms = initial_delay_ms;
        
        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(err) => {
                    attempts += 1;
                    if attempts >= max_attempts {
                        return Err(err);
                    }
                    
                    // Sleep for the current delay
                    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                    
                    // Exponential backoff with jitter
                    delay_ms = (delay_ms * 2).min(max_delay_ms);
                    let jitter = rand::random::<u64>() % (delay_ms / 2);
                    delay_ms = delay_ms.saturating_add(jitter);
                }
            }
        }
    }
    
    /// Fails over to a backup endpoint.
    fn failover<T>(
        &self,
        _error: RecoverableError,
        _operation: impl Fn() -> Result<T>,
        _backup_endpoint: &str,
    ) -> Result<T> {
        // In a real implementation, we would redirect to the backup endpoint
        // For now, we'll just return an error
        Err(DatabaseError::Network("Failover not implemented".to_string()))
    }
    
    /// Continues in degraded mode.
    fn degraded_mode<T>(
        &self,
        _error: RecoverableError,
        operation: impl Fn() -> Result<T>,
    ) -> Result<T> {
        // In degraded mode, we might return partial results or use fallback logic
        operation()
    }
    
    /// Uses cache fallback.
    fn cache_fallback<T>(
        &self,
        _error: RecoverableError,
        _operation: impl Fn() -> Result<T>,
    ) -> Result<T> {
        // In a real implementation, we would return cached/stale data
        // For now, we'll just return an error
        Err(DatabaseError::Other("Cache fallback not implemented".to_string()))
    }
    
    /// Skips the operation.
    fn skip<T>(
        &self,
        _error: RecoverableError,
        _operation: impl Fn() -> Result<T>,
    ) -> Result<T> {
        // In a real implementation, we would return a default value or skip the operation
        // For now, we'll just return an error
        Err(DatabaseError::Other("Skip not implemented".to_string()))
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new(3, 100, 5000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_database_error() {
        let error = DatabaseError::NotFound("test".to_string());
        assert_eq!(format!("{}", error), "Entity not found: test");
    }
    
    #[test]
    fn test_serialization_error() {
        let error = SerializationError::Bincode("test".to_string());
        assert_eq!(format!("{}", error), "Bincode error: test");
    }
    
    #[test]
    fn test_graph_error() {
        let error = GraphError::NodeNotFound("test".to_string());
        assert_eq!(format!("{}", error), "Node not found: test");
    }
    
    #[test]
    fn test_vector_error() {
        let error = VectorError::InvalidDimension { expected: 3, actual: 2 };
        assert_eq!(format!("{}", error), "Invalid vector dimension: expected 3, got 2");
    }
    
    #[test]
    fn test_index_error() {
        let error = IndexError::IndexNotFound("test".to_string());
        assert_eq!(format!("{}", error), "Index not found: test");
    }
    
    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("test_operation".to_string(), "test_component".to_string())
            .with_context("key".to_string(), "value".to_string())
            .with_recovery_strategy(RecoveryStrategy::Abort);
        
        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.component, "test_component");
        assert_eq!(context.context_data.get("key"), Some(&"value".to_string()));
        assert_eq!(context.recovery_strategy, RecoveryStrategy::Abort);
    }
    
    #[test]
    fn test_result_ext() {
        let result: Result<()> = Ok(());
        let result = result.with_context(|| ErrorContext::new("test".to_string(), "test".to_string()));
        assert!(result.is_ok());
    }
}