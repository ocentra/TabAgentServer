use crate::error::{ModelCacheError, Result};
use sled::Db;
use std::sync::Arc;

const CHUNK_SIZE: usize = 100 * 1024 * 1024; // 100MB chunks (matching extension logic)

/// Manages chunked storage of model files in sled
pub struct ChunkStorage {
    /// Shared reference to the database (to keep it alive)
    _db: Arc<Db>,
    /// Tree for storing file chunks: key = "file:{repo}:{file_path}:chunk:{n}", value = bytes
    chunks: sled::Tree,
    /// Tree for storing file metadata: key = "meta:{repo}:{file_path}", value = FileMetadata
    metadata: sled::Tree,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, bincode::Encode, bincode::Decode)]
pub struct FileMetadata {
    pub repo_id: String,
    pub file_path: String,
    pub total_size: u64,
    pub chunk_count: usize,
    pub content_type: Option<String>,
    pub etag: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
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
    /// Create a new ChunkStorage using an existing database instance
    pub fn new(db: Arc<Db>) -> Result<Self> {
        let chunks = db.open_tree(b"model_chunks")?;
        let metadata = db.open_tree(b"model_metadata")?;
        
        Ok(Self {
            _db: db,
            chunks,
            metadata,
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
            self.chunks.insert(chunk_key.as_bytes(), chunk)?;
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
        let meta_bytes = bincode::encode_to_vec(&metadata, bincode::config::standard())?;
        self.metadata.insert(meta_key.as_bytes(), meta_bytes)?;
        
        self._db.flush()?;
        
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
        
        // 1. Create and save manifest FIRST (like extension does)
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
        let manifest_bytes = bincode::encode_to_vec(&manifest, bincode::config::standard())?;
        self.metadata.insert(manifest_key.as_bytes(), manifest_bytes)?;
        
        // 2. Stream and save chunks (exactly like extension's saveChunkedFileSafe)
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
                    self.chunks.insert(chunk_key.as_bytes(), &current_chunk_buffer[..current_chunk_offset])?;
                    
                    total_bytes_processed += current_chunk_offset as u64;
                    chunk_index += 1;
                    current_chunk_offset = 0;
                    
                    // Report progress every 20 chunks (like extension)
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
        
        // 3. **CRITICAL**: Save the last partial chunk if there's remaining data
        //    (Extension lines 1172-1184)
        if current_chunk_offset > 0 {
            let chunk_key = format!("file:{}:{}:chunk:{}", repo_id, file_path, chunk_index);
            self.chunks.insert(chunk_key.as_bytes(), &current_chunk_buffer[..current_chunk_offset])?;
            
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
        let meta_bytes = bincode::encode_to_vec(&metadata, bincode::config::standard())?;
        self.metadata.insert(meta_key.as_bytes(), meta_bytes)?;
        
        // 5. Update manifest status to "present"
        let mut final_manifest = manifest;
        final_manifest.status = "present".to_string();
        final_manifest.total_chunks = chunk_index;
        let manifest_bytes = bincode::encode_to_vec(&final_manifest, bincode::config::standard())?;
        self.metadata.insert(manifest_key.as_bytes(), manifest_bytes)?;
        
        self._db.flush()?;
        
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
        let meta_bytes = match self.metadata.get(meta_key.as_bytes())? {
            Some(bytes) => bytes,
            None => return Ok(None),
        };
        
        let (metadata, _): (FileMetadata, usize) = bincode::decode_from_slice(&meta_bytes, bincode::config::standard())?;
        
        // Reconstruct file from chunks
        let mut file_data = Vec::with_capacity(metadata.total_size as usize);
        
        for i in 0..metadata.chunk_count {
            let chunk_key = format!("file:{}:{}:chunk:{}", repo_id, file_path, i);
            let chunk_bytes = self.chunks.get(chunk_key.as_bytes())?
                .ok_or_else(|| ModelCacheError::Download(format!("Missing chunk {} for file {}", i, file_path)))?;
            
            file_data.extend_from_slice(&chunk_bytes);
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
        let meta_bytes = match self.metadata.get(meta_key.as_bytes())? {
            Some(bytes) => bytes,
            None => return Ok(None),
        };
        
        let (metadata, _): (FileMetadata, usize) = bincode::decode_from_slice(&meta_bytes, bincode::config::standard())?;
        
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
            let chunk_bytes = self.chunks.get(chunk_key.as_bytes())?
                .ok_or_else(|| ModelCacheError::Download(format!("Missing chunk {} for file {}", i, file_path)))?;
            
            temp_file.write_all(&chunk_bytes)?;
        }
        
        temp_file.flush()?;
        
        Ok(Some(temp_file_path))
    }
    
    /// Get file metadata
    pub fn get_metadata(&self, repo_id: &str, file_path: &str) -> Result<Option<FileMetadata>> {
        let meta_key = format!("meta:{}:{}", repo_id, file_path);
        let meta_bytes = match self.metadata.get(meta_key.as_bytes())? {
            Some(bytes) => bytes,
            None => return Ok(None),
        };
        
        let (metadata, _): (FileMetadata, usize) = bincode::decode_from_slice(&meta_bytes, bincode::config::standard())?;
        Ok(Some(metadata))
    }
    
    /// Check if a file exists
    pub fn has_file(&self, repo_id: &str, file_path: &str) -> Result<bool> {
        let meta_key = format!("meta:{}:{}", repo_id, file_path);
        Ok(self.metadata.contains_key(meta_key.as_bytes())?)
    }
    
    /// Delete a file and its chunks
    pub fn delete_file(&self, repo_id: &str, file_path: &str) -> Result<()> {
        // Get metadata to know how many chunks to delete
        let meta_key = format!("meta:{}:{}", repo_id, file_path);
        if let Some(meta_bytes) = self.metadata.get(meta_key.as_bytes())? {
            let (metadata, _): (FileMetadata, usize) = bincode::decode_from_slice(&meta_bytes, bincode::config::standard())?;
            
            // Delete all chunks
            for i in 0..metadata.chunk_count {
                let chunk_key = format!("file:{}:{}:chunk:{}", repo_id, file_path, i);
                self.chunks.remove(chunk_key.as_bytes())?;
            }
        }
        
        // Delete metadata
        self.metadata.remove(meta_key.as_bytes())?;
        
        self._db.flush()?;
        
        Ok(())
    }
    
    /// List all files for a repo
    pub fn list_files(&self, repo_id: &str) -> Result<Vec<String>> {
        let prefix = format!("meta:{}:", repo_id);
        let mut files = Vec::new();
        
        for item in self.metadata.scan_prefix(prefix.as_bytes()) {
            let (_key, value) = item?;
            let (metadata, _): (FileMetadata, usize) = bincode::decode_from_slice(&value, bincode::config::standard())?;
            files.push(metadata.file_path);
        }
        
        Ok(files)
    }
    
    /// Get total storage size for a repo
    pub fn get_repo_size(&self, repo_id: &str) -> Result<u64> {
        let prefix = format!("meta:{}:", repo_id);
        let mut total = 0u64;
        
        for item in self.metadata.scan_prefix(prefix.as_bytes()) {
            let (_key, value) = item?;
            let (metadata, _): (FileMetadata, usize) = bincode::decode_from_slice(&value, bincode::config::standard())?;
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
    /// **Resource note:**
    /// - Sled stays open for application lifetime (~5-10MB overhead)
    /// - Flush = sync writes, NOT close connection
    /// - RAII handles cleanup on Drop
    pub fn flush(&self) -> Result<()> {
        self._db.flush()
            .map_err(|e| ModelCacheError::Storage(format!("Failed to flush: {}", e)))?;
        Ok(())
    }
    
    /// Stream a file chunk-by-chunk (NO RAM OVERHEAD for large files)
    /// 
    /// This is the KEY to serving 20GB models without loading them into memory
    /// Returns an iterator over chunks
    pub fn stream_file_chunks<'a>(
        &'a self,
        repo_id: &'a str,
        file_path: &'a str,
    ) -> Result<Option<FileChunkIterator<'a>>> {
        // Get metadata first
        let meta_key = format!("meta:{}:{}", repo_id, file_path);
        let meta_bytes = match self.metadata.get(meta_key.as_bytes())? {
            Some(bytes) => bytes,
            None => return Ok(None),
        };
        
        let (metadata, _): (FileMetadata, usize) = bincode::decode_from_slice(&meta_bytes, bincode::config::standard())?;
        
        Ok(Some(FileChunkIterator {
            storage: self,
            repo_id: repo_id.to_string(),
            file_path: file_path.to_string(),
            chunk_count: metadata.chunk_count,
            current_chunk: 0,
        }))
    }
}

/// Iterator for streaming file chunks without loading entire file into RAM
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
        
        let result = self.storage.chunks.get(chunk_key.as_bytes())
            .map_err(ModelCacheError::from)
            .and_then(|opt| {
                opt.map(|bytes| bytes.to_vec())
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
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_store_and_retrieve() {
        let temp_dir = TempDir::new().unwrap();
        let db = Arc::new(sled::open(temp_dir.path()).unwrap());
        let storage = ChunkStorage::new(db).unwrap();
        
        let test_data = b"Hello, this is a test file for chunked storage!".repeat(100);
        
        storage.store_file(
            "test-repo",
            "model.bin",
            &test_data,
            Some("application/octet-stream".to_string()),
            Some("etag123".to_string()),
        ).unwrap();
        
        let retrieved = storage.get_file("test-repo", "model.bin").unwrap();
        assert_eq!(retrieved, Some(test_data.to_vec()));
    }
    
    #[test]
    fn test_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let db = Arc::new(sled::open(temp_dir.path()).unwrap());
        let storage = ChunkStorage::new(db).unwrap();
        
        let test_data = vec![0u8; 10_000_000]; // 10MB
        
        storage.store_file("test-repo", "large.bin", &test_data, None, None).unwrap();
        
        let meta = storage.get_metadata("test-repo", "large.bin").unwrap().unwrap();
        assert_eq!(meta.total_size, 10_000_000);
        assert_eq!(meta.chunk_count, 1); // 10MB / 100MB chunks = 1 chunk
    }
}

