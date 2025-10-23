/// Generic Text Generation Pipeline Handler
///
/// For standard LLMs without special architecture requirements.
/// Supports: Llama, Mistral, Qwen, Phi, etc.

use crate::error::Result;
use tabagent_model_cache::detection::ModelInfo as DetectionModelInfo;

/// Generic text generation handler
///
/// Used for standard LLMs that don't need specialized preprocessing.
pub struct TextGenerationHandler;

impl TextGenerationHandler {
    /// Create new text generation handler
    pub fn new() -> Self {
        Self
    }
    
    /// Validate model info (generic - accepts any text-generation model)
    pub fn validate_model(_model_info: &DetectionModelInfo) -> Result<()> {
        // Generic handler - no specific validation needed
        Ok(())
    }
    
    /// Get model-specific configuration hints
    pub fn get_model_config_hints() -> Vec<(&'static str, &'static str)> {
        vec![
            ("context_size", "4096"),
            ("task", "text-generation"),
            ("streaming", "true"),
        ]
    }
}

