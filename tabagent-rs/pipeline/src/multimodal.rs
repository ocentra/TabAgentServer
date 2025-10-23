/// Multimodal Pipeline Handler
///
/// Generic handler for multimodal models (text + vision).
/// Examples: Phi-3.5-vision, Llama-3.2-vision, Qwen2-VL

use crate::error::Result;
use tabagent_model_cache::detection::ModelInfo as DetectionModelInfo;

/// Generic multimodal handler
pub struct MultimodalHandler;

impl MultimodalHandler {
    pub fn new() -> Self {
        Self
    }
    
    pub fn validate_model(_model_info: &DetectionModelInfo) -> Result<()> {
        // Generic - no strict validation
        Ok(())
    }
    
    pub fn get_model_config_hints() -> Vec<(&'static str, &'static str)> {
        vec![
            ("task", "text-generation"),
            ("supports_vision", "true"),
            ("multimodal", "true"),
        ]
    }
}

