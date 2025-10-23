use serde::{Serialize, Deserialize};
use crate::manifest::ManifestEntry;
use crate::hf_client::HfModelConfig;

/// Supported model types for inference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelType {
    /// GGUF format models (used by llama.cpp)
    /// Standard quantized models for general use
    GGUF,
    /// BitNet 1.58-bit models (stored as .gguf files)
    /// Special handling needed: custom DLL, ternary weights (-1/0/+1), optimized for extreme compression
    BitNet,
    /// ONNX Runtime models (Transformers.js)
    ONNX,
    /// SafeTensors format models
    SafeTensors,
    /// LiteRT models (MediaPipe)
    LiteRT,
    /// Unknown or unsupported model type
    Unknown,
}

/// Backend execution engine for a model
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Backend {
    /// Rust-based backend with engine name
    Rust { engine: String },
    /// Python-based backend with engine name
    Python { engine: String },
    /// External API endpoint
    API { endpoint: String },
}

/// Complete model information including type, backend, and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Repository ID (e.g., "microsoft/Phi-3-mini-4k-instruct-onnx")
    pub repo: String,
    /// Detected model type
    pub model_type: ModelType,
    /// Recommended backend for execution
    pub backend: Backend,
    /// Available model variants (e.g., different quantizations)
    pub variants: Vec<String>,
    /// Model task (e.g., "text-generation", "text2text-generation")
    pub task: Option<String>,
    /// Model architecture (e.g., "florence2", "janus", "whisper", "clip")
    /// Used to route to specialized pipelines that need architecture-specific preprocessing
    pub architecture: Option<String>,
    /// Extension-compatible manifest (for ONNX models)
    pub manifest: Option<ManifestEntry>,
}

// Known repository patterns for fast detection
const BITNET_PATTERNS: &[&str] = &["1bitLLM", "1.58", "b1.58", "BitNet", "bitnet", "Falcon-E"];
const GGUF_PATTERNS: &[&str] = &["GGUF", "gguf"];
const ONNX_PATTERNS: &[&str] = &["-onnx", "-ONNX", "onnx/"];
const LITERT_PATTERNS: &[&str] = &["litert", "LiteRT", "gemma-3n"];

// Model-specific patterns for task detection (Layer 1)
const FLORENCE_PATTERNS: &[&str] = &["florence", "Florence"];
const JANUS_PATTERNS: &[&str] = &["janus", "Janus"];
const WHISPER_PATTERNS: &[&str] = &["whisper", "Whisper", "moonshine", "Moonshine"];
const CLIP_PATTERNS: &[&str] = &["clip", "CLIP"];
const CLAP_PATTERNS: &[&str] = &["clap", "CLAP"];
const RERANK_PATTERNS: &[&str] = &["rerank", "cross-encoder"];
const TTS_PATTERNS: &[&str] = &["speecht5", "SpeechT5", "-tts", "TTS"];
const CODE_PATTERNS: &[&str] = &["code", "codellama", "CodeLlama", "starcoder", "StarCoder"];
const DINO_PATTERNS: &[&str] = &["dino", "DINOv2", "with-attentions"];

/// Detect model architecture from repo name or path
/// Returns None for generic models, Some("florence2") for specialized architectures
fn detect_architecture(source: &str) -> Option<String> {
    let source_lower = source.to_lowercase();
    
    // Check in priority order (most specific first)
    if FLORENCE_PATTERNS.iter().any(|p| source_lower.contains(&p.to_lowercase())) {
        return Some("florence2".to_string());
    }
    if JANUS_PATTERNS.iter().any(|p| source_lower.contains(&p.to_lowercase())) {
        return Some("janus".to_string());
    }
    if WHISPER_PATTERNS.iter().any(|p| source_lower.contains(&p.to_lowercase())) {
        return Some("whisper".to_string());
    }
    if CLIP_PATTERNS.iter().any(|p| source_lower.contains(&p.to_lowercase())) {
        return Some("clip".to_string());
    }
    if CLAP_PATTERNS.iter().any(|p| source_lower.contains(&p.to_lowercase())) {
        return Some("clap".to_string());
    }
    
    // No special architecture detected - generic model
    None
}

