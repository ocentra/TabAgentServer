//! Advanced memory mapping support for large datasets.
//!
//! This module provides enhanced memory mapping capabilities for handling
//! very large datasets that don't fit in memory. It includes features like
//! memory-mapped vector storage, memory-mapped payload storage, and
//! memory-mapped graph storage.

use common::{DbError, DbResult, EmbeddingId, NodeId, EdgeId};
use memmap2::{Mmap, MmapMut};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Memory mapping configuration.
#[derive(Debug, Clone)]
pub struct MemoryMapConfig {
    /// Whether to use memory mapping
    pub enabled: bool,
    
    /// Maximum size of the memory-mapped region
    pub max_size: usize,
    
    /// Whether to use copy-on-write mapping
    pub copy_on_write: bool,
    
    /// Whether to populate the mapping eagerly
    pub populate: bool,
    
    /// Page size for alignment
    pub page_size: usize,
}

impl Default for MemoryMapConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_size: 1024 * 1024 * 1024, // 1GB default
            copy_on_write: false,
            populate: false,
            page_size: 4096, // 4KB page size
        }
    }
}

/// Memory-mapped vector storage for large datasets.
pub struct MemoryMappedVectorStorage {
    /// Path to the storage file
    file_path: PathBuf,
    
    /// File handle
    file: File,
    
    /// Memory-mapped region for reading
    mmap: Option<Mmap>,
    
    /// Memory-mapped region for writing
    mmap_mut: Option<MmapMut>,
    
    /// Vector dimension
    dimension: usize,
    
    /// Number of vectors stored
    vector_count: usize,
    
    /// Mapping from embedding ID to file offset
    id_to_offset: HashMap<EmbeddingId, u64>,
    
    /// Mapping from file offset to embedding ID
    offset_to_id: HashMap<u64, EmbeddingId>,
    
    /// Current write position
    write_pos: u64,
    
    /// Configuration
    config: MemoryMapConfig,
}

impl MemoryMappedVectorStorage {
    /// Creates a new memory-mapped vector storage.
    pub fn new<P: AsRef<Path>>(
        file_path: P,
        dimension: usize,
        config: MemoryMapConfig,
    ) -> DbResult<Self> {
        let file_path = file_path.as_ref().to_path_buf();
        
        // Create or open the file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&file_path)?;
        
        // Get file size
        let file_size = file.metadata()?.len();
        
        // Initialize memory mapping
        let (mmap, mmap_mut, vector_count, write_pos) = if file_size > 0 {
            // Load existing data
            let mmap = if config.enabled {
                // SAFETY: Mmap::map is safe because:
                // 1. The file was successfully opened and exists
                // 2. file_size > 0 ensures the file has content
                // 3. memmap2 guarantees safety when mapping a valid, readable file
                // 4. The file descriptor remains valid for the lifetime of the mapping
                Some(unsafe { Mmap::map(&file)? })
            } else {
                None
            };
            
            let mmap_mut = if config.enabled && !config.copy_on_write {
                // SAFETY: MmapMut::map_mut is safe because:
                // 1. The file was successfully opened with write permissions
                // 2. file_size > 0 ensures the file has content
                // 3. memmap2 guarantees safety when mapping a valid, writable file
                // 4. The file descriptor remains valid for the lifetime of the mapping
                Some(unsafe { MmapMut::map_mut(&file)? })
            } else {
                None
            };
            
            // Parse header to get vector count
            let vector_count = if file_size >= 16 {
                let mut header = [0u8; 16];
                let mut file_for_read = File::open(&file_path)?;
                file_for_read.read_exact(&mut header)?;
                
                let count = u64::from_le_bytes([
                    header[0], header[1], header[2], header[3],
                    header[4], header[5], header[6], header[7],
                ]) as usize;
                
                let dim = u64::from_le_bytes([
                    header[8], header[9], header[10], header[11],
                    header[12], header[13], header[14], header[15],
                ]) as usize;
                
                if dim != dimension {
                    return Err(DbError::InvalidOperation(
                        "Dimension mismatch in existing file".to_string()
                    ));
                }
                
                count
            } else {
                0
            };
            
            let write_pos = if vector_count > 0 {
                16 + (vector_count as u64) * (dimension as u64) * 4
            } else {
                16
            };
            
            (mmap, mmap_mut, vector_count, write_pos)
        } else {
            // Create new file with header
            let header = [
                0u64.to_le_bytes(), // vector count
                (dimension as u64).to_le_bytes(), // dimension
            ];
            
            let mut file_for_write = OpenOptions::new()
                .write(true)
                .open(&file_path)?;
            file_for_write.write_all(&header[0])?;
            file_for_write.write_all(&header[1])?;
            
            (None, None, 0, 16)
        };
        
        Ok(Self {
            file_path,
            file,
            mmap,
            mmap_mut,
            dimension,
            vector_count,
            id_to_offset: HashMap::new(),
            offset_to_id: HashMap::new(),
            write_pos,
            config,
        })
    }
    
