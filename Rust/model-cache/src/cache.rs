use crate::error::{ModelCacheError, Result};
use crate::manifest::ManifestEntry;
use crate::schema::QuantStatus;
use crate::storage::ChunkStorage;
use crate::download::{ModelDownloader, ProgressCallback};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main model cache manager - combines storage, downloads, and manifest management
pub struct ModelCache {
    storage: Arc<ChunkStorage>,
    downloader: Arc<ModelDownloader>,
    manifests: Arc<RwLock<sled::Tree>>,
}

impl ModelCache {
    /// Create a new model cache at the specified path
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let storage = Arc::new(ChunkStorage::new(&db_path)?);
        let downloader = Arc::new(ModelDownloader::new());
        
        // Open manifests tree in the same sled DB
        let db = sled::open(&db_path)?;
        let manifests = Arc::new(RwLock::new(db.open_tree(b"model_manifests")?));
        
        Ok(Self {
            storage,
            downloader,
            manifests,
        })
    }
    
    /// Get or create manifest entry for a repo
    pub async fn get_manifest(&self, repo_id: &str) -> Result<Option<ManifestEntry>> {
        let manifests = self.manifests.read().await;
        let key = format!("manifest:{}", repo_id);
        
        if let Some(bytes) = manifests.get(key.as_bytes())? {
            let entry: ManifestEntry = bincode::deserialize(&bytes)?;
            return Ok(Some(entry));
        }
        
        Ok(None)
    }
    
    /// Save manifest entry
    pub async fn save_manifest(&self, entry: &ManifestEntry) -> Result<()> {
        let manifests = self.manifests.write().await;
        let key = format!("manifest:{}", entry.repo_id);
        let bytes = bincode::serialize(entry)?;
        manifests.insert(key.as_bytes(), bytes)?;
        manifests.flush()?;
        Ok(())
    }
    
    /// Scan a HuggingFace repo and create/update manifest
    /// SIMPLE & GENERIC: Just tracks what files exist, no type/quant detection
    pub async fn scan_repo(&self, repo_id: &str) -> Result<ManifestEntry> {
        log::info!("Scanning repository: {}", repo_id);
        
        // Validate repo_id format (owner/repo)
        if !Self::is_valid_repo_id(repo_id) {
            return Err(ModelCacheError::Manifest(format!(
                "Invalid repo_id format: '{}'. Expected 'owner/repo'", 
                repo_id
            )));
        }
        
        // Get model info (task, pipeline_tag)
        let model_info = self.downloader.get_model_info(repo_id).await?;
        
        // List all files in repo
        let files = self.downloader.list_repo_files(repo_id).await?;
        
        // Create or update manifest
        let mut manifest = self.get_manifest(repo_id).await?
            .unwrap_or_else(|| ManifestEntry::new(repo_id.to_string(), model_info.task.clone()));
        
        // Update task if changed
        if model_info.task.is_some() {
            manifest.task = model_info.task;
        }
        
        // SIMPLE APPROACH: Just group ALL files by their natural variant names
        // No hardcoded detection - let the filename speak for itself
        let mut file_groups: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        
        for file in files {
            // Extract variant from filename (generic, works for any format)
            let variant = extract_variant_from_filename(&file);
            file_groups.entry(variant).or_default().push(file);
        }
        
        // Update manifest with what we found
        for (variant_key, files) in file_groups {
            // Check if files are already cached
            let all_cached = files.iter().all(|f| {
                self.storage.has_file(repo_id, f).unwrap_or(false)
            });
            
            let status = if all_cached {
                QuantStatus::Downloaded
            } else {
                QuantStatus::Available
            };
            
            manifest.quants.entry(variant_key.clone()).or_insert_with(|| {
                crate::manifest::QuantInfo {
                    status: status.clone(),
                    files: files.clone(),
                    total_size: None,
                    downloaded_size: None,
                    last_updated: chrono::Utc::now().timestamp_millis(),
                }
            });
        }
        
        self.save_manifest(&manifest).await?;
        
        // Flush after creating new manifest
        self.flush().await?;
        
        log::info!("✅ Scanned and persisted {} - found {} variants", repo_id, manifest.quants.len());
        
        Ok(manifest)
    }
    
    /// Validate HuggingFace repo_id format and safety
    fn is_valid_repo_id(repo_id: &str) -> bool {
        // Must be owner/repo format
        let parts: Vec<&str> = repo_id.split('/').collect();
        if parts.len() != 2 {
            return false;
        }
        
        // No empty parts
        if parts[0].is_empty() || parts[1].is_empty() {
            return false;
        }
        
        // Basic safety: no path traversal attempts
        if repo_id.contains("..") || repo_id.contains("//") {
            return false;
        }
        
        // Only alphanumeric, dash, underscore, dot allowed
        repo_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '/')
    }
    
    /// Download and cache a specific file
    pub async fn download_file(
        &self,
        repo_id: &str,
        file_path: &str,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<()> {
        log::info!("Downloading {} from {}", file_path, repo_id);
        
        // Download the file
        let data = self.downloader.download_file(repo_id, file_path, progress_callback).await?;
        
        // Store in chunks
        self.storage.store_file(
            repo_id,
            file_path,
            &data,
            Some("application/octet-stream".to_string()),
            None,
        )?;
        
        log::info!("Downloaded and cached {} ({} bytes)", file_path, data.len());
        
        Ok(())
    }
    
    /// Download all files for a quantization variant
    pub async fn download_quant(
        &self,
        repo_id: &str,
        quant_key: &str,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<()> {
        log::info!("Downloading quant {} for {}", quant_key, repo_id);
        
        // Get manifest
        let mut manifest = self.get_manifest(repo_id).await?
            .ok_or_else(|| ModelCacheError::Manifest(format!("No manifest for {}", repo_id)))?;
        
        // Get file list for this quant
        let files = manifest.quants.get(quant_key)
            .ok_or_else(|| ModelCacheError::Manifest(format!("Quant {} not found", quant_key)))?
            .files.clone();
        
        // Update status to Downloading
        manifest.update_quant_status(quant_key, QuantStatus::Downloading);
        self.save_manifest(&manifest).await?;
        
        // Download each file
        let total_files = files.len();
        for (i, file_path) in files.iter().enumerate() {
            // Skip if already cached
            if self.storage.has_file(repo_id, file_path)? {
                log::info!("File already cached: {}", file_path);
                continue;
            }
            
            // Create per-file progress callback
            let file_progress = if let Some(ref cb) = progress_callback {
                let cb = Arc::clone(cb);
                let file_idx = i;
                Some(Arc::new(move |loaded: u64, total: u64| {
                    // Map file progress to overall progress
                    let file_progress = if total > 0 { loaded as f64 / total as f64 } else { 0.0 };
                    let overall_progress = (file_idx as f64 + file_progress) / total_files as f64;
                    cb((overall_progress * 100.0) as u64, 100);
                }) as ProgressCallback)
            } else {
                None
            };
            
            self.download_file(repo_id, file_path, file_progress).await?;
        }
        
        // Update status to Downloaded
        manifest.update_quant_status(quant_key, QuantStatus::Downloaded);
        self.save_manifest(&manifest).await?;
        
        // Flush after complete download (not after each chunk!)
        self.flush().await?;
        
        log::info!("✅ Downloaded and persisted quant {} for {}", quant_key, repo_id);
        
        Ok(())
    }
    
    /// **MANIFEST-FIRST**: Ensure manifest exists before operations
    /// 
    /// This matches extension's pattern of ALWAYS checking manifest first
    /// (sidepanel.ts lines 1690-1695)
    pub async fn ensure_manifest(&self, repo_id: &str) -> Result<ManifestEntry> {
        // Check if manifest exists
        if let Some(manifest) = self.get_manifest(repo_id).await? {
            log::debug!("[ensure_manifest] Manifest for {} already exists", repo_id);
            return Ok(manifest);
        }
        
        // Manifest doesn't exist - scan repo and create it
        log::info!("[ensure_manifest] Manifest for {} not found, scanning repo...", repo_id);
        self.scan_repo(repo_id).await?;
        
        // Get the newly created manifest
        self.get_manifest(repo_id).await?
            .ok_or_else(|| ModelCacheError::Manifest(
                format!("Failed to create manifest for {}", repo_id)
            ))
    }
    
    /// Get a cached file as bytes
    /// 
    /// **MANIFEST-FIRST**: Checks manifest exists before file access
    pub async fn get_file(&self, repo_id: &str, file_path: &str) -> Result<Option<Vec<u8>>> {
        // Ensure manifest exists first (extension pattern)
        let _manifest = self.ensure_manifest(repo_id).await?;
        
        self.storage.get_file(repo_id, file_path)
    }
    
    /// Get a cached file as a filesystem path (for model loaders that need paths)
    /// This writes the cached file to a temporary location and returns the path
    /// 
    /// **MANIFEST-FIRST**: Checks manifest exists before file access
    /// 
    /// IMPORTANT: Caller is responsible for cleaning up the temp file
    pub async fn get_file_path(&self, repo_id: &str, file_path: &str) -> Result<Option<std::path::PathBuf>> {
        // Ensure manifest exists first (extension pattern)
        let _manifest = self.ensure_manifest(repo_id).await?;
        
        self.storage.get_file_as_temp_path(repo_id, file_path)
    }
    
    /// Stream a file chunk-by-chunk (ZERO RAM OVERHEAD for large models)
    /// 
    /// This is critical for serving 20GB models without exhausting memory
    /// Returns an iterator that yields 5MB chunks
    /// 
    /// Example:
    /// ```rust,ignore
    /// let chunks = cache.stream_file_chunks("owner/repo", "model.gguf")?;
    /// for chunk_result in chunks {
    ///     let chunk = chunk_result?;
    ///     // Send chunk via HTTP, write to file, etc.
    ///     // Only 5MB in memory at a time!
    /// }
    /// ```
    pub fn stream_file_chunks<'a>(
        &'a self,
        repo_id: &'a str,
        file_path: &'a str,
    ) -> Result<Option<crate::storage::FileChunkIterator<'a>>> {
        self.storage.stream_file_chunks(repo_id, file_path)
    }
    
    /// Check if a file is cached
    pub fn has_file(&self, repo_id: &str, file_path: &str) -> Result<bool> {
        self.storage.has_file(repo_id, file_path)
    }
    
    /// Delete a model and all its files
    pub async fn delete_model(&self, repo_id: &str) -> Result<()> {
        log::info!("Deleting model: {}", repo_id);
        
        // Get file list
        let files = self.storage.list_files(repo_id)?;
        
        // Delete all files
        for file_path in files {
            self.storage.delete_file(repo_id, &file_path)?;
        }
        
        // Delete manifest
        let manifests = self.manifests.write().await;
        let key = format!("manifest:{}", repo_id);
        manifests.remove(key.as_bytes())?;
        manifests.flush()?;
        
        log::info!("Deleted model: {}", repo_id);
        
        Ok(())
    }
    
    /// Get storage statistics
    pub async fn get_stats(&self) -> Result<CacheStats> {
        let manifests = self.manifests.read().await;
        let mut total_repos = 0;
        let mut total_size = 0u64;
        
        for item in manifests.iter() {
            let (_key, value) = item?;
            let entry: ManifestEntry = bincode::deserialize(&value)?;
            total_repos += 1;
            total_size += self.storage.get_repo_size(&entry.repo_id)?;
        }
        
        Ok(CacheStats {
            total_repos,
            total_size,
        })
    }
    
    /// Flush pending writes to disk
    /// 
    /// **Resource Analysis (RAG Rule 15.1):**
    /// - Sled is memory-mapped; keeping open ≈ 5-10MB overhead (metadata/cache)
    /// - Reopening = expensive (file I/O, index rebuild)
    /// - **Strategy: Keep open, flush at logical boundaries**
    /// 
    /// **When to flush:**
    /// - After complete download (all files)
    /// - After scan_repo (manifest created)
    /// - On explicit request (e.g., before backup)
    /// 
    /// **When NOT to flush:**
    /// - After each chunk (would destroy streaming performance!)
    /// - After single file read (no writes!)
    /// - Between model operations (DB stays open)
    pub async fn flush(&self) -> Result<()> {
        log::debug!("[ModelCache] Flushing pending writes...");
        
        // Flush storage
        self.storage.flush()?;
        
        // Flush manifests
        let manifests = self.manifests.read().await;
        manifests.flush()?;
        
        log::debug!("[ModelCache] Flush complete");
        Ok(())
    }
    
    /// Graceful shutdown - final flush before application exit
    /// 
    /// **ONLY call on application shutdown!**
    /// 
    /// The cache is designed to stay open for application lifetime (RAG Rule 4.2).
    /// Sled's memory-mapped design means minimal overhead (~5-10MB) when idle.
    pub async fn shutdown(&self) -> Result<()> {
        log::info!("[ModelCache] Application shutdown - final flush...");
        self.flush().await?;
        log::info!("[ModelCache] Shutdown complete. DB will close on Drop.");
        Ok(())
    }
}

