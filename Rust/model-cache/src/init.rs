/// Database Initialization System
/// 
/// Matches extension's initialization flow from db.ts:
/// 1. Create/Open databases
/// 2. Validate schema (stores, versions)
/// 3. Run migrations if needed
/// 4. Set ready state

use crate::error::{ModelCacheError, Result};
use crate::schema::{
    ModelCacheSchema, DatabaseName, StoreName, SchemaValidation,
    CURRENT_SCHEMA_VERSION,
};
use sled::Db;
use std::path::Path;
use std::collections::HashMap;

/// Initialization state
#[derive(Debug, Clone, PartialEq)]
pub enum InitState {
    NotInitialized,
    Initializing,
    Ready,
    Failed(String),
}

/// Database coordinator - manages all databases systematically
pub struct DatabaseCoordinator {
    /// Main database instance
    db: Db,
    /// Schema definition
    schema: ModelCacheSchema,
    /// Tree handles (organized by StoreName)
    trees: HashMap<StoreName, sled::Tree>,
    /// Current initialization state
    state: InitState,
    /// Schema version stored in DB
    stored_version: u32,
}

impl DatabaseCoordinator {
    /// Initialize database system with schema validation
    /// 
    /// This matches extension's initialization flow:
    /// 1. Open DB
    /// 2. Check stored version
    /// 3. Run migrations if needed
    /// 4. Open all trees
    /// 5. Validate schema
    pub fn initialize<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        log::info!("[DatabaseCoordinator] Starting initialization...");
        
        // 1. Open main database
        let db = sled::open(db_path.as_ref())
            .map_err(|e| ModelCacheError::Storage(format!("Failed to open database: {}", e)))?;
        
        // 2. Load schema
        let schema = ModelCacheSchema::new();
        
        // 3. Check stored version
        let stored_version = Self::get_stored_version(&db)?;
        log::info!("[DatabaseCoordinator] Stored version: {}, Current version: {}", 
                   stored_version, CURRENT_SCHEMA_VERSION);
        
        // 4. Run migrations if needed
        if stored_version < CURRENT_SCHEMA_VERSION {
            log::warn!("[DatabaseCoordinator] Schema upgrade needed: {} → {}", 
                      stored_version, CURRENT_SCHEMA_VERSION);
            Self::run_migrations(&db, stored_version, CURRENT_SCHEMA_VERSION)?;
            Self::set_stored_version(&db, CURRENT_SCHEMA_VERSION)?;
        }
        
        // 5. Open all trees (stores)
        let mut trees = HashMap::new();
        
        // Files tree - stores model file chunks
        let files_tree = db.open_tree(b"model_files")
            .map_err(|e| ModelCacheError::Storage(format!("Failed to open files tree: {}", e)))?;
        trees.insert(StoreName::Files, files_tree);
        
        // Manifest tree - stores repo manifests
        let manifest_tree = db.open_tree(b"model_manifests")
            .map_err(|e| ModelCacheError::Storage(format!("Failed to open manifest tree: {}", e)))?;
        trees.insert(StoreName::Manifest, manifest_tree);
        
        // Chunk manifest tree - stores chunk metadata
        let chunk_manifest_tree = db.open_tree(b"chunk_manifests")
            .map_err(|e| ModelCacheError::Storage(format!("Failed to open chunk manifest tree: {}", e)))?;
        trees.insert(StoreName::ChunkManifest, chunk_manifest_tree);
        
        // Settings tree - stores user settings
        let settings_tree = db.open_tree(b"inference_settings")
            .map_err(|e| ModelCacheError::Storage(format!("Failed to open settings tree: {}", e)))?;
        trees.insert(StoreName::InferenceSettings, settings_tree);
        
        // 6. Validate schema
        let validation = Self::validate_schema(&schema, &trees);
        if !validation.valid {
            let error_msg = format!("Schema validation failed: {:?}", validation.errors);
            log::error!("[DatabaseCoordinator] {}", error_msg);
            return Err(ModelCacheError::Storage(error_msg));
        }
        
        if !validation.warnings.is_empty() {
            for warning in &validation.warnings {
                log::warn!("[DatabaseCoordinator] Schema warning: {}", warning);
            }
        }
        
        log::info!("[DatabaseCoordinator] ✅ Initialization complete. All stores validated.");
        