    /// Reads a vector from the memory-mapped file at the given offset.
    pub fn read_vector(&self, offset: u64) -> DbResult<Vec<f32>> {
        if !self.config.enabled {
            return Err(DbError::InvalidOperation(
                "Memory mapping is disabled".to_string()
            ));
        }
        
        let mmap = self.mmap.as_ref().ok_or_else(|| {
            DbError::InvalidOperation("Memory mapping not initialized".to_string())
        })?;
        
        if offset + (self.dimension as u64) * 4 > mmap.len() as u64 {
            return Err(DbError::InvalidOperation("Invalid offset".to_string()));
        }
        
        let start = offset as usize;
        let end = start + self.dimension * 4;
        let bytes = &mmap[start..end];
        
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
    pub fn write_vector(&mut self, vector: &[f32]) -> DbResult<u64> {
        if vector.len() != self.dimension {
            return Err(DbError::InvalidOperation(
                "Vector dimension mismatch".to_string()
            ));
        }
        
        if !self.config.enabled {
            return Err(DbError::InvalidOperation(
                "Memory mapping is disabled".to_string()
            ));
        }
        
        // Ensure the file is large enough
        let required_size = self.write_pos + (self.dimension as u64) * 4;
        if required_size > self.config.max_size as u64 {
            return Err(DbError::InvalidOperation(
                "Maximum file size exceeded".to_string()
            ));
        }
        
        if required_size > self.file.metadata()?.len() {
            self.file.set_len(required_size)?;
            
            // Initialize or remap the memory mapping
            if self.config.enabled {
                // SAFETY: Mmap::map is safe because:
                // 1. The file was successfully extended with set_len
                // 2. The file descriptor remains valid
                // 3. memmap2 guarantees safety when mapping a valid, readable file
                self.mmap = Some(unsafe { Mmap::map(&self.file)? });
                
                if !self.config.copy_on_write {
                    // SAFETY: MmapMut::map_mut is safe because:
                    // 1. The file was successfully extended with set_len
                    // 2. The file descriptor remains valid
                    // 3. memmap2 guarantees safety when mapping a valid, writable file
                    self.mmap_mut = Some(unsafe { MmapMut::map_mut(&self.file)? });
                }
            }
        }
        
        let offset = self.write_pos;
        let start = offset as usize;
        let _end = start + self.dimension * 4;
        
        // Write vector data
        if let Some(mmap_mut) = self.mmap_mut.as_mut() {
            for (i, &value) in vector.iter().enumerate() {
                let byte_offset = start + i * 4;
                let bytes = value.to_le_bytes();
                mmap_mut[byte_offset..byte_offset + 4].copy_from_slice(&bytes);
            }
        } else {
            // Fallback to regular file I/O
            self.file.seek(SeekFrom::Start(offset))?;
            for &value in vector {
                self.file.write_all(&value.to_le_bytes())?;
            }
        }
        
        // Update write position
        self.write_pos += (self.dimension as u64) * 4;
        self.vector_count += 1;
        
        Ok(offset)
    }
    
    /// Adds a vector to the storage.
    pub fn add_vector(&mut self, id: EmbeddingId, vector: Vec<f32>) -> DbResult<()> {
        let offset = self.write_vector(&vector)?;
        self.id_to_offset.insert(id.clone(), offset);
        self.offset_to_id.insert(offset, id);
        Ok(())
    }
    
    /// Gets a vector from the storage.
    pub fn get_vector(&self, id: &EmbeddingId) -> DbResult<Option<Vec<f32>>> {
        if let Some(&offset) = self.id_to_offset.get(id) {
            let vector = self.read_vector(offset)?;
            Ok(Some(vector))
        } else {
            Ok(None)
        }
    }
    
    /// Removes a vector from the storage.
    pub fn remove_vector(&mut self, id: &EmbeddingId) -> DbResult<bool> {
        if let Some(offset) = self.id_to_offset.remove(id) {
            self.offset_to_id.remove(&offset);
            // In a real implementation, we would mark the space as free
            // for reuse, but for simplicity we'll just leave it
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Flushes the memory-mapped region to disk.
    pub fn flush(&mut self) -> DbResult<()> {
        if let Some(mmap_mut) = self.mmap_mut.as_mut() {
            mmap_mut.flush()?;
        }
        
        // Update header with vector count
        let header = [
            (self.vector_count as u64).to_le_bytes(),
            (self.dimension as u64).to_le_bytes(),
        ];
        
        let mut file_for_write = OpenOptions::new()
            .write(true)
            .open(&self.file_path)?;
        file_for_write.write_all(&header[0])?;
        file_for_write.write_all(&header[1])?;
        
        Ok(())
    }
    
    /// Gets the number of vectors in the storage.
    pub fn len(&self) -> usize {
        self.vector_count
    }
    
    /// Checks if the storage is empty.
    pub fn is_empty(&self) -> bool {
        self.vector_count == 0
    }
}

/// Memory-mapped payload storage.
pub struct MemoryMappedPayloadStorage {
    /// Path to the storage file
    file_path: PathBuf,
    
    /// File handle
    file: File,
    
    /// Memory-mapped region
    mmap: Option<Mmap>,
    
    /// Memory-mapped region for writing
    mmap_mut: Option<MmapMut>,
    
    /// Mapping from ID to file offset
    id_to_offset: HashMap<String, u64>,
    
    /// Current write position
    write_pos: u64,
    
    /// Configuration
    config: MemoryMapConfig,
}

impl MemoryMappedPayloadStorage {
    /// Creates a new memory-mapped payload storage.
    pub fn new<P: AsRef<Path>>(file_path: P, config: MemoryMapConfig) -> DbResult<Self> {
        let file_path = file_path.as_ref().to_path_buf();
        
        // Create or open the file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&file_path)?;
        
        // Get file size
        let file_size = file.metadata()?.len();
        
        // Initialize memory mapping
        let (mmap, mmap_mut, write_pos) = if file_size > 0 {
            let mmap = if config.enabled {
                // SAFETY: Mmap::map is safe because:
                // 1. The file was successfully opened and exists
                // 2. file_size > 0 ensures the file has content
                // 3. memmap2 guarantees safety when mapping a valid, readable file
                Some(unsafe { Mmap::map(&file)? })
            } else {
                None
            };
            
            let mmap_mut = if config.enabled && !config.copy_on_write {
                // SAFETY: MmapMut::map_mut is safe because:
                // 1. The file was successfully opened with write permissions
                // 2. file_size > 0 ensures the file has content
                // 3. memmap2 guarantees safety when mapping a valid, writable file
                Some(unsafe { MmapMut::map_mut(&file)? })
            } else {
                None
            };
            
            let write_pos = file_size;
            
            (mmap, mmap_mut, write_pos)
        } else {
            (None, None, 0)
        };
        
        Ok(Self {
            file_path,
            file,
            mmap,
            mmap_mut,
            id_to_offset: HashMap::new(),
            write_pos,
            config,
        })
    }
    
    /// Writes payload data to the memory-mapped file.
    pub fn write_payload(&mut self, data: &[u8]) -> DbResult<u64> {
        if !self.config.enabled {
            return Err(DbError::InvalidOperation(
                "Memory mapping is disabled".to_string()
            ));
        }
        
        // Ensure the file is large enough
        let data_len = data.len() as u64;
        let required_size = self.write_pos + 8 + data_len; // 8 bytes for length + data
        if required_size > self.config.max_size as u64 {
            return Err(DbError::InvalidOperation(
                "Maximum file size exceeded".to_string()
            ));
        }
        
        if required_size > self.file.metadata()?.len() {
            self.file.set_len(required_size)?;
            
            // Initialize or remap the memory mapping
            if self.config.enabled {
                // SAFETY: Mmap::map is safe because:
                // 1. The file was successfully extended with set_len
                // 2. The file descriptor remains valid
                // 3. memmap2 guarantees safety when mapping a valid, readable file
                self.mmap = Some(unsafe { Mmap::map(&self.file)? });
                
                if !self.config.copy_on_write {
                    // SAFETY: MmapMut::map_mut is safe because:
                    // 1. The file was successfully extended with set_len
                    // 2. The file descriptor remains valid
                    // 3. memmap2 guarantees safety when mapping a valid, writable file
                    self.mmap_mut = Some(unsafe { MmapMut::map_mut(&self.file)? });
                }
            }
        }
        
        let offset = self.write_pos;
        
        // Write length followed by data
        if let Some(mmap_mut) = self.mmap_mut.as_mut() {
            // Write length
            let len_bytes = data_len.to_le_bytes();
            let len_start = offset as usize;
            mmap_mut[len_start..len_start + 8].copy_from_slice(&len_bytes);
            
            // Write data
            let data_start = len_start + 8;
            mmap_mut[data_start..data_start + data.len()].copy_from_slice(data);
        } else {
            // Fallback to regular file I/O
            self.file.seek(SeekFrom::Start(offset))?;
            self.file.write_all(&data_len.to_le_bytes())?;
            self.file.write_all(data)?;
        }
        
        // Update write position
        self.write_pos = offset + 8 + data_len;
        
        Ok(offset)
    }
    
    /// Reads payload data from the memory-mapped file.
    pub fn read_payload(&self, offset: u64) -> DbResult<Vec<u8>> {
        if !self.config.enabled {
            return Err(DbError::InvalidOperation(
                "Memory mapping is disabled".to_string()
            ));
        }
        
        let mmap = self.mmap.as_ref().ok_or_else(|| {
            DbError::InvalidOperation("Memory mapping not initialized".to_string())
        })?;
        
        if offset + 8 > mmap.len() as u64 {
            return Err(DbError::InvalidOperation("Invalid offset".to_string()));
        }
        
        // Read length
        let len_start = offset as usize;
        let len_bytes = &mmap[len_start..len_start + 8];
        let data_len = u64::from_le_bytes([
            len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3],
            len_bytes[4], len_bytes[5], len_bytes[6], len_bytes[7],
        ]) as usize;
        
        // Read data
        let data_start = len_start + 8;
        if data_start + data_len > mmap.len() {
            return Err(DbError::InvalidOperation("Invalid data length".to_string()));
        }
        
        let data = mmap[data_start..data_start + data_len].to_vec();
        Ok(data)
    }
    
    /// Adds payload data to the storage.
    pub fn add_payload(&mut self, id: &str, data: Vec<u8>) -> DbResult<()> {
        let offset = self.write_payload(&data)?;
        self.id_to_offset.insert(id.to_string(), offset);
        Ok(())
    }
    
    /// Gets payload data from the storage.
    pub fn get_payload(&self, id: &str) -> DbResult<Option<Vec<u8>>> {
        if let Some(&offset) = self.id_to_offset.get(id) {
            let data = self.read_payload(offset)?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }
    
    /// Flushes the memory-mapped region to disk.
    pub fn flush(&mut self) -> DbResult<()> {
        if let Some(mmap_mut) = self.mmap_mut.as_mut() {
            mmap_mut.flush()?;
        }
        Ok(())
    }
}

/// Memory-mapped graph storage.
pub struct MemoryMappedGraphStorage {
    /// Path to the nodes file
    nodes_file_path: PathBuf,
    
    /// Path to the edges file
    edges_file_path: PathBuf,
    
    /// Nodes file handle
    nodes_file: File,
    
    /// Edges file handle
    edges_file: File,
    
    /// Memory-mapped region for nodes
    nodes_mmap: Option<Mmap>,
    
    /// Memory-mapped region for edges
    edges_mmap: Option<Mmap>,
    
    /// Memory-mapped region for nodes (mutable)
    nodes_mmap_mut: Option<MmapMut>,
    
    /// Memory-mapped region for edges (mutable)
    edges_mmap_mut: Option<MmapMut>,
    
    /// Number of nodes
    node_count: usize,
    
    /// Number of edges
    edge_count: usize,
    
    /// Current write position for nodes
    nodes_write_pos: u64,
    
    /// Current write position for edges
    edges_write_pos: u64,
    
    /// Configuration
    config: MemoryMapConfig,
}

impl MemoryMappedGraphStorage {
    /// Creates a new memory-mapped graph storage.
    pub fn new<P: AsRef<Path>>(base_path: P, config: MemoryMapConfig) -> DbResult<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        let nodes_file_path = base_path.join("nodes.dat");
        let edges_file_path = base_path.join("edges.dat");
        
        // Create base directory if it doesn't exist
        std::fs::create_dir_all(&base_path)?;
        
        // Create or open the nodes file
        let nodes_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&nodes_file_path)?;
        
        // Create or open the edges file
        let edges_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&edges_file_path)?;
        
        // Get file sizes
        let nodes_file_size = nodes_file.metadata()?.len();
        let edges_file_size = edges_file.metadata()?.len();
        
        // Initialize memory mapping
        let (nodes_mmap, nodes_mmap_mut, nodes_write_pos) = if nodes_file_size > 0 {
            let mmap = if config.enabled {
                // SAFETY: Mmap::map is safe because:
                // 1. The file was successfully opened and exists
                // 2. nodes_file_size > 0 ensures the file has content
                // 3. memmap2 guarantees safety when mapping a valid, readable file
                Some(unsafe { Mmap::map(&nodes_file)? })
            } else {
                None
            };
            
            let mmap_mut = if config.enabled && !config.copy_on_write {
                // SAFETY: MmapMut::map_mut is safe because:
                // 1. The file was successfully opened with write permissions
                // 2. nodes_file_size > 0 ensures the file has content
                // 3. memmap2 guarantees safety when mapping a valid, writable file
                Some(unsafe { MmapMut::map_mut(&nodes_file)? })
            } else {
                None
            };
            
            let write_pos = nodes_file_size;
            
            (mmap, mmap_mut, write_pos)
        } else {
            (None, None, 0)
        };
        
        let (edges_mmap, edges_mmap_mut, edges_write_pos) = if edges_file_size > 0 {
            let mmap = if config.enabled {
                // SAFETY: Mmap::map is safe because:
                // 1. The file was successfully opened and exists
                // 2. edges_file_size > 0 ensures the file has content
                // 3. memmap2 guarantees safety when mapping a valid, readable file
                Some(unsafe { Mmap::map(&edges_file)? })
            } else {
                None
            };
            
            let mmap_mut = if config.enabled && !config.copy_on_write {
                // SAFETY: MmapMut::map_mut is safe because:
                // 1. The file was successfully opened with write permissions
                // 2. edges_file_size > 0 ensures the file has content
                // 3. memmap2 guarantees safety when mapping a valid, writable file
                Some(unsafe { MmapMut::map_mut(&edges_file)? })
            } else {
                None
            };
            
            let write_pos = edges_file_size;
            
            (mmap, mmap_mut, write_pos)
        } else {
            (None, None, 0)
        };
        
        Ok(Self {
            nodes_file_path,
            edges_file_path,
            nodes_file,
            edges_file,
            nodes_mmap,
            edges_mmap,
            nodes_mmap_mut,
            edges_mmap_mut,
            node_count: 0,
            edge_count: 0,
            nodes_write_pos,
            edges_write_pos,
            config,
        })
    }
    
