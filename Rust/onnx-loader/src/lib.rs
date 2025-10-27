//! ONNX Runtime inference loader
//!
//! Provides high-level wrapper around ort crate with:
//! - Automatic execution provider selection (CUDA, TensorRT, DirectML, CPU)
//! - Integrated tokenization via tabagent-tokenization
//! - Hardware-aware provider configuration

pub mod error;
pub mod providers;
pub mod providers_bridge;
pub mod session;
pub mod text_generation;

pub use error::{OnnxError, Result};
pub use session::OnnxSession;
pub use providers_bridge::{bridge_to_ort, auto_select_providers};
pub use text_generation::{TextGenerator, GenerationConfig};

/// Re-export full `ort` crate for advanced usage
/// 
/// **Philosophy**: We don't limit you!
/// If `ort` can do it, you can do it through us.
/// 
/// Use our convenience API for common tasks,
/// OR use `ort` directly for full power (GPT-2, YOLOv8, async, training, etc.)
pub use ort;

/// Initialize ONNX Runtime environment
pub fn init() -> Result<()> {
    log::info!("Initializing ONNX Runtime loader");
    Ok(())
}

