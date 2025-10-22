//! Enrichment modules for autonomous knowledge processing.
//!
//! Each module implements a specific cognitive task:
//! - `semantic_indexer`: Generate vector embeddings
//! - `entity_linker`: Extract and link named entities
//! - `associative_linker`: Create similarity-based connections
//! - `summarizer`: Generate conversation summaries

pub mod semantic_indexer;
pub mod entity_linker;
pub mod associative_linker;
pub mod summarizer;

