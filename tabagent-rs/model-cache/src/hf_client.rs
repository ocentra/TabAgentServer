use serde::{Deserialize, Serialize};
use crate::error::{ModelCacheError, Result};

/// HuggingFace API endpoint
const HF_API_BASE: &str = "https://huggingface.co/api";

/// File information from HuggingFace repo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HfFile {
    /// Relative path in the repo (e.g., "onnx/model_q4f16.onnx")
    #[serde(rename = "rfilename")]
    pub path: String,
    /// File size in bytes
    pub size: Option<u64>,
}

/// Repository metadata from HuggingFace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HfRepoMetadata {
    /// Repository ID (e.g., "microsoft/Phi-3-mini-4k-instruct-onnx")
    #[serde(rename = "modelId")]
    pub repo_id: Option<String>,
    /// Model task (e.g., "text-generation", "text2text-generation")
    pub pipeline_tag: Option<String>,
    /// List of all files in the repository
    pub siblings: Vec<HfFile>,
    /// Tags (for additional model info)
    pub tags: Option<Vec<String>>,
}

/// Model configuration from config.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HfModelConfig {
    /// Model type (e.g., "whisper", "clip", "phi3")
    pub model_type: Option<String>,
    /// Model architectures (e.g., ["WhisperForConditionalGeneration"])
    pub architectures: Option<Vec<String>>,
    /// Task hint (sometimes present in config)
    pub task: Option<String>,
    /// Max position embeddings (context length)
    pub max_position_embeddings: Option<i64>,
}

/// Fetch repository metadata from HuggingFace API
///
/// # Arguments
/// * `repo` - Repository ID (e.g., "microsoft/Phi-3-mini-4k-instruct-onnx")
/// * `auth_token` - Optional HuggingFace API token for private repos
///
/// # Returns
/// Repository metadata including files list and pipeline_tag
///
/// # Errors
/// Returns `ModelCacheError::Network` or `ModelCacheError::Download` if the request fails or the API returns an error
///
/// # Examples
/// ```no_run
/// use model_cache::hf_client::fetch_repo_metadata;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let metadata = fetch_repo_metadata("microsoft/Phi-3-mini-4k-instruct-onnx", None).await?;
/// println!("Task: {:?}", metadata.pipeline_tag);
/// println!("Files: {}", metadata.siblings.len());
/// # Ok(())
/// # }
/// ```
pub async fn fetch_repo_metadata(
    repo: &str,
    auth_token: Option<&str>,
) -> Result<HfRepoMetadata> {
    let url = format!("{}/models/{}", HF_API_BASE, repo);
    
    let client = reqwest::Client::new();
    let mut request = client.get(&url);
    
    // Add authorization header if token provided
    if let Some(token) = auth_token {
        request = request.header("Authorization", format!("Bearer {}", token));
    }
    
    // Send request
    let response = request
        .send()
        .await
        .map_err(|e| ModelCacheError::Download(format!("Failed to fetch repo metadata: {}", e)))?;
    
    // Check status
    if !response.status().is_success() {
        return Err(ModelCacheError::Download(format!(
            "HuggingFace API returned error: {} - {}",
            response.status(),
            response.text().await.unwrap_or_else(|_| "Unknown error".to_string())
        )));
    }
    
    // Parse JSON
    let metadata = response
        .json::<HfRepoMetadata>()
        .await
        .map_err(|e| ModelCacheError::Download(format!("Failed to parse repo metadata: {}", e)))?;
    
    Ok(metadata)
}

/// Fetch model config.json from HuggingFace
///
/// # Arguments
/// * `repo` - Repository ID (e.g., "openai/whisper-tiny")
/// * `auth_token` - Optional HuggingFace API token for private repos
///
/// # Returns
/// Model configuration containing model_type, architectures, and other metadata
///
/// # Errors
/// Returns error if config.json doesn't exist or can't be parsed
pub async fn fetch_model_config(
    repo: &str,
    auth_token: Option<&str>,
) -> Result<HfModelConfig> {
    let url = format!("https://huggingface.co/{}/resolve/main/config.json", repo);
    
    let client = reqwest::Client::new();
    let mut request = client.get(&url);
    
    // Add authorization header if token provided
    if let Some(token) = auth_token {
        request = request.header("Authorization", format!("Bearer {}", token));
    }
    
    // Send request
    let response = request
        .send()
        .await
        .map_err(|e| ModelCacheError::Download(format!("Failed to fetch config.json: {}", e)))?;
    
    // Check status
    if !response.status().is_success() {
        return Err(ModelCacheError::Download(format!(
            "config.json not found or inaccessible: {} - {}",
            response.status(),
            response.text().await.unwrap_or_else(|_| "Unknown error".to_string())
        )));
    }
    
    // Parse JSON
    let config = response
        .json::<HfModelConfig>()
        .await
        .map_err(|e| ModelCacheError::Download(format!("Failed to parse config.json: {}", e)))?;
    
    Ok(config)
}

