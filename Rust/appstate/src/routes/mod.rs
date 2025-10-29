//! AppState route handlers - REAL business logic.
//!
//! This module contains the actual implementation of all TabAgent features.
//! Transport layers (API, Native Messaging, WebRTC) forward requests here.

pub mod chat;
pub mod generate;
pub mod embeddings;
pub mod models;
pub mod system;
pub mod hf_auth;
pub mod hardware;
pub mod sessions;
pub mod rag;
pub mod params;
pub mod stats;
pub mod resources;
pub mod management;
pub mod media;

// Re-export handlers for convenience
pub use chat::handle as handle_chat;
pub use generate::handle as handle_generate;
pub use embeddings::handle as handle_embeddings;

pub use models::{
    handle_load as handle_load_model,
    handle_unload as handle_unload_model,
    handle_list as handle_list_models,
    handle_info as handle_model_info,
};

pub use system::{
    handle_health,
    handle_system_info,
};

pub use hf_auth::{
    handle_set_token as handle_set_hf_token,
    handle_get_token_status as handle_get_hf_token_status,
    handle_clear_token as handle_clear_hf_token,
};

pub use hardware::{
    handle_get_info as handle_get_hardware_info,
    handle_check_feasibility as handle_check_model_feasibility,
    handle_get_recommended as handle_get_recommended_models,
};

pub use sessions::{
    handle_chat_history,
    handle_save_message,
};

pub use rag::{
    handle_query as handle_rag_query,
    handle_rerank,
    handle_stop_generation,
};

pub use params::{
    handle_get_params,
    handle_set_params,
};

pub use stats::handle_get_stats;

pub use resources::{
    handle_get_resources,
    handle_estimate_memory,
};

pub use management::{
    handle_get_recipes,
    handle_get_embedding_models,
    handle_get_loaded_models,
    handle_select_model,
    handle_pull_model,
    handle_delete_model,
};

pub use media::{
    handle_audio_stream,
    handle_video_stream,
};

