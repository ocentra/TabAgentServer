//! Database Coordinator for MIA's multi-tier memory architecture
//!
//! This module provides the `DatabaseCoordinator` which manages all 7 database types
//! and their temperature tiers, providing a unified interface for cross-DB operations.
//!
//! # Concurrency Safety
//!
//! The DatabaseCoordinator is designed to be thread-safe and can be safely shared
//! across multiple threads. It uses the following concurrency primitives:
//!
//! - `Arc<StorageManager>`: For shared ownership of database instances
//! - `Arc<RwLock<Option<StorageManager>>>`: For lazy-loaded tiers that may be initialized concurrently
//! - `Arc<RwLock<HashMap<String, StorageManager>>>`: For archive tiers that may be accessed concurrently
//!
//! The underlying sled database is thread-safe, so multiple threads can safely
//! perform operations on the same database instances without additional synchronization.

use crate::{
    conversations::ConversationManager, embeddings::EmbeddingManager,
    experience::ExperienceManager, knowledge::KnowledgeManager, summaries::SummaryManager,
    tool_results::ToolResultManager, traits::*, DatabaseType, StorageManager, TemperatureTier,
};
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
    // Composed managers for different database types
    pub conversation_manager: ConversationManager,
    pub knowledge_manager: KnowledgeManager,
    pub embedding_manager: EmbeddingManager,
    pub tool_result_manager: ToolResultManager,
    pub experience_manager: ExperienceManager,
    pub summary_manager: SummaryManager,

    // ========== INDEX DATABASE ==========
    /// Meta: Query routing, performance stats, confidence maps
    pub meta: Arc<StorageManager>,
}

// Implement DirectAccessOperations for safe public access to storage managers
impl crate::traits::DirectAccessOperations for DatabaseCoordinator {
    fn conversations_active(&self) -> Arc<StorageManager> {
        self.conversation_manager.conversations_active.clone()
    }
    
    fn knowledge_active(&self) -> Arc<StorageManager> {
        self.knowledge_manager.knowledge_active.clone()
    }
    
    fn knowledge_stable(&self) -> Arc<StorageManager> {
        self.knowledge_manager.knowledge_stable.clone()
    }
    
    fn embeddings_active(&self) -> Arc<StorageManager> {
        self.embedding_manager.embeddings_active.clone()
    }
    
    fn tool_results(&self) -> Arc<StorageManager> {
        self.tool_result_manager.tool_results.clone()
    }
    
    fn experience(&self) -> Arc<StorageManager> {
        self.experience_manager.experience.clone()
    }
    
    fn meta(&self) -> Arc<StorageManager> {
        self.meta.clone()
    }
}

impl DatabaseCoordinator {
    /// Initialize ALL databases at platform-specific paths
    ///
    /// Opens HOT tiers immediately, leaves WARM/COLD tiers for lazy loading.
    pub fn new() -> DbResult<Self> {
        Self::with_base_path(None)
    }