    /// Adds a node to the storage.
    pub fn add_node(&mut self, _node_id: NodeId, _data: Vec<u8>) -> DbResult<()> {
        // Implementation would write node data to the memory-mapped file
        // This is a simplified placeholder
        self.node_count += 1;
        Ok(())
    }
    
    /// Adds an edge to the storage.
    pub fn add_edge(&mut self, _edge_id: EdgeId, _from: NodeId, _to: NodeId, _data: Vec<u8>) -> DbResult<()> {
        // Implementation would write edge data to the memory-mapped file
        // This is a simplified placeholder
        self.edge_count += 1;
        Ok(())
    }
    
    /// Gets a node from the storage.
    pub fn get_node(&self, _node_id: &NodeId) -> DbResult<Option<Vec<u8>>> {
        // Implementation would read node data from the memory-mapped file
        // This is a simplified placeholder
        Ok(None)
    }
    
    /// Gets an edge from the storage.
    pub fn get_edge(&self, _edge_id: &EdgeId) -> DbResult<Option<Vec<u8>>> {
        // Implementation would read edge data from the memory-mapped file
        // This is a simplified placeholder
        Ok(None)
    }
    
    /// Flushes the memory-mapped regions to disk.
    pub fn flush(&mut self) -> DbResult<()> {
        if let Some(mmap_mut) = self.nodes_mmap_mut.as_mut() {
            mmap_mut.flush()?;
        }
        
        if let Some(mmap_mut) = self.edges_mmap_mut.as_mut() {
            mmap_mut.flush()?;
        }
        
        Ok(())
    }
    
