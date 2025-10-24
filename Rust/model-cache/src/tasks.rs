/// Task and engine constants for HuggingFace compatibility
///
/// These constants define the string values used by HuggingFace's pipeline_tag
/// field and backend engine names. They're kept as constants (not enums) at this
/// layer because:
/// 1. They come from external HF API as strings
/// 2. model-cache is low-level and shouldn't depend on pipeline crate
/// 3. Pipeline crate will parse these into type-safe enums for routing
///
/// **Rule 13.5**: Use enums in routing logic, but constants for API boundaries.

// ============================================================================
// File Extension Constants
// ============================================================================

// Model file extensions
pub const EXT_GGUF: &str = ".gguf";
pub const EXT_ONNX: &str = ".onnx";
pub const EXT_ONNX_DATA: &str = ".onnx_data";
pub const EXT_SAFETENSORS: &str = ".safetensors";
pub const EXT_TFLITE: &str = ".tflite";
pub const EXT_BIN: &str = ".bin";

// Config/tokenizer file extensions
pub const EXT_CONFIG_JSON: &str = "config.json";
pub const EXT_TOKENIZER_JSON: &str = "tokenizer.json";
pub const EXT_TOKENIZER_CONFIG_JSON: &str = "tokenizer_config.json";
pub const EXT_GENERATION_CONFIG_JSON: &str = "generation_config.json";
pub const EXT_VOCAB_JSON: &str = "vocab.json";
pub const EXT_MERGES_TXT: &str = "merges.txt";
pub const EXT_SPECIAL_TOKENS_MAP_JSON: &str = "special_tokens_map.json";

// ============================================================================
// Backend Engine Constants
// ============================================================================

// Rust engines
pub const ENGINE_LLAMA_CPP: &str = "llama.cpp";
pub const ENGINE_BITNET: &str = "bitnet";

// Python engines (will migrate to Rust)
pub const ENGINE_ONNXRUNTIME: &str = "onnxruntime";
pub const ENGINE_MEDIAPIPE: &str = "mediapipe";
pub const ENGINE_TRANSFORMERS: &str = "transformers";
pub const ENGINE_LITERT: &str = "litert";

// ============================================================================
// Task Constants (HuggingFace pipeline_tag values)
// ============================================================================

// Text tasks
pub const TASK_TEXT_GENERATION: &str = "text-generation";
pub const TASK_TEXT2TEXT_GENERATION: &str = "text2text-generation";
pub const TASK_FILL_MASK: &str = "fill-mask";
pub const TASK_TOKEN_CLASSIFICATION: &str = "token-classification";
pub const TASK_QUESTION_ANSWERING: &str = "question-answering";
pub const TASK_SUMMARIZATION: &str = "summarization";
pub const TASK_TRANSLATION: &str = "translation";
pub const TASK_TEXT_CLASSIFICATION: &str = "text-classification";
pub const TASK_CONVERSATIONAL: &str = "conversational";

// Vision tasks
pub const TASK_IMAGE_CLASSIFICATION: &str = "image-classification";
pub const TASK_IMAGE_TO_TEXT: &str = "image-to-text";
pub const TASK_IMAGE_SEGMENTATION: &str = "image-segmentation";
pub const TASK_OBJECT_DETECTION: &str = "object-detection";
pub const TASK_DEPTH_ESTIMATION: &str = "depth-estimation";
pub const TASK_IMAGE_TO_IMAGE: &str = "image-to-image";

// Audio tasks
pub const TASK_AUTOMATIC_SPEECH_RECOGNITION: &str = "automatic-speech-recognition";
pub const TASK_AUDIO_CLASSIFICATION: &str = "audio-classification";
pub const TASK_TEXT_TO_SPEECH: &str = "text-to-speech";
pub const TASK_AUDIO_TO_AUDIO: &str = "audio-to-audio";

// Multimodal tasks
pub const TASK_FEATURE_EXTRACTION: &str = "feature-extraction";
pub const TASK_SENTENCE_SIMILARITY: &str = "sentence-similarity";
pub const TASK_ZERO_SHOT_CLASSIFICATION: &str = "zero-shot-classification";
pub const TASK_ZERO_SHOT_IMAGE_CLASSIFICATION: &str = "zero-shot-image-classification";
pub const TASK_ZERO_SHOT_AUDIO_CLASSIFICATION: &str = "zero-shot-audio-classification";
pub const TASK_ZERO_SHOT_OBJECT_DETECTION: &str = "zero-shot-object-detection";

// Embedding tasks  
pub const TASK_EMBEDDING: &str = "embedding";
pub const TASK_RERANKING: &str = "reranking";

/// Check if a task string is a text generation variant
pub fn is_text_generation_task(task: &str) -> bool {
    matches!(
        task,
        TASK_TEXT_GENERATION | TASK_TEXT2TEXT_GENERATION | TASK_CONVERSATIONAL
    )
}

/// Check if a task string is a vision task
pub fn is_vision_task(task: &str) -> bool {
    matches!(
        task,
        TASK_IMAGE_CLASSIFICATION
            | TASK_IMAGE_TO_TEXT
            | TASK_IMAGE_SEGMENTATION
            | TASK_OBJECT_DETECTION
            | TASK_DEPTH_ESTIMATION
            | TASK_IMAGE_TO_IMAGE
            | TASK_ZERO_SHOT_IMAGE_CLASSIFICATION
            | TASK_ZERO_SHOT_OBJECT_DETECTION
    )
}

/// Check if a task string is an audio task
pub fn is_audio_task(task: &str) -> bool {
    matches!(
        task,
        TASK_AUTOMATIC_SPEECH_RECOGNITION
            | TASK_AUDIO_CLASSIFICATION
            | TASK_TEXT_TO_SPEECH
            | TASK_AUDIO_TO_AUDIO
            | TASK_ZERO_SHOT_AUDIO_CLASSIFICATION
    )
}

/// Check if a task string is an embedding/feature extraction task
pub fn is_embedding_task(task: &str) -> bool {
    matches!(
        task,
        TASK_FEATURE_EXTRACTION | TASK_EMBEDDING | TASK_SENTENCE_SIMILARITY | TASK_RERANKING
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_classification() {
        assert!(is_text_generation_task(TASK_TEXT_GENERATION));
        assert!(is_text_generation_task(TASK_TEXT2TEXT_GENERATION));
        
        assert!(is_vision_task(TASK_IMAGE_TO_TEXT));
        assert!(is_vision_task(TASK_OBJECT_DETECTION));
        
        assert!(is_audio_task(TASK_AUTOMATIC_SPEECH_RECOGNITION));
        assert!(is_audio_task(TASK_TEXT_TO_SPEECH));
        
        assert!(is_embedding_task(TASK_FEATURE_EXTRACTION));
        assert!(is_embedding_task(TASK_EMBEDDING));
    }
}

