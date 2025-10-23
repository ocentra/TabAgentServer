//! Error types for model loading and inference

use thiserror::Error;

pub type Result<T> = std::result::Result<T, ModelError>;

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Failed to load library: {0}")]
    LibraryLoadError(String),

    #[error("Failed to load model: {0}")]
    ModelLoadError(String),

    #[error("Failed to create context: {0}")]
    ContextCreationError(String),

    #[error("Inference failed: {0}")]
    InferenceError(String),

    #[error("Invalid model path: {0}")]
    InvalidPath(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Hardware detection failed: {0}")]
    HardwareError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("FFI error: {0}")]
    FfiError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

