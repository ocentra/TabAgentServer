/// Model Cache Database Schema System
/// 
/// This module defines the structured schema for model storage,
/// matching the extension's disciplined IndexedDB architecture.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Current schema version for migrations
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Database names (matching extension's DBNames)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DatabaseName {
    /// Model file storage and cache
    Models,
    /// Model manifests and metadata
    Manifests,
    /// User inference settings
    Settings,
}

impl DatabaseName {
    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseName::Models => "TabAgentModels",
            DatabaseName::Manifests => "TabAgentManifests",
            DatabaseName::Settings => "TabAgentSettings",
        }
    }
}

/// Store names within databases
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StoreName {
    /// File chunks (keyPath: url)
    Files,
    /// Repository manifests (keyPath: repo)
    Manifest,
    /// Chunk metadata (keyPath: "{repo}:{file}:manifest")
    ChunkManifest,
    /// Inference settings (keyPath: id)
    InferenceSettings,
}

impl StoreName {
    pub fn as_str(&self) -> &'static str {
        match self {
            StoreName::Files => "files",
            StoreName::Manifest => "manifest",
            StoreName::ChunkManifest => "chunk_manifest",
            StoreName::InferenceSettings => "inferenceSettings",
        }
    }
    
    pub fn key_path(&self) -> &'static str {
        match self {
            StoreName::Files => "url",
            StoreName::Manifest => "repo",
            StoreName::ChunkManifest => "id",
            StoreName::InferenceSettings => "id",
        }
    }
}

/// Quant status enum (matching extension's QuantStatus)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuantStatus {
    /// Model files are available on HuggingFace
    Available,
    /// Currently downloading
    Downloading,
    /// Model files are downloaded and cached locally
    Downloaded,
    /// Download failed
    Failed,
    /// Quant variant not found in repo
    NotFound,
    /// Model unavailable
    Unavailable,
    /// Model type not supported by transformers.js
    Unsupported,
    /// Model too large for browser, server-only
    ServerOnly,
}

/// Schema definition for a store
#[derive(Debug, Clone)]
pub struct StoreSchema {
    pub name: StoreName,
    pub key_path: String,
    pub version: u32,
}

/// Schema definition for a database
#[derive(Debug, Clone)]
pub struct DatabaseSchema {
    pub name: DatabaseName,
    pub version: u32,
    pub stores: Vec<StoreSchema>,
}

/// Complete model cache schema (matching extension's modelCacheSchema)
pub struct ModelCacheSchema {
    databases: HashMap<DatabaseName, DatabaseSchema>,
}

impl ModelCacheSchema {
    pub fn new() -> Self {
        let mut databases = HashMap::new();
        
        // Model Cache Database (files, manifest, settings)
        databases.insert(
            DatabaseName::Models,
            DatabaseSchema {
                name: DatabaseName::Models,
                version: CURRENT_SCHEMA_VERSION,
                stores: vec![
                    StoreSchema {
                        name: StoreName::Files,
                        key_path: StoreName::Files.key_path().to_string(),
                        version: CURRENT_SCHEMA_VERSION,
                    },
                    StoreSchema {
                        name: StoreName::Manifest,
                        key_path: StoreName::Manifest.key_path().to_string(),
                        version: CURRENT_SCHEMA_VERSION,
                    },
                    StoreSchema {
                        name: StoreName::ChunkManifest,
                        key_path: StoreName::ChunkManifest.key_path().to_string(),
                        version: CURRENT_SCHEMA_VERSION,
                    },
                    StoreSchema {
                        name: StoreName::InferenceSettings,
                        key_path: StoreName::InferenceSettings.key_path().to_string(),
                        version: CURRENT_SCHEMA_VERSION,
                    },
                ],
            },
        );
        
        Self { databases }
    }
    
    pub fn get_database(&self, name: &DatabaseName) -> Option<&DatabaseSchema> {
        self.databases.get(name)
    }
    
    pub fn databases(&self) -> &HashMap<DatabaseName, DatabaseSchema> {
        &self.databases
    }
}

impl Default for ModelCacheSchema {
    fn default() -> Self {
        Self::new()
    }
}

/// Schema validation result
#[derive(Debug)]
pub struct SchemaValidation {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl SchemaValidation {
    pub fn success() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    pub fn add_error(&mut self, error: String) {
        self.valid = false;
        self.errors.push(error);
    }
    
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_schema_creation() {
        let schema = ModelCacheSchema::new();
        let models_db = schema.get_database(&DatabaseName::Models).unwrap();
        
        assert_eq!(models_db.version, CURRENT_SCHEMA_VERSION);
        assert_eq!(models_db.stores.len(), 4);
        
        let files_store = models_db.stores.iter()
            .find(|s| s.name == StoreName::Files)
            .unwrap();
        assert_eq!(files_store.key_path, "url");
    }
    
    #[test]
    fn test_database_names() {
        assert_eq!(DatabaseName::Models.as_str(), "TabAgentModels");
        assert_eq!(StoreName::Files.as_str(), "files");
        assert_eq!(StoreName::Manifest.key_path(), "repo");
    }
}

