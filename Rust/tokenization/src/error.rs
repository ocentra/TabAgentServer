use thiserror::Error;

pub type Result<T> = std::result::Result<T, TokenizationError>;

#[derive(Error, Debug)]
pub enum TokenizationError {
    #[error("Failed to load tokenizer: {0}")]
    LoadFailed(String),
    
    #[error("Failed to encode text: {0}")]
    EncodeFailed(String),
    
    #[error("Failed to decode tokens: {0}")]
    DecodeFailed(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