/// Layer 1: Detect model type from file path (fast, no network)
/// 
/// # Examples
/// ```
/// use model_cache::detection::detect_from_file_path;
/// 
/// let info = detect_from_file_path("models/Qwen3-30B/model-Q4_K_M.gguf");
/// assert!(info.is_some());
/// assert_eq!(info.unwrap().model_type, ModelType::GGUF);
/// ```
pub fn detect_from_file_path(path: &str) -> Option<ModelInfo> {
    // GGUF files (both regular and BitNet use .gguf extension)
    if path.ends_with(".gguf") {
        // Check if it's a BitNet model by path patterns
        // BitNet models are GGUF files but need special handling (different DLL, 1.58-bit quantization)
        let is_bitnet = BITNET_PATTERNS.iter().any(|pattern| path.contains(pattern));
        
        if is_bitnet {
            let repo = extract_repo_from_path(path);
            return Some(ModelInfo {
                repo: repo.clone(),
                model_type: ModelType::BitNet,
                backend: Backend::Rust { 
                    engine: "bitnet".to_string() 
                },
                variants: vec![path.to_string()],
                task: Some("text-generation".to_string()),
                architecture: detect_architecture(&repo),
                manifest: None,
            });
        }
        
        // Regular GGUF model
        let repo = extract_repo_from_path(path);
        return Some(ModelInfo {
            repo: repo.clone(),
            model_type: ModelType::GGUF,
            backend: Backend::Rust { 
                engine: "llama.cpp".to_string() 
            },
            variants: vec![path.to_string()],
            task: Some("text-generation".to_string()),
            architecture: detect_architecture(&repo),
            manifest: None,
        });
    }
    
    // ONNX files
    if path.ends_with(".onnx") {
        let repo = extract_repo_from_path(path);
        return Some(ModelInfo {
            repo: repo.clone(),
            model_type: ModelType::ONNX,
            backend: Backend::Python { 
                engine: "onnxruntime".to_string()  // Will migrate to Rust later
            },
            variants: vec![path.to_string()],
            task: None, // Will be fetched from HF if needed
            architecture: detect_architecture(&repo),
            manifest: None,
        });
    }
    
    // SafeTensors files
    if path.ends_with(".safetensors") {
        let repo = extract_repo_from_path(path);
        return Some(ModelInfo {
            repo: repo.clone(),
            model_type: ModelType::SafeTensors,
            backend: Backend::Python { 
                engine: "transformers".to_string() 
            },
            variants: vec![path.to_string()],
            task: None,
            architecture: detect_architecture(&repo),
            manifest: None,
        });
    }
    
    // LiteRT/TFLite files
    if path.ends_with(".tflite") || path.ends_with(".bin") {
        let repo = extract_repo_from_path(path);
        return Some(ModelInfo {
            repo: repo.clone(),
            model_type: ModelType::LiteRT,
            backend: Backend::Python { 
                engine: "mediapipe".to_string() 
            },
            variants: vec![path.to_string()],
            task: Some("text-generation".to_string()),
            architecture: detect_architecture(&repo),
            manifest: None,
        });
    }
    
    None
}

