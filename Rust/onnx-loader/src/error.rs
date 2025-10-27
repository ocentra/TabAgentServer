use thiserror::Error;

pub type Result<T> = std::result::Result<T, OnnxError>;

#[derive(Error, Debug)]
pub enum OnnxError {
    #[error("Failed to create ONNX environment: {0}")]
    EnvironmentCreationFailed(String),
    
    #[error("Failed to load model: {0}")]
    ModelLoadFailed(String),
    
    #[error("Failed to create session: {0}")]
    SessionCreationFailed(String),
    
    #[error("Inference failed: {0}")]
    InferenceFailed(String),
    
    #[error("Tokenization error: {0}")]
    TokenizationError(#[from] tabagent_tokenization::TokenizationError),
    
    #[error("Tokenizer load failed: {0}")]
    TokenizerLoadFailed(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Model not loaded")]
    ModelNotLoaded,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

