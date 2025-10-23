/// CLIP Pipeline Handler
///
/// Architecture-specific logic for CLIP dual-encoder models.
/// Handles embeddings and zero-shot classification.

use crate::error::{Result, PipelineError};
use tabagent_model_cache::detection::ModelInfo as DetectionModelInfo;

/// CLIP pipeline handler
///
/// Handles CLIP-specific:
/// - Dual-encoder architecture (text + vision)
/// - Embedding generation
/// - Zero-shot classification
pub struct ClipHandler;

impl ClipHandler {
    /// Create new CLIP handler
    pub fn new() -> Self {
        Self
    }
    
    /// Validate CLIP model info
    pub fn validate_model(model_info: &DetectionModelInfo) -> Result<()> {
        if let Some(arch) = &model_info.architecture {
            let arch_lower = arch.to_lowercase();
            if arch_lower != "clip" && arch_lower != "clap" {
                return Err(PipelineError::InvalidArchitecture(
                    format!("Expected CLIP, got: {}", arch)
                ));
            }
        } else {
            return Err(PipelineError::InvalidArchitecture(
                "No architecture specified for CLIP".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Get model-specific configuration hints
    pub fn get_model_config_hints() -> Vec<(&'static str, &'static str)> {
        vec![
            ("task", "feature-extraction"),
            ("supports_vision", "true"),
            ("supports_text", "true"),
            ("output_type", "embeddings"),
        ]
    }
}

