//! Vector storage backends for the TabAgent indexing system.
//!
//! This module provides multiple storage backends for vector data,
//! including in-memory, memory-mapped, and chunked storage options.
//! These implementations follow the Rust Architecture Guidelines
//! for safety, performance, and clarity.

use common::{DbError, DbResult, EmbeddingId};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use memmap2::MmapMut;

/// Trait for vector storage backends.
pub trait VectorStorage: Send + Sync {
    /// Adds a vector to the storage.
    fn add_vector(&mut self, id: EmbeddingId, vector: Vec<f32>) -> DbResult<()>;
    
    /// Removes a vector from the storage.
    fn remove_vector(&mut self, id: &EmbeddingId) -> DbResult<bool>;
    
    /// Gets a vector from the storage.
    fn get_vector(&self, id: &EmbeddingId) -> DbResult<Option<&Vec<f32>>>;
    
    /// Gets a mutable reference to a vector in the storage.
    fn get_vector_mut(&mut self, id: &EmbeddingId) -> DbResult<Option<&mut Vec<f32>>>;
    
    /// Gets the number of vectors in the storage.
    fn len(&self) -> usize;
    
    /// Checks if the storage is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Gets all vector IDs in the storage.
    fn get_all_ids(&self) -> Vec<EmbeddingId>;
    
    /// Flushes any pending writes to disk.
    fn flush(&mut self) -> DbResult<()>;
}

/// In-memory vector storage backend.
///
/// This storage backend keeps all vectors in memory for maximum performance.
/// It's suitable for small to medium datasets that fit comfortably in RAM.
pub struct InMemoryVectorStorage {
    /// Mapping from embedding ID to vector data
    vectors: HashMap<EmbeddingId, Vec<f32>>,
}

impl InMemoryVectorStorage {
    /// Creates a new in-memory vector storage.
    pub fn new() -> Self {
        Self {
            vectors: HashMap::new(),
        }
    }
    
    /// Creates a new in-memory vector storage with initial capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vectors: HashMap::with_capacity(capacity),
        }
    }
}

impl Default for InMemoryVectorStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorStorage for InMemoryVectorStorage {
    fn add_vector(&mut self, id: EmbeddingId, vector: Vec<f32>) -> DbResult<()> {
        self.vectors.insert(id, vector);
        Ok(())
    }
    
    fn remove_vector(&mut self, id: &EmbeddingId) -> DbResult<bool> {
        Ok(self.vectors.remove(id).is_some())
    }
    
    fn get_vector(&self, id: &EmbeddingId) -> DbResult<Option<&Vec<f32>>> {
        Ok(self.vectors.get(id))
    }
    
    fn get_vector_mut(&mut self, id: &EmbeddingId) -> DbResult<Option<&mut Vec<f32>>> {
        Ok(self.vectors.get_mut(id))
    }
    
    fn len(&self) -> usize {
        self.vectors.len()
    }
    
    fn get_all_ids(&self) -> Vec<EmbeddingId> {
        self.vectors.keys().cloned().collect()
    }
    
    fn flush(&mut self) -> DbResult<()> {
        // Nothing to flush for in-memory storage
        Ok(())
    }
}

/// Memory-mapped vector storage backend.
///
/// This storage backend uses memory-mapped files for efficient storage
/// of large datasets. It provides a good balance between performance
/// and memory usage.
pub struct MmapVectorStorage {
    /// Path to the storage file
    file_path: PathBuf,
    
    /// File handle
    file: File,
    
    /// Memory-mapped region
    mmap: MmapMut,
    
    /// Mapping from embedding ID to file offset
    id_to_offset: HashMap<EmbeddingId, u64>,
    
    /// Mapping from file offset to embedding ID
    offset_to_id: HashMap<u64, EmbeddingId>,
    
    /// Current write position in the file
    write_pos: u64,
    
    /// Vector dimension
    dimension: usize,
}

