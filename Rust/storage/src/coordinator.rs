//! Database Coordinator for MIA's multi-tier memory architecture
//!
//! This module provides the `DatabaseCoordinator` which manages all 7 database types
//! and their temperature tiers, providing a unified interface for cross-DB operations.

use crate::{DatabaseType, StorageManager, TemperatureTier};
use common::{models::*, DbResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// High-level coordinator for ALL 7 database types in MIA's memory system
///
/// The coordinator manages:
/// - 2 SOURCE databases (Conversations, Experience)
/// - 3 DERIVED databases (Knowledge, Embeddings, Summaries)
/// - 1 EXTERNAL database (ToolResults)
/// - 1 INDEX database (Meta)
///
/// HOT tiers (Active, Stable, Session) are always loaded.
/// WARM/COLD tiers (Recent, Archive) are lazy-loaded on first access.
pub struct DatabaseCoordinator {
    // ========== SOURCE DATABASES (CRITICAL!) ==========
    /// Conversations/active: 0-30 days (HOT - always loaded)
    conversations_active: Arc<StorageManager>,
    /// Conversations/recent: 30-90 days (WARM - lazy load)
    conversations_recent: Arc<RwLock<Option<StorageManager>>>,
    /// Conversations/archive: 90+ days by quarter (COLD - on-demand)
    conversations_archives: Arc<RwLock<HashMap<String, StorageManager>>>,
    
    // ========== DERIVED: KNOWLEDGE ==========
    /// Knowledge/active: Recently mentioned entities (HOT)
    knowledge_active: Arc<StorageManager>,
    /// Knowledge/stable: Proven important entities 10+ mentions (HOT)
    knowledge_stable: Arc<StorageManager>,
    /// Knowledge/inferred: Experimental/low-confidence (COLD)
    knowledge_inferred: Arc<StorageManager>,
    
    // ========== DERIVED: EMBEDDINGS ==========
    /// Embeddings/active: Vectors for 0-30 days (HOT)
    embeddings_active: Arc<StorageManager>,
    /// Embeddings/recent: Vectors for 30-90 days (WARM - lazy load)
    embeddings_recent: Arc<RwLock<Option<StorageManager>>>,
    /// Embeddings/archive: Vectors for 90+ days (COLD - on-demand)
    embeddings_archives: Arc<RwLock<HashMap<String, StorageManager>>>,
    
    // ========== DERIVED: SUMMARIES ==========
    /// Summaries: Session/daily/weekly/monthly (varies by tier)
    summaries: Arc<RwLock<HashMap<String, StorageManager>>>,
    
    // ========== EXTERNAL & LEARNING DATABASES ==========
    /// Tool-results: Cached searches, scrapes, API responses
    tool_results: Arc<StorageManager>,
    /// Experience: Agent action outcomes, user feedback, patterns (CRITICAL for learning!)
    experience: Arc<StorageManager>,
    /// Meta: Query routing, performance stats, confidence maps
    meta: Arc<StorageManager>,
}

impl DatabaseCoordinator {
    /// Initialize ALL databases at platform-specific paths
    ///
    /// Opens HOT tiers immediately, leaves WARM/COLD tiers for lazy loading.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use storage::coordinator::DatabaseCoordinator;
    ///
    /// # fn main() -> Result<(), common::DbError> {
    /// let coordinator = DatabaseCoordinator::new()?;
    /// // All HOT tier databases are now ready!
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> DbResult<Self> {
        Ok(Self {
            // ========== SOURCE: CONVERSATIONS (3 tiers) ==========
            conversations_active: Arc::new(StorageManager::open_typed_with_indexing(
                DatabaseType::Conversations,
                Some(TemperatureTier::Active),
            )?),
            conversations_recent: Arc::new(RwLock::new(None)),
            conversations_archives: Arc::new(RwLock::new(HashMap::new())),
            
            // ========== DERIVED: KNOWLEDGE (3 tiers) ==========
            knowledge_active: Arc::new(StorageManager::open_typed_with_indexing(
                DatabaseType::Knowledge,
                Some(TemperatureTier::Active),
            )?),
            knowledge_stable: Arc::new(StorageManager::open_typed_with_indexing(
                DatabaseType::Knowledge,
                Some(TemperatureTier::Stable),
            )?),
            knowledge_inferred: Arc::new(StorageManager::open_typed_with_indexing(
                DatabaseType::Knowledge,
                Some(TemperatureTier::Inferred),
            )?),
            
            // ========== DERIVED: EMBEDDINGS (3 tiers) ==========
            embeddings_active: Arc::new(StorageManager::open_typed_with_indexing(
                DatabaseType::Embeddings,
                Some(TemperatureTier::Active),
            )?),
            embeddings_recent: Arc::new(RwLock::new(None)),
            embeddings_archives: Arc::new(RwLock::new(HashMap::new())),
            
            // ========== DERIVED: SUMMARIES (lazy load all) ==========
            summaries: Arc::new(RwLock::new(HashMap::new())),
            
            // ========== EXTERNAL & LEARNING ==========
            tool_results: Arc::new(StorageManager::open_typed_with_indexing(
                DatabaseType::ToolResults,
                None,
            )?),
            experience: Arc::new(StorageManager::open_typed_with_indexing(
                DatabaseType::Experience,
                None,
            )?),
            meta: Arc::new(StorageManager::open_typed_with_indexing(
                DatabaseType::Meta,
                None,
            )?),
        })
    }
    
    // ========== CONVERSATION OPERATIONS ==========
    
    /// Insert a message into conversations/active
    ///
    /// # Arguments
    ///
    /// * `message` - The message to insert
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use storage::coordinator::DatabaseCoordinator;
    /// use common::models::Message;
    ///
    /// # fn main() -> Result<(), common::DbError> {
    /// # use common::{NodeId, models::Message};
    /// let coordinator = DatabaseCoordinator::new()?;
    /// let message = Message {
    ///     id: NodeId::from("msg_123"),
    ///     chat_id: NodeId::from("chat_1"),
    ///     sender: "user".to_string(),
    ///     timestamp: 0,
    ///     text_content: "Hello".to_string(),
    ///     attachment_ids: vec![],
    ///     embedding_id: None,
    ///     metadata: serde_json::json!({}),
    /// };
    /// coordinator.insert_message(message)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn insert_message(&self, message: Message) -> DbResult<()> {
        self.conversations_active.insert_node(&Node::Message(message))
    }
    
    /// Get a message by ID, searching across all conversation tiers
    ///
    /// Searches Active → Recent → Archive tiers in order.
    ///
    /// # Arguments
    ///
    /// * `message_id` - The message ID to find
    ///
    /// # Returns
    ///
    /// The message if found, None otherwise
    pub fn get_message(&self, message_id: &str) -> DbResult<Option<Message>> {
        // Try active first (HOT - most common)
        if let Some(Node::Message(msg)) = self.conversations_active.get_node(message_id)? {
            return Ok(Some(msg));
        }
        
        // Try recent (WARM - lazy load if needed)
        if let Some(recent) = self.get_or_load_conversations_recent()? {
            if let Some(Node::Message(msg)) = recent.get_node(message_id)? {
                return Ok(Some(msg));
            }
        }
        
        // Try archives (COLD - search all loaded quarters)
        let archives = self.conversations_archives.read().unwrap();
        for (_quarter, storage) in archives.iter() {
            if let Some(Node::Message(msg)) = storage.get_node(message_id)? {
                return Ok(Some(msg));
            }
        }
        
        Ok(None)
    }
    
    /// Insert a chat into conversations/active
    pub fn insert_chat(&self, chat: Chat) -> DbResult<()> {
        self.conversations_active.insert_node(&Node::Chat(chat))
    }
    
    /// Get a chat by ID, searching across all conversation tiers
    pub fn get_chat(&self, chat_id: &str) -> DbResult<Option<Chat>> {
        // Try active
        if let Some(Node::Chat(chat)) = self.conversations_active.get_node(chat_id)? {
            return Ok(Some(chat));
        }
        
        // Try recent
        if let Some(recent) = self.get_or_load_conversations_recent()? {
            if let Some(Node::Chat(chat)) = recent.get_node(chat_id)? {
                return Ok(Some(chat));
            }
        }
        
        // Try archives
        let archives = self.conversations_archives.read().unwrap();
        for (_quarter, storage) in archives.iter() {
            if let Some(Node::Chat(chat)) = storage.get_node(chat_id)? {
                return Ok(Some(chat));
            }
        }
        
        Ok(None)
    }
    
    // ========== KNOWLEDGE OPERATIONS ==========
    
    /// Insert an entity into knowledge/active
    pub fn insert_entity(&self, entity: Entity) -> DbResult<()> {
        self.knowledge_active.insert_node(&Node::Entity(entity))
    }
    
    /// Get an entity by ID, searching active → stable → inferred
    pub fn get_entity(&self, entity_id: &str) -> DbResult<Option<Entity>> {
        // Try active
        if let Some(Node::Entity(entity)) = self.knowledge_active.get_node(entity_id)? {
            return Ok(Some(entity));
        }
        
        // Try stable
        if let Some(Node::Entity(entity)) = self.knowledge_stable.get_node(entity_id)? {
            return Ok(Some(entity));
        }
        
        // Try inferred
        if let Some(Node::Entity(entity)) = self.knowledge_inferred.get_node(entity_id)? {
            return Ok(Some(entity));
        }
        
        Ok(None)
    }
    
    /// Promote an entity from active to stable (after 10+ mentions)
    pub fn promote_entity_to_stable(&self, entity_id: &str) -> DbResult<()> {
        if let Some(entity) = self.get_entity(entity_id)? {
            // Insert to stable
            self.knowledge_stable.insert_node(&Node::Entity(entity))?;
            // Remove from active
            self.knowledge_active.delete_node(entity_id)?;
        }
        Ok(())
    }
    
    // ========== TOOL RESULTS OPERATIONS ==========
    
    /// Insert a web search result
    pub fn insert_web_search(&self, search: WebSearch) -> DbResult<()> {
        self.tool_results.insert_node(&Node::WebSearch(search))
    }
    
    /// Get a web search by ID
    pub fn get_web_search(&self, search_id: &str) -> DbResult<Option<WebSearch>> {
        match self.tool_results.get_node(search_id)? {
            Some(Node::WebSearch(search)) => Ok(Some(search)),
            _ => Ok(None),
        }
    }
    
    /// Insert a scraped page
    pub fn insert_scraped_page(&self, page: ScrapedPage) -> DbResult<()> {
        self.tool_results.insert_node(&Node::ScrapedPage(page))
    }
    
    // ========== EXPERIENCE OPERATIONS (CRITICAL for learning!) ==========
    
    // TODO: Uncomment when ActionOutcome type is added to common::models
    // /// Insert an action outcome (what happened when agent acted)
    // pub fn insert_action_outcome(&self, outcome: ActionOutcome) -> DbResult<()> {
    //     self.experience.insert_node(&Node::ActionOutcome(outcome))
    // }
    // 
    // /// Get an action outcome by ID
    // pub fn get_action_outcome(&self, outcome_id: &str) -> DbResult<Option<ActionOutcome>> {
    //     match self.experience.get_node(outcome_id)? {
    //         Some(Node::ActionOutcome(outcome)) => Ok(Some(outcome)),
    //         _ => Ok(None),
    //     }
    // }
    
    // ========== LAZY LOADING HELPERS ==========
    
    /// Get or lazy-load conversations/recent tier
    fn get_or_load_conversations_recent(&self) -> DbResult<Option<Arc<StorageManager>>> {
        let mut recent_guard = self.conversations_recent.write().unwrap();
        
        if recent_guard.is_none() {
            // Lazy load recent tier
            match StorageManager::open_typed_with_indexing(
                DatabaseType::Conversations,
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
        
        Ok(recent_guard.as_ref().map(|s| Arc::new(s.clone())))
    }
    
    /// Get or lazy-load a specific archive quarter
    pub fn get_or_load_archive(&self, quarter: &str) -> DbResult<Option<Arc<StorageManager>>> {
        let archives = self.conversations_archives.write().unwrap();
        
        if !archives.contains_key(quarter) {
            // TODO: Implement archive loading with quarter-specific paths
            // For now, archives are not yet implemented
        }
        
        Ok(archives.get(quarter).map(|s| Arc::new(s.clone())))
    }
    
    // ========== DIRECT ACCESS TO DATABASES (for specialized operations) ==========
    
    /// Get direct access to conversations/active database
    pub fn conversations_active(&self) -> Arc<StorageManager> {
        Arc::clone(&self.conversations_active)
    }
    
    /// Get direct access to knowledge/active database
    pub fn knowledge_active(&self) -> Arc<StorageManager> {
        Arc::clone(&self.knowledge_active)
    }
    
    /// Get direct access to knowledge/stable database
    pub fn knowledge_stable(&self) -> Arc<StorageManager> {
        Arc::clone(&self.knowledge_stable)
    }
    
    /// Get direct access to embeddings/active database
    pub fn embeddings_active(&self) -> Arc<StorageManager> {
        Arc::clone(&self.embeddings_active)
    }
    
    /// Get direct access to tool-results database
    pub fn tool_results(&self) -> Arc<StorageManager> {
        Arc::clone(&self.tool_results)
    }
    
    /// Get direct access to experience database
    pub fn experience(&self) -> Arc<StorageManager> {
        Arc::clone(&self.experience)
    }
    
    /// Get direct access to meta database
    pub fn meta(&self) -> Arc<StorageManager> {
        Arc::clone(&self.meta)
    }
}

// Make StorageManager cloneable for lazy loading
impl Clone for StorageManager {
    fn clone(&self) -> Self {
        // This is a shallow clone - all clones share the same underlying sled::Db
        Self {
            db: self.db.clone(),
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            embeddings: self.embeddings.clone(),
            index_manager: self.index_manager.as_ref().map(|im| Arc::clone(im)),
            db_type: self.db_type,
            tier: self.tier,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::NodeId;

    #[test]
    fn test_coordinator_initialization() {
        let coordinator = DatabaseCoordinator::new();
        assert!(coordinator.is_ok());
    }
    
    // Integration tests for coordinator require temp databases to avoid file locks.
    // Basic CRUD functionality is tested in storage_tests.rs with proper isolation.
}

