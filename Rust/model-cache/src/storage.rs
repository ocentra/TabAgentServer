use crate::error::{ModelCacheError, Result};
use storage::engine::{MdbxEngine, StorageEngine};
use std::sync::Arc;

const CHUNK_SIZE: usize = 100 * 1024 * 1024; // 100MB chunks (matching extension logic)

/// Manages chunked storage of model files using libmdbx
pub struct ChunkStorage {
    /// Low-level storage engine for chunks and metadata
    engine: Arc<MdbxEngine>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct FileMetadata {
    pub repo_id: String,
    pub file_path: String,
    pub total_size: u64,
    pub chunk_count: usize,
    pub content_type: Option<String>,
    pub etag: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ChunkManifest {
    pub id: String, // "{repo_id}/{file_path}:manifest"
    pub manifest_type: String, // "manifest"
    pub chunk_group_id: String, // "{repo_id}/{file_path}"
    pub file_name: String,
    pub total_chunks: usize,
    pub chunk_size_used: usize,
    pub size: u64,
    pub status: String, // "present", "downloading", "failed"
}

/// Progress callback: (bytes_downloaded, total_bytes, current_chunk, total_chunks)
pub type StorageProgressCallback = Box<dyn Fn(u64, u64, usize, usize) + Send + Sync>;

impl ChunkStorage {
    /// Create a new ChunkStorage using libmdbx
    pub fn new(db_path: &str) -> Result<Self> {
        let engine = MdbxEngine::open(db_path)
            .map_err(|e| ModelCacheError::Storage(e.to_string()))?;
        
        Ok(Self {
            engine: Arc::new(engine),
        })
    }
    
    /// Store a file in chunks
    pub fn store_file(
        &self,
        repo_id: &str,
        file_path: &str,
        data: &[u8],
        content_type: Option<String>,
        etag: Option<String>,
    ) -> Result<()> {
        let chunk_count = (data.len() + CHUNK_SIZE - 1) / CHUNK_SIZE;
        
        // Store chunks
        for (i, chunk) in data.chunks(CHUNK_SIZE).enumerate() {
            let chunk_key = format!("file:{}:{}:chunk:{}", repo_id, file_path, i);
        self.engine.insert("chunks", chunk_key.as_bytes(), chunk.to_vec())
            .map_err(|e| ModelCacheError::Storage(e.to_string()))?;
        }
        
        // Store metadata
        let metadata = FileMetadata {
            repo_id: repo_id.to_string(),
            file_path: file_path.to_string(),
            total_size: data.len() as u64,
            chunk_count,
            content_type,
            etag,
            created_at: chrono::Utc::now().timestamp_millis(),
        };
        
        let meta_key = format!("meta:{}:{}", repo_id, file_path);
        let meta_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&metadata)
            .map_err(|e| ModelCacheError::Serialization(e.to_string()))?;
        self.engine.insert("metadata", meta_key.as_bytes(), meta_bytes.to_vec())
            .map_err(|e| ModelCacheError::Storage(e.to_string()))?;
        
        self.engine.flush()
            .map_err(|e| ModelCacheError::Storage(e.to_string()))?;
        
        Ok(())
    }
    
    /// **STREAMING STORAGE** - Store file chunks as they arrive from network
    /// 
    /// This matches extension's `saveChunkedFileSafe()` - NO RAM ACCUMULATION!
    /// Saves chunks to storage immediately as they fill up
    /// 
    /// # Arguments
    /// * `repo_id` - Repository ID (e.g., "microsoft/phi-2")
    /// * `file_path` - File path within repo (e.g., "onnx/model.onnx")
    /// * `stream` - Async stream of byte chunks from network
    /// * `total_size` - Total file size (from Content-Length header)
    /// * `progress_callback` - Optional callback for progress updates
    pub async fn store_file_streaming<S>(
        &self,
        repo_id: &str,
        file_path: &str,
        mut stream: S,
        total_size: u64,
        content_type: Option<String>,
        etag: Option<String>,
        progress_callback: Option<StorageProgressCallback>,
    ) -> Result<()>
    where
        S: futures::Stream<Item = Result<bytes::Bytes>> + Unpin,
    {
        use futures::StreamExt;
        
        let total_chunks = ((total_size as usize) + CHUNK_SIZE - 1) / CHUNK_SIZE;
        
        log::info!(
            "[store_file_streaming] Starting chunking {}: {} bytes into {} chunks (last chunk will be {} bytes)",
            file_path,
            total_size,
            total_chunks,
            total_size as usize % CHUNK_SIZE
        );
        
        // 1. Create and save manifest first
        let manifest = ChunkManifest {
            id: format!("{}:{}:manifest", repo_id, file_path),
            manifest_type: "manifest".to_string(),
            chunk_group_id: format!("{}:{}", repo_id, file_path),
            file_name: file_path.to_string(),
            total_chunks,
            chunk_size_used: CHUNK_SIZE,
            size: total_size,
            status: "downloading".to_string(),
        };
        
        let manifest_key = format!("manifest:{}:{}", repo_id, file_path);
        let manifest_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&manifest)
            .map_err(|e| ModelCacheError::Serialization(e.to_string()))?;
        self.engine.insert("metadata", manifest_key.as_bytes(), manifest_bytes.to_vec())
            .map_err(|e| ModelCacheError::Storage(e.to_string()))?;
        
        // 2. Stream and save chunks
        let mut chunk_index = 0;
        let mut total_bytes_processed = 0u64;
        let mut current_chunk_buffer = vec![0u8; CHUNK_SIZE];
        let mut current_chunk_offset = 0;
        
        while let Some(network_chunk) = stream.next().await {
            let network_chunk = network_chunk?;
            let mut data_offset = 0;
            
            while data_offset < network_chunk.len() {
                let remaining_in_chunk = CHUNK_SIZE - current_chunk_offset;
                let remaining_in_data = network_chunk.len() - data_offset;
                let bytes_to_copy = remaining_in_chunk.min(remaining_in_data);
                
                // Copy data to current chunk buffer
                current_chunk_buffer[current_chunk_offset..current_chunk_offset + bytes_to_copy]
                    .copy_from_slice(&network_chunk[data_offset..data_offset + bytes_to_copy]);
                
                current_chunk_offset += bytes_to_copy;
                data_offset += bytes_to_copy;
                
                // If chunk is full, save it immediately
                if current_chunk_offset == CHUNK_SIZE {
                    let chunk_key = format!("file:{}:{}:chunk:{}", repo_id, file_path, chunk_index);
                    self.engine.insert("chunks", chunk_key.as_bytes(), current_chunk_buffer[..current_chunk_offset].to_vec())
                        .map_err(|e| ModelCacheError::Storage(e.to_string()))?;
                    
                    total_bytes_processed += current_chunk_offset as u64;
                    chunk_index += 1;
                    current_chunk_offset = 0;
                    
                    // Report progress every 20 chunks
                    if chunk_index % 20 == 0 || chunk_index == total_chunks {
                        log::info!(
                            "[store_file_streaming] Chunks {}-{} saved ({} bytes, total: {}/{})",
                            chunk_index.saturating_sub(20),
                            chunk_index - 1,
                            current_chunk_offset,
                            total_bytes_processed,
                            total_size
                        );
                        
                        if let Some(ref callback) = progress_callback {
                            callback(total_bytes_processed, total_size, chunk_index, total_chunks);
                        }
                    }
                }
            }
        }
        
        // 3. Save the last partial chunk if there's remaining data
        if current_chunk_offset > 0 {
            let chunk_key = format!("file:{}:{}:chunk:{}", repo_id, file_path, chunk_index);
            self.engine.insert("chunks", chunk_key.as_bytes(), current_chunk_buffer[..current_chunk_offset].to_vec())
                .map_err(|e| ModelCacheError::Storage(e.to_string()))?;
            
            total_bytes_processed += current_chunk_offset as u64;
            chunk_index += 1;
            
            log::info!(
                "[store_file_streaming] Streamed final chunk {}/{} ({} bytes, total: {}/{})",
                chunk_index,
                total_chunks,
                current_chunk_offset,
                total_bytes_processed,
                total_size
            );
            
            if let Some(ref callback) = progress_callback {
                callback(total_bytes_processed, total_size, chunk_index, total_chunks);
            }
        }
        
        // 4. Save metadata with actual chunk count
        let metadata = FileMetadata {
            repo_id: repo_id.to_string(),
            file_path: file_path.to_string(),
            total_size,
            chunk_count: chunk_index,
            content_type,
            etag,
            created_at: chrono::Utc::now().timestamp_millis(),
        };
        
        let meta_key = format!("meta:{}:{}", repo_id, file_path);
        let meta_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&metadata)
            .map_err(|e| ModelCacheError::Serialization(e.to_string()))?;
        self.engine.insert("metadata", meta_key.as_bytes(), meta_bytes.to_vec())
            .map_err(|e| ModelCacheError::Storage(e.to_string()))?;
        
