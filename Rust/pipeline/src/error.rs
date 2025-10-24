/// Pipeline errors
///
/// Represents all possible failure modes for pipeline operations.
use thiserror::Error;
use crate::types::Architecture;

#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("Pipeline type not supported: {0}")]
    UnsupportedPipelineType(String),

    #[error("Model not loaded")]
    ModelNotLoaded,

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Invalid architecture: expected {expected}, got {actual}")]
    InvalidArchitecture {
        expected: Architecture,
        actual: Architecture,
    },

    #[error("Inference failed: {0}")]
    InferenceFailed(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, PipelineError>;

