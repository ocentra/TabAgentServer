use crate::error::{ModelCacheError, Result};
use futures::StreamExt;
use reqwest::Client;
use std::sync::Arc;

const HUGGINGFACE_BASE: &str = "https://huggingface.co";

/// Progress callback for downloads
pub type ProgressCallback = Arc<dyn Fn(u64, u64) + Send + Sync>;

/// Downloader for HuggingFace model files
pub struct ModelDownloader {
    client: Client,
}

impl ModelDownloader {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("tabagent-model-cache/0.1.0")
            .build()
            .unwrap_or_else(|_| Client::new());
        
        Self { client }
    }
    
    /// Download a file from HuggingFace with progress tracking and validation
    pub async fn download_file(
        &self,
        repo_id: &str,
        file_path: &str,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<Vec<u8>> {
        self.download_file_with_token(repo_id, file_path, None, progress_callback).await
    }
    
    /// Download a file with explicit token (bypasses ENV check)
    pub async fn download_file_with_token(
        &self,
        repo_id: &str,
        file_path: &str,
        auth_token: Option<&str>,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<Vec<u8>> {
        // ===== SAFETY CHECKS =====
        
        // 1. Validate repo_id format
        if !Self::is_valid_repo_id(repo_id) {
            return Err(ModelCacheError::Download(format!(
                "Invalid repo_id format: '{}'. Expected 'owner/repo'",
                repo_id
            )));
        }
        
        // 2. Validate file_path (no path traversal)
        if !Self::is_safe_file_path(file_path) {
            return Err(ModelCacheError::Download(format!(
                "Unsafe file path detected: '{}'",
                file_path
            )));
        }
        
        // 3. Build and validate URL
        let url = format!("{}/{}/resolve/main/{}", HUGGINGFACE_BASE, repo_id, file_path);
        
        // 4. Ensure URL is actually pointing to HuggingFace
        if !url.starts_with(HUGGINGFACE_BASE) {
            return Err(ModelCacheError::Download(format!(
                "URL does not point to HuggingFace: '{}'",
                url
            )));
        }
        
        log::info!("✅ Validated and downloading {} from {}", file_path, url);
        
        // ===== DOWNLOAD =====
        
        // Build request with optional HuggingFace token for gated/private repos
        let mut request = self.client.get(&url);
        
        // Priority: explicit token > ENV variable
        let token = auth_token
            .map(|t| t.to_string())
            .or_else(|| std::env::var("HF_TOKEN").ok())
            .or_else(|| std::env::var("HUGGINGFACE_TOKEN").ok());
        
        if let Some(token) = token {
            if !token.is_empty() {
                log::debug!("Using HuggingFace token for authentication");
                request = request.header("Authorization", format!("Bearer {}", token));
            }
        }
        
        let response = request.send().await?;
        
        if !response.status().is_success() {
            return Err(ModelCacheError::Download(format!(
                "HTTP {} while downloading {}",
                response.status(),
                file_path
            )));
        }
        
        let total_size = response.content_length().unwrap_or(0);
        
        let mut downloaded = 0u64;
        let mut buffer = Vec::with_capacity(total_size as usize);
        
        let mut stream = response.bytes_stream();
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            buffer.extend_from_slice(&chunk);
            downloaded += chunk.len() as u64;
            
            if let Some(ref callback) = progress_callback {
                callback(downloaded, total_size);
            }
        }
        
        log::info!("Downloaded {} ({} bytes)", file_path, downloaded);
        
        Ok(buffer)
    }
    
    /// List files in a HuggingFace repository
    pub async fn list_repo_files(&self, repo_id: &str) -> Result<Vec<String>> {
        let url = format!("https://huggingface.co/api/models/{}", repo_id);
        
        log::info!("Fetching file list for repo: {}", repo_id);
        
        // Build request with optional HuggingFace token
        let mut request = self.client.get(&url);
        let token = std::env::var("HF_TOKEN")
            .or_else(|_| std::env::var("HUGGINGFACE_TOKEN"))
            .ok();
        if let Some(token) = token {
            if !token.is_empty() {
                request = request.header("Authorization", format!("Bearer {}", token));
            }
        }
        
        let response = request.send().await?;
        
        if !response.status().is_success() {
            return Err(ModelCacheError::Download(format!(
                "HTTP {} while listing files for {}",
                response.status(),
                repo_id
            )));
        }
        
        #[derive(serde::Deserialize)]
        struct HFModel {
            #[serde(default)]
            siblings: Vec<HFFile>,
        }
        
        #[derive(serde::Deserialize)]
        struct HFFile {
            rfilename: String,
        }
        
        let model_info: HFModel = response.json().await?;
        
        let files: Vec<String> = model_info
            .siblings
            .into_iter()
            .map(|f| f.rfilename)
            .collect();
        
        log::info!("Found {} files in {}", files.len(), repo_id);
        
        Ok(files)
    }
    
    /// Get model metadata (task, pipeline_tag, etc.)
    pub async fn get_model_info(&self, repo_id: &str) -> Result<ModelInfo> {
        let url = format!("https://huggingface.co/api/models/{}", repo_id);
        
        // Build request with optional HuggingFace token
        let mut request = self.client.get(&url);
        let token = std::env::var("HF_TOKEN")
            .or_else(|_| std::env::var("HUGGINGFACE_TOKEN"))
            .ok();
        if let Some(token) = token {
            if !token.is_empty() {
                request = request.header("Authorization", format!("Bearer {}", token));
            }
        }
        
        let response = request.send().await?;
        
        if !response.status().is_success() {
            return Err(ModelCacheError::Download(format!(
                "HTTP {} while fetching info for {}",
                response.status(),
                repo_id
            )));
        }
        
        #[derive(serde::Deserialize)]
        struct HFModel {
            #[serde(default)]
            pipeline_tag: Option<String>,
            #[serde(default)]
            library_name: Option<String>,
        }
        
        let model_info: HFModel = response.json().await?;
        
        Ok(ModelInfo {
            repo_id: repo_id.to_string(),
            task: model_info.pipeline_tag,
            library: model_info.library_name,
        })
    }
    
    /// Validate repo_id format (owner/repo)
    fn is_valid_repo_id(repo_id: &str) -> bool {
        let parts: Vec<&str> = repo_id.split('/').collect();
        
        // Must be exactly owner/repo
        if parts.len() != 2 {
            return false;
        }
        
        // No empty parts
        if parts[0].is_empty() || parts[1].is_empty() {
            return false;
        }
        
        // No path traversal
        if repo_id.contains("..") || repo_id.contains("//") {
            return false;
        }
        
        // Only safe characters
        repo_id.chars().all(|c| {
            c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/'
        })
    }
    
    /// Validate file_path for safety (no path traversal)
    fn is_safe_file_path(file_path: &str) -> bool {
        // No path traversal
        if file_path.contains("..") || file_path.starts_with('/') || file_path.starts_with('\\') {
            return false;
        }
        
        // No double slashes
        if file_path.contains("//") || file_path.contains("\\\\") {
            return false;
        }
        
        // No null bytes or other dangerous characters
        if file_path.contains('\0') {
            return false;
        }
        
        // Must not be empty
        if file_path.is_empty() {
            return false;
        }
        
        true
    }
}

impl Default for ModelDownloader {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub repo_id: String,
    pub task: Option<String>,
    pub library: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_list_files() {
        let downloader = ModelDownloader::new();
        
        // Use a small, known test model
        let files = downloader
            .list_repo_files("hf-internal-testing/tiny-random-gpt2")
            .await
            .unwrap();
        
        assert!(!files.is_empty());
        assert!(files.iter().any(|f| f.contains("config.json")));
    }
    
    #[tokio::test]
    async fn test_get_model_info() {
        let downloader = ModelDownloader::new();
        
        let info = downloader
            .get_model_info("hf-internal-testing/tiny-random-gpt2")
            .await
            .unwrap();
        
        assert_eq!(info.repo_id, "hf-internal-testing/tiny-random-gpt2");
    }
}