/// Automatic cleanup on drop (RAG Rule 4.2: RAII)
/// 
/// Sled's Drop implementation will:
/// 1. Flush any pending writes
/// 2. Close file handles
/// 3. Release memory-mapped regions (OS handles this)
/// 
/// **Resource behavior:**
/// - Connection open during app lifetime: ~5-10MB overhead
/// - On drop: All resources released automatically
/// - No explicit close needed (RAII pattern)
impl Drop for ModelCache {
    fn drop(&mut self) {
        log::debug!("[ModelCache] Drop - ensuring final flush...");
        
        // Best-effort sync flush (can't do async in Drop)
        if let Err(e) = self.storage.flush() {
            log::warn!("[ModelCache] Failed to flush on drop: {}", e);
        }
        
        // Sled's Drop will handle file handles and memory cleanup
        log::debug!("[ModelCache] Resources released");
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub total_repos: usize,
    pub total_size: u64,
}

/// Extract variant identifier from filename (GENERIC - works for any format)
/// 
/// Philosophy: Don't try to parse/understand the format, just group by natural names
/// Examples:
///   "model-Q4_K_M.gguf" -> "Q4_K_M"
///   "model_q4f16.onnx" -> "q4f16"  
///   "model_fp16.safetensors" -> "fp16"
///   "model.bin" -> "default"
///   "llama-2-7b-Q8_0.gguf" -> "Q8_0"
fn extract_variant_from_filename(file_path: &str) -> String {
    let file_name = file_path.split('/').last().unwrap_or(file_path);
    
    // Config/tokenizer files always go to "common"
    use crate::tasks::{
        EXT_CONFIG_JSON, EXT_TOKENIZER_JSON, EXT_TOKENIZER_CONFIG_JSON,
        EXT_GENERATION_CONFIG_JSON, EXT_VOCAB_JSON, EXT_MERGES_TXT,
        EXT_SPECIAL_TOKENS_MAP_JSON,
    };
    
    if file_name.ends_with(EXT_CONFIG_JSON) 
        || file_name.ends_with(EXT_TOKENIZER_JSON)
        || file_name.ends_with(EXT_TOKENIZER_CONFIG_JSON)
        || file_name.ends_with(EXT_GENERATION_CONFIG_JSON)
        || file_name.ends_with(EXT_VOCAB_JSON)
        || file_name.ends_with(EXT_MERGES_TXT)
        || file_name.ends_with(EXT_SPECIAL_TOKENS_MAP_JSON) {
        return "common".to_string();
    }
    
    // Remove extension to get base name
    let name_without_ext = file_name
        .rsplit_once('.')
        .map(|(base, _)| base)
        .unwrap_or(file_name);
    
    // Look for common variant patterns (dash or underscore separated)
    // Pattern: "base-VARIANT" or "base_VARIANT"
    
    // Try dash separator first: "model-Q4_K_M" -> "Q4_K_M"
    if let Some(last_dash) = name_without_ext.rfind('-') {
        let potential_variant = &name_without_ext[last_dash + 1..];
        // If it looks like a variant (contains numbers/underscores), use it
        if potential_variant.chars().any(|c| c.is_numeric() || c == '_') {
            return potential_variant.to_string();
        }
    }
    
    // Try underscore separator: "model_q4f16" -> "q4f16"
    if let Some(last_underscore) = name_without_ext.rfind('_') {
        let potential_variant = &name_without_ext[last_underscore + 1..];
        // If it looks like a variant (contains numbers), use it
        if potential_variant.chars().any(|c| c.is_numeric()) {
            return potential_variant.to_string();
        }
    }
    
    // No variant detected - this is the default/base model
    "default".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_create_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache = ModelCache::new(temp_dir.path()).unwrap();
        
        let stats = cache.get_stats().await.unwrap();
        assert_eq!(stats.total_repos, 0);
    }
    