/// Layer 2: Detect model type from repository name (medium, pattern matching)
/// 
/// # Examples
/// ```
/// use model_cache::detection::detect_from_repo_name;
/// 
/// let info = detect_from_repo_name("Qwen/Qwen2.5-3B-GGUF");
/// assert!(info.is_some());
/// ```
pub fn detect_from_repo_name(repo: &str) -> Option<ModelInfo> {
    // Pattern 1: BitNet repositories (check BEFORE generic GGUF!)
    // BitNet models ARE GGUF files but need special handling (different DLL, 1.58-bit quantization)
    if BITNET_PATTERNS.iter().any(|pattern| repo.contains(pattern)) {
        return Some(ModelInfo {
            repo: repo.to_string(),
            model_type: ModelType::BitNet,
            backend: Backend::Rust { 
                engine: "bitnet".to_string() 
            },
            variants: vec![],
            task: Some("text-generation".to_string()),
            architecture: detect_architecture(repo),
            manifest: None,
        });
    }
    
    // Pattern 2: Generic GGUF repositories
    let repo_upper = repo.to_uppercase();
    if GGUF_PATTERNS.iter().any(|pattern| repo_upper.contains(&pattern.to_uppercase())) {
        return Some(ModelInfo {
            repo: repo.to_string(),
            model_type: ModelType::GGUF,
            backend: Backend::Rust { 
                engine: "llama.cpp".to_string() 
            },
            variants: vec![],
            task: Some("text-generation".to_string()),
            architecture: detect_architecture(repo),
            manifest: None,
        });
    }
    
    // Pattern 3: ONNX repositories
    if ONNX_PATTERNS.iter().any(|pattern| repo.to_lowercase().contains(&pattern.to_lowercase())) {
        return Some(ModelInfo {
            repo: repo.to_string(),
            model_type: ModelType::ONNX,
            backend: Backend::Python { 
                engine: "onnxruntime".to_string()  // Will migrate to Rust later
            },
            variants: vec![],
            task: None,
            architecture: detect_architecture(repo),
            manifest: None,
        });
    }
    
    // Pattern 4: LiteRT/MediaPipe repositories
    if LITERT_PATTERNS.iter().any(|pattern| repo.to_lowercase().contains(&pattern.to_lowercase())) {
        return Some(ModelInfo {
            repo: repo.to_string(),
            model_type: ModelType::LiteRT,
            backend: Backend::Python { 
                engine: "mediapipe".to_string() 
            },
            variants: vec![],
            task: Some("text-generation".to_string()),
            architecture: detect_architecture(repo),
            manifest: None,
        });
    }
    
    None
}

/// Extract repository ID from file path
/// 
/// # Examples
/// ```
/// use model_cache::detection::extract_repo_from_path;
/// 
/// let repo = extract_repo_from_path("models/Qwen/Qwen2.5-3B/model.gguf");
/// assert_eq!(repo, "Qwen/Qwen2.5-3B");
/// ```
pub fn extract_repo_from_path(path: &str) -> String {
    // Remove "models/" prefix if present
    let clean_path = path.strip_prefix("models/").unwrap_or(path);
    
    // Split by / and take first two components (org/model)
    let parts: Vec<&str> = clean_path.split('/').collect();
    if parts.len() >= 2 {
        format!("{}/{}", parts[0], parts[1])
    } else {
        clean_path.to_string()
    }
}

/// Detect task from model name patterns (Layer 1: fastest, most specific)
///
/// Uses model-specific patterns to infer task types for specialized models
/// like Florence2, Janus, Whisper, CLIP, etc.
///
/// # Arguments
/// * `repo_name` - Repository or model name to check
///
/// # Returns
/// Task string if pattern matches, None otherwise
pub fn detect_task_from_name(repo_name: &str) -> Option<String> {
    let name_lower = repo_name.to_lowercase();
    
    // Florence2 models: image-to-text
    if FLORENCE_PATTERNS.iter().any(|p| repo_name.contains(p)) {
        return Some("image-to-text".to_string());
    }
    
    // Janus models: visual-language
    if JANUS_PATTERNS.iter().any(|p| repo_name.contains(p)) {
        return Some("visual-language".to_string());
    }
    
    // Whisper/Moonshine models: automatic-speech-recognition
    if WHISPER_PATTERNS.iter().any(|p| repo_name.contains(p)) {
        return Some("automatic-speech-recognition".to_string());
    }
    
    // CLAP models (audio): zero-shot-audio-classification (check before CLIP!)
    if CLAP_PATTERNS.iter().any(|p| repo_name.contains(p)) {
        return Some("zero-shot-audio-classification".to_string());
    }
    
    // CLIP models (image): feature-extraction / zero-shot-image-classification
    if CLIP_PATTERNS.iter().any(|p| repo_name.contains(p)) {
        return Some("feature-extraction".to_string());
    }
    
    // Rerank/Cross-encoder models: text-classification
    if RERANK_PATTERNS.iter().any(|p| name_lower.contains(p)) {
        return Some("text-classification".to_string());
    }
    
    // TTS models: text-to-speech
    if TTS_PATTERNS.iter().any(|p| repo_name.contains(p)) {
        return Some("text-to-speech".to_string());
    }
    
    // Code models: text-generation (specialized)
    if CODE_PATTERNS.iter().any(|p| repo_name.contains(p)) {
        return Some("text-generation".to_string());
    }
    
    // DINOv2 / attention visualization: image-classification
    if DINO_PATTERNS.iter().any(|p| repo_name.contains(p)) {
        return Some("image-classification".to_string());
    }
    
    None
}

