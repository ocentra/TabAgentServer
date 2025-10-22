//! Model Loader - FFI bindings to llama.cpp/BitNet inference engines
//!
//! This crate provides safe Rust wrappers around the llama.cpp C API,
//! enabling efficient GGUF model loading and inference.

pub mod ffi;
pub mod model;
pub mod context;
pub mod error;

pub use error::{ModelError, Result};
pub use model::{Model, ModelConfig};
pub use context::{Context, GenerationParams};

/// Initialize the model loader library
pub fn init() -> Result<()> {
    log::info!("Initializing model-loader");
    // Any global initialization goes here
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }
}