impl MmapVectorStorage {
    /// Creates a new memory-mapped vector storage.
    pub fn new<P: AsRef<Path>>(file_path: P, dimension: usize) -> DbResult<Self> {
        let file_path = file_path.as_ref().to_path_buf();
        
        // Create or open the file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&file_path)?;
        
        // Get file size
        let file_size = file.metadata()?.len();
        
        // If file is empty, initialize it with header
        if file_size == 0 {
            // Write header: dimension (u32) + reserved space
            let header_size = 4 + 256; // 4 bytes for dimension + 256 bytes reserved
            file.set_len(header_size)?;
            
            // Write dimension to header
            let mut header_data = vec![0u8; 4];
            header_data[0..4].copy_from_slice(&(dimension as u32).to_le_bytes());
            
            let mut file_for_write = OpenOptions::new()
                .write(true)
                .open(&file_path)?;
            file_for_write.write_all(&header_data)?;
        }
        
        // Memory-map the file
        // SAFETY: MmapMut::map_mut is safe because:
        // 1. The file was successfully opened with write permissions
        // 2. The file was created/truncated to the required size above
        // 3. memmap2 guarantees safety when mapping a valid, writable file
        // 4. The file descriptor remains valid for the lifetime of the mapping
        let mmap = unsafe { MmapMut::map_mut(&file)? };
        
        // Load existing mappings from file (simplified implementation)
        let id_to_offset = HashMap::new();
        let offset_to_id = HashMap::new();
        let write_pos = 4 + 256; // Skip header
        
        Ok(Self {
            file_path,
            file,
            mmap,
            id_to_offset,
            offset_to_id,
            write_pos,
            dimension,
        })
    }
    
    /// Reads a vector from the memory-mapped file at the given offset.
    fn read_vector(&self, offset: u64) -> DbResult<Vec<f32>> {
        if offset + (self.dimension as u64) * 4 > self.mmap.len() as u64 {
            return Err(DbError::InvalidOperation("Invalid offset".to_string()));
        }
        
        let start = offset as usize;
        let end = start + self.dimension * 4;
        let bytes = &self.mmap[start..end];
        
        let mut vector = Vec::with_capacity(self.dimension);
        for i in 0..self.dimension {
            let byte_offset = i * 4;
            let bytes_slice = &bytes[byte_offset..byte_offset + 4];
            let value = f32::from_le_bytes([
                bytes_slice[0],
                bytes_slice[1],
                bytes_slice[2],
                bytes_slice[3],
            ]);
            vector.push(value);
        }
        
        Ok(vector)
    }
    
    /// Writes a vector to the memory-mapped file at the current write position.
    fn write_vector(&mut self, vector: &[f32]) -> DbResult<u64> {
        if vector.len() != self.dimension {
            return Err(DbError::InvalidOperation("Vector dimension mismatch".to_string()));
        }
        
        // Ensure the file is large enough
        let required_size = self.write_pos + (self.dimension as u64) * 4;
        if required_size > self.mmap.len() as u64 {
            // Extend the file and remap
            self.file.set_len(required_size)?;
            // SAFETY: MmapMut::map_mut is safe because:
            // 1. The file was successfully extended with set_len
            // 2. The file descriptor remains valid
            // 3. memmap2 guarantees safety when mapping a valid, writable file
            self.mmap = unsafe { MmapMut::map_mut(&self.file)? };
        }
        
        let offset = self.write_pos;
        let start = offset as usize;
        let end = start + self.dimension * 4;
        
        // Write vector data
        for (i, &value) in vector.iter().enumerate() {
            let byte_offset = start + i * 4;
            let bytes = value.to_le_bytes();
            self.mmap[byte_offset..byte_offset + 4].copy_from_slice(&bytes);
        }
        
        // Update write position
        self.write_pos += (self.dimension as u64) * 4;
        
        Ok(offset)
    }
}

impl VectorStorage for MmapVectorStorage {
    fn add_vector(&mut self, id: EmbeddingId, vector: Vec<f32>) -> DbResult<()> {
        let offset = self.write_vector(&vector)?;
        self.id_to_offset.insert(id.clone(), offset);
        self.offset_to_id.insert(offset, id);
        Ok(())
    }
    