    /// Initialize ALL databases at a custom base path (for testing)
    ///
    /// This method allows specifying a custom base path for all databases,
    /// which is useful for testing to avoid file locking conflicts.
    pub fn with_base_path(base_path: Option<std::path::PathBuf>) -> DbResult<Self> {
        // Helper function to open a database with optional custom base path
        let open_db = |db_type: DatabaseType,
                       tier: Option<TemperatureTier>|
         -> DbResult<StorageManager> {
            if let Some(ref base) = base_path {
                // Use custom path
                let path = if let Some(t) = tier {
                    base.join(db_type.name()).join(t.name())
                } else {
                    base.join(db_type.name())
                };

                // Ensure directory exists
                common::platform::ensure_db_directory(&path)?;

                let path_str = path.to_str().ok_or_else(|| {
                    common::DbError::InvalidOperation("Invalid UTF-8 in database path".to_string())
                })?;

                StorageManager::with_indexing(path_str)
            } else {
                // Use default platform paths
                StorageManager::open_typed_with_indexing(db_type, tier)
            }
        };

        // Helper function to open a database without indexing (for single-tier DBs)
        let open_db_no_index = |db_type: DatabaseType,
                                tier: Option<TemperatureTier>|
         -> DbResult<StorageManager> {
            if let Some(ref base) = base_path {
                // Use custom path
                let path = if let Some(t) = tier {
                    base.join(db_type.name()).join(t.name())
                } else {
                    base.join(db_type.name())
                };

                // Ensure directory exists
                common::platform::ensure_db_directory(&path)?;

                let path_str = path.to_str().ok_or_else(|| {
                    common::DbError::InvalidOperation("Invalid UTF-8 in database path".to_string())
                })?;

                StorageManager::new(path_str)
            } else {
                // Use default platform paths
                StorageManager::open_typed(db_type, tier)
            }
        };

        // Initialize all the managers
        let conversation_manager = ConversationManager {
            conversations_active: Arc::new(open_db(
                DatabaseType::Conversations,
                Some(TemperatureTier::Active),
            )?),
            conversations_recent: Arc::new(RwLock::new(None)),
            conversations_archives: Arc::new(RwLock::new(HashMap::new())),
        };

        let knowledge_manager = KnowledgeManager {
            knowledge_active: Arc::new(open_db(
                DatabaseType::Knowledge,
                Some(TemperatureTier::Active),
            )?),
            knowledge_stable: Arc::new(open_db(
                DatabaseType::Knowledge,
                Some(TemperatureTier::Stable),
            )?),
            knowledge_inferred: Arc::new(open_db(
                DatabaseType::Knowledge,
                Some(TemperatureTier::Inferred),
            )?),
        };

        let embedding_manager = EmbeddingManager {
            embeddings_active: Arc::new(open_db(
                DatabaseType::Embeddings,
                Some(TemperatureTier::Active),
            )?),
            embeddings_recent: Arc::new(RwLock::new(None)),
            embeddings_archives: Arc::new(RwLock::new(HashMap::new())),
        };

        let tool_result_manager = ToolResultManager {
            tool_results: Arc::new(open_db_no_index(DatabaseType::ToolResults, None)?),
        };

        let experience_manager = ExperienceManager {
            experience: Arc::new(open_db_no_index(DatabaseType::Experience, None)?),
        };

        let summary_manager = SummaryManager {
            summaries: Arc::new(RwLock::new(HashMap::new())),
        };

        Ok(Self {
            conversation_manager,
            knowledge_manager,
            embedding_manager,
            tool_result_manager,
            experience_manager,
            summary_manager,
            meta: Arc::new(open_db_no_index(DatabaseType::Meta, None)?),
        })
    }

    // ========== CONVERSATION OPERATIONS ==========

    /// Insert a message into conversations/active
    pub fn insert_message(&self, message: Message) -> DbResult<()> {
        self.conversation_manager.insert_message(message)
    }

    /// Get a message by ID, searching across all conversation tiers
    pub fn get_message(&self, message_id: &str) -> DbResult<Option<Message>> {
        self.conversation_manager.get_message(message_id)
    }

    /// Get a message by ID with a hint about which quarter it might be in
    pub fn get_message_with_hint(
        &self,
        message_id: &str,
        timestamp_hint_ms: i64,
    ) -> DbResult<Option<Message>> {
        self.conversation_manager
            .get_message_with_hint(message_id, timestamp_hint_ms)
    }

    /// Insert a chat into conversations/active
    pub fn insert_chat(&self, chat: Chat) -> DbResult<()> {
        self.conversation_manager.insert_chat(chat)
    }

    /// Get a chat by ID, searching across all conversation tiers
    pub fn get_chat(&self, chat_id: &str) -> DbResult<Option<Chat>> {
        self.conversation_manager.get_chat(chat_id)
    }

    /// Promote a message from active to recent tier based on age
    pub fn promote_message_to_recent(
        &self,
        message_id: &str,
        current_timestamp_ms: i64,
    ) -> DbResult<bool> {
        self.conversation_manager
            .promote_message_to_recent(message_id, current_timestamp_ms)
    }

    // ========== KNOWLEDGE OPERATIONS ==========

    /// Insert an entity into knowledge/active
    pub fn insert_entity(&self, entity: Entity) -> DbResult<()> {
        self.knowledge_manager.insert_entity(entity)
    }

    /// Get an entity by ID, searching active → stable → inferred
    pub fn get_entity(&self, entity_id: &str) -> DbResult<Option<Entity>> {
        self.knowledge_manager.get_entity(entity_id)
    }

    /// Promote an entity from active to stable (after 10+ mentions)
    pub fn promote_entity_to_stable(&self, entity_id: &str) -> DbResult<()> {
        self.knowledge_manager.promote_entity_to_stable(entity_id)
    }

    // ========== TOOL RESULTS OPERATIONS ==========

    /// Insert a web search result
    pub fn insert_web_search(&self, search: WebSearch) -> DbResult<()> {
        self.tool_result_manager.insert_web_search(search)
    }

    /// Get a web search by ID
    pub fn get_web_search(&self, search_id: &str) -> DbResult<Option<WebSearch>> {
        self.tool_result_manager.get_web_search(search_id)
    }

