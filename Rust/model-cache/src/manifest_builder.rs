use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::{
    hf_client::{HfRepoMetadata, HfFile, extract_clean_dtype, is_onnx_file, is_supporting_file},
    schema::QuantStatus,
    error::Result,
};

/// Extension-compatible quantization information
/// Matches the TypeScript QuantInfo interface in idbModel.ts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionQuantInfo {
    /// Full paths (rfilename) to all required files for this quant
    pub files: Vec<String>,
    /// Quantization status
    pub status: QuantStatus,
    /// Clean quantization type: "q4f16", "fp16", "fp32", etc.
    pub dtype: String,
    /// Whether this quant uses external data format (split files)
    pub has_external_data: bool,
}

/// Extension-compatible manifest entry
/// Matches the TypeScript ManifestEntry interface in idbModel.ts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionManifestEntry {
    /// Repository ID (e.g., "microsoft/Phi-3-mini-4k-instruct-onnx")
    pub repo: String,
    /// Map of main ONNX file path to quantization info
    pub quants: HashMap<String, ExtensionQuantInfo>,
    /// Model task (e.g., "text-generation", "text2text-generation")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    /// Version of the manifest structure
    pub manifest_version: u32,
}

/// Constants for server-only size limits (2.1GB default)
pub const DEFAULT_SERVER_ONLY_SIZE: u64 = 2_252_341_248; // 2.1 * 1024 * 1024 * 1024

/// Bypass models that should always be allowed regardless of size
pub const DEFAULT_BYPASS_MODELS: &[&str] = &[
    "google/gemma-3n-E4B-it-litert-lm",
];

/// Build an extension-compatible manifest from HuggingFace repository metadata
///
/// # Arguments
/// * `metadata` - Repository metadata from HuggingFace API
/// * `server_only_size_limit` - Maximum size before marking as server_only (default 2.1GB)
/// * `bypass_models` - Models that bypass size limits
///
/// # Returns
/// Extension-compatible manifest with all ONNX quantizations detected
///
/// # Examples
/// ```no_run
/// use model_cache::manifest_builder::{build_manifest_from_hf, DEFAULT_SERVER_ONLY_SIZE, DEFAULT_BYPASS_MODELS};
/// use model_cache::hf_client::fetch_repo_metadata;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let metadata = fetch_repo_metadata("microsoft/Phi-3-mini-4k-instruct-onnx", None).await?;
/// let manifest = build_manifest_from_hf(
///     &metadata,
///     DEFAULT_SERVER_ONLY_SIZE,
///     DEFAULT_BYPASS_MODELS
/// )?;
/// println!("Found {} quantizations", manifest.quants.len());
/// # Ok(())
/// # }
/// ```
pub fn build_manifest_from_hf(
    metadata: &HfRepoMetadata,
    _server_only_size_limit: u64, // Kept for API compatibility; unused in desktop (no browser limits)
    bypass_models: &[&str],
) -> Result<ExtensionManifestEntry> {
    let repo = metadata.repo_id.clone().unwrap_or_else(|| "unknown".to_string());
    let task = metadata.pipeline_tag.clone();
    
    // Check if this repo is in bypass list
    let _is_bypass = bypass_models.iter().any(|&model| repo.contains(model));
    
    // Step 1: Find all .onnx files (excluding .onnx_data)
    let onnx_files: Vec<&HfFile> = metadata.siblings.iter()
        .filter(|f| is_onnx_file(&f.path))
        .collect();
    
    // Step 2: For each .onnx file, collect its dependencies
    let mut quants = HashMap::new();
    
    for onnx_file in onnx_files {
        let onnx_path = &onnx_file.path;
        
        // Extract dtype from filename
        let dtype = extract_clean_dtype(onnx_path);
        
        // Collect all files needed for this quant
        let mut files = vec![onnx_path.clone()];
        let mut has_external_data = false;
        // Note: total_size accumulated but not used for status in desktop builds (no browser size limits)
        #[allow(unused_variables, unused_assignments)]
        let mut total_size: u64 = onnx_file.size.unwrap_or(0);
        
        // Check for corresponding .onnx_data OR .onnx.data file
        // Extension checks BOTH patterns (sidepanel.ts:1777)
        let base_name = onnx_path.strip_suffix(".onnx").unwrap_or(onnx_path);
        let data_path_underscore = format!("{}.onnx_data", base_name);
        let data_path_dot = format!("{}.onnx.data", base_name);
        
        if let Some(data_file) = metadata.siblings.iter()
            .find(|f| f.path == data_path_underscore || f.path == data_path_dot) 
        {
            files.push(data_file.path.clone());
            has_external_data = true;
            total_size += data_file.size.unwrap_or(0);
        }
        
        // Collect supporting files (config, tokenizer, etc.)
        // Only add root-level supporting files
        for file in &metadata.siblings {
            if is_supporting_file(&file.path) {
                // Only include if it's in the root or same directory as the onnx file
                let file_dir = get_file_directory(&file.path);
                let onnx_dir = get_file_directory(onnx_path);
                
                if file_dir == onnx_dir || file_dir.is_empty() {
                    if !files.contains(&file.path) {
                        files.push(file.path.clone());
                        total_size += file.size.unwrap_or(0);
                    }
                }
            }
        }
        
        // Desktop: All models are "Available" - no browser size limits
        // Actual loading will check VRAM/RAM and auto-split GPU/CPU layers
        // The "server_only_size_limit" parameter is kept for extension compatibility
        // but not used in desktop - we rely on hardware detection instead
        let status = QuantStatus::Available;
        
        quants.insert(onnx_path.clone(), ExtensionQuantInfo {
            files,
            status,
            dtype,
            has_external_data,
        });
    }
    
    Ok(ExtensionManifestEntry {
        repo,
        quants,
        task,
        manifest_version: 1,
    })
}