    /// Gets the number of nodes in the storage.
    pub fn node_count(&self) -> usize {
        self.node_count
    }
    
    /// Gets the number of edges in the storage.
    pub fn edge_count(&self) -> usize {
        self.edge_count
    }
}

/// A unified memory-mapped storage manager.
pub struct MemoryMappedStorageManager {
    /// Vector storage
    vector_storage: Option<MemoryMappedVectorStorage>,
    
    /// Payload storage
    payload_storage: Option<MemoryMappedPayloadStorage>,
    
    /// Graph storage
    graph_storage: Option<MemoryMappedGraphStorage>,
    
    /// Configuration
    config: MemoryMapConfig,
}

impl MemoryMappedStorageManager {
    /// Creates a new memory-mapped storage manager.
    pub fn new(config: MemoryMapConfig) -> Self {
        Self {
            vector_storage: None,
            payload_storage: None,
            graph_storage: None,
            config,
        }
    }
    
    /// Initializes vector storage.
    pub fn init_vector_storage<P: AsRef<Path>>(
        &mut self,
        file_path: P,
        dimension: usize,
    ) -> DbResult<()> {
        if self.config.enabled {
            self.vector_storage = Some(MemoryMappedVectorStorage::new(
                file_path,
                dimension,
                self.config.clone(),
            )?);
        }
        Ok(())
    }
    