        Ok(Self {
            db,
            schema,
            trees,
            state: InitState::Ready,
            stored_version: CURRENT_SCHEMA_VERSION,
        })
    }
    
    /// Get a tree by store name
    pub fn get_tree(&self, store: &StoreName) -> Result<&sled::Tree> {
        self.trees.get(store)
            .ok_or_else(|| ModelCacheError::Storage(format!("Store {:?} not initialized", store)))
    }
    
    /// Get current initialization state
    pub fn state(&self) -> &InitState {
        &self.state
    }
    
    /// Get schema version
    pub fn version(&self) -> u32 {
        self.stored_version
    }
    
    /// Flush all trees to disk
    pub fn flush(&self) -> Result<()> {
        self.db.flush()
            .map_err(|e| ModelCacheError::Storage(format!("Failed to flush database: {}", e)))?;
        Ok(())
    }
    
    // ===== Private Helper Methods =====
    
    /// Get stored schema version from metadata
    fn get_stored_version(db: &Db) -> Result<u32> {
        const VERSION_KEY: &[u8] = b"__schema_version__";
        
        match db.get(VERSION_KEY)? {
            Some(bytes) => {
                let version_bytes: [u8; 4] = bytes.as_ref()
                    .try_into()
                    .map_err(|_| ModelCacheError::Storage("Invalid version bytes".to_string()))?;
                Ok(u32::from_le_bytes(version_bytes))
            }
            None => {
                // First time initialization
                log::info!("[DatabaseCoordinator] No stored version found, initializing fresh database");
                Ok(0)
            }
        }
    }
    
    /// Set stored schema version
    fn set_stored_version(db: &Db, version: u32) -> Result<()> {
        const VERSION_KEY: &[u8] = b"__schema_version__";
        db.insert(VERSION_KEY, &version.to_le_bytes())?;
        db.flush()?;
        log::info!("[DatabaseCoordinator] Stored version updated to: {}", version);
        Ok(())
    }
    
    /// Run schema migrations
    fn run_migrations(_db: &Db, from_version: u32, to_version: u32) -> Result<()> {
        log::info!("[DatabaseCoordinator] Running migrations: {} → {}", from_version, to_version);
        
        // Future migrations will go here
        match (from_version, to_version) {
            (0, 1) => {
                log::info!("[DatabaseCoordinator] Migration 0→1: Initial schema setup");
                // Initial setup, no migration needed
                Ok(())
            }
            _ => {
                log::warn!("[DatabaseCoordinator] No specific migration path for {} → {}", 
                          from_version, to_version);
                Ok(())
            }
        }
    }
    
    /// Validate schema against actual database structure
    fn validate_schema(
        schema: &ModelCacheSchema,
        trees: &HashMap<StoreName, sled::Tree>,
    ) -> SchemaValidation {
        let mut validation = SchemaValidation::success();
        
        // Check that all expected stores exist
        let expected_stores = [
            StoreName::Files,
            StoreName::Manifest,
            StoreName::ChunkManifest,
            StoreName::InferenceSettings,
        ];
        
        for store in &expected_stores {
            if !trees.contains_key(store) {
                validation.add_error(format!("Required store {:?} is missing", store));
            }
        }
        
        // Check database schema
        if let Some(db_schema) = schema.get_database(&DatabaseName::Models) {
            if db_schema.stores.len() != trees.len() {
                validation.add_warning(format!(
                    "Store count mismatch: schema has {}, database has {}",
                    db_schema.stores.len(),
                    trees.len()
                ));
            }
        }
        
        validation
    }
}

/// Initialization result matching extension's DbInitializationCompleteNotification
#[derive(Debug)]
pub struct InitializationResult {
    pub success: bool,
    pub version: u32,
    pub stores_initialized: Vec<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl InitializationResult {
    pub fn from_coordinator(coordinator: &DatabaseCoordinator, validation: SchemaValidation) -> Self {
        Self {
            success: coordinator.state() == &InitState::Ready,
            version: coordinator.version(),
            stores_initialized: coordinator.trees.keys()
                .map(|s| format!("{:?}", s))
                .collect(),
            errors: validation.errors,
            warnings: validation.warnings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let coordinator = DatabaseCoordinator::initialize(temp_dir.path()).unwrap();
        
        assert_eq!(coordinator.state(), &InitState::Ready);
        assert_eq!(coordinator.version(), CURRENT_SCHEMA_VERSION);
        
        // Verify all trees exist
        assert!(coordinator.get_tree(&StoreName::Files).is_ok());
        assert!(coordinator.get_tree(&StoreName::Manifest).is_ok());
        assert!(coordinator.get_tree(&StoreName::ChunkManifest).is_ok());
        assert!(coordinator.get_tree(&StoreName::InferenceSettings).is_ok());
    }
    
    #[test]
    fn test_version_persistence() {
        let temp_dir = TempDir::new().unwrap();
        
        // First initialization
        {
            let coordinator = DatabaseCoordinator::initialize(temp_dir.path()).unwrap();
            assert_eq!(coordinator.version(), CURRENT_SCHEMA_VERSION);
        }
        
        // Second initialization (should read stored version)
        {
            let coordinator = DatabaseCoordinator::initialize(temp_dir.path()).unwrap();
            assert_eq!(coordinator.version(), CURRENT_SCHEMA_VERSION);
        }
    }
    
    #[test]
    fn test_flush() {
        let temp_dir = TempDir::new().unwrap();
        let coordinator = DatabaseCoordinator::initialize(temp_dir.path()).unwrap();
        
        // Should flush without errors
        coordinator.flush().unwrap();
    }
}

