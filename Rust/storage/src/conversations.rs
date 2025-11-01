//! Conversation operations for the MIA storage system
//!
//! This module provides implementations for conversation-related operations
//! including messages and chats across different temperature tiers.

use crate::{traits::ConversationOperations, DefaultStorageManager, TemperatureTier};
use common::{models::*, platform::get_quarter_from_timestamp, DbResult};
use std::sync::{Arc, RwLock};

/// Implementation of conversation operations
pub struct ConversationManager {
    /// Conversations/active: 0-30 days (HOT - always loaded)
    pub(crate) conversations_active: Arc<DefaultStorageManager>,
    /// Conversations/recent: 30-90 days (WARM - lazy load)
    pub(crate) conversations_recent: Arc<RwLock<Option<DefaultStorageManager>>>,
    /// Conversations/archive: 90+ days by quarter (COLD - on-demand)
    pub(crate) conversations_archives:
        Arc<RwLock<std::collections::HashMap<String, DefaultStorageManager>>>,
}

impl ConversationOperations for ConversationManager {
    /// Insert a message into conversations/active
    fn insert_message(&self, message: Message) -> DbResult<()> {
        self.conversations_active
            .insert_node(&Node::Message(message))
    }

    /// Get a message by ID, searching across all conversation tiers
    fn get_message(&self, message_id: &str) -> DbResult<Option<Message>> {
        // Try active first (HOT - most common)
        if let Some(node_ref) = self.conversations_active.get_node_ref(message_id)? {
            let node = node_ref.deserialize()?;
            if let Node::Message(msg) = node {
                return Ok(Some(msg));
            }
        }

        // Try recent (WARM - lazy load if needed)
        if let Some(recent) = self.get_or_load_conversations_recent()? {
            if let Some(node_ref) = recent.get_node_ref(message_id)? {
                let node = node_ref.deserialize()?;
                if let Node::Message(msg) = node {
                    return Ok(Some(msg));
                }
            }
        }

        // Try archives (COLD - search all loaded quarters)
        let archives = self.conversations_archives.read()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
        for (_quarter, storage) in archives.iter() {
            if let Some(node_ref) = storage.get_node_ref(message_id)? {
                let node = node_ref.deserialize()?;
                if let Node::Message(msg) = node {
                    return Ok(Some(msg));
                }
            }
        }

        Ok(None)
    }

    /// Get a message by ID with a hint about which quarter it might be in
    fn get_message_with_hint(
        &self,
        message_id: &str,
        timestamp_hint_ms: i64,
    ) -> DbResult<Option<Message>> {
        // Try active first (HOT - most common)
        if let Some(node_ref) = self.conversations_active.get_node_ref(message_id)? {
            let node = node_ref.deserialize()?;
            if let Node::Message(msg) = node {
                return Ok(Some(msg));
            }
        }

        // Try recent (WARM - lazy load if needed)
        if let Some(recent) = self.get_or_load_conversations_recent()? {
            if let Some(node_ref) = recent.get_node_ref(message_id)? {
                let node = node_ref.deserialize()?;
                if let Node::Message(msg) = node {
                    return Ok(Some(msg));
                }
            }
        }

        // Try the hinted quarter first
        let quarter = self.get_quarter_for_timestamp(timestamp_hint_ms);
        if let Some(msg) = self.get_message_from_archive(message_id, &quarter)? {
            return Ok(Some(msg));
        }

        // If not found in the hinted quarter, search all other loaded quarters
        let archives = self.conversations_archives.read()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
        for (quarter_name, storage) in archives.iter() {
            // Skip the quarter we already searched
            if quarter_name == &quarter {
                continue;
            }

            if let Some(node_ref) = storage.get_node_ref(message_id)? {
                let node = node_ref.deserialize()?;
                if let Node::Message(msg) = node {
                    return Ok(Some(msg));
                }
            }
        }

        Ok(None)
    }

    /// Insert a chat into conversations/active
    fn insert_chat(&self, chat: Chat) -> DbResult<()> {
        self.conversations_active.insert_node(&Node::Chat(chat))
    }

    /// Get a chat by ID, searching across all conversation tiers
    fn get_chat(&self, chat_id: &str) -> DbResult<Option<Chat>> {
        // Try active
        if let Some(node_ref) = self.conversations_active.get_node_ref(chat_id)? {
            let node = node_ref.deserialize()?;
            if let Node::Chat(chat) = node {
                return Ok(Some(chat));
            }
        }

        // Try recent
        if let Some(recent) = self.get_or_load_conversations_recent()? {
            if let Some(node_ref) = recent.get_node_ref(chat_id)? {
                let node = node_ref.deserialize()?;
                if let Node::Chat(chat) = node {
                    return Ok(Some(chat));
                }
            }
        }

        // Try archives
        let archives = self.conversations_archives.read()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
        for (_quarter, storage) in archives.iter() {
            if let Some(node_ref) = storage.get_node_ref(chat_id)? {
                let node = node_ref.deserialize()?;
                if let Node::Chat(chat) = node {
                    return Ok(Some(chat));
                }
            }
        }

        Ok(None)
    }

