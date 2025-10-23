//! Action type constants - SINGLE SOURCE OF TRUTH
//! 
//! ⚠️  CRITICAL: MUST stay 100% synced with Python Server/core/message_fields.py
//! ⚠️  Both Python and Rust MUST have IDENTICAL constant names and values
//! ⚠️  Any change here MUST be reflected in Python, and vice versa
//! 
//! All action names, status codes, and message field names
//! Synced with Python core/message_types.py ActionType enum
//! 
//! All clients (Chrome Native Messaging, FastAPI, WebRTC) speak the same language.
//! 
//! NO STRING LITERALS ALLOWED IN HANDLERS - ALL MUST REFERENCE THESE CONSTANTS

// ============================================================
// ACTION TYPES - Commands from clients (synced with Python)
// ============================================================

/// Core system actions
pub mod system_actions {
    pub const PING: &str = "ping";
    pub const GET_SYSTEM_INFO: &str = "get_system_info";
    pub const EXECUTE_COMMAND: &str = "execute_command";
}

/// Model lifecycle actions (matches Python ActionType)
pub mod model_lifecycle {
    pub const PULL_MODEL: &str = "pull_model";
    pub const LOAD_MODEL: &str = "load_model";
    pub const UNLOAD_MODEL: &str = "unload_model";
    pub const DELETE_MODEL: &str = "delete_model";
    pub const GENERATE: &str = "generate";
    pub const GET_MODEL_STATE: &str = "get_model_state";
    pub const UPDATE_SETTINGS: &str = "update_settings";
    pub const STOP_GENERATION: &str = "stop_generation";
}

/// Embeddings and RAG actions
pub mod embeddings {
    pub const GENERATE_EMBEDDINGS: &str = "generate_embeddings";
    pub const SEMANTIC_SEARCH: &str = "semantic_search";
    pub const RERANK_DOCUMENTS: &str = "rerank_documents";
    pub const CLUSTER_TEXTS: &str = "cluster_texts";
    pub const RECOMMEND_ITEMS: &str = "recommend_items";
    pub const COMPUTE_SIMILARITY: &str = "compute_similarity";
}

/// Configuration actions
pub mod config {
    pub const GET_PARAMS: &str = "get_params";
    pub const SET_PARAMS: &str = "set_params";
    pub const GET_RECIPES: &str = "get_recipes";
    pub const GET_REGISTERED_MODELS: &str = "get_registered_models";
}

/// Resource management
pub mod resources {
    pub const QUERY_RESOURCES: &str = "query_resources";
    pub const LIST_LOADED_MODELS: &str = "list_loaded_models";
    pub const SELECT_ACTIVE_MODEL: &str = "select_active_model";
    pub const ESTIMATE_MODEL_SIZE: &str = "estimate_model_size";
}

/// Chat history & sync
pub mod chat {
    pub const CREATE_CONVERSATION: &str = "create_conversation";
    pub const GET_CONVERSATION: &str = "get_conversation";
    pub const LIST_CONVERSATIONS: &str = "list_conversations";
    pub const SEARCH_CONVERSATIONS: &str = "search_conversations";
    pub const ADD_MESSAGES: &str = "add_messages";
    pub const SYNC_PUSH: &str = "sync_push";
    pub const SYNC_PULL: &str = "sync_pull";
}

/// LM Studio specific actions
pub mod lmstudio {
    pub const CHECK_LMSTUDIO: &str = "check_lmstudio";
    pub const START_LMSTUDIO: &str = "start_lmstudio";
    pub const STOP_LMSTUDIO: &str = "stop_lmstudio";
    pub const LMSTUDIO_STATUS: &str = "lmstudio_status";
}

/// Rust-specific extended actions (not in Python yet)
pub mod rust_extended {
    pub const DOWNLOAD_MODEL: &str = "download_model";
    pub const GET_LOADED_MODELS: &str = "get_loaded_models";
    pub const GET_DOWNLOADED_MODELS: &str = "get_downloaded_models";
    pub const GET_AVAILABLE_MODELS: &str = "get_available_models";
    pub const GET_DOWNLOAD_STATUS: &str = "get_download_status";
    pub const GET_SYSTEM_RESOURCES: &str = "get_system_resources";
    pub const RECOMMEND_SPLIT: &str = "recommend_split";
    pub const ADD_MODEL_TO_LIST: &str = "add_model_to_list";
    pub const UNLOAD_ALL_MODELS: &str = "unload_all_models";
    pub const GET_CURRENT_MODEL: &str = "get_current_model";
    pub const SET_ACTIVE_MODEL: &str = "set_active_model";
    pub const GET_MEMORY_USAGE: &str = "get_memory_usage";
    pub const GET_ACTIVE_DOWNLOADS: &str = "get_active_downloads";
    pub const GET_MODELS_BY_TYPE: &str = "get_models_by_type";
    pub const GET_DEFAULT_MODEL: &str = "get_default_model";
}

