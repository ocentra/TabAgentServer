//! Model Catalog - Database-backed model registry
//!
//! Stores available models in the ModelCache database (sled)
//! Supports JSON import/export for easy configuration management
//! Can be updated dynamically without recompilation

use crate::error::{ModelCacheError, Result};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::path::Path;

/// Model catalog entry (stored in database)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCatalogEntry {
    /// Unique model ID
    pub id: String,
    
    /// Display name
    pub name: String,
    
    /// HuggingFace repo/checkpoint
    pub repo_id: String,
    
    /// File path within repo (optional for BitNet)
    pub file_path: Option<String>,
    
    /// Model type: gguf, bitnet, onnx, safetensors
    pub model_type: String,
    
    /// Approximate size in GB
    pub size_gb: f32,
    
    /// Tags/labels for filtering
    pub tags: Vec<String>,
    
    /// Is this model suggested/recommended?
    pub suggested: bool,
    
    /// Is this model currently downloaded?
    pub downloaded: bool,
    
    /// Optional description
    pub description: Option<String>,
    
    /// Default quantization level
    pub default_quant: Option<String>,
    
    /// Source: "builtin", "test", "user"
    pub source: Option<String>,
    
    /// Does this model require HuggingFace token?
    pub requires_token: Option<bool>,
}

/// Model catalog configuration (JSON format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCatalogConfig {
    pub version: u32,
    pub models: Vec<ModelCatalogEntry>,
}

/// Model catalog manager
pub struct ModelCatalog {
    db: Db,
}

impl ModelCatalog {
    const CATALOG_TREE: &'static str = "model_catalog";
    const VERSION_KEY: &'static [u8] = b"__catalog_version__";
    
    /// Open or create model catalog
    pub fn open(cache_dir: &Path) -> Result<Self> {
        let db_path = cache_dir.join("model_cache.db");
        let db = sled::open(db_path)?;
        
        Ok(Self { db })
    }
    
    /// Initialize catalog from JSON config file
    pub fn init_from_json(&self, json_path: &Path) -> Result<()> {
        // Read JSON file
        let json_content = std::fs::read_to_string(json_path)
            .map_err(|e| ModelCacheError::Storage(format!("Failed to read catalog JSON: {}", e)))?;
        
        // Parse JSON
        let config: ModelCatalogConfig = serde_json::from_str(&json_content)
            .map_err(|e| ModelCacheError::Storage(format!("Failed to parse catalog JSON: {}", e)))?;
        
        // Store version
        let catalog_tree = self.db.open_tree(Self::CATALOG_TREE)
            .map_err(|e| ModelCacheError::Storage(format!("Failed to open catalog tree: {}", e)))?;
        
        catalog_tree.insert(Self::VERSION_KEY, &config.version.to_le_bytes())
            .map_err(|e| ModelCacheError::Storage(format!("Failed to store catalog version: {}", e)))?;
        
        // Insert all models
        for model in config.models {
            self.insert_model(model)?;
        }
        
        catalog_tree.flush()
            .map_err(|e| ModelCacheError::Storage(format!("Failed to flush catalog: {}", e)))?;
        
        Ok(())
    }
    
    /// Export catalog to JSON
    pub fn export_to_json(&self, json_path: &Path) -> Result<()> {
        let models = self.get_all_models()?;
        
        let catalog_tree = self.db.open_tree(Self::CATALOG_TREE)
            .map_err(|e| ModelCacheError::Storage(format!("Failed to open catalog tree: {}", e)))?;
        
        let version = catalog_tree.get(Self::VERSION_KEY)
            .map_err(|e| ModelCacheError::Storage(format!("Failed to read catalog version: {}", e)))?
            .and_then(|v| {
                if v.len() == 4 {
                    Some(u32::from_le_bytes([v[0], v[1], v[2], v[3]]))
                } else {
                    None
                }
            })
            .unwrap_or(1);
        
        let config = ModelCatalogConfig {
            version,
            models,
        };
        
        let json = serde_json::to_string_pretty(&config)
            .map_err(|e| ModelCacheError::Storage(format!("Failed to serialize catalog: {}", e)))?;
        
        std::fs::write(json_path, json)
            .map_err(|e| ModelCacheError::Storage(format!("Failed to write catalog JSON: {}", e)))?;
        
        Ok(())
    }
    
    /// Insert or update a model
    pub fn insert_model(&self, model: ModelCatalogEntry) -> Result<()> {
        let catalog_tree = self.db.open_tree(Self::CATALOG_TREE)
            .map_err(|e| ModelCacheError::Storage(format!("Failed to open catalog tree: {}", e)))?;
        
        let key = model.id.as_bytes();
        let value = serde_json::to_vec(&model)
            .map_err(|e| ModelCacheError::Storage(format!("Failed to serialize model: {}", e)))?;
        
        catalog_tree.insert(key, value)
            .map_err(|e| ModelCacheError::Storage(format!("Failed to insert model: {}", e)))?;
        
        Ok(())
    }
    
    /// Get a specific model by ID
    pub fn get_model(&self, model_id: &str) -> Result<Option<ModelCatalogEntry>> {
        let catalog_tree = self.db.open_tree(Self::CATALOG_TREE)
            .map_err(|e| ModelCacheError::Storage(format!("Failed to open catalog tree: {}", e)))?;
        
        let key = model_id.as_bytes();
        let value = catalog_tree.get(key)
            .map_err(|e| ModelCacheError::Storage(format!("Failed to get model: {}", e)))?;
        
        match value {
            Some(ref data) => {
                let model = serde_json::from_slice(data)?;
                Ok(Some(model))
            }
            None => Ok(None),
        }
    }
    