    fn remove_vector(&mut self, id: &EmbeddingId) -> DbResult<bool> {
        if let Some(offset) = self.id_to_offset.remove(id) {
            self.offset_to_id.remove(&offset);
            // In a real implementation, we would mark the space as free
            // for reuse, but for simplicity we'll just leave it
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    fn get_vector(&self, id: &EmbeddingId) -> DbResult<Option<&Vec<f32>>> {
        if let Some(&offset) = self.id_to_offset.get(id) {
            // In a real implementation, we would cache the vectors
            // For now, we'll return None to indicate the vector needs to be read from disk
            Ok(None)
        } else {
            Ok(None)
        }
    }
    
    fn get_vector_mut(&mut self, id: &EmbeddingId) -> DbResult<Option<&mut Vec<f32>>> {
        // Memory-mapped storage doesn't keep vectors in memory
        Ok(None)
    }
    
    fn len(&self) -> usize {
        self.id_to_offset.len()
    }
    
    fn get_all_ids(&self) -> Vec<EmbeddingId> {
        self.id_to_offset.keys().cloned().collect()
    }
    
    fn flush(&mut self) -> DbResult<()> {
        self.mmap.flush()?;
        Ok(())
    }
}

/// Chunked vector storage backend.
///
/// This storage backend divides vectors into chunks for efficient memory usage.
/// It's suitable for very large datasets that don't fit in memory.
pub struct ChunkedVectorStorage {
    /// Path to the storage directory
    storage_dir: PathBuf,
    
    /// Chunk size (number of vectors per chunk)
    chunk_size: usize,
    
    /// Vector dimension
    dimension: usize,
    
    /// Mapping from embedding ID to (chunk_id, index_in_chunk)
    id_to_location: HashMap<EmbeddingId, (usize, usize)>,
    
    /// Loaded chunks
    chunks: HashMap<usize, Chunk>,
    
    /// Next chunk ID to use
    next_chunk_id: usize,
    
    /// Next index in the current chunk
    next_index_in_chunk: usize,
}

/// A chunk of vectors stored in memory.
struct Chunk {
    /// Vectors in this chunk
    vectors: Vec<Vec<f32>>,
    
    /// Chunk ID
    id: usize,
    
    /// Whether this chunk has been modified
    dirty: bool,
}

impl Chunk {
    /// Creates a new chunk.
    fn new(id: usize) -> Self {
        Self {
            vectors: Vec::new(),
            id,
            dirty: false,
        }
    }
    
    /// Adds a vector to the chunk.
    fn add_vector(&mut self, vector: Vec<f32>) -> usize {
        let index = self.vectors.len();
        self.vectors.push(vector);
        self.dirty = true;
        index
    }
    
    /// Gets a vector from the chunk.
    fn get_vector(&self, index: usize) -> Option<&Vec<f32>> {
        self.vectors.get(index)
    }
    
    /// Gets a mutable reference to a vector in the chunk.
    fn get_vector_mut(&mut self, index: usize) -> Option<&mut Vec<f32>> {
        self.vectors.get_mut(index)
    }
}

impl ChunkedVectorStorage {
    /// Creates a new chunked vector storage.
    pub fn new<P: AsRef<Path>>(storage_dir: P, chunk_size: usize, dimension: usize) -> DbResult<Self> {
        let storage_dir = storage_dir.as_ref().to_path_buf();
        
        // Create the storage directory if it doesn't exist
        std::fs::create_dir_all(&storage_dir)?;
        
        Ok(Self {
            storage_dir,
            chunk_size,
            dimension,
            id_to_location: HashMap::new(),
            chunks: HashMap::new(),
            next_chunk_id: 0,
            next_index_in_chunk: 0,
        })
    }
    
    /// Gets or creates the current chunk.
    fn get_current_chunk(&mut self) -> &mut Chunk {
        self.chunks.entry(self.next_chunk_id).or_insert_with(|| Chunk::new(self.next_chunk_id))
    }
    
    /// Saves a chunk to disk.
    fn save_chunk(&self, chunk: &Chunk) -> DbResult<()> {
        let chunk_file_path = self.storage_dir.join(format!("chunk_{}.bin", chunk.id));
        let mut file = File::create(chunk_file_path)?;
        
        // Write chunk header: number of vectors (u32) + dimension (u32)
        let header = [
            (chunk.vectors.len() as u32).to_le_bytes(),
            (self.dimension as u32).to_le_bytes(),
        ];
        file.write_all(&header[0])?;
        file.write_all(&header[1])?;
        
        // Write vectors
        for vector in &chunk.vectors {
            for &value in vector {
                file.write_all(&value.to_le_bytes())?;
            }
        }
        
        Ok(())
    }
    
    /// Loads a chunk from disk.
    fn load_chunk(&self, chunk_id: usize) -> DbResult<Chunk> {
        let chunk_file_path = self.storage_dir.join(format!("chunk_{}.bin", chunk_id));
        
        if !chunk_file_path.exists() {
            return Err(DbError::NotFound(format!("Chunk {} not found", chunk_id)));
        }
        
        let mut file = File::open(chunk_file_path)?;
        
        // Read chunk header
        let mut header = [0u8; 8];
        file.read_exact(&mut header)?;
        
        let num_vectors = u32::from_le_bytes([header[0], header[1], header[2], header[3]]) as usize;
        let dimension = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;
        
        if dimension != self.dimension {
            return Err(DbError::InvalidOperation("Dimension mismatch".to_string()));
        }
        
        // Read vectors
        let mut chunk = Chunk::new(chunk_id);
        chunk.vectors.reserve(num_vectors);
        
        for _ in 0..num_vectors {
            let mut vector = Vec::with_capacity(dimension);
            for _ in 0..dimension {
                let mut bytes = [0u8; 4];
                file.read_exact(&mut bytes)?;
                let value = f32::from_le_bytes(bytes);
                vector.push(value);
            }
            chunk.vectors.push(vector);
        }
        
        Ok(chunk)
    }
}

impl VectorStorage for ChunkedVectorStorage {
    fn add_vector(&mut self, id: EmbeddingId, vector: Vec<f32>) -> DbResult<()> {
        if vector.len() != self.dimension {
            return Err(DbError::InvalidOperation("Vector dimension mismatch".to_string()));
        }
        
        // Get current chunk
        let chunk = self.get_current_chunk();
        
        // Add vector to chunk
        let index_in_chunk = chunk.add_vector(vector);
        
        // Record location
        self.id_to_location.insert(id.clone(), (self.next_chunk_id, index_in_chunk));
        
        // Update indices
        self.next_index_in_chunk += 1;
        if self.next_index_in_chunk >= self.chunk_size {
            // Save current chunk
            if let Some(chunk) = self.chunks.get(&self.next_chunk_id) {
                self.save_chunk(chunk)?;
            }
            
            // Move to next chunk
            self.next_chunk_id += 1;
            self.next_index_in_chunk = 0;
        }
        
        Ok(())
    }
    
    fn remove_vector(&mut self, id: &EmbeddingId) -> DbResult<bool> {
        if let Some((chunk_id, index_in_chunk)) = self.id_to_location.remove(id) {
            // In a real implementation, we would mark the vector as deleted
            // For now, we'll just remove the mapping
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    fn get_vector(&self, id: &EmbeddingId) -> DbResult<Option<&Vec<f32>>> {
        if let Some((chunk_id, index_in_chunk)) = self.id_to_location.get(id) {
            // Load chunk if not already loaded
            let chunk = if let Some(chunk) = self.chunks.get(chunk_id) {
                chunk
            } else {
                // In a real implementation, we would load the chunk from disk
                // For now, we'll return None
                return Ok(None);
            };
            
            Ok(chunk.get_vector(*index_in_chunk))
        } else {
            Ok(None)
        }
    }
    
    fn get_vector_mut(&mut self, id: &EmbeddingId) -> DbResult<Option<&mut Vec<f32>>> {
        if let Some((chunk_id, index_in_chunk)) = self.id_to_location.get(id) {
            // Load chunk if not already loaded
            let chunk = if let Some(chunk) = self.chunks.get_mut(chunk_id) {
                chunk
            } else {
                // In a real implementation, we would load the chunk from disk
                // For now, we'll return None
                return Ok(None);
            };
            
            Ok(chunk.get_vector_mut(*index_in_chunk))
        } else {
            Ok(None)
        }
    }
    
    fn len(&self) -> usize {
        self.id_to_location.len()
    }
    
    fn get_all_ids(&self) -> Vec<EmbeddingId> {
        self.id_to_location.keys().cloned().collect()
    }
    
    fn flush(&mut self) -> DbResult<()> {
        // Save all dirty chunks
        for chunk in self.chunks.values() {
            if chunk.dirty {
                self.save_chunk(chunk)?;
            }
        }
        Ok(())
    }
}

/// Enum for different vector storage backends.
pub enum VectorStorageBackend {
    /// In-memory storage
    InMemory(InMemoryVectorStorage),
    
    /// Memory-mapped storage
    Mmap(MmapVectorStorage),
    
    /// Chunked storage
    Chunked(ChunkedVectorStorage),
}

impl VectorStorage for VectorStorageBackend {
    fn add_vector(&mut self, id: EmbeddingId, vector: Vec<f32>) -> DbResult<()> {
        match self {
            VectorStorageBackend::InMemory(storage) => storage.add_vector(id, vector),
            VectorStorageBackend::Mmap(storage) => storage.add_vector(id, vector),
            VectorStorageBackend::Chunked(storage) => storage.add_vector(id, vector),
        }
    }
    
    fn remove_vector(&mut self, id: &EmbeddingId) -> DbResult<bool> {
        match self {
            VectorStorageBackend::InMemory(storage) => storage.remove_vector(id),
            VectorStorageBackend::Mmap(storage) => storage.remove_vector(id),
            VectorStorageBackend::Chunked(storage) => storage.remove_vector(id),
        }
    }
    
    fn get_vector(&self, id: &EmbeddingId) -> DbResult<Option<&Vec<f32>>> {
        match self {
            VectorStorageBackend::InMemory(storage) => storage.get_vector(id),
            VectorStorageBackend::Mmap(storage) => storage.get_vector(id),
            VectorStorageBackend::Chunked(storage) => storage.get_vector(id),
        }
    }
    
    fn get_vector_mut(&mut self, id: &EmbeddingId) -> DbResult<Option<&mut Vec<f32>>> {
        match self {
            VectorStorageBackend::InMemory(storage) => storage.get_vector_mut(id),
            VectorStorageBackend::Mmap(storage) => storage.get_vector_mut(id),
            VectorStorageBackend::Chunked(storage) => storage.get_vector_mut(id),
        }
    }
    
    fn len(&self) -> usize {
        match self {
            VectorStorageBackend::InMemory(storage) => storage.len(),
            VectorStorageBackend::Mmap(storage) => storage.len(),
            VectorStorageBackend::Chunked(storage) => storage.len(),
        }
    }
    
    fn get_all_ids(&self) -> Vec<EmbeddingId> {
        match self {
            VectorStorageBackend::InMemory(storage) => storage.get_all_ids(),
            VectorStorageBackend::Mmap(storage) => storage.get_all_ids(),
            VectorStorageBackend::Chunked(storage) => storage.get_all_ids(),
        }
    }
    
    fn flush(&mut self) -> DbResult<()> {
        match self {
            VectorStorageBackend::InMemory(storage) => storage.flush(),
            VectorStorageBackend::Mmap(storage) => storage.flush(),
            VectorStorageBackend::Chunked(storage) => storage.flush(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_in_memory_storage() {
        let mut storage = InMemoryVectorStorage::new();
        
        let id = EmbeddingId::from("test_vector");
        let vector = vec![1.0, 2.0, 3.0];
        
        // Add vector
        assert!(storage.add_vector(id.clone(), vector.clone()).is_ok());
        assert_eq!(storage.len(), 1);
        
        // Get vector
        let retrieved = storage.get_vector(&id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &vector);
        
        // Remove vector
        assert!(storage.remove_vector(&id).unwrap());
        assert_eq!(storage.len(), 0);
        assert!(storage.get_vector(&id).unwrap().is_none());
    }
    
    #[test]
    fn test_mmap_storage() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_vectors.bin");
        
        let mut storage = MmapVectorStorage::new(&file_path, 3).unwrap();
        
        let id = EmbeddingId::from("test_vector");
        let vector = vec![1.0, 2.0, 3.0];
        
        // Add vector
        assert!(storage.add_vector(id.clone(), vector.clone()).is_ok());
        assert_eq!(storage.len(), 1);
        
        // Remove vector
        assert!(storage.remove_vector(&id).unwrap());
        assert_eq!(storage.len(), 0);
    }
    
    #[test]
    fn test_chunked_storage() {
        let temp_dir = TempDir::new().unwrap();
        
        let mut storage = ChunkedVectorStorage::new(&temp_dir, 10, 3).unwrap();
        
        let id = EmbeddingId::from("test_vector");
        let vector = vec![1.0, 2.0, 3.0];
        
        // Add vector
        assert!(storage.add_vector(id.clone(), vector.clone()).is_ok());
        assert_eq!(storage.len(), 1);
        
        // Remove vector
        assert!(storage.remove_vector(&id).unwrap());
        assert_eq!(storage.len(), 0);
    }
}