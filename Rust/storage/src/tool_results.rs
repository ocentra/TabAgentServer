//! Tool result operations for the MIA storage system
//!
//! This module provides implementations for tool result operations
//! including web searches and scraped pages.

use crate::{traits::ToolResultOperations, StorageManager};
use common::{models::*, DbResult};
use std::sync::Arc;

/// Implementation of tool result operations
pub struct ToolResultManager {
    /// Tool-results: Cached searches, scrapes, API responses
    pub(crate) tool_results: Arc<StorageManager>,
}

impl ToolResultOperations for ToolResultManager {
    /// Insert a web search result
    fn insert_web_search(&self, search: WebSearch) -> DbResult<()> {
        self.tool_results.insert_node(&Node::WebSearch(search))
    }

    /// Get a web search by ID
    fn get_web_search(&self, search_id: &str) -> DbResult<Option<WebSearch>> {
        match self.tool_results.get_node(search_id)? {
            Some(Node::WebSearch(search)) => Ok(Some(search)),
            _ => Ok(None),
        }
    }

    /// Insert a scraped page
    fn insert_scraped_page(&self, page: ScrapedPage) -> DbResult<()> {
        self.tool_results.insert_node(&Node::ScrapedPage(page))
    }
}