    /// Initializes payload storage.
    pub fn init_payload_storage<P: AsRef<Path>>(&mut self, file_path: P) -> DbResult<()> {
        if self.config.enabled {
            self.payload_storage = Some(MemoryMappedPayloadStorage::new(
                file_path,
                self.config.clone(),
            )?);
        }
        Ok(())
    }
    
    /// Initializes graph storage.
    pub fn init_graph_storage<P: AsRef<Path>>(&mut self, base_path: P) -> DbResult<()> {
        if self.config.enabled {
            self.graph_storage = Some(MemoryMappedGraphStorage::new(
                base_path,
                self.config.clone(),
            )?);
        }
        Ok(())
    }
    
    /// Gets the vector storage.
    pub fn vector_storage(&self) -> Option<&MemoryMappedVectorStorage> {
        self.vector_storage.as_ref()
    }
    
    /// Gets the mutable vector storage.
    pub fn vector_storage_mut(&mut self) -> Option<&mut MemoryMappedVectorStorage> {
        self.vector_storage.as_mut()
    }
    
    /// Gets the payload storage.
    pub fn payload_storage(&self) -> Option<&MemoryMappedPayloadStorage> {
        self.payload_storage.as_ref()
    }
    
    /// Gets the mutable payload storage.
    pub fn payload_storage_mut(&mut self) -> Option<&mut MemoryMappedPayloadStorage> {
        self.payload_storage.as_mut()
    }
    
