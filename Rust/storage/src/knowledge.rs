//! Knowledge operations for the MIA storage system
//!
//! This module provides implementations for knowledge-related operations
//! including entities across different temperature tiers.

use crate::{traits::KnowledgeOperations, StorageManager};
use common::{models::*, DbResult};
use std::sync::Arc;

/// Implementation of knowledge operations
pub struct KnowledgeManager {
    /// Knowledge/active: Recently mentioned entities (HOT)
    pub(crate) knowledge_active: Arc<StorageManager>,
    /// Knowledge/stable: Proven important entities 10+ mentions (HOT)
    pub(crate) knowledge_stable: Arc<StorageManager>,
    /// Knowledge/inferred: Experimental/low-confidence (COLD)
    pub(crate) knowledge_inferred: Arc<StorageManager>,
}

impl KnowledgeOperations for KnowledgeManager {
    /// Insert an entity into knowledge/active
    fn insert_entity(&self, entity: Entity) -> DbResult<()> {
        self.knowledge_active.insert_node(&Node::Entity(entity))
    }

    /// Get an entity by ID, searching active → stable → inferred
    fn get_entity(&self, entity_id: &str) -> DbResult<Option<Entity>> {
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
    fn promote_entity_to_stable(&self, entity_id: &str) -> DbResult<()> {
        if let Some(entity) = self.get_entity(entity_id)? {
            // Insert to stable
            self.knowledge_stable.insert_node(&Node::Entity(entity))?;
            // Remove from active
            self.knowledge_active.delete_node(entity_id)?;
        }
        Ok(())
    }
}
