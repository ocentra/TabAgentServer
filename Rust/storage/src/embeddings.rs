//! Embedding operations for the MIA storage system
//!
//! This module provides implementations for embedding-related operations
//! across different temperature tiers.

use crate::{traits::EmbeddingOperations, DefaultStorageManager, TemperatureTier};
use common::{models::*, DbResult};
use std::sync::{Arc, RwLock};

/// Implementation of embedding operations
pub struct EmbeddingManager {
    /// Embeddings/active: Vectors for 0-30 days (HOT)
    pub(crate) embeddings_active: Arc<DefaultStorageManager>,
    /// Embeddings/recent: Vectors for 30-90 days (WARM - lazy load)
    pub(crate) embeddings_recent: Arc<RwLock<Option<DefaultStorageManager>>>,
    /// Embeddings/archive: Vectors for 90+ days (COLD - on-demand)
    pub(crate) embeddings_archives: Arc<RwLock<std::collections::HashMap<String, DefaultStorageManager>>>,
}

impl EmbeddingOperations for EmbeddingManager {
    /// Get an embedding by ID, searching across all embedding tiers
    fn get_embedding(&self, embedding_id: &str) -> DbResult<Option<Embedding>> {
        // Try active first (HOT - most common)
        if let Some(emb_ref) = self.embeddings_active.get_embedding_ref(embedding_id)? {
            return Ok(Some(emb_ref.deserialize()?));
        }

        // Try recent (WARM - lazy load if needed)
        if let Some(recent) = self.get_or_load_embeddings_recent()? {
            if let Some(emb_ref) = recent.get_embedding_ref(embedding_id)? {
                return Ok(Some(emb_ref.deserialize()?));
            }
        }

        // Try archives (COLD - search all loaded quarters)
        let archives = self.embeddings_archives.read()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
        for (_quarter, storage) in archives.iter() {
            if let Some(emb_ref) = storage.get_embedding_ref(embedding_id)? {
                return Ok(Some(emb_ref.deserialize()?));
            }
        }

        Ok(None)
    }

    /// Get an embedding by ID with a hint about which quarter it might be in
    fn get_embedding_with_hint(
        &self,
        embedding_id: &str,
        timestamp_hint_ms: i64,
    ) -> DbResult<Option<Embedding>> {
        // Try active first (HOT - most common)
        if let Some(emb_ref) = self.embeddings_active.get_embedding_ref(embedding_id)? {
            return Ok(Some(emb_ref.deserialize()?));
        }

        // Try recent (WARM - lazy load if needed)
        if let Some(recent) = self.get_or_load_embeddings_recent()? {
            if let Some(emb_ref) = recent.get_embedding_ref(embedding_id)? {
                return Ok(Some(emb_ref.deserialize()?));
            }
        }

        // Try the hinted quarter first
        let quarter = common::platform::get_quarter_from_timestamp(timestamp_hint_ms);
        if let Some(embedding) = self.get_embedding_from_archive(embedding_id, &quarter)? {
            return Ok(Some(embedding));
        }

        // If not found in the hinted quarter, search all other loaded quarters
        let archives = self.embeddings_archives.read()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
        for (quarter_name, storage) in archives.iter() {
            // Skip the quarter we already searched
            if quarter_name == &quarter {
                continue;
            }

            if let Some(emb_ref) = storage.get_embedding_ref(embedding_id)? {
                return Ok(Some(emb_ref.deserialize()?));
            }
        }

        Ok(None)
    }

    /// Insert an embedding into embeddings/active
    fn insert_embedding(&self, embedding: Embedding) -> DbResult<()> {
        self.embeddings_active.insert_embedding(&embedding)
    }

    /// Get or lazy-load embeddings/recent tier
    fn get_or_load_embeddings_recent(&self) -> DbResult<Option<Arc<DefaultStorageManager>>> {
        let mut recent_guard = self.embeddings_recent.write()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;

        if recent_guard.is_none() {
            // Lazy load recent tier
            match DefaultStorageManager::open_typed(
                crate::DatabaseType::Embeddings,
                Some(TemperatureTier::Recent),
            ) {
                Ok(storage) => *recent_guard = Some(storage),
                Err(e) => {
                    // If recent doesn't exist yet, that's OK
                    if !matches!(e, common::DbError::InvalidOperation(_)) {
                        return Err(e);
                    }
                }
            }
        }

        let storage = recent_guard.as_ref()
            .ok_or_else(|| common::DbError::Other("Recent storage not loaded".to_string()))?;
        Ok(Some(Arc::new(storage.clone())))
    }

    /// Get or lazy-load a specific embeddings archive quarter
    fn get_or_load_embeddings_archive(
        &self,
        quarter: &str,
    ) -> DbResult<Option<Arc<DefaultStorageManager>>> {
        let mut archives = self.embeddings_archives.write()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;

        if !archives.contains_key(quarter) {
            // Implement archive loading with quarter-specific paths
            match DefaultStorageManager::open_typed(
                crate::DatabaseType::Embeddings,
                Some(TemperatureTier::Archive),
            ) {
                Ok(_storage) => {
                    // Modify the path to include the quarter
                    let base_path =
                        crate::DatabaseType::Embeddings.get_path(Some(TemperatureTier::Archive));
                    let quarter_path = base_path.join(quarter);

                    // Ensure the quarter directory exists
                    common::platform::ensure_db_directory(&quarter_path)?;

                    let path_str = quarter_path.to_str().ok_or_else(|| {
                        common::DbError::InvalidOperation(
                            "Invalid UTF-8 in database path".to_string(),
                        )
                    })?;

                    // Reopen storage with the quarter-specific path
                    let quarter_storage = DefaultStorageManager::new(path_str)?;
                    archives.insert(quarter.to_string(), quarter_storage);
                }
                Err(e) => {
                    // If archive doesn't exist yet, that's OK for lazy loading
                    if !matches!(e, common::DbError::InvalidOperation(_)) {
                        return Err(e);
                    }
                }
            }
        }

        Ok(archives.get(quarter).map(|s| Arc::new(s.clone())))
    }

    /// Get an embedding from a specific archive quarter
    fn get_embedding_from_archive(
        &self,
        embedding_id: &str,
        quarter: &str,
    ) -> DbResult<Option<Embedding>> {
        if let Some(archive_storage) = self.get_or_load_embeddings_archive(quarter)? {
            if let Some(emb_ref) = archive_storage.get_embedding_ref(embedding_id)? {
                return Ok(Some(emb_ref.deserialize()?));
            }
        }
        Ok(None)
    }
}