    /// Delete a model from catalog
    pub fn delete_model(&self, model_id: &str) -> Result<bool> {
        let catalog_tree = self.db.open_tree(Self::CATALOG_TREE)
            .map_err(|e| ModelCacheError::Storage(format!("Failed to open catalog tree: {}", e)))?;
        
        let key = model_id.as_bytes();
        let removed = catalog_tree.remove(key)
            .map_err(|e| ModelCacheError::Storage(format!("Failed to delete model: {}", e)))?;
        
        Ok(removed.is_some())
    }
    
    /// Get all models
    pub fn get_all_models(&self) -> Result<Vec<ModelCatalogEntry>> {
        let catalog_tree = self.db.open_tree(Self::CATALOG_TREE)
            .map_err(|e| ModelCacheError::Storage(format!("Failed to open catalog tree: {}", e)))?;
        
        let mut models = Vec::new();
        
        for item in catalog_tree.iter() {
            let (_key, value) = item
                .map_err(|e| ModelCacheError::Storage(format!("Failed to iterate catalog: {}", e)))?;
            
            // Skip version key
            if value.starts_with(&[0u8, 0, 0, 0]) && value.len() == 4 {
                continue;
            }
            
            let model: ModelCatalogEntry = serde_json::from_slice(&value)
                .map_err(|e| ModelCacheError::Storage(format!("Failed to deserialize model: {}", e)))?;
            
            models.push(model);
        }
        
        Ok(models)
    }
    
    /// Get models by type
    pub fn get_models_by_type(&self, model_type: &str) -> Result<Vec<ModelCatalogEntry>> {
        let all_models = self.get_all_models()?;
        Ok(all_models
            .into_iter()
            .filter(|m| m.model_type.eq_ignore_ascii_case(model_type))
            .collect())
    }
    
    /// Get suggested/recommended models
    pub fn get_suggested_models(&self) -> Result<Vec<ModelCatalogEntry>> {
        let all_models = self.get_all_models()?;
        Ok(all_models
            .into_iter()
            .filter(|m| m.suggested)
            .collect())
    }
    
    /// Get downloaded models
    pub fn get_downloaded_models(&self) -> Result<Vec<ModelCatalogEntry>> {
        let all_models = self.get_all_models()?;
        Ok(all_models
            .into_iter()
            .filter(|m| m.downloaded)
            .collect())
    }
    
    /// Get models by tag
    pub fn get_models_by_tag(&self, tag: &str) -> Result<Vec<ModelCatalogEntry>> {
        let all_models = self.get_all_models()?;
        Ok(all_models
            .into_iter()
            .filter(|m| m.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)))
            .collect())
    }
    
    /// Search models by name or repo
    pub fn search_models(&self, query: &str) -> Result<Vec<ModelCatalogEntry>> {
        let all_models = self.get_all_models()?;
        let query_lower = query.to_lowercase();
        
        Ok(all_models
            .into_iter()
            .filter(|m| {
                m.name.to_lowercase().contains(&query_lower)
                    || m.repo_id.to_lowercase().contains(&query_lower)
                    || m.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&query_lower))
            })
            .collect())
    }
    
    /// Mark model as downloaded
    pub fn mark_downloaded(&self, model_id: &str, downloaded: bool) -> Result<()> {
        if let Some(mut model) = self.get_model(model_id)? {
            model.downloaded = downloaded;
            self.insert_model(model)?;
        }
        Ok(())
    }
    
    /// Get default model for a type
    pub fn get_default_for_type(&self, model_type: &str) -> Result<Option<ModelCatalogEntry>> {
        let models = self.get_models_by_type(model_type)?;
        
        // First, try to find a model tagged with "default"
        if let Some(default_model) = models.iter()
            .find(|m| m.suggested && m.tags.iter().any(|t| t.eq_ignore_ascii_case("default")))
            .cloned() 
        {
            return Ok(Some(default_model));
        }
        
        // Fallback: Find smallest suggested model
        Ok(models
            .into_iter()
            .filter(|m| m.suggested)
            .min_by(|a, b| a.size_gb.partial_cmp(&b.size_gb).unwrap_or(std::cmp::Ordering::Equal)))
    }
    
    /// Count models by type
    pub fn count_by_type(&self) -> Result<std::collections::HashMap<String, usize>> {
        let all_models = self.get_all_models()?;
        let mut counts = std::collections::HashMap::new();
        
        for model in all_models {
            *counts.entry(model.model_type.clone()).or_insert(0) += 1;
        }
        
        Ok(counts)
    }
    
    /// Clear entire catalog (use with caution!)
    pub fn clear_all(&self) -> Result<()> {
        let catalog_tree = self.db.open_tree(Self::CATALOG_TREE)
            .map_err(|e| ModelCacheError::Storage(format!("Failed to open catalog tree: {}", e)))?;
        
        catalog_tree.clear()
            .map_err(|e| ModelCacheError::Storage(format!("Failed to clear catalog: {}", e)))?;
        
        Ok(())
    }
}