/// Detect task from config.json (Layer 2: authoritative)
///
/// Infers task from model_type and architectures fields in config.json
///
/// # Arguments
/// * `config` - Parsed model configuration
///
/// # Returns
/// Task string if inferable, None otherwise
pub fn detect_task_from_config(config: &HfModelConfig) -> Option<String> {
    // Check architectures first (most specific)
    if let Some(archs) = &config.architectures {
        for arch in archs {
            let arch_lower = arch.to_lowercase();
            
            // Causal LM: text-generation
            if arch_lower.contains("forcausallm") || arch_lower.contains("lmheadmodel") {
                return Some("text-generation".to_string());
            }
            
            // Conditional generation models (need model_type for disambiguation)
            if arch_lower.contains("forconditionalgeneration") {
                if let Some(ref model_type) = config.model_type {
                    let mt_lower = model_type.to_lowercase();
                    if mt_lower.contains("whisper") {
                        return Some("automatic-speech-recognition".to_string());
                    }
                    if mt_lower.contains("vision") || mt_lower.contains("florence") {
                        return Some("image-to-text".to_string());
                    }
                }
                // Default for conditional generation
                return Some("text2text-generation".to_string());
            }
            
            // Sequence classification
            if arch_lower.contains("forsequenceclassification") {
                return Some("text-classification".to_string());
            }
            
            // Token classification (NER, POS tagging)
            if arch_lower.contains("fortokenclassification") {
                return Some("token-classification".to_string());
            }
            
            // Question answering
            if arch_lower.contains("forquestionanswering") {
                return Some("question-answering".to_string());
            }
            
            // Masked LM (BERT-style)
            if arch_lower.contains("formaskedlm") {
                return Some("fill-mask".to_string());
            }
            
            // Image classification
            if arch_lower.contains("forimageclassification") {
                return Some("image-classification".to_string());
            }
            
            // Object detection
            if arch_lower.contains("forobjectdetection") {
                return Some("object-detection".to_string());
            }
        }
    }
    
    // Fallback to model_type
    if let Some(ref model_type) = config.model_type {
        let mt_lower = model_type.to_lowercase();
        
        match mt_lower.as_str() {
            "whisper" => return Some("automatic-speech-recognition".to_string()),
            "clip" => return Some("feature-extraction".to_string()),
            "clap" => return Some("zero-shot-audio-classification".to_string()),
            "bert" | "roberta" | "distilbert" => return Some("feature-extraction".to_string()),
            "gpt2" | "gpt_neox" | "llama" | "mistral" | "phi" | "phi3" | "qwen2" => {
                return Some("text-generation".to_string());
            }
            "t5" | "bart" | "pegasus" => return Some("text2text-generation".to_string()),
            _ => {}
        }
    }
    
    // Check if task is explicitly defined in config
    config.task.clone()
}

