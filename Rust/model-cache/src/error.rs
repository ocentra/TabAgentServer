use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModelCacheError {
    #[error("Sled database error: {0}")]
    Sled(#[from] sled::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    #[error("File not found in model: {0}")]
    FileNotFound(String),
    
    #[error("Manifest error: {0}")]
    Manifest(String),
    
    #[error("Download error: {0}")]
    Download(String),
    
    #[error("Invalid model URL: {0}")]
    InvalidUrl(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ModelCacheError>;

