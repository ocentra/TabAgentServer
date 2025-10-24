//! Experience operations for the MIA storage system
//!
//! This module provides implementations for experience-related operations
//! including action outcomes and learning patterns.

use crate::{traits::ExperienceOperations, StorageManager};
use common::{models::*, DbResult};
use std::sync::Arc;

/// Implementation of experience operations
pub struct ExperienceManager {
    /// Experience: Agent action outcomes, user feedback, patterns (CRITICAL for learning!)
    pub(crate) experience: Arc<StorageManager>,
}

impl ExperienceOperations for ExperienceManager {
    /// Insert an action outcome (what happened when agent acted)
    fn insert_action_outcome(&self, outcome: ActionOutcome) -> DbResult<()> {
        self.experience.insert_node(&Node::ActionOutcome(outcome))
    }

    /// Get an action outcome by ID
    fn get_action_outcome(&self, outcome_id: &str) -> DbResult<Option<ActionOutcome>> {
        match self.experience.get_node(outcome_id)? {
            Some(Node::ActionOutcome(outcome)) => Ok(Some(outcome)),
            _ => Ok(None),
        }
    }

    /// Update an existing action outcome with user feedback
    fn update_action_outcome_with_feedback(
        &self,
        outcome_id: &str,
        feedback: UserFeedback,
    ) -> DbResult<()> {
        if let Some(Node::ActionOutcome(mut outcome)) = self.experience.get_node(outcome_id)? {
            outcome.user_feedback = Some(feedback);
            self.experience.insert_node(&Node::ActionOutcome(outcome))
        } else {
            Err(common::DbError::NotFound(outcome_id.to_string()))
        }
    }

    /// Get all action outcomes with a specific action type
    fn get_action_outcomes_by_type(&self, _action_type: &str) -> DbResult<Vec<ActionOutcome>> {
        // This is a simplified implementation
        // In a real system, we would want to use indexes for efficient querying
        let outcomes = Vec::new();

        // We would need to iterate through all nodes in the experience database
        // and filter for ActionOutcome nodes with the matching action_type
        // This is a placeholder implementation

        Ok(outcomes)
    }

    /// Record a success pattern by creating a new ActionOutcome to represent the pattern
    fn record_success_pattern(
        &self,
        pattern_id: &str,
        action_type: &str,
        confidence: f32,
    ) -> DbResult<()> {
        let pattern_outcome = ActionOutcome {
            id: common::NodeId::new(format!("pattern_{}", pattern_id)),
            action_type: format!("success_pattern_{}", action_type),
            action_args: serde_json::json!({"confidence": confidence}),
            result: serde_json::json!({"pattern_id": pattern_id, "type": "success"}),
            user_feedback: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0),
            conversation_context: "pattern_learning".to_string(),
        };

        self.experience
            .insert_node(&Node::ActionOutcome(pattern_outcome))
    }

    /// Record an error pattern by creating a new ActionOutcome to represent the pattern
    fn record_error_pattern(
        &self,
        pattern_id: &str,
        action_type: &str,
        error_count: u32,
    ) -> DbResult<()> {
        let pattern_outcome = ActionOutcome {
            id: common::NodeId::new(format!("pattern_{}", pattern_id)),
            action_type: format!("error_pattern_{}", action_type),
            action_args: serde_json::json!({"error_count": error_count}),
            result: serde_json::json!({"pattern_id": pattern_id, "type": "error"}),
            user_feedback: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0),
            conversation_context: "pattern_learning".to_string(),
        };

        self.experience
            .insert_node(&Node::ActionOutcome(pattern_outcome))
    }
}
