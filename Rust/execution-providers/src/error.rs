use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Errors that can occur when working with execution providers
#[derive(Debug, Error, Serialize, Deserialize)]
#[serde(tag = "type", content = "message")]
pub enum ProviderError {
    #[error("Provider not available: {0}")]
    NotAvailable(String),
    
    #[error("Unsupported provider: {0}")]
    Unsupported(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Registration failed: {0}")]
    Registration(String),
    
    #[error("Hardware detection failed: {0}")]
    Hardware(String),
}

pub type Result<T> = std::result::Result<T, ProviderError>;

