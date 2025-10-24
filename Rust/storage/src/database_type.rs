//! Database type system for MIA's multi-tier memory architecture
//!
//! This module defines the 7 database types and temperature tiers
//! that make up MIA's cognitive memory system.

use common::platform;
use std::path::PathBuf;

/// Database types in the MIA memory system
///
/// Each type serves a specific purpose in the cognitive architecture:
/// - SOURCE: Critical user data (cannot lose!)
/// - DERIVED: Regeneratable from SOURCE
/// - EXTERNAL: Cached external data
/// - LEARNING: Agent experience and feedback
/// - INDEXES: Performance optimization metadata
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DatabaseType {
    /// SOURCE: User conversations (CRITICAL - cannot lose!)
    /// Contains chats, messages, attachments
    Conversations,

    /// DERIVED: Extracted entities and relationships (regeneratable)
    /// Contains entities extracted by weaver
    Knowledge,

    /// DERIVED: Semantic embeddings for vector search (regeneratable)
    /// Contains vectors and HNSW indexes
    Embeddings,

    /// EXTERNAL: Cached tool results (searches, scrapes, APIs)
    /// Contains web search results, scraped pages, API responses
    ToolResults,

    /// LEARNING: Agent experience and feedback (CRITICAL for learning!)
    /// Contains action outcomes, user feedback, success/error patterns
    Experience,

    /// DERIVED: Hierarchical summaries (regeneratable)
    /// Contains daily/weekly/monthly summaries
    Summaries,

    /// INDEXES: Query optimization metadata (rebuildable)
    /// Contains query routing cache, performance stats
    Meta,

    /// MODELS: Model files and manifests (separate system)
    /// Already implemented in model-cache crate
    ModelCache,
}

/// Temperature tier for data lifecycle management
///
/// Hot data is always in RAM, cold data is loaded on-demand.
/// This keeps queries fast even with years of accumulated data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TemperatureTier {
    /// HOT: 0-30 days, always in RAM, <1ms queries
    Active,

    /// WARM: 30-90 days, lazy load on first access, <10ms queries
    Recent,

    /// COLD: 90+ days, on-demand load per quarter, 100ms queries (acceptable)
    Archive,

    /// STABLE: For knowledge - proven entities (10+ mentions)
    Stable,

    /// INFERRED: For knowledge - experimental/low-confidence entities
    Inferred,

    /// SESSION: For summaries - current session (in-memory, volatile)
    Session,

    /// DAILY: For summaries - daily summaries (last 30 days)
    Daily,

    /// WEEKLY: For summaries - weekly summaries (last 6 months)
    Weekly,

    /// MONTHLY: For summaries - monthly summaries (all time)
    Monthly,
}

impl DatabaseType {
    /// Get platform-specific path for this database type and tier
    ///
    /// Returns the full path where this database should be stored.
    /// Creates parent directories if they don't exist.
    pub fn get_path(&self, tier: Option<TemperatureTier>) -> PathBuf {
        let base = platform::get_default_db_path();

        match (self, tier) {
            // === CONVERSATIONS (3 tiers) ===
            (DatabaseType::Conversations, Some(TemperatureTier::Active)) => {
                base.join("conversations").join("active")
            }
            (DatabaseType::Conversations, Some(TemperatureTier::Recent)) => {
                base.join("conversations").join("recent")
            }
            (DatabaseType::Conversations, Some(TemperatureTier::Archive)) => {
                base.join("conversations").join("archive")
            }
            (DatabaseType::Conversations, None) => base.join("conversations"),

            // === KNOWLEDGE (3 tiers) ===
            (DatabaseType::Knowledge, Some(TemperatureTier::Active)) => {
                base.join("knowledge").join("active")
            }
            (DatabaseType::Knowledge, Some(TemperatureTier::Stable)) => {
                base.join("knowledge").join("stable")
            }
            (DatabaseType::Knowledge, Some(TemperatureTier::Inferred)) => {
                base.join("knowledge").join("inferred")
            }
            (DatabaseType::Knowledge, None) => base.join("knowledge"),

            // === EMBEDDINGS (3 tiers) ===
            (DatabaseType::Embeddings, Some(TemperatureTier::Active)) => {
                base.join("embeddings").join("active")
            }
            (DatabaseType::Embeddings, Some(TemperatureTier::Recent)) => {
                base.join("embeddings").join("recent")
            }
            (DatabaseType::Embeddings, Some(TemperatureTier::Archive)) => {
                base.join("embeddings").join("archive")
            }
            (DatabaseType::Embeddings, None) => base.join("embeddings"),

            // === SUMMARIES (4 tiers) ===
            (DatabaseType::Summaries, Some(TemperatureTier::Session)) => {
                base.join("summaries").join("session")
            }
            (DatabaseType::Summaries, Some(TemperatureTier::Daily)) => {
                base.join("summaries").join("daily")
            }
            (DatabaseType::Summaries, Some(TemperatureTier::Weekly)) => {
                base.join("summaries").join("weekly")
            }
            (DatabaseType::Summaries, Some(TemperatureTier::Monthly)) => {
                base.join("summaries").join("monthly")
            }
            (DatabaseType::Summaries, None) => base.join("summaries"),

            // === TOOL-RESULTS (single tier) ===
            (DatabaseType::ToolResults, _) => base.join("tool-results"),

            // === EXPERIENCE (single tier) ===
            (DatabaseType::Experience, _) => base.join("experience"),

            // === META (single tier) ===
            (DatabaseType::Meta, _) => base.join("meta"),

            // === MODEL-CACHE (single tier, already exists) ===
            (DatabaseType::ModelCache, _) => base.join("model-cache"),

            // === CATCH-ALL for invalid tier combinations ===
            // This handles cases like Conversations with Stable tier, etc.
            _ => {
                // For any other combination, use the base database path
                base.join(self.name())
            }
        }
    }