    /// Gets the graph storage.
    pub fn graph_storage(&self) -> Option<&MemoryMappedGraphStorage> {
        self.graph_storage.as_ref()
    }
    
    /// Gets the mutable graph storage.
    pub fn graph_storage_mut(&mut self) -> Option<&mut MemoryMappedGraphStorage> {
        self.graph_storage.as_mut()
    }
    
    /// Flushes all memory-mapped regions to disk.
    pub fn flush(&mut self) -> DbResult<()> {
        if let Some(vector_storage) = self.vector_storage_mut() {
            vector_storage.flush()?;
        }
        
        if let Some(payload_storage) = self.payload_storage_mut() {
            payload_storage.flush()?;
        }
        
        if let Some(graph_storage) = self.graph_storage_mut() {
            graph_storage.flush()?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_memory_mapped_vector_storage() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("vectors.dat");
        
        let config = MemoryMapConfig::default();
        let mut storage = MemoryMappedVectorStorage::new(&file_path, 3, config).unwrap();
        
        // Add vectors
        let vector1 = vec![1.0, 0.0, 0.0];
        let vector2 = vec![0.0, 1.0, 0.0];
        
        storage.add_vector(EmbeddingId::from("vector1"), vector1.clone()).unwrap();
        storage.add_vector(EmbeddingId::from("vector2"), vector2.clone()).unwrap();
        
        // Get vectors
        let retrieved1 = storage.get_vector(&EmbeddingId::from("vector1")).unwrap();
        let retrieved2 = storage.get_vector(&EmbeddingId::from("vector2")).unwrap();
        
        assert_eq!(retrieved1, Some(vector1));
        assert_eq!(retrieved2, Some(vector2));
        
        // Flush to disk
        storage.flush().unwrap();
        
        // Check that file exists and has content
        assert!(file_path.exists());
        assert!(file_path.metadata().unwrap().len() > 0);
    }
    
    #[test]
    fn test_memory_mapped_payload_storage() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("payloads.dat");
        
        let config = MemoryMapConfig::default();
        let mut storage = MemoryMappedPayloadStorage::new(&file_path, config).unwrap();
        
        // Add payload data
        let data1 = b"test payload 1".to_vec();
        let data2 = b"test payload 2".to_vec();
        
        storage.add_payload("payload1", data1.clone()).unwrap();
        storage.add_payload("payload2", data2.clone()).unwrap();
        
        // Get payload data
        let retrieved1 = storage.get_payload("payload1").unwrap();
        let retrieved2 = storage.get_payload("payload2").unwrap();
        
        assert_eq!(retrieved1, Some(data1));
        assert_eq!(retrieved2, Some(data2));
        
        // Flush to disk
        storage.flush().unwrap();
        
        // Check that file exists and has content
        assert!(file_path.exists());
        assert!(file_path.metadata().unwrap().len() > 0);
    }
    
    #[test]
    fn test_memory_mapped_storage_manager() {
        let temp_dir = TempDir::new().unwrap();
        
        let config = MemoryMapConfig::default();
        let mut manager = MemoryMappedStorageManager::new(config);
        
        // Initialize storages
        let vectors_path = temp_dir.path().join("vectors.dat");
        let payloads_path = temp_dir.path().join("payloads.dat");
        let graph_path = temp_dir.path().join("graph");
        
        manager.init_vector_storage(&vectors_path, 3).unwrap();
        manager.init_payload_storage(&payloads_path).unwrap();
        manager.init_graph_storage(&graph_path).unwrap();
        
        // Check that storages are initialized
        assert!(manager.vector_storage().is_some());
        assert!(manager.payload_storage().is_some());
        assert!(manager.graph_storage().is_some());
        
        // Flush all storages
        manager.flush().unwrap();
    }
}