        // 5. Update manifest status to "present"
        let mut final_manifest = manifest;
        final_manifest.status = "present".to_string();
        final_manifest.total_chunks = chunk_index;
        let manifest_bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&final_manifest)
            .map_err(|e| ModelCacheError::Serialization(e.to_string()))?;
        self.engine.insert("metadata", manifest_key.as_bytes(), manifest_bytes.to_vec())
            .map_err(|e| ModelCacheError::Storage(e.to_string()))?;
        
        self.engine.flush().map_err(|e| ModelCacheError::Storage(e.to_string()))?;
        
        log::info!(
            "[store_file_streaming] ✅ Successfully streamed {}: {} chunks saved",
            file_path,
            chunk_index
        );
        
        Ok(())
    }
    
    /// Retrieve a file from chunks
    pub fn get_file(&self, repo_id: &str, file_path: &str) -> Result<Option<Vec<u8>>> {
        // Get metadata first
        let meta_key = format!("meta:{}:{}", repo_id, file_path);
        let meta_bytes = match self.engine.get("metadata", meta_key.as_bytes()).map_err(|e| ModelCacheError::Storage(e.to_string()))? {
            Some(guard) => guard.data,
            None => return Ok(None),
        };
        
        let metadata = rkyv::from_bytes::<FileMetadata, rkyv::rancor::Error>(&meta_bytes)
            .map_err(|e| ModelCacheError::Serialization(e.to_string()))?;
        
        // Reconstruct file from chunks
        let mut file_data = Vec::with_capacity(metadata.total_size as usize);
        
        for i in 0..metadata.chunk_count {
            let chunk_key = format!("file:{}:{}:chunk:{}", repo_id, file_path, i);
            let chunk_guard = self.engine.get("chunks", chunk_key.as_bytes()).map_err(|e| ModelCacheError::Storage(e.to_string()))?
                .ok_or_else(|| ModelCacheError::Download(format!("Missing chunk {} for file {}", i, file_path)))?;
            
            file_data.extend_from_slice(&chunk_guard.data);
        }
        
        Ok(Some(file_data))
    }
    
    /// Get a file as a temporary filesystem path
    /// This writes the cached chunks to a temp file and returns the path
    /// 
    /// This is needed for model loaders (like llama.cpp) that require file paths
    /// instead of in-memory bytes
    pub fn get_file_as_temp_path(&self, repo_id: &str, file_path: &str) -> Result<Option<std::path::PathBuf>> {
        use std::io::Write;
        
        // Get metadata first
        let meta_key = format!("meta:{}:{}", repo_id, file_path);
        let meta_bytes = match self.engine.get("metadata", meta_key.as_bytes()).map_err(|e| ModelCacheError::Storage(e.to_string()))? {
            Some(guard) => guard.data,
            None => return Ok(None),
        };
        
        let metadata = rkyv::from_bytes::<FileMetadata, rkyv::rancor::Error>(&meta_bytes)
            .map_err(|e| ModelCacheError::Serialization(e.to_string()))?;
        
        // Create temp file with proper extension
        let file_name = file_path.split('/').last().unwrap_or("model");
        
        let temp_dir = std::env::temp_dir().join("tabagent_model_cache");
        std::fs::create_dir_all(&temp_dir)?;
        
        let temp_file_path = temp_dir.join(format!("{}_{}", 
            repo_id.replace('/', "_"),
            file_name
        ));
        
        // Write chunks to temp file
        let mut temp_file = std::fs::File::create(&temp_file_path)?;
        
        for i in 0..metadata.chunk_count {
            let chunk_key = format!("file:{}:{}:chunk:{}", repo_id, file_path, i);
            let chunk_guard = self.engine.get("chunks", chunk_key.as_bytes()).map_err(|e| ModelCacheError::Storage(e.to_string()))?
                .ok_or_else(|| ModelCacheError::Download(format!("Missing chunk {} for file {}", i, file_path)))?;
            
            temp_file.write_all(&chunk_guard.data)?;
        }
        
        temp_file.flush()?;
        
        Ok(Some(temp_file_path))
    }
    
    /// Get file metadata
    pub fn get_metadata(&self, repo_id: &str, file_path: &str) -> Result<Option<FileMetadata>> {
        let meta_key = format!("meta:{}:{}", repo_id, file_path);
        let meta_bytes = match self.engine.get("metadata", meta_key.as_bytes()).map_err(|e| ModelCacheError::Storage(e.to_string()))? {
            Some(guard) => guard.data,
            None => return Ok(None),
        };
        
        let metadata = rkyv::from_bytes::<FileMetadata, rkyv::rancor::Error>(&meta_bytes)
            .map_err(|e| ModelCacheError::Serialization(e.to_string()))?;
        Ok(Some(metadata))
    }
    
    /// Check if a file exists
    pub fn has_file(&self, repo_id: &str, file_path: &str) -> Result<bool> {
        let meta_key = format!("meta:{}:{}", repo_id, file_path);
        Ok(self.engine.get("metadata", meta_key.as_bytes())
            .map_err(|e| ModelCacheError::Storage(e.to_string()))?
            .is_some())
    }
    
    /// Delete a file and its chunks
    pub fn delete_file(&self, repo_id: &str, file_path: &str) -> Result<()> {
        // Get metadata to know how many chunks to delete
        let meta_key = format!("meta:{}:{}", repo_id, file_path);
        if let Some(guard) = self.engine.get("metadata", meta_key.as_bytes()).map_err(|e| ModelCacheError::Storage(e.to_string()))? {
            let metadata = rkyv::from_bytes::<FileMetadata, rkyv::rancor::Error>(&guard.data)
            .map_err(|e| ModelCacheError::Serialization(e.to_string()))?;
            
            // Delete all chunks
            for i in 0..metadata.chunk_count {
                let chunk_key = format!("file:{}:{}:chunk:{}", repo_id, file_path, i);
                self.engine.remove("chunks", chunk_key.as_bytes())
                    .map_err(|e| ModelCacheError::Storage(e.to_string()))?;
            }
        }
        
        // Delete metadata
        self.engine.remove("metadata", meta_key.as_bytes())
            .map_err(|e| ModelCacheError::Storage(e.to_string()))?;
        
        self.engine.flush().map_err(|e| ModelCacheError::Storage(e.to_string()))?;
        
        Ok(())
    }
    
    /// List all files for a repo
    pub fn list_files(&self, repo_id: &str) -> Result<Vec<String>> {
        let prefix = format!("meta:{}:", repo_id);
        let mut files = Vec::new();
        
        for item in self.engine.scan_prefix("metadata", prefix.as_bytes()) {
            let (_key, guard) = item.map_err(|e| ModelCacheError::Storage(e.to_string()))?;
            let metadata = rkyv::from_bytes::<FileMetadata, rkyv::rancor::Error>(&guard.data)
                .map_err(|e| ModelCacheError::Serialization(e.to_string()))?;
            files.push(metadata.file_path);
        }
        
        Ok(files)
    }
    
    /// Get total storage size for a repo
    pub fn get_repo_size(&self, repo_id: &str) -> Result<u64> {
        let prefix = format!("meta:{}:", repo_id);
        let mut total = 0u64;
        
        for item in self.engine.scan_prefix("metadata", prefix.as_bytes()) {
            let (_key, guard) = item.map_err(|e| ModelCacheError::Storage(e.to_string()))?;
            let metadata = rkyv::from_bytes::<FileMetadata, rkyv::rancor::Error>(&guard.data)
                .map_err(|e| ModelCacheError::Serialization(e.to_string()))?;
            total += metadata.total_size;
        }
        
        Ok(total)
    }
    
    /// Flush all pending writes to disk
    /// 
    /// **When to call:**
    /// - After `store_file_streaming()` completes (called internally)
    /// - After `delete_file()` (called internally)
    /// - Periodically by higher-level cache manager
    /// 
    /// Synchronizes pending writes to disk.
    /// Database connection remains open and is managed by RAII.
    pub fn flush(&self) -> Result<()> {
        self.engine.flush()
            .map_err(|e| ModelCacheError::Storage(e.to_string()))?;
        Ok(())
    }
    
    /// Stream a file chunk-by-chunk without loading the entire file into memory.
    /// Returns an iterator over file chunks.
    pub fn stream_file_chunks<'a>(
        &'a self,
        repo_id: &'a str,
        file_path: &'a str,
    ) -> Result<Option<FileChunkIterator<'a>>> {
        // Get metadata first
        let meta_key = format!("meta:{}:{}", repo_id, file_path);
        let meta_bytes = match self.engine.get("metadata", meta_key.as_bytes()).map_err(|e| ModelCacheError::Storage(e.to_string()))? {
            Some(guard) => guard.data,
            None => return Ok(None),
        };
        
        let metadata = rkyv::from_bytes::<FileMetadata, rkyv::rancor::Error>(&meta_bytes)
            .map_err(|e| ModelCacheError::Serialization(e.to_string()))?;
        
        Ok(Some(FileChunkIterator {
            storage: self,
            repo_id: repo_id.to_string(),
            file_path: file_path.to_string(),
            chunk_count: metadata.chunk_count,
            current_chunk: 0,
        }))
    }
}

/// Iterator for streaming file chunks.
pub struct FileChunkIterator<'a> {
    storage: &'a ChunkStorage,
    repo_id: String,
    file_path: String,
    chunk_count: usize,
    current_chunk: usize,
}

impl<'a> Iterator for FileChunkIterator<'a> {
    type Item = Result<Vec<u8>>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_chunk >= self.chunk_count {
            return None;
        }
        
        let chunk_key = format!(
            "file:{}:{}:chunk:{}",
            self.repo_id,
            self.file_path,
            self.current_chunk
        );
        
        let result = self.storage.engine.get("chunks", chunk_key.as_bytes())
            .map_err(|e| ModelCacheError::Storage(e.to_string()))
            .and_then(|opt| {
                opt.map(|guard| guard.data)
                    .ok_or_else(|| ModelCacheError::Download(format!(
                        "Missing chunk {} for file {}",
                        self.current_chunk,
                        self.file_path
                    )))
            });
        
        self.current_chunk += 1;
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_store_and_retrieve() {
        // Test implementation pending
    }
    
    #[test]
    fn test_metadata() {
        // Test implementation pending
    }
}

