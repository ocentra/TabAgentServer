/// Pipeline type enum - NO string literals!
///
/// Represents all supported ML task types.
/// Rule 13.5: Use enums, not strings, for domain-specific values.
///
/// NOTE: This maps detected ModelType + task string → specialized pipeline.
/// We don't duplicate ModelType/Backend from model-cache; we build on top of it.
use serde::{Deserialize, Serialize};
use std::fmt;
use tabagent_model_cache::ModelType as CacheModelType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PipelineType {
    /// Text generation (LLMs, GPT, Llama, etc.)
    #[serde(rename = "text-generation")]
    TextGeneration,

    /// Image to text (Florence2, BLIP, etc.)
    #[serde(rename = "image-to-text")]
    ImageToText,

    /// Feature extraction (CLIP, sentence-transformers)
    #[serde(rename = "feature-extraction")]
    FeatureExtraction,

    /// Automatic speech recognition (Whisper)
    #[serde(rename = "automatic-speech-recognition")]
    AutomaticSpeechRecognition,

    /// Text to speech
    #[serde(rename = "text-to-speech")]
    TextToSpeech,

    /// Zero-shot image classification
    #[serde(rename = "zero-shot-image-classification")]
    ZeroShotImageClassification,

    /// Image classification
    #[serde(rename = "image-classification")]
    ImageClassification,

    /// Object detection
    #[serde(rename = "object-detection")]
    ObjectDetection,

    /// Depth estimation
    #[serde(rename = "depth-estimation")]
    DepthEstimation,

    /// Embedding generation
    #[serde(rename = "embedding")]
    Embedding,
}

impl PipelineType {
    /// Detect pipeline type from model-cache ModelInfo
    ///
    /// This is the bridge between model-cache detection and pipeline routing.
    pub fn from_model_info(model_type: &CacheModelType, task: Option<&str>) -> Self {
        // Use task if available (authoritative)
        if let Some(task_str) = task {
            if let Some(pipeline_type) = Self::from_hf_tag(task_str) {
                return pipeline_type;
            }
        }

        // Fallback to model type defaults
        match model_type {
            CacheModelType::GGUF | CacheModelType::BitNet | CacheModelType::SafeTensors => {
                Self::TextGeneration
            }
            CacheModelType::ONNX => {
                // Could be anything - check task or default to text-gen
                task.and_then(Self::from_hf_tag)
                    .unwrap_or(Self::TextGeneration)
            }
            CacheModelType::LiteRT => Self::TextGeneration,
            CacheModelType::Unknown => Self::TextGeneration,
        }
    }

    /// Convert to HuggingFace pipeline_tag string (for API compatibility)
    pub fn to_hf_tag(&self) -> &'static str {
        match self {
            Self::TextGeneration => "text-generation",
            Self::ImageToText => "image-to-text",
            Self::FeatureExtraction => "feature-extraction",
            Self::AutomaticSpeechRecognition => "automatic-speech-recognition",
            Self::TextToSpeech => "text-to-speech",
            Self::ZeroShotImageClassification => "zero-shot-image-classification",
            Self::ImageClassification => "image-classification",
            Self::ObjectDetection => "object-detection",
            Self::DepthEstimation => "depth-estimation",
            Self::Embedding => "embedding",
        }
    }

    /// Parse from HuggingFace pipeline_tag string
    pub fn from_hf_tag(tag: &str) -> Option<Self> {
        match tag {
            "text-generation" => Some(Self::TextGeneration),
            "image-to-text" => Some(Self::ImageToText),
            "feature-extraction" => Some(Self::FeatureExtraction),
            "automatic-speech-recognition" => Some(Self::AutomaticSpeechRecognition),
            "text-to-speech" => Some(Self::TextToSpeech),
            "zero-shot-image-classification" => Some(Self::ZeroShotImageClassification),
            "image-classification" => Some(Self::ImageClassification),
            "object-detection" => Some(Self::ObjectDetection),
            "depth-estimation" => Some(Self::DepthEstimation),
            "embedding" => Some(Self::Embedding),
            _ => None,
        }
    }

    /// Returns true if this pipeline requires specialized handling
    pub fn is_specialized(&self) -> bool {
        !matches!(self, Self::TextGeneration | Self::FeatureExtraction)
    }
}

impl fmt::Display for PipelineType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hf_tag())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_type_serialization() {
        let pt = PipelineType::ImageToText;
        let json = serde_json::to_string(&pt).expect("Serialization failed");
        assert_eq!(json, r#""image-to-text""#);

        let deserialized: PipelineType = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(deserialized, pt);
    }

    #[test]
    fn test_hf_tag_conversion() {
        assert_eq!(PipelineType::TextGeneration.to_hf_tag(), "text-generation");
        assert_eq!(
            PipelineType::from_hf_tag("image-to-text"),
            Some(PipelineType::ImageToText)
        );
        assert_eq!(PipelineType::from_hf_tag("invalid"), None);
    }

    #[test]
    fn test_specialized_detection() {
        assert!(!PipelineType::TextGeneration.is_specialized());
        assert!(PipelineType::ImageToText.is_specialized());
        assert!(PipelineType::AutomaticSpeechRecognition.is_specialized());
    }
}

