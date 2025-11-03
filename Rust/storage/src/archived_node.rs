//! Archived data access wrappers.

use crate::engine::ReadGuard;
use common::{DbResult, models::Node};
use rkyv::Archive;

/// Wrapper that holds a ReadGuard and provides access to archived Node data.
pub struct ArchivedNodeRef<G: ReadGuard> {
    _guard: G,
    archived: &'static <Node as Archive>::Archived,
}

impl<G: ReadGuard> ArchivedNodeRef<G> {
    pub(crate) fn new(guard: G) -> DbResult<Self> {
        let archived = guard.archived::<Node>()
            .map_err(|e| common::DbError::InvalidOperation(e))?;
        let archived = unsafe { std::mem::transmute(archived) };
        Ok(Self { _guard: guard, archived })
    }
    
    #[inline]
    pub fn archived(&self) -> &<Node as Archive>::Archived {
        self.archived
    }
    
    /// Get message text without deserializing
    #[inline]
    pub fn message_text(&self) -> Option<&str> {
        match self.archived {
            rkyv::Archived::<Node>::Message(m) => Some(m.text_content.as_str()),
            _ => None,
        }
    }
    
    /// Get message timestamp without deserializing
    #[inline]
    pub fn message_timestamp(&self) -> Option<i64> {
        match self.archived {
            rkyv::Archived::<Node>::Message(m) => Some(m.timestamp.into()),
            _ => None,
        }
    }
    
    /// Get message sender without deserializing
    #[inline]
    pub fn message_sender(&self) -> Option<&str> {
        match self.archived {
            rkyv::Archived::<Node>::Message(m) => Some(m.sender.as_str()),
            _ => None,
        }
    }
    
    /// Get message ID without deserializing
    #[inline]
    pub fn message_id(&self) -> Option<&str> {
        match self.archived {
            rkyv::Archived::<Node>::Message(m) => Some(m.id.0.as_str()),
            _ => None,
        }
    }
    
    /// Get message chat_id without deserializing
    #[inline]
    pub fn message_chat_id(&self) -> Option<&str> {
        match self.archived {
            rkyv::Archived::<Node>::Message(m) => Some(m.chat_id.0.as_str()),
            _ => None,
        }
    }
    
    /// Get chat title without deserializing
    #[inline]
    pub fn chat_title(&self) -> Option<&str> {
        match self.archived {
            rkyv::Archived::<Node>::Chat(c) => Some(c.title.as_str()),
            _ => None,
        }
    }
    
    /// Get entity label without deserializing
    #[inline]
    pub fn entity_label(&self) -> Option<&str> {
        match self.archived {
            rkyv::Archived::<Node>::Entity(e) => Some(e.label.as_str()),
            _ => None,
        }
    }
    
    /// Get entity type without deserializing
    #[inline]
    pub fn entity_type(&self) -> Option<&str> {
        match self.archived {
            rkyv::Archived::<Node>::Entity(e) => Some(e.entity_type.as_str()),
            _ => None,
        }
    }
    
    /// Get summary content without deserializing
    #[inline]
    pub fn summary_content(&self) -> Option<&str> {
        match self.archived {
            rkyv::Archived::<Node>::Summary(s) => Some(s.content.as_str()),
            _ => None,
        }
    }
    
    /// Check if node is a Message variant
    #[inline]
    pub fn is_message(&self) -> bool {
        matches!(self.archived, rkyv::Archived::<Node>::Message(_))
    }
    
    /// Check if node is a Chat variant
    #[inline]
    pub fn is_chat(&self) -> bool {
        matches!(self.archived, rkyv::Archived::<Node>::Chat(_))
    }
    
    /// Check if node is an Entity variant
    #[inline]
    pub fn is_entity(&self) -> bool {
        matches!(self.archived, rkyv::Archived::<Node>::Entity(_))
    }
    
    /// Deserialize to owned type
    pub fn deserialize(&self) -> DbResult<Node> {
        // Use from_bytes with the guard's data
        rkyv::from_bytes::<Node, rkyv::rancor::Error>(self._guard.data())
            .map_err(|e| common::DbError::Serialization(e.to_string()))
    }
}

/// Wrapper for archived Edge data.
pub struct ArchivedEdgeRef<G: ReadGuard> {
    _guard: G,
    archived: &'static <common::models::Edge as Archive>::Archived,
}

impl<G: ReadGuard> ArchivedEdgeRef<G> {
    pub(crate) fn new(guard: G) -> DbResult<Self> {
        let archived = guard.archived::<common::models::Edge>()
            .map_err(|e| common::DbError::InvalidOperation(e))?;
        let archived = unsafe { std::mem::transmute(archived) };
        Ok(Self { _guard: guard, archived })
    }
    
    #[inline]
    pub fn archived(&self) -> &<common::models::Edge as Archive>::Archived {
        self.archived
    }
    
    /// Get edge type without deserializing
    #[inline]
    pub fn edge_type(&self) -> &str {
        self.archived.edge_type.as_str()
    }
    
    /// Get from_node ID without deserializing
    #[inline]
    pub fn from_node(&self) -> &str {
        self.archived.from_node.0.as_str()
    }
    
    /// Get to_node ID without deserializing
    #[inline]
    pub fn to_node(&self) -> &str {
        self.archived.to_node.0.as_str()
    }
    
    /// Get created_at timestamp without deserializing
    #[inline]
    pub fn created_at(&self) -> i64 {
        self.archived.created_at.into()
    }
    
    /// Deserialize to owned type (ALLOCATES)
    pub fn deserialize(&self) -> DbResult<common::models::Edge> {
        // Use from_bytes with the guard's data
        rkyv::from_bytes::<common::models::Edge, rkyv::rancor::Error>(self._guard.data())
            .map_err(|e| common::DbError::Serialization(e.to_string()))
    }
}

/// Wrapper for archived Embedding data.
pub struct ArchivedEmbeddingRef<G: ReadGuard> {
    _guard: G,
    archived: &'static <common::models::Embedding as Archive>::Archived,
}

impl<G: ReadGuard> ArchivedEmbeddingRef<G> {
    pub(crate) fn new(guard: G) -> DbResult<Self> {
        let archived = guard.archived::<common::models::Embedding>()
            .map_err(|e| common::DbError::InvalidOperation(e))?;
        let archived = unsafe { std::mem::transmute(archived) };
        Ok(Self { _guard: guard, archived })
    }
    
    #[inline]
    pub fn archived(&self) -> &<common::models::Embedding as Archive>::Archived {
        self.archived
    }
    
    /// Get vector slice without deserializing
    #[inline]
    pub fn vector(&self) -> Vec<f32> {
        // Convert from archived f32_le to native f32
        self.archived.vector.iter().map(|&v| v.into()).collect()
    }
    
    /// Get vector dimension without deserializing
    #[inline]
    pub fn dimension(&self) -> usize {
        self.archived.vector.len()
    }
    
    /// Get model name without deserializing
    #[inline]
    pub fn model(&self) -> &str {
        self.archived.model.as_str()
    }
    
    /// Deserialize to owned type (ALLOCATES)
    pub fn deserialize(&self) -> DbResult<common::models::Embedding> {
        // Use from_bytes with the guard's data
        rkyv::from_bytes::<common::models::Embedding, rkyv::rancor::Error>(self._guard.data())
            .map_err(|e| common::DbError::Serialization(e.to_string()))
    }
}