// ============================================================
// STATUS CODES - Response status values
// ============================================================

pub mod status {
    pub const SUCCESS: &str = "success";
    pub const ERROR: &str = "error";
    pub const PENDING: &str = "pending";
}

// ============================================================
// DOWNLOAD STATUS - Download state tracking
// ============================================================

pub mod download_status {
    pub const DOWNLOADING: &str = "downloading";
    pub const COMPLETED: &str = "completed";
    pub const FAILED: &str = "failed";
    pub const CANCELLED: &str = "cancelled";
}

// ============================================================
// MESSAGE FIELDS - JSON message field names
// ============================================================

pub mod message_fields {
    // Request fields
    pub const ACTION: &str = "action";
    pub const MODEL_PATH: &str = "modelPath";
    pub const MODEL_ID: &str = "modelId";
    pub const MODEL_NAME: &str = "model_name";
    pub const MODEL: &str = "model";
    pub const REPO_ID: &str = "repoId";
    pub const FILE_PATH: &str = "filePath";
    pub const FILE_NAME: &str = "fileName";
    pub const MODEL_FILE: &str = "modelFile";
    pub const MODEL_SIZE: &str = "modelSize";
    pub const TOTAL_LAYERS: &str = "totalLayers";
    pub const CHECKPOINT: &str = "checkpoint";
    pub const MODEL_TYPE: &str = "modelType";
    pub const TYPE: &str = "type";
    pub const SIZE_GB: &str = "sizeGb";
    pub const LABELS: &str = "labels";
    pub const SETTINGS: &str = "settings";
    
    // Response fields
    pub const STATUS: &str = "status";
    pub const MESSAGE: &str = "message";
    pub const PAYLOAD: &str = "payload";
    pub const IS_READY: &str = "isReady";
    pub const BACKEND: &str = "backend";
    pub const LOADED_TO: &str = "loadedTo";
    pub const VRAM_USED: &str = "vramUsed";
    pub const RAM_USED: &str = "ramUsed";
    pub const VRAM_FREED: &str = "vramFreed";
    pub const RAM_FREED: &str = "ramFreed";
    pub const CONFIG: &str = "config";
    pub const MODELS: &str = "models";
    pub const COUNT: &str = "count";
    pub const LOADED_COUNT: &str = "loadedCount";
    pub const UNLOADED_MODELS: &str = "unloadedModels";
    pub const DOWNLOADS: &str = "downloads";
    pub const RECEIVED: &str = "received";
    
    // Memory & resource fields
    pub const CURRENT_MODEL: &str = "currentModel";
    pub const RAM: &str = "ram";
    pub const VRAM: &str = "vram";
    pub const LOADED_MODELS_COUNT: &str = "loadedModelsCount";
    pub const MEMORY_USED_BY_MODELS: &str = "memoryUsedByModels";
    pub const CACHED: &str = "cached";
    pub const DEFAULT_MODEL: &str = "defaultModel";
    pub const TIMESTAMP: &str = "timestamp";
    pub const TOTAL: &str = "total";
    pub const USED: &str = "used";
    pub const AVAILABLE: &str = "available";
}

// ============================================================
// LOAD TARGETS - Where models are loaded
// ============================================================

pub mod load_targets {
    pub const GPU: &str = "gpu";
    pub const CPU: &str = "cpu";
    pub const SPLIT: &str = "split";
}

// ============================================================
// MODEL TYPES - Model format identifiers
// ============================================================

pub mod model_types {
    pub const GGUF: &str = "gguf";
    pub const BITNET: &str = "bitnet";
    pub const ONNX: &str = "onnx";
    pub const SAFETENSORS: &str = "safetensors";
    pub const MEDIAPIPE: &str = "mediapipe";
}

// ============================================================
// BACKEND IDENTIFIERS
// ============================================================

pub mod backends {
    pub const RUST: &str = "Rust";
    pub const RUST_GGUF: &str = "Rust-GGUF";
    pub const RUST_BITNET: &str = "Rust-BitNet";
    pub const PYTHON_ONNX: &str = "Python-ONNX";
    pub const PYTHON_MEDIAPIPE: &str = "Python-MediaPipe";
}

// ============================================================
// EVENT TYPES - For progress/status events
// ============================================================

pub mod events {
    pub const DOWNLOAD_PROGRESS: &str = "download_progress";
    pub const LOAD_PROGRESS: &str = "load_progress";
    pub const GENERATION_PROGRESS: &str = "generation_progress";
    pub const MODEL_LOADED: &str = "model_loaded";
    pub const MODEL_UNLOADED: &str = "model_unloaded";
    pub const ERROR: &str = "error";
}