/// Extract clean dtype from file path
///
/// Parses quantization precision from file paths like:
/// - "onnx/model_q4f16.onnx" → "q4f16"
/// - "onnx/model_fp16.onnx" → "fp16"
/// - "onnx/model.onnx" → "fp32" (default)
///
/// Matches the logic in extension's `extractCleanDtype` function
///
/// # Examples
/// ```
/// use model_cache::hf_client::extract_clean_dtype;
///
/// assert_eq!(extract_clean_dtype("onnx/model_q4f16.onnx"), "q4f16");
/// assert_eq!(extract_clean_dtype("onnx/model_fp16.onnx"), "fp16");
/// assert_eq!(extract_clean_dtype("onnx/model.onnx"), "fp32");
/// ```
pub fn extract_clean_dtype(path: &str) -> String {
    // Get filename without extension
    let filename = path.rsplit('/').next().unwrap_or(path);
    let name_without_ext = filename
        .strip_suffix(".onnx")
        .or_else(|| filename.strip_suffix(".onnx_data"))
        .unwrap_or(filename);
    
    // Common quantization patterns
    let quant_patterns = [
        "q4", "q4f16", "q8", "int4", "uint4",
        "fp16", "float16", "fp32", "float32",
    ];
    
    for pattern in quant_patterns {
        if name_without_ext.to_lowercase().contains(pattern) {
            // Extract the actual quant string (e.g., "q4f16" from "model_q4f16")
            if let Some(idx) = name_without_ext.to_lowercase().find(pattern) {
                let quant_part = &name_without_ext[idx..];
                // Take only the quant identifier (stop at underscore or end)
                let quant = quant_part
                    .split('_')
                    .next()
                    .unwrap_or(quant_part)
                    .to_lowercase();
                return quant;
            }
        }
    }
    
    // Default to fp32 if no quant pattern found
    "fp32".to_string()
}

/// Check if a file is a supporting file (config, tokenizer, etc.)
///
/// Matches the SUPPORTING_FILE_REGEX patterns from the extension:
/// - config.json, generation_config.json, tokenizer_config.json
/// - tokenizer.json, vocab.json, added_tokens.json
/// - tokenizer.model, special_tokens_map.json
///
/// # Examples
/// ```
/// use model_cache::hf_client::is_supporting_file;
///
/// assert!(is_supporting_file("config.json"));
/// assert!(is_supporting_file("tokenizer.json"));
/// assert!(!is_supporting_file("model.onnx"));
/// ```
pub fn is_supporting_file(path: &str) -> bool {
    let filename = path.rsplit('/').next().unwrap_or(path).to_lowercase();
    
    const SUPPORTING_FILES: &[&str] = &[
        "config.json",
        "generation_config.json",
        "tokenizer_config.json",
        "tokenizer.json",
        "vocab.json",
        "added_tokens.json",
        "tokenizer.model",
        "special_tokens_map.json",
    ];
    
    SUPPORTING_FILES.contains(&filename.as_str())
}

/// Check if a file is an ONNX model file
///
/// # Examples
/// ```
/// use model_cache::hf_client::is_onnx_file;
///
/// assert!(is_onnx_file("onnx/model.onnx"));
/// assert!(!is_onnx_file("onnx/model.onnx_data"));
/// assert!(!is_onnx_file("config.json"));
/// ```
pub fn is_onnx_file(path: &str) -> bool {
    path.ends_with(".onnx") && !path.ends_with(".onnx_data")
}

/// Check if a file is ONNX external data
///
/// # Examples
/// ```
/// use model_cache::hf_client::is_onnx_external_data;
///
/// assert!(is_onnx_external_data("onnx/model.onnx_data"));
/// assert!(!is_onnx_external_data("onnx/model.onnx"));
/// ```
pub fn is_onnx_external_data(path: &str) -> bool {
    path.ends_with(".onnx_data")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_clean_dtype() {
        assert_eq!(extract_clean_dtype("onnx/model_q4f16.onnx"), "q4f16");
        assert_eq!(extract_clean_dtype("onnx/model_q4.onnx"), "q4");
        assert_eq!(extract_clean_dtype("onnx/model_fp16.onnx"), "fp16");
        assert_eq!(extract_clean_dtype("onnx/model_float16.onnx"), "float16");
        assert_eq!(extract_clean_dtype("onnx/model.onnx"), "fp32");
        assert_eq!(extract_clean_dtype("model_int4.onnx"), "int4");
    }
    
    #[test]
    fn test_is_supporting_file() {
        assert!(is_supporting_file("config.json"));
        assert!(is_supporting_file("tokenizer.json"));
        assert!(is_supporting_file("vocab.json"));
        assert!(is_supporting_file("onnx/config.json"));
        assert!(!is_supporting_file("model.onnx"));
        assert!(!is_supporting_file("random.txt"));
    }
    
    #[test]
    fn test_is_onnx_file() {
        assert!(is_onnx_file("onnx/model.onnx"));
        assert!(is_onnx_file("model_q4f16.onnx"));
        assert!(!is_onnx_file("model.onnx_data"));
        assert!(!is_onnx_file("config.json"));
    }
    
    #[test]
    fn test_is_onnx_external_data() {
        assert!(is_onnx_external_data("onnx/model.onnx_data"));
        assert!(is_onnx_external_data("model_q4f16.onnx_data"));
        assert!(!is_onnx_external_data("model.onnx"));
        assert!(!is_onnx_external_data("config.json"));
    }
}