    /// Get human-readable name for this database type
    pub fn name(&self) -> &'static str {
        match self {
            DatabaseType::Conversations => "conversations",
            DatabaseType::Knowledge => "knowledge",
            DatabaseType::Embeddings => "embeddings",
            DatabaseType::ToolResults => "tool-results",
            DatabaseType::Experience => "experience",
            DatabaseType::Summaries => "summaries",
            DatabaseType::Meta => "meta",
            DatabaseType::ModelCache => "model-cache",
        }
    }

    /// Is this a SOURCE database? (cannot lose data!)
    pub fn is_source(&self) -> bool {
        matches!(self, DatabaseType::Conversations | DatabaseType::Experience)
    }

    /// Is this a DERIVED database? (can regenerate from source)
    pub fn is_derived(&self) -> bool {
        matches!(
            self,
            DatabaseType::Knowledge | DatabaseType::Embeddings | DatabaseType::Summaries
        )
    }

    /// Is this an EXTERNAL database? (cached external data)
    pub fn is_external(&self) -> bool {
        matches!(self, DatabaseType::ToolResults)
    }

    /// Is this an INDEX database? (rebuildable)
    pub fn is_index(&self) -> bool {
        matches!(self, DatabaseType::Meta)
    }

    /// Get default tiers for this database type
    pub fn default_tiers(&self) -> Vec<TemperatureTier> {
        match self {
            DatabaseType::Conversations => vec![
                TemperatureTier::Active,
                TemperatureTier::Recent,
                TemperatureTier::Archive,
            ],
            DatabaseType::Knowledge => vec![
                TemperatureTier::Active,
                TemperatureTier::Stable,
                TemperatureTier::Inferred,
            ],
            DatabaseType::Embeddings => vec![
                TemperatureTier::Active,
                TemperatureTier::Recent,
                TemperatureTier::Archive,
            ],
            DatabaseType::Summaries => vec![
                TemperatureTier::Session,
                TemperatureTier::Daily,
                TemperatureTier::Weekly,
                TemperatureTier::Monthly,
            ],
            // Single-tier databases
            DatabaseType::ToolResults
            | DatabaseType::Experience
            | DatabaseType::Meta
            | DatabaseType::ModelCache => vec![],
        }
    }
}

impl TemperatureTier {
    /// Get human-readable name for this tier
    pub fn name(&self) -> &'static str {
        match self {
            TemperatureTier::Active => "active",
            TemperatureTier::Recent => "recent",
            TemperatureTier::Archive => "archive",
            TemperatureTier::Stable => "stable",
            TemperatureTier::Inferred => "inferred",
            TemperatureTier::Session => "session",
            TemperatureTier::Daily => "daily",
            TemperatureTier::Weekly => "weekly",
            TemperatureTier::Monthly => "monthly",
        }
    }

    /// Is this a HOT tier? (always loaded)
    pub fn is_hot(&self) -> bool {
        matches!(
            self,
            TemperatureTier::Active | TemperatureTier::Stable | TemperatureTier::Session
        )
    }

    /// Is this a WARM tier? (lazy load)
    pub fn is_warm(&self) -> bool {
        matches!(self, TemperatureTier::Recent | TemperatureTier::Daily)
    }

    /// Is this a COLD tier? (on-demand)
    pub fn is_cold(&self) -> bool {
        matches!(
            self,
            TemperatureTier::Archive
                | TemperatureTier::Inferred
                | TemperatureTier::Weekly
                | TemperatureTier::Monthly
        )
    }
}