    /// Insert a scraped page
    pub fn insert_scraped_page(&self, page: ScrapedPage) -> DbResult<()> {
        self.tool_result_manager.insert_scraped_page(page)
    }

    // ========== EXPERIENCE OPERATIONS (CRITICAL for learning!) ==========

    /// Insert an action outcome (what happened when agent acted)
    pub fn insert_action_outcome(&self, outcome: ActionOutcome) -> DbResult<()> {
        self.experience_manager.insert_action_outcome(outcome)
    }

    /// Get an action outcome by ID
    pub fn get_action_outcome(&self, outcome_id: &str) -> DbResult<Option<ActionOutcome>> {
        self.experience_manager.get_action_outcome(outcome_id)
    }

    /// Update an existing action outcome with user feedback
    pub fn update_action_outcome_with_feedback(
        &self,
        outcome_id: &str,
        feedback: UserFeedback,
    ) -> DbResult<()> {
        self.experience_manager
            .update_action_outcome_with_feedback(outcome_id, feedback)
    }

    /// Get all action outcomes with a specific action type
    pub fn get_action_outcomes_by_type(&self, action_type: &str) -> DbResult<Vec<ActionOutcome>> {
        self.experience_manager
            .get_action_outcomes_by_type(action_type)
    }

    /// Record a success pattern by creating a new ActionOutcome to represent the pattern
    pub fn record_success_pattern(
        &self,
        pattern_id: &str,
        action_type: &str,
        confidence: f32,
    ) -> DbResult<()> {
        self.experience_manager
            .record_success_pattern(pattern_id, action_type, confidence)
    }

    /// Record an error pattern by creating a new ActionOutcome to represent the pattern
    pub fn record_error_pattern(
        &self,
        pattern_id: &str,
        action_type: &str,
        error_count: u32,
    ) -> DbResult<()> {
        self.experience_manager
            .record_error_pattern(pattern_id, action_type, error_count)
    }

    // ========== EMBEDDINGS OPERATIONS ==========

    /// Get an embedding by ID, searching across all embedding tiers
    pub fn get_embedding(&self, embedding_id: &str) -> DbResult<Option<Embedding>> {
        self.embedding_manager.get_embedding(embedding_id)
    }

    /// Get an embedding by ID with a hint about which quarter it might be in
    pub fn get_embedding_with_hint(
        &self,
        embedding_id: &str,
        timestamp_hint_ms: i64,
    ) -> DbResult<Option<Embedding>> {
        self.embedding_manager
            .get_embedding_with_hint(embedding_id, timestamp_hint_ms)
    }

    /// Insert an embedding into embeddings/active
    pub fn insert_embedding(&self, embedding: Embedding) -> DbResult<()> {
        self.embedding_manager.insert_embedding(embedding)
    }

    // ========== SUMMARIES OPERATIONS ==========

    /// Insert a summary into the appropriate tier
    pub fn insert_summary(&self, summary: Summary) -> DbResult<()> {
        self.summary_manager.insert_summary(summary)
    }

    /// Get a summary by ID, searching across all summary tiers
    pub fn get_summary(&self, summary_id: &str) -> DbResult<Option<Summary>> {
        self.summary_manager.get_summary(summary_id)
    }

    // ========== DIRECT ACCESS TO DATABASES (for specialized operations) ==========

    /// Get direct access to conversations/active database
    pub fn conversations_active(&self) -> Arc<StorageManager> {
        Arc::clone(&self.conversation_manager.conversations_active)
    }

    /// Get direct access to knowledge/active database
    pub fn knowledge_active(&self) -> Arc<StorageManager> {
        Arc::clone(&self.knowledge_manager.knowledge_active)
    }

    /// Get direct access to knowledge/stable database
    pub fn knowledge_stable(&self) -> Arc<StorageManager> {
        Arc::clone(&self.knowledge_manager.knowledge_stable)
    }

    /// Get direct access to embeddings/active database
    pub fn embeddings_active(&self) -> Arc<StorageManager> {
        Arc::clone(&self.embedding_manager.embeddings_active)
    }

    /// Get direct access to tool-results database
    pub fn tool_results(&self) -> Arc<StorageManager> {
        Arc::clone(&self.tool_result_manager.tool_results)
    }

    /// Get direct access to experience database
    pub fn experience(&self) -> Arc<StorageManager> {
        Arc::clone(&self.experience_manager.experience)
    }

    /// Get direct access to meta database
    pub fn meta(&self) -> Arc<StorageManager> {
        Arc::clone(&self.meta)
    }
}
