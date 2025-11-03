/// Configuration for a database in the storage registry
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// Path to the database file (relative to base path or absolute)
    pub path: String,
    
    /// Collections (tables/trees) to create in this database
    pub collections: Vec<String>,
    
    /// Maximum database size in bytes (default: 10GB)
    pub max_size: Option<usize>,
    
    /// Enable read-ahead for this database
    pub enable_readahead: bool,
    
    /// Enable write-map mode (unsafe but faster)
    pub enable_writemap: bool,
}

impl DbConfig {
    /// Create a new database configuration
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            collections: Vec::new(),
            max_size: Some(10 * 1024 * 1024 * 1024), // 10GB default
            enable_readahead: true,
            enable_writemap: false,
        }
    }
    
    /// Add a collection to this database
    pub fn with_collection(mut self, name: impl Into<String>) -> Self {
        self.collections.push(name.into());
        self
    }
    
    /// Add multiple collections to this database
    pub fn with_collections(mut self, names: Vec<String>) -> Self {
        self.collections.extend(names);
        self
    }
    
    /// Set maximum database size
    pub fn with_max_size(mut self, size: usize) -> Self {
        self.max_size = Some(size);
        self
    }
    
    /// Enable or disable read-ahead
    pub fn with_readahead(mut self, enabled: bool) -> Self {
        self.enable_readahead = enabled;
        self
    }
    
    /// Enable or disable write-map mode
    pub fn with_writemap(mut self, enabled: bool) -> Self {
        self.enable_writemap = enabled;
        self
    }
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            path: String::new(),
            collections: Vec::new(),
            max_size: Some(10 * 1024 * 1024 * 1024),
            enable_readahead: true,
            enable_writemap: false,
        }
    }
}

