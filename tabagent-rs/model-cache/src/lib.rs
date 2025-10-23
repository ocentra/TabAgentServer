pub mod error;
pub mod schema;
pub mod init;
pub mod manifest;
pub mod storage;
pub mod download;
pub mod cache;
pub mod catalog;

pub use error::{ModelCacheError, Result};
pub use schema::{
    ModelCacheSchema, DatabaseName, StoreName, QuantStatus,
    SchemaValidation, CURRENT_SCHEMA_VERSION,
};
pub use init::{DatabaseCoordinator, InitState, InitializationResult};
pub use manifest::{ManifestEntry, QuantInfo};
pub use storage::{ChunkStorage, FileMetadata, FileChunkIterator, ChunkManifest, StorageProgressCallback};
pub use download::{ModelDownloader, ModelInfo, ProgressCallback};
pub use cache::{ModelCache, CacheStats};
pub use catalog::{ModelCatalog, ModelCatalogEntry, ModelCatalogConfig};