/// Get directory path from file path
fn get_file_directory(path: &str) -> String {
    if let Some(last_slash) = path.rfind('/') {
        path[..last_slash].to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_build_manifest_basic() {
        let metadata = HfRepoMetadata {
            repo_id: Some("test/model-onnx".to_string()),
            pipeline_tag: Some("text-generation".to_string()),
            siblings: vec![
                HfFile { path: "config.json".to_string(), size: Some(1024) },
                HfFile { path: "tokenizer.json".to_string(), size: Some(2048) },
                HfFile { path: "onnx/model_q4f16.onnx".to_string(), size: Some(1_000_000) },
                HfFile { path: "onnx/model_fp16.onnx".to_string(), size: Some(2_000_000) },
            ],
            tags: None,
        };
        
        let manifest = build_manifest_from_hf(
            &metadata,
            DEFAULT_SERVER_ONLY_SIZE,
            DEFAULT_BYPASS_MODELS
        ).expect("Should build manifest");
        
        assert_eq!(manifest.repo, "test/model-onnx");
        assert_eq!(manifest.task, Some("text-generation".to_string()));
        assert_eq!(manifest.manifest_version, 1);
        assert_eq!(manifest.quants.len(), 2);
        
        // Check q4f16 quant
        let q4f16 = manifest.quants.get("onnx/model_q4f16.onnx").expect("Should have q4f16");
        assert_eq!(q4f16.dtype, "q4f16");
        assert_eq!(q4f16.status, QuantStatus::Available);
        assert!(!q4f16.has_external_data);
        assert!(q4f16.files.contains(&"onnx/model_q4f16.onnx".to_string()));
        
        // Check fp16 quant
        let fp16 = manifest.quants.get("onnx/model_fp16.onnx").expect("Should have fp16");
        assert_eq!(fp16.dtype, "fp16");
    }
    
    #[test]
    fn test_build_manifest_with_external_data() {
        let metadata = HfRepoMetadata {
            repo_id: Some("test/large-model".to_string()),
            pipeline_tag: None,
            siblings: vec![
                HfFile { path: "config.json".to_string(), size: Some(1024) },
                HfFile { path: "onnx/model.onnx".to_string(), size: Some(5_000) },
                HfFile { path: "onnx/model.onnx_data".to_string(), size: Some(1_000_000_000) },
            ],
            tags: None,
        };
        
        let manifest = build_manifest_from_hf(
            &metadata,
            DEFAULT_SERVER_ONLY_SIZE,
            DEFAULT_BYPASS_MODELS
        ).expect("Should build manifest");
        
        assert_eq!(manifest.quants.len(), 1);
        
        let quant = manifest.quants.get("onnx/model.onnx").expect("Should have quant");
        assert_eq!(quant.dtype, "fp32"); // Default dtype
        assert!(quant.has_external_data);
        assert_eq!(quant.files.len(), 3); // onnx + onnx_data + config.json (root supporting file)
        assert!(quant.files.contains(&"onnx/model.onnx_data".to_string()));
        assert!(quant.files.contains(&"config.json".to_string()));
    }
    
    #[test]
    fn test_desktop_all_available() {
        // Desktop builds: All models are "Available" regardless of size
        // Browser size limits don't apply - we rely on VRAM/RAM detection instead
        let large_size = 3_000_000_000u64; // 3GB
        
        let metadata = HfRepoMetadata {
            repo_id: Some("test/huge-model".to_string()),
            pipeline_tag: None,
            siblings: vec![
                HfFile { path: "model.onnx".to_string(), size: Some(large_size) },
            ],
            tags: None,
        };
        
        let manifest = build_manifest_from_hf(
            &metadata,
            DEFAULT_SERVER_ONLY_SIZE, // Kept for API compat but unused in desktop
            DEFAULT_BYPASS_MODELS
        ).expect("Should build manifest");
        
        let quant = manifest.quants.get("model.onnx").expect("Should have quant");
        // Desktop: Always Available - actual loading will check hardware and auto-split layers
        assert_eq!(quant.status, QuantStatus::Available);
    }
    
    #[test]
    fn test_bypass_model() {
        let large_size = 3_000_000_000u64; // 3GB
        
        let metadata = HfRepoMetadata {
            repo_id: Some("google/gemma-3n-E4B-it-litert-lm".to_string()),
            pipeline_tag: None,
            siblings: vec![
                HfFile { path: "model.onnx".to_string(), size: Some(large_size) },
            ],
            tags: None,
        };
        
        let manifest = build_manifest_from_hf(
            &metadata,
            DEFAULT_SERVER_ONLY_SIZE,
            DEFAULT_BYPASS_MODELS
        ).expect("Should build manifest");
        
        let quant = manifest.quants.get("model.onnx").expect("Should have quant");
        // Should be Available despite large size because it's in bypass list
        assert_eq!(quant.status, QuantStatus::Available);
    }
    
    #[test]
    fn test_get_file_directory() {
        assert_eq!(get_file_directory("onnx/model.onnx"), "onnx");
        assert_eq!(get_file_directory("model.onnx"), "");
        assert_eq!(get_file_directory("a/b/c/file.txt"), "a/b/c");
    }
}

