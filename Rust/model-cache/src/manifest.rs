use crate::schema::QuantStatus;
use tabagent_values::InferenceSettings;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about a specific quantization variant
#[derive(Debug, Clone, Serialize, Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct QuantInfo {
    pub status: QuantStatus,
    pub files: Vec<String>,
    pub total_size: Option<u64>,
    pub downloaded_size: Option<u64>,
    pub last_updated: i64, // Unix timestamp
}

/// Manifest entry for a model repository
#[derive(Debug, Clone, Serialize, Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ManifestEntry {
    pub repo_id: String,
    pub task: Option<String>, // e.g., "text-generation", "feature-extraction"
    pub quants: HashMap<String, QuantInfo>, // quant_key -> info
    pub settings: HashMap<String, InferenceSettings>, // variant -> inference settings
    pub created_at: i64,
    pub updated_at: i64,
}

impl ManifestEntry {
    pub fn new(repo_id: String, task: Option<String>) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            repo_id,
            task,
            quants: HashMap::new(),
            settings: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn add_quant(&mut self, quant_key: String, files: Vec<String>, total_size: Option<u64>) {
        let now = chrono::Utc::now().timestamp_millis();
        self.quants.insert(quant_key, QuantInfo {
            status: QuantStatus::Available,
            files,
            total_size,
            downloaded_size: None,
            last_updated: now,
        });
        self.updated_at = now;
    }
    
    pub fn update_quant_status(&mut self, quant_key: &str, status: QuantStatus) {
        if let Some(quant) = self.quants.get_mut(quant_key) {
            quant.status = status;
            quant.last_updated = chrono::Utc::now().timestamp_millis();
            self.updated_at = quant.last_updated;
        }
    }
    
    pub fn update_download_progress(&mut self, quant_key: &str, downloaded_size: u64) {
        if let Some(quant) = self.quants.get_mut(quant_key) {
            quant.downloaded_size = Some(downloaded_size);
            quant.last_updated = chrono::Utc::now().timestamp_millis();
            self.updated_at = quant.last_updated;
        }
    }
}