/// Comprehensive task detection combining all sources
///
/// Priority order:
/// 1. Model name patterns (highest confidence for specialized models)
/// 2. config.json (authoritative when available)
/// 3. HF API pipeline_tag (fallback)
/// 4. Default to "text-generation"
///
/// # Arguments
/// * `repo_name` - Repository or model name
/// * `config` - Optional model configuration from config.json
/// * `pipeline_tag` - Optional pipeline_tag from HuggingFace API
///
/// # Returns
/// Detected task string
pub fn detect_task_unified(
    repo_name: &str,
    config: Option<&HfModelConfig>,
    pipeline_tag: Option<&str>,
) -> String {
    // Layer 1: Model name patterns (highest confidence for specialized models)
    if let Some(task) = detect_task_from_name(repo_name) {
        return task;
    }
    
    // Layer 2: config.json (authoritative)
    if let Some(cfg) = config {
        if let Some(task) = detect_task_from_config(cfg) {
            return task;
        }
    }
    
    // Layer 3: HF API pipeline_tag
    if let Some(tag) = pipeline_tag {
        return tag.to_string();
    }
    
    // Layer 4: Default fallback
    "text-generation".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_gguf_file_path() {
        let info = detect_from_file_path("models/Qwen/Qwen2.5-3B/model-Q4_K_M.gguf");
        assert!(info.is_some());
        
        let info = info.expect("Should detect GGUF file");
        assert_eq!(info.model_type, ModelType::GGUF);
        assert_eq!(info.repo, "Qwen/Qwen2.5-3B");
        
        if let Backend::Rust { engine } = info.backend {
            assert_eq!(engine, "llama.cpp");
        } else {
            panic!("Expected Rust backend");
        }
    }
    
    #[test]
    fn test_detect_bitnet_file_path() {
        // BitNet models are .gguf files but need special handling
        let info = detect_from_file_path("models/1bitLLM/Falcon3-1B-Instruct-1.58bit/model.gguf");
        assert!(info.is_some());
        
        let info = info.expect("Should detect BitNet file");
        assert_eq!(info.model_type, ModelType::BitNet);
        assert_eq!(info.repo, "1bitLLM/Falcon3-1B-Instruct-1.58bit");
        
        if let Backend::Rust { engine } = info.backend {
            assert_eq!(engine, "bitnet");
        } else {
            panic!("Expected Rust backend");
        }
    }
    
    #[test]
    fn test_detect_onnx_file_path() {
        let info = detect_from_file_path("models/microsoft/Phi-3-mini/onnx/model_q4f16.onnx");
        assert!(info.is_some());
        
        let info = info.expect("Should detect ONNX file");
        assert_eq!(info.model_type, ModelType::ONNX);
        assert_eq!(info.repo, "microsoft/Phi-3-mini");
    }
    
    #[test]
    fn test_detect_gguf_repo_name() {
        let test_cases = vec![
            "Qwen/Qwen2.5-3B-GGUF",
            "bartowski/Llama-3.2-3B-gguf",
            "TheBloke/Mistral-7B-GGUF",
        ];
        
        for repo in test_cases {
            let info = detect_from_repo_name(repo);
            assert!(info.is_some(), "Failed to detect GGUF repo: {}", repo);
            
            let info = info.expect("Should detect GGUF repo");
            assert_eq!(info.model_type, ModelType::GGUF);
            assert_eq!(info.repo, repo);
        }
    }
    
    #[test]
    fn test_detect_bitnet_repo_name() {
        let test_cases = vec![
            "microsoft/BitNet-b1.58-2B-4T",
            "1bitLLM/bitnet-3b",
        ];
        
        for repo in test_cases {
            let info = detect_from_repo_name(repo);
            assert!(info.is_some(), "Failed to detect BitNet repo: {}", repo);
            
            let info = info.expect("Should detect BitNet repo");
            assert_eq!(info.model_type, ModelType::BitNet);
        }
    }
    
    #[test]
    fn test_detect_onnx_repo_name() {
        let test_cases = vec![
            "microsoft/Phi-3-mini-4k-instruct-onnx",
            "HuggingFaceTB/SmolLM3-3B-ONNX",
            "Xenova/gpt2-onnx",
        ];
        
        for repo in test_cases {
            let info = detect_from_repo_name(repo);
            assert!(info.is_some(), "Failed to detect ONNX repo: {}", repo);
            
            let info = info.expect("Should detect ONNX repo");
            assert_eq!(info.model_type, ModelType::ONNX);
        }
    }
    
    #[test]
    fn test_detect_litert_repo_name() {
        let info = detect_from_repo_name("google/gemma-3n-E4B-it-litert-lm");
        assert!(info.is_some());
        
        let info = info.expect("Should detect LiteRT repo");
        assert_eq!(info.model_type, ModelType::LiteRT);
    }
    
    #[test]
    fn test_extract_repo_from_path() {
        assert_eq!(
            extract_repo_from_path("models/Qwen/Qwen2.5-3B/model.gguf"),
            "Qwen/Qwen2.5-3B"
        );
        
        assert_eq!(
            extract_repo_from_path("microsoft/Phi-3-mini/onnx/model.onnx"),
            "microsoft/Phi-3-mini"
        );
        
        assert_eq!(
            extract_repo_from_path("localmodel/file.gguf"),
            "localmodel/file.gguf"
        );
    }
    
    #[test]
    fn test_unknown_file_type() {
        let info = detect_from_file_path("models/unknown/file.txt");
        assert!(info.is_none());
    }
    
    #[test]
    fn test_unknown_repo_pattern() {
        let info = detect_from_repo_name("random-org/random-model");
        assert!(info.is_none());
    }
}

