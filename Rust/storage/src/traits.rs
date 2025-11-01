//! Traits for the MIA storage system
//!
//! This module defines traits that provide interfaces for different types of
//! database operations, making the system more modular and maintainable.

use crate::{time_scope::TimeScope, DefaultStorageManager, TemperatureTier};
use common::{models::*, DbResult};
use std::sync::Arc;

/// Trait for conversation-related operations
pub trait ConversationOperations {
    /// Insert a message into conversations/active
    fn insert_message(&self, message: Message) -> DbResult<()>;

    /// Get a message by ID, searching across all conversation tiers
    fn get_message(&self, message_id: &str) -> DbResult<Option<Message>>;

    /// Get a message by ID with a hint about which quarter it might be in
    fn get_message_with_hint(
        &self,
        message_id: &str,
        timestamp_hint_ms: i64,
    ) -> DbResult<Option<Message>>;

    /// Insert a chat into conversations/active
    fn insert_chat(&self, chat: Chat) -> DbResult<()>;

    /// Get a chat by ID, searching across all conversation tiers
    fn get_chat(&self, chat_id: &str) -> DbResult<Option<Chat>>;

    /// Promote a message from active to recent tier based on age
    fn promote_message_to_recent(
        &self,
        message_id: &str,
        current_timestamp_ms: i64,
    ) -> DbResult<bool>;

    /// Get or lazy-load conversations/recent tier
    fn get_or_load_conversations_recent(&self) -> DbResult<Option<Arc<DefaultStorageManager>>>;

    /// Get or lazy-load a specific archive quarter
    fn get_or_load_archive(&self, quarter: &str) -> DbResult<Option<Arc<DefaultStorageManager>>>;

    /// Get the quarter string for a given timestamp
    fn get_quarter_for_timestamp(&self, timestamp_ms: i64) -> String;

    /// Get a message from a specific archive quarter
    fn get_message_from_archive(
        &self,
        message_id: &str,
        quarter: &str,
    ) -> DbResult<Option<Message>>;
}

/// Trait for knowledge-related operations
pub trait KnowledgeOperations {
    /// Insert an entity into knowledge/active
    fn insert_entity(&self, entity: Entity) -> DbResult<()>;

    /// Get an entity by ID, searching active → stable → inferred
    fn get_entity(&self, entity_id: &str) -> DbResult<Option<Entity>>;

    /// Promote an entity from active to stable (after 10+ mentions)
    fn promote_entity_to_stable(&self, entity_id: &str) -> DbResult<()>;
}

/// Trait for embedding-related operations
pub trait EmbeddingOperations {
    /// Get an embedding by ID, searching across all embedding tiers
    fn get_embedding(&self, embedding_id: &str) -> DbResult<Option<Embedding>>;

    /// Get an embedding by ID with a hint about which quarter it might be in
    fn get_embedding_with_hint(
        &self,
        embedding_id: &str,
        timestamp_hint_ms: i64,
    ) -> DbResult<Option<Embedding>>;

    /// Insert an embedding into embeddings/active
    fn insert_embedding(&self, embedding: Embedding) -> DbResult<()>;

    /// Get or lazy-load embeddings/recent tier
    fn get_or_load_embeddings_recent(&self) -> DbResult<Option<Arc<DefaultStorageManager>>>;

    /// Get or lazy-load a specific embeddings archive quarter
    fn get_or_load_embeddings_archive(
        &self,
        quarter: &str,
    ) -> DbResult<Option<Arc<DefaultStorageManager>>>;

    /// Get an embedding from a specific archive quarter
    fn get_embedding_from_archive(
        &self,
        embedding_id: &str,
        quarter: &str,
    ) -> DbResult<Option<Embedding>>;
}

/// Trait for tool result operations
pub trait ToolResultOperations {
    /// Insert a web search result
    fn insert_web_search(&self, search: WebSearch) -> DbResult<()>;

    /// Get a web search by ID
    fn get_web_search(&self, search_id: &str) -> DbResult<Option<WebSearch>>;

    /// Insert a scraped page
    fn insert_scraped_page(&self, page: ScrapedPage) -> DbResult<()>;
}

/// Trait for experience operations (critical for learning)
pub trait ExperienceOperations {
    /// Insert an action outcome (what happened when agent acted)
    fn insert_action_outcome(&self, outcome: ActionOutcome) -> DbResult<()>;