    /// Promote a message from active to recent tier based on age
    fn promote_message_to_recent(
        &self,
        message_id: &str,
        current_timestamp_ms: i64,
    ) -> DbResult<bool> {
        // Check timestamp before deserializing
        if let Some(node_ref) = self.conversations_active.get_node_ref(message_id)? {
            // Access timestamp field directly
            if let Some(timestamp) = node_ref.message_timestamp() {
                // Check if message is older than 30 days
                let age_ms = current_timestamp_ms - timestamp;
                let thirty_days_ms = 30 * 24 * 60 * 60 * 1000;
                
                if age_ms < thirty_days_ms {
                    return Ok(false);  // Not old enough
                }
                
                // Deserialize to move
                let node = node_ref.deserialize()?;
                if let Node::Message(msg) = node {
                    // Load recent tier if needed
                    let recent_storage = if let Some(recent) = self.get_or_load_conversations_recent()? {
                        recent
                    } else {
                        let recent_db = StorageManager::open_typed_with_indexing(
                            crate::DatabaseType::Conversations,
                            Some(TemperatureTier::Recent),
                        )?;
                        let mut recent_guard = self.conversations_recent.write()
                            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;
                        *recent_guard = Some(recent_db.clone());
                        Arc::new(recent_db)
                    };

                    // Move message to recent tier
                    recent_storage.insert_node(&Node::Message(msg))?;
                    // Remove from active tier
                    self.conversations_active.delete_node(message_id)?;
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Get or lazy-load conversations/recent tier
    fn get_or_load_conversations_recent(&self) -> DbResult<Option<Arc<StorageManager>>> {
        let mut recent_guard = self.conversations_recent.write()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;

        if recent_guard.is_none() {
            // Lazy load recent tier
            match StorageManager::open_typed_with_indexing(
                crate::DatabaseType::Conversations,
                Some(TemperatureTier::Recent),
            ) {
                Ok(storage) => *recent_guard = Some(storage),
                Err(e) => {
                    // If recent doesn't exist yet, that's OK
                    if !matches!(e, common::DbError::Sled(_)) {
                        return Err(e);
                    }
                }
            }
        }

        let storage = recent_guard.as_ref()
            .ok_or_else(|| common::DbError::Other("Recent storage not loaded".to_string()))?;
        Ok(Some(Arc::new(storage.clone())))
    }

    /// Get or lazy-load a specific archive quarter
    fn get_or_load_archive(&self, quarter: &str) -> DbResult<Option<Arc<StorageManager>>> {
        let mut archives = self.conversations_archives.write()
            .map_err(|e| common::DbError::Other(format!("Lock poisoned: {}", e)))?;

        if !archives.contains_key(quarter) {
            // Implement archive loading with quarter-specific paths
            match StorageManager::open_typed(
                crate::DatabaseType::Conversations,
                Some(TemperatureTier::Archive),
            ) {
                Ok(_storage) => {
                    // Modify the path to include the quarter
                    let base_path =
                        crate::DatabaseType::Conversations.get_path(Some(TemperatureTier::Archive));
                    let quarter_path = base_path.join(quarter);

                    // Ensure the quarter directory exists
                    common::platform::ensure_db_directory(&quarter_path)?;

                    let path_str = quarter_path.to_str().ok_or_else(|| {
                        common::DbError::InvalidOperation(
                            "Invalid UTF-8 in database path".to_string(),
                        )
                    })?;

                    // Reopen storage with the quarter-specific path
                    let quarter_storage = StorageManager::new(path_str)?;
                    archives.insert(quarter.to_string(), quarter_storage);
                }
                Err(e) => {
                    // If archive doesn't exist yet, that's OK for lazy loading
                    if !matches!(e, common::DbError::Sled(_)) {
                        return Err(e);
                    }
                }
            }
        }

        Ok(archives.get(quarter).map(|s| Arc::new(s.clone())))
    }

    /// Get the quarter string for a given timestamp
    fn get_quarter_for_timestamp(&self, timestamp_ms: i64) -> String {
        get_quarter_from_timestamp(timestamp_ms)
    }

    /// Get a message from a specific archive quarter
    fn get_message_from_archive(
        &self,
        message_id: &str,
        quarter: &str,
    ) -> DbResult<Option<Message>> {
        if let Some(archive_storage) = self.get_or_load_archive(quarter)? {
            if let Some(node_ref) = archive_storage.get_node_ref(message_id)? {
                let node = node_ref.deserialize()?;
                if let Node::Message(msg) = node {
                    return Ok(Some(msg));
                }
            }
        }
        Ok(None)
    }
}
