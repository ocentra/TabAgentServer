/// Florence2 Pipeline Handler
///
/// Architecture-specific logic for Florence2 vision models.
/// Handles Florence2 special tokens and multimodal processing.

use crate::error::{Result, PipelineError};
use tabagent_model_cache::detection::ModelInfo as DetectionModelInfo;

/// Florence2 pipeline handler
///
/// Handles Florence2-specific:
/// - Special tokens (<OCR>, <CAPTION>, <OD>, etc.)
/// - Image preprocessing
/// - Multimodal inputs (text + image)
pub struct Florence2Handler;

impl Florence2Handler {
    /// Create new Florence2 handler
    pub fn new() -> Self {
        Self
    }
    
    /// Validate Florence2 model info
    ///
    /// Ensures model has required components for Florence2
    pub fn validate_model(model_info: &DetectionModelInfo) -> Result<()> {
        // Check if Florence2 architecture
        if let Some(arch) = &model_info.architecture {
            if arch.to_lowercase() != "florence2" && arch.to_lowercase() != "florence" {
                return Err(PipelineError::InvalidArchitecture(
                    format!("Expected Florence2, got: {}", arch)
                ));
            }
        } else {
            return Err(PipelineError::InvalidArchitecture(
                "No architecture specified for Florence2".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Preprocess Florence2 prompt
    ///
    /// Handles special tokens and formatting
    pub fn preprocess_prompt(prompt: &str) -> String {
        // Florence2 special tokens:
        // <OD>: Object detection
        // <CAPTION>: Dense captioning
        // <DETAILED_CAPTION>: Detailed image description
        // <MORE_DETAILED_CAPTION>: Very detailed description
        // <OCR>: Optical character recognition
        // <OCR_WITH_REGION>: OCR with bounding boxes
        
        // If prompt doesn't start with special token, default to <CAPTION>
        if prompt.starts_with('<') {
            prompt.to_string()
        } else {
            format!("<CAPTION> {}", prompt)
        }
    }
    
    /// Get model-specific configuration hints
    ///
    /// Returns recommended settings for Florence2 models
    pub fn get_model_config_hints() -> Vec<(&'static str, &'static str)> {
        vec![
            ("context_size", "2048"),
            ("supports_vision", "true"),
            ("requires_processor", "true"),
            ("special_tokens", "<OD>,<CAPTION>,<OCR>"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_preprocess_prompt() {
        // With special token
        assert_eq!(
            Florence2Handler::preprocess_prompt("<OCR>"),
            "<OCR>"
        );
        
        // Without special token - add default
        assert_eq!(
            Florence2Handler::preprocess_prompt("Describe this image"),
            "<CAPTION> Describe this image"
        );
    }
}