    /// Get an action outcome by ID
    fn get_action_outcome(&self, outcome_id: &str) -> DbResult<Option<ActionOutcome>>;

    /// Update an existing action outcome with user feedback
    fn update_action_outcome_with_feedback(
        &self,
        outcome_id: &str,
        feedback: UserFeedback,
    ) -> DbResult<()>;

    /// Get all action outcomes with a specific action type
    fn get_action_outcomes_by_type(&self, action_type: &str) -> DbResult<Vec<ActionOutcome>>;

    /// Record a success pattern by creating a new ActionOutcome to represent the pattern
    fn record_success_pattern(
        &self,
        pattern_id: &str,
        action_type: &str,
        confidence: f32,
    ) -> DbResult<()>;

    /// Record an error pattern by creating a new ActionOutcome to represent the pattern
    fn record_error_pattern(
        &self,
        pattern_id: &str,
        action_type: &str,
        error_count: u32,
    ) -> DbResult<()>;
}

/// Trait for summary operations
pub trait SummaryOperations {
    /// Get or lazy-load a specific summary tier
    fn get_or_load_summary(&self, tier: TemperatureTier) -> DbResult<Arc<DefaultStorageManager>>;

    /// Insert a summary into the appropriate tier
    fn insert_summary(&self, summary: Summary) -> DbResult<()>;

    /// Get a summary by ID, searching across all summary tiers
    fn get_summary(&self, summary_id: &str) -> DbResult<Option<Summary>>;
}

/// Trait for time-based query operations
pub trait TimeBasedQueryOperations {
    /// Get messages within a specific time scope
    fn get_messages_in_time_scope(
        &self,
        time_scope: TimeScope,
        current_chat_id: Option<&str>,
        current_time_ms: i64,
    ) -> DbResult<Vec<Message>>;

    /// Get related messages within a time scope
    fn get_related_messages_in_time_scope(
        &self,
        message_id: &str,
        time_scope: TimeScope,
        current_time_ms: i64,
    ) -> DbResult<Vec<Message>>;

    /// Get messages for a specific entity within a time scope
    fn get_messages_for_entity_in_time_scope(
        &self,
        entity_id: &str,
        time_scope: TimeScope,
        current_time_ms: i64,
    ) -> DbResult<Vec<Message>>;

    /// Get entities mentioned in a specific message
    fn get_entities_for_message(&self, message_id: &str) -> DbResult<Vec<Entity>>;

    /// Get entities mentioned in a specific chat
    fn get_entities_for_chat(&self, chat_id: &str) -> DbResult<Vec<Entity>>;

    /// Get related chats within a time scope
    fn get_related_chats_in_time_scope(
        &self,
        chat_id: &str,
        time_scope: TimeScope,
        current_time_ms: i64,
    ) -> DbResult<Vec<Chat>>;

    /// Get chats for a specific entity within a time scope
    fn get_chats_for_entity_in_time_scope(
        &self,
        entity_id: &str,
        time_scope: TimeScope,
        current_time_ms: i64,
    ) -> DbResult<Vec<Chat>>;

    /// Get recent messages within a chat (last N messages)
    fn get_recent_messages_in_chat(&self, chat_id: &str, count: usize) -> DbResult<Vec<Message>>;

    /// Get messages from the last N messages in a chat within a time scope
    fn get_recent_messages_in_chat_in_time_scope(
        &self,
        chat_id: &str,
        count: usize,
        time_scope: TimeScope,
        current_time_ms: i64,
    ) -> DbResult<Vec<Message>>;
}

/// Trait for direct database access
pub trait DirectAccessOperations {
    /// Get direct access to conversations/active database
    fn conversations_active(&self) -> Arc<DefaultStorageManager>;

    /// Get direct access to knowledge/active database
    fn knowledge_active(&self) -> Arc<DefaultStorageManager>;

    /// Get direct access to knowledge/stable database
    fn knowledge_stable(&self) -> Arc<DefaultStorageManager>;

    /// Get direct access to embeddings/active database
    fn embeddings_active(&self) -> Arc<DefaultStorageManager>;

    /// Get direct access to tool-results database
    fn tool_results(&self) -> Arc<DefaultStorageManager>;

    /// Get direct access to experience database
    fn experience(&self) -> Arc<DefaultStorageManager>;

    /// Get direct access to meta database
    fn meta(&self) -> Arc<DefaultStorageManager>;
}
