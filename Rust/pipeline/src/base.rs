/// Base pipeline trait - Thin unification layer
///
/// **Composable**: Pipelines delegate to model-cache/model-loader, don't reimplement.
/// This trait just defines the unified interface.
use crate::error::Result;
use crate::types::PipelineType;
use serde_json::Value;

/// Base trait that all pipelines must implement
///
/// **Thin interface**: Actual loading/inference delegated to:
/// - `model-cache` for detection, download
/// - `model-loader` for GGUF/BitNet loading
/// - Python backends for ONNX/SafeTensors
pub trait Pipeline: Send + Sync {
    /// Get the pipeline type (for routing)
    fn pipeline_type(&self) -> PipelineType;

    /// Check if the model is loaded
    fn is_loaded(&self) -> bool;

    /// Load the model
    ///
    /// **Delegates to**: model-cache (download) → model-loader (FFI) or Python
    fn load(&mut self, model_id: &str, options: Option<Value>) -> Result<()>;

    /// Generate/infer output
    ///
    /// **Delegates to**: Underlying backend (Rust FFI or Python)
    fn generate(&self, input: Value) -> Result<Value>;

    /// Unload the model to free resources
    fn unload(&mut self) -> Result<()>;
}

// No tests needed - this is just a trait definition

