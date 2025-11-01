//! Summary operations for the MIA storage system
//!
//! This module provides implementations for summary-related operations
//! across different temperature tiers.

use crate::{traits::SummaryOperations, DefaultStorageManager, TemperatureTier};
use common::{models::*, DbResult};
use std::sync::{Arc, RwLock};

/// Implementation of summary operations
pub struct SummaryManager {
    /// Summaries: Session/daily/weekly/monthly (varies by tier)
    pub(crate) summaries: Arc<RwLock<std::collections::HashMap<String, DefaultStorageManager>>>,
}

impl SummaryOperations for SummaryManager {
    /// Get or lazy-load a specific summary tier
    fn get_or_load_summary(&self, tier: TemperatureTier) -> DbResult<Arc<DefaultStorageManager>> {
        let mut summaries = self.summaries.write()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;

        let tier_key = tier.name().to_string();
        if !summaries.contains_key(&tier_key) {
            // Lazy load summary tier
            let storage = DefaultStorageManager::open_typed_with_indexing(
                crate::DatabaseType::Summaries,
                Some(tier),
            )?;
            summaries.insert(tier_key.clone(), storage);
        }

        let storage = summaries.get(&tier_key)
            .ok_or_else(|| common::DbError::Other("Storage not found after insert".to_string()))?;
        Ok(Arc::new(storage.clone()))
    }

    /// Insert a summary into the appropriate tier
    fn insert_summary(&self, summary: Summary) -> DbResult<()> {
        // Determine the appropriate tier based on summary properties
        // For now, we'll use a simple approach - this could be enhanced later
        let tier = if summary.content.len() < 1000 {
            TemperatureTier::Daily
        } else if summary.content.len() < 5000 {
            TemperatureTier::Weekly
        } else {
            TemperatureTier::Monthly
        };

        let storage = self.get_or_load_summary(tier)?;
        storage.insert_node(&Node::Summary(summary))
    }

    /// Get a summary by ID, searching across all summary tiers
    fn get_summary(&self, summary_id: &str) -> DbResult<Option<Summary>> {
        // Try all summary tiers
        let summary_tiers = [
            TemperatureTier::Session,
            TemperatureTier::Daily,
            TemperatureTier::Weekly,
            TemperatureTier::Monthly,
        ];

        let summaries = self.summaries.read()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;

        for tier in &summary_tiers {
            let tier_key = tier.name();
            if let Some(storage) = summaries.get(tier_key) {
                if let Some(node_ref) = storage.get_node_ref(summary_id)? {
                    let node = node_ref.deserialize()?;
                    if let Node::Summary(summary) = node {
                        return Ok(Some(summary));
                    }
                }
            }
        }

        // Also check lazy-loaded tiers that might not be in the map yet
        for tier in &summary_tiers {
            if let Ok(storage) = self.get_or_load_summary(*tier) {
                if let Some(node_ref) = storage.get_node_ref(summary_id)? {
                    let node = node_ref.deserialize()?;
                    if let Node::Summary(summary) = node {
                        return Ok(Some(summary));
                    }
                }
            }
        }

        Ok(None)
    }
}
