/// Model-specific inference settings management
use crate::error::Result;
use common::inference_settings::InferenceSettings;
use sled::Db;
use std::sync::Arc;

pub struct ModelSettingsStore {
    #[allow(dead_code)]
    db: Arc<Db>,
    tree: sled::Tree,
}

impl ModelSettingsStore {
    pub fn new(db: Arc<Db>) -> Result<Self> {
        let tree = db.open_tree(b"model_settings")?;
        Ok(Self { db: db.clone(), tree })
    }
    
    /// Get settings for a model, returns None if not saved yet
    pub fn get(&self, repo_id: &str, variant: &str) -> Result<Option<InferenceSettings>> {
        let key = format!("{}:{}", repo_id, variant);
        if let Some(bytes) = self.tree.get(key.as_bytes())? {
            let (settings, _): (InferenceSettings, usize) = bincode::decode_from_slice(&bytes, bincode::config::standard())?;
            Ok(Some(settings))
        } else {
            Ok(None)
        }
    }
    
    /// Get or create default settings for a model
    pub fn get_or_default(&self, repo_id: &str, variant: &str) -> Result<InferenceSettings> {
        if let Some(settings) = self.get(repo_id, variant)? {
            Ok(settings)
        } else {
            // Generate defaults based on model pattern
            let settings = InferenceSettings::for_model(repo_id);
            // Save for next time
            self.save(repo_id, variant, &settings)?;
            Ok(settings)
        }
    }
    
    /// Save settings for a model
    pub fn save(&self, repo_id: &str, variant: &str, settings: &InferenceSettings) -> Result<()> {
        let key = format!("{}:{}", repo_id, variant);
        let bytes = bincode::encode_to_vec(settings, bincode::config::standard())?;
        self.tree.insert(key.as_bytes(), bytes)?;
        self.tree.flush()?;
        Ok(())
    }
    
    /// Delete settings for a model
    pub fn delete(&self, repo_id: &str, variant: &str) -> Result<()> {
        let key = format!("{}:{}", repo_id, variant);
        self.tree.remove(key.as_bytes())?;
        self.tree.flush()?;
        Ok(())
    }
}

