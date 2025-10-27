/// TabAgent Pipeline Crate
///
/// **High-level orchestration for specialized ML pipelines.**
///
/// # Architecture
///
/// This crate is a **composable layer** that builds on top of existing crates:
/// - **`model-cache`**: Detection, download, HF metadata
/// - **`model-loader`**: GGUF/BitNet FFI loading
/// - **`pipeline`**: Task-specific orchestration (Florence2, Whisper, etc.)
///
/// **Key Principle**: Don't duplicate detection/loading logic - compose existing crates!
///
/// # Design Principles
///
/// 1. **Composability** (Rule 1.3): Reuse `model-cache` and `model-loader`
/// 2. **No Duplication**: Don't reimplement detection/loading
/// 3. **Type-Safe Routing**: `PipelineType` enum → specialized implementation
/// 4. **Task-Specific**: Each pipeline adds specialized logic
///
/// # Example (Composable)
///
/// ```no_run
/// use tabagent_pipeline::{detect_from_repo_name, PipelineFactory};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // 1. Detect using model-cache (composed)
/// if let Some(model_info) = detect_from_repo_name("microsoft/Florence-2-large") {
///     // 2. Route to backend (composed - no duplication!)
///     let backend = PipelineFactory::route_backend(&model_info)?;
///     let pipeline_type = PipelineFactory::get_pipeline_type(&model_info);
///
///     // 3. Delegate to correct backend:
///     //    - Rust: model-loader::Model::load() for GGUF/BitNet
///     //    - Python: transformers_backend for ONNX/SafeTensors
/// }
///
/// // That's it! Pipeline crate just routes - doesn't reimplement.
/// # Ok(())
/// # }
/// ```

pub mod base;
pub mod error;
pub mod factory;
pub mod types;

// Architecture-specific handlers (separate files like extension)
pub mod florence2;
pub mod whisper;
pub mod text_generation;
pub mod clip;
pub mod janus;
pub mod multimodal;
pub mod onnx;
pub mod yolo;
pub mod segmentation;

// Re-export public API (thin layer)
pub use base::Pipeline;
pub use error::{PipelineError, Result};
pub use factory::PipelineFactory;
pub use types::{Architecture, PipelineType};

// Re-export architecture-specific handlers
pub use florence2::Florence2Handler;
pub use whisper::WhisperHandler;
pub use text_generation::TextGenerationHandler;
pub use clip::ClipHandler;
pub use janus::JanusHandler;
pub use multimodal::MultimodalHandler;
pub use onnx::OnnxPipeline;
pub use yolo::{YoloPipeline, YOLO_CLASS_LABELS};
pub use segmentation::{SegmentationPipeline, SegmentationType};

// Re-export model-cache types for convenience (DRY - don't duplicate!)
pub use tabagent_model_cache::{
    ModelType as CacheModelType,
    detect_from_repo_name,
    detect_from_file_path,
    ModelCache,
};

// Re-export detection types specifically
pub use tabagent_model_cache::detection::{
    ModelInfo as DetectionModelInfo,
    Backend as CacheBackend,
};

