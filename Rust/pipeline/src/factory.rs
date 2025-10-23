/// Pipeline factory - Thin routing layer
///
/// **Composable**: Routes PipelineType → correct backend (Rust or Python).
/// Does NOT reimplement loading/inference - just delegates.
use crate::error::Result;
use crate::types::PipelineType;
use tabagent_model_cache::detection::{ModelInfo as DetectionModelInfo, Backend};

/// Factory for routing pipelines
///
/// **Thin router**: Maps (ModelType, task) → correct backend implementation.
/// Actual loading delegated to model-cache/model-loader.
pub struct PipelineFactory;

impl PipelineFactory {
    /// Determine which backend should handle this model
    ///
    /// **Routes to**:
    /// - Rust: GGUF/BitNet via model-loader FFI
    /// - Python: ONNX/SafeTensors via transformers_backend
    ///
    /// # Arguments
    /// * `model_info` - Detection result from model-cache
    ///
    /// # Returns
    /// Backend routing decision (Rust/Python engine name)
    pub fn route_backend(model_info: &DetectionModelInfo) -> Result<Backend> {
        // Just return the backend that model-cache detected!
        // We don't duplicate routing logic - model-cache already did it.
        Ok(model_info.backend.clone())
    }

    /// Get pipeline type for routing specialized logic
    ///
    /// # Arguments
    /// * `model_info` - Detection result from model-cache
    ///
    /// # Returns
    /// PipelineType for task-specific handling
    pub fn get_pipeline_type(model_info: &DetectionModelInfo) -> PipelineType {
        PipelineType::from_model_info(&model_info.model_type, model_info.task.as_deref())
    }

    /// Check if a pipeline type is supported
    pub fn is_supported(pipeline_type: PipelineType) -> bool {
        // TODO: Maintain a registry of supported pipelines
        // For now, return false - will be updated when Python bindings are added
        matches!(
            pipeline_type,
            PipelineType::TextGeneration
                | PipelineType::ImageToText
                | PipelineType::AutomaticSpeechRecognition
                | PipelineType::FeatureExtraction
        )
    }

    /// Get list of all supported pipeline types
    pub fn supported_types() -> Vec<PipelineType> {
        vec![
            PipelineType::TextGeneration,
            PipelineType::ImageToText,
            PipelineType::AutomaticSpeechRecognition,
            PipelineType::FeatureExtraction,
        ]
    }
}

// Tests moved to tests/pipeline_tests.rs

