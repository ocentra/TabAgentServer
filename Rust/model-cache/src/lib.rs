pub mod error;
pub mod schema;
pub mod manifest;
pub mod storage;
pub mod download;
pub mod cache;
pub mod detection;
pub mod hf_client;
pub mod manifest_builder;
pub mod tasks;

pub use error::{ModelCacheError, Result};
pub use schema::{
    ModelCacheSchema, DatabaseName, StoreName, QuantStatus,
    SchemaValidation, CURRENT_SCHEMA_VERSION,
};
pub use manifest::{ManifestEntry, QuantInfo};
pub use storage::{ChunkStorage, FileMetadata, FileChunkIterator, ChunkManifest, StorageProgressCallback};
pub use download::{ModelDownloader, ModelInfo, ProgressCallback};
pub use cache::{ModelCache, CacheStats};
pub use detection::{
    ModelType, Backend, ModelInfo as DetectionModelInfo, 
    detect_from_file_path, detect_from_repo_name, extract_repo_from_path,
    detect_task_from_name, detect_task_from_config, detect_task_unified
};
pub use hf_client::{fetch_repo_metadata, fetch_model_config, HfRepoMetadata, HfModelConfig, HfFile, extract_clean_dtype, is_supporting_file, is_onnx_file, is_onnx_external_data};
pub use manifest_builder::{build_manifest_from_hf, ExtensionManifestEntry, ExtensionQuantInfo, DEFAULT_SERVER_ONLY_SIZE, DEFAULT_BYPASS_MODELS};
pub use tasks::*;

