/// Janus Pipeline Handler
///
/// Architecture-specific logic for Janus multimodal models.

use crate::error::{Result, PipelineError};
use tabagent_model_cache::detection::ModelInfo as DetectionModelInfo;

/// Janus pipeline handler
pub struct JanusHandler;

impl JanusHandler {
    pub fn new() -> Self {
        Self
    }
    
    pub fn validate_model(model_info: &DetectionModelInfo) -> Result<()> {
        if let Some(arch) = &model_info.architecture {
            if arch.to_lowercase() != "janus" {
                return Err(PipelineError::InvalidArchitecture(
                    format!("Expected Janus, got: {}", arch)
                ));
            }
        } else {
            return Err(PipelineError::InvalidArchitecture(
                "No architecture specified for Janus".to_string()
            ));
        }
        Ok(())
    }
    
    pub fn get_model_config_hints() -> Vec<(&'static str, &'static str)> {
        vec![
            ("task", "image-to-text"),
            ("supports_vision", "true"),
            ("architecture", "janus"),
        ]
    }
}