    #[test]
    fn test_extract_variant_from_filename() {
        // ONNX formats
        assert_eq!(extract_variant_from_filename("onnx/model_q4f16.onnx"), "q4f16");
        assert_eq!(extract_variant_from_filename("onnx/model_fp16.onnx"), "fp16");
        assert_eq!(extract_variant_from_filename("model.onnx"), "default");
        assert_eq!(extract_variant_from_filename("onnx/decoder_model_q4.onnx"), "q4");
        
        // GGUF formats
        assert_eq!(extract_variant_from_filename("model-Q4_K_M.gguf"), "Q4_K_M");
        assert_eq!(extract_variant_from_filename("llama-2-7b-Q5_K_S.gguf"), "Q5_K_S");
        assert_eq!(extract_variant_from_filename("phi-3_Q8_0.gguf"), "Q8_0");
        assert_eq!(extract_variant_from_filename("model_q4_k_m.gguf"), "q4_k_m");
        
        // SafeTensors
        assert_eq!(extract_variant_from_filename("model-fp16.safetensors"), "fp16");
        assert_eq!(extract_variant_from_filename("model.safetensors"), "default");
        
        // Common files (config, tokenizer)
        assert_eq!(extract_variant_from_filename("config.json"), "common");
        assert_eq!(extract_variant_from_filename("tokenizer.json"), "common");
        assert_eq!(extract_variant_from_filename("generation_config.json"), "common");
        
        // Edge cases
        assert_eq!(extract_variant_from_filename("model-v2.bin"), "default"); // v2 has no numbers
        assert_eq!(extract_variant_from_filename("random_file_100.txt"), "100"); // Has number
    }
}

