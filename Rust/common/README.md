# Common Crate

**Foundation types and utilities shared across all workspace crates.**

## Purpose

The `common` crate sits at the bottom of the dependency hierarchy with **zero workspace dependencies**. It provides the fundamental building blocks that all other crates depend on.

## Responsibilities

### 1. Type Aliases
- `NodeId` - Unique identifier for nodes
- `EdgeId` - Unique identifier for edges  
- `EmbeddingId` - Unique identifier for vector embeddings

### 2. Error Types
- `DbError` - Unified error type for all database operations
- `DbResult<T>` - Standard Result type alias

### 3. Data Models
All core data structures using the **Hybrid Schema Model**:
- **Strongly-typed fields** for queryable, critical data
- **Flexible `metadata` field** for extensibility
- **Node Types**: Chat, Message, Summary, Attachment, Entity, WebSearch, ScrapedPage, Bookmark, ImageMetadata, AudioTranscript, ModelInfo
- **Edge**: Directed, typed relationships between nodes
- **Embedding**: High-dimensional vector embeddings (384/768/1536 dimensions)

### 4. Serialization Helpers
- `json_metadata` - Custom `serde` module for `bincode` compatibility with JSON metadata fields

## Architecture Principle

**Zero Dependencies**: This crate depends on NO other workspace crates, ensuring:
- No circular dependencies
- Easy to reason about
- Stable foundation for all other crates

## Usage

``rust
use common::{NodeId, DbError, DbResult};
use common::models::{Node, Chat, Message, Edge};

fn process_node(id: NodeId) -> DbResult<()> {
    if id.is_empty() {
        return Err(DbError::NotFound("Empty ID".to_string()));
    }
    Ok(())
}
```

## Testing

The common crate includes comprehensive tests:
- 7 documentation tests covering all major APIs
- 8 unit tests covering newtype wrappers, error types, and platform functionality
- All serialization/deserialization functionality validated

## Dependencies

- `serde` - Serialization framework
- `serde_json` - JSON support for metadata
- `thiserror` - Error type derivation
- `bincode` - Binary serialization
- `sled` - For error interop

## See Also

- Parent: [Main README](../README.md)
- Usage: [storage/README.md](../storage/README.md)
- Progress: [TODO.md](./TODO.md)
