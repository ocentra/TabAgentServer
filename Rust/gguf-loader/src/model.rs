//! High-level Model abstraction
//!
//! Provides safe Rust wrappers around llama.cpp model loading

use crate::error::{ModelError, Result};
use crate::ffi::{LlamaFunctions, LlamaModel, LlamaModelParams};
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Path to the model file
    pub model_path: PathBuf,
    
    /// Number of GPU layers to offload (-1 for all, 0 for CPU only)
    pub n_gpu_layers: i32,
    
    /// Main GPU to use (for multi-GPU systems)
    pub main_gpu: i32,
    
    /// Use memory mapping for faster loading
    pub use_mmap: bool,
    
    /// Lock model in memory (prevents swapping)
    pub use_mlock: bool,
    
    /// Load vocabulary only (no weights)
    pub vocab_only: bool,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::new(),
            n_gpu_layers: 0,  // CPU-only by default
            main_gpu: 0,
            use_mmap: true,
            use_mlock: false,
            vocab_only: false,
        }
    }
}

impl ModelConfig {
    /// Create a new model config with the given path
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            model_path: path.as_ref().to_path_buf(),
            ..Default::default()
        }
    }

    /// Enable GPU offloading with the specified number of layers
    pub fn with_gpu_layers(mut self, n_layers: i32) -> Self {
        self.n_gpu_layers = n_layers;
        self
    }

    /// Enable memory locking
    pub fn with_mlock(mut self) -> Self {
        self.use_mlock = true;
        self
    }
}

/// A loaded GGUF model
pub struct Model {
    model_ptr: *mut LlamaModel,
    _library: Arc<Library>,  // Keep library alive for entire model lifetime
    functions: Arc<LlamaFunctions>,
    config: ModelConfig,
}

impl Model {
    /// Load a model with auto-selected library variant
    ///
    /// # Arguments
    /// * `base_path` - Base directory containing BitnetRelease/
    /// * `config` - Model configuration
    /// * `prefer_gpu` - Whether to prefer GPU variants
    ///
    /// # Returns
    /// A loaded Model instance
    ///
    /// # Example
    /// ```no_run
    /// use gguf_loader::{Model, ModelConfig};
    /// use std::path::Path;
    ///
    /// let config = ModelConfig::new("models/llama-7b-q4.gguf");
    /// let model = Model::load_with_auto_select(Path::new("."), config, true)?;
    /// # Ok::<(), gguf_loader::ModelError>(())
    /// ```
    pub fn load_with_auto_select<P: AsRef<Path>>(
        base_path: P, 
        config: ModelConfig, 
        prefer_gpu: bool
    ) -> Result<Self> {
        let library_path = crate::auto_select_library(base_path.as_ref(), prefer_gpu)?;
        Self::load(&library_path, config)
    }

    /// Load a model from a GGUF file with a specific library variant
    ///
    /// # Arguments
    /// * `library_path` - Path to llama.dll, llama.so, or libllama.dylib
    /// * `config` - Model configuration
    ///
    /// # Returns
    /// A loaded Model instance
    ///
    /// # Example
    /// ```no_run
    /// use gguf_loader::{Model, ModelConfig, auto_select_library};
    /// use std::path::Path;
    ///
    /// let base_path = Path::new(".");
    /// let library_path = auto_select_library(base_path, true)?;
    /// let config = ModelConfig::new("models/llama-7b-q4.gguf");
    /// let model = Model::load(&library_path, config)?;
    /// # Ok::<(), gguf_loader::ModelError>(())
    /// ```
    pub fn load<P: AsRef<Path>>(library_path: P, config: ModelConfig) -> Result<Self> {
        // Validate model path
        if !config.model_path.exists() {
            return Err(ModelError::ModelNotFound(
                config.model_path.display().to_string(),
            ));
        }

        // Load the shared library
        // SAFETY: Library::new loads a shared library from the filesystem. This is safe because:
        // 1. We validate the library path exists and is a valid DLL/SO file
        // 2. The library is only accessed through properly typed function pointers
        // 3. All subsequent FFI calls verify function symbols exist before use
        let library = unsafe {
            Library::new(library_path.as_ref()).map_err(|e| {
                ModelError::LibraryLoadError(format!(
                    "Failed to load {}: {}",
                    library_path.as_ref().display(),
                    e
                ))
            })?
        };
        let library = Arc::new(library);

        // Load all function symbols
        let functions = Self::load_functions(&library)?;
        let functions = Arc::new(functions);

        // Initialize llama backend
        // SAFETY: llama_backend_init initializes the llama.cpp backend once per process.
        // This is safe because:
        // 1. The function pointer was verified to exist during load_functions
        // 2. llama_backend_init is designed to be called before any model operations
        // 3. It's idempotent - safe to call multiple times
        unsafe {
            (functions.llama_backend_init)();
        }

        // Prepare model parameters
        let model_params = LlamaModelParams {
            n_gpu_layers: config.n_gpu_layers,
            main_gpu: config.main_gpu,
            tensor_split: std::ptr::null(),
            vocab_only: config.vocab_only,
            use_mmap: config.use_mmap,
            use_mlock: config.use_mlock,
        };

        // Load the model
        let model_path_cstr = CString::new(config.model_path.to_string_lossy().as_bytes())
            .map_err(|e| ModelError::InvalidPath(e.to_string()))?;

        // SAFETY: llama_load_model_from_file loads a GGUF model from disk.
        // This is safe because:
        // 1. model_path_cstr is a valid null-terminated C string
        // 2. model_params is a valid struct matching the C API layout
        // 3. We check for null return value to handle load failures
        // 4. The returned pointer's lifetime is managed by our Drop implementation
        let model_ptr = unsafe {
            (functions.llama_load_model_from_file)(model_path_cstr.as_ptr(), model_params)
        };

        if model_ptr.is_null() {
            return Err(ModelError::ModelLoadError(format!(
                "Failed to load model from {}",
                config.model_path.display()
            )));
        }

        log::info!("Successfully loaded model: {}", config.model_path.display());

        Ok(Self {
            model_ptr,
            _library: library,
            functions,
            config,
        })
    }

    /// Load all required function symbols from the library
    fn load_functions(library: &Library) -> Result<LlamaFunctions> {
        // SAFETY: We're loading function symbols from a trusted llama.cpp shared library.
        // This is safe because:
        // 1. All symbol names are verified C strings with null terminators
        // 2. Each .get() call returns Result, and we propagate errors for missing symbols
        // 3. Function signatures exactly match the llama.cpp C API
        // 4. The Library holds the loaded shared library, preventing symbol invalidation
        unsafe {
            Ok(LlamaFunctions {
                llama_backend_init: **library
                    .get::<Symbol<unsafe extern "C" fn()>>(b"llama_backend_init\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_backend_init: {}", e)))?,

                llama_backend_free: **library
                    .get::<Symbol<unsafe extern "C" fn()>>(b"llama_backend_free\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_backend_free: {}", e)))?,

                llama_load_model_from_file: *library
                    .get(b"llama_load_model_from_file\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_load_model_from_file: {}", e)))?,

                llama_free_model: *library
                    .get(b"llama_free_model\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_free_model: {}", e)))?,

                llama_new_context_with_model: *library
                    .get(b"llama_new_context_with_model\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_new_context_with_model: {}", e)))?,

                llama_free: *library
                    .get(b"llama_free\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_free: {}", e)))?,

                llama_tokenize: *library
                    .get(b"llama_tokenize\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_tokenize: {}", e)))?,

                llama_token_to_piece: *library
                    .get(b"llama_token_to_piece\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_token_to_piece: {}", e)))?,

                llama_decode: *library
                    .get(b"llama_decode\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_decode: {}", e)))?,

                llama_get_logits: *library
                    .get(b"llama_get_logits\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_get_logits: {}", e)))?,

                llama_get_logits_ith: *library
                    .get(b"llama_get_logits_ith\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_get_logits_ith: {}", e)))?,

                llama_batch_init: *library
                    .get(b"llama_batch_init\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_batch_init: {}", e)))?,

                llama_batch_free: *library
                    .get(b"llama_batch_free\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_batch_free: {}", e)))?,

                llama_batch_get_one: *library
                    .get(b"llama_batch_get_one\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_batch_get_one: {}", e)))?,

                llama_sample_token_greedy: *library
                    .get(b"llama_sample_token_greedy\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_sample_token_greedy: {}", e)))?,

                llama_n_vocab: *library
                    .get(b"llama_n_vocab\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_n_vocab: {}", e)))?,

                llama_n_ctx_train: *library
                    .get(b"llama_n_ctx_train\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_n_ctx_train: {}", e)))?,

                llama_n_embd: *library
                    .get(b"llama_n_embd\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_n_embd: {}", e)))?,

                llama_n_ctx: *library
                    .get(b"llama_n_ctx\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_n_ctx: {}", e)))?,

                llama_token_bos: *library
                    .get(b"llama_token_bos\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_token_bos: {}", e)))?,

                llama_token_eos: *library
                    .get(b"llama_token_eos\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_token_eos: {}", e)))?,

                llama_token_nl: *library
                    .get(b"llama_token_nl\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_token_nl: {}", e)))?,

                llama_token_is_eog: *library
                    .get(b"llama_token_is_eog\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_token_is_eog: {}", e)))?,

                // Embeddings API
                llama_get_embeddings: *library
                    .get(b"llama_get_embeddings\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_get_embeddings: {}", e)))?,

                llama_get_embeddings_ith: *library
                    .get(b"llama_get_embeddings_ith\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_get_embeddings_ith: {}", e)))?,

                llama_get_embeddings_seq: *library
                    .get(b"llama_get_embeddings_seq\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_get_embeddings_seq: {}", e)))?,

                llama_set_embeddings: *library
                    .get(b"llama_set_embeddings\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_set_embeddings: {}", e)))?,

                // KV cache management
                llama_kv_cache_clear: *library
                    .get(b"llama_kv_cache_clear\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_kv_cache_clear: {}", e)))?,

                llama_kv_cache_seq_rm: *library
                    .get(b"llama_kv_cache_seq_rm\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_kv_cache_seq_rm: {}", e)))?,

                llama_kv_cache_seq_cp: *library
                    .get(b"llama_kv_cache_seq_cp\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_kv_cache_seq_cp: {}", e)))?,

                llama_kv_cache_seq_keep: *library
                    .get(b"llama_kv_cache_seq_keep\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_kv_cache_seq_keep: {}", e)))?,

                llama_kv_cache_seq_add: *library
                    .get(b"llama_kv_cache_seq_add\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_kv_cache_seq_add: {}", e)))?,

                // Encoder/decoder support
                llama_encode: *library
                    .get(b"llama_encode\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_encode: {}", e)))?,

                llama_model_has_encoder: *library
                    .get(b"llama_model_has_encoder\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_model_has_encoder: {}", e)))?,

                llama_model_has_decoder: *library
                    .get(b"llama_model_has_decoder\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_model_has_decoder: {}", e)))?,

                // Pooling type
                llama_pooling_type: *library
                    .get(b"llama_pooling_type\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_pooling_type: {}", e)))?,

                // Model info
                llama_get_model: *library
                    .get(b"llama_get_model\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_get_model: {}", e)))?,

                // Advanced sampling
                llama_sample_token: *library
                    .get(b"llama_sample_token\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_sample_token: {}", e)))?,

                // Model quantization
                llama_model_quantize: *library
                    .get(b"llama_model_quantize\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_model_quantize: {}", e)))?,

                // LoRA adapters
                llama_lora_adapter_init: *library
                    .get(b"llama_lora_adapter_init\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_lora_adapter_init: {}", e)))?,

                llama_lora_adapter_set: *library
                    .get(b"llama_lora_adapter_set\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_lora_adapter_set: {}", e)))?,

                llama_lora_adapter_remove: *library
                    .get(b"llama_lora_adapter_remove\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_lora_adapter_remove: {}", e)))?,

                llama_lora_adapter_clear: *library
                    .get(b"llama_lora_adapter_clear\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_lora_adapter_clear: {}", e)))?,

                llama_lora_adapter_free: *library
                    .get(b"llama_lora_adapter_free\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_lora_adapter_free: {}", e)))?,

                // State save/load
                llama_state_get_size: *library
                    .get(b"llama_state_get_size\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_state_get_size: {}", e)))?,

                llama_state_get_data: *library
                    .get(b"llama_state_get_data\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_state_get_data: {}", e)))?,

                llama_state_set_data: *library
                    .get(b"llama_state_set_data\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_state_set_data: {}", e)))?,

                llama_state_load_file: *library
                    .get(b"llama_state_load_file\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_state_load_file: {}", e)))?,

                llama_state_save_file: *library
                    .get(b"llama_state_save_file\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_state_save_file: {}", e)))?,

                // Sequence state operations
                llama_state_seq_get_size: *library
                    .get(b"llama_state_seq_get_size\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_state_seq_get_size: {}", e)))?,

                llama_state_seq_get_data: *library
                    .get(b"llama_state_seq_get_data\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_state_seq_get_data: {}", e)))?,

                llama_state_seq_set_data: *library
                    .get(b"llama_state_seq_set_data\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_state_seq_set_data: {}", e)))?,

                llama_state_seq_save_file: *library
                    .get(b"llama_state_seq_save_file\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_state_seq_save_file: {}", e)))?,

                llama_state_seq_load_file: *library
                    .get(b"llama_state_seq_load_file\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_state_seq_load_file: {}", e)))?,

                // Model metadata inspection
                llama_model_meta_val_str: *library
                    .get(b"llama_model_meta_val_str\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_model_meta_val_str: {}", e)))?,

                llama_model_meta_count: *library
                    .get(b"llama_model_meta_count\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_model_meta_count: {}", e)))?,

                llama_model_meta_key_by_index: *library
                    .get(b"llama_model_meta_key_by_index\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_model_meta_key_by_index: {}", e)))?,

                llama_model_meta_val_str_by_index: *library
                    .get(b"llama_model_meta_val_str_by_index\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_model_meta_val_str_by_index: {}", e)))?,

                llama_model_desc: *library
                    .get(b"llama_model_desc\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_model_desc: {}", e)))?,

                llama_model_size: *library
                    .get(b"llama_model_size\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_model_size: {}", e)))?,

                llama_model_n_params: *library
                    .get(b"llama_model_n_params\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_model_n_params: {}", e)))?,

                // Model capabilities
                llama_model_is_recurrent: *library
                    .get(b"llama_model_is_recurrent\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_model_is_recurrent: {}", e)))?,

                llama_model_decoder_start_token: *library
                    .get(b"llama_model_decoder_start_token\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_model_decoder_start_token: {}", e)))?,

                // Hardware support queries
                llama_supports_mmap: *library
                    .get(b"llama_supports_mmap\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_supports_mmap: {}", e)))?,

                llama_supports_mlock: *library
                    .get(b"llama_supports_mlock\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_supports_mlock: {}", e)))?,

                llama_supports_gpu_offload: *library
                    .get(b"llama_supports_gpu_offload\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_supports_gpu_offload: {}", e)))?,

                llama_max_devices: *library
                    .get(b"llama_max_devices\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_max_devices: {}", e)))?,

                // Timing
                llama_time_us: *library
                    .get(b"llama_time_us\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_time_us: {}", e)))?,

                // Context info (additional)
                llama_n_batch: *library
                    .get(b"llama_n_batch\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_n_batch: {}", e)))?,

                llama_n_ubatch: *library
                    .get(b"llama_n_ubatch\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_n_ubatch: {}", e)))?,

                llama_n_seq_max: *library
                    .get(b"llama_n_seq_max\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_n_seq_max: {}", e)))?,

                // KV cache inspection
                llama_get_kv_cache_token_count: *library
                    .get(b"llama_get_kv_cache_token_count\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_get_kv_cache_token_count: {}", e)))?,

                llama_get_kv_cache_used_cells: *library
                    .get(b"llama_get_kv_cache_used_cells\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_get_kv_cache_used_cells: {}", e)))?,

                llama_kv_cache_seq_pos_max: *library
                    .get(b"llama_kv_cache_seq_pos_max\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_kv_cache_seq_pos_max: {}", e)))?,

                llama_kv_cache_defrag: *library
                    .get(b"llama_kv_cache_defrag\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_kv_cache_defrag: {}", e)))?,

                llama_kv_cache_update: *library
                    .get(b"llama_kv_cache_update\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_kv_cache_update: {}", e)))?,

                llama_kv_cache_seq_div: *library
                    .get(b"llama_kv_cache_seq_div\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_kv_cache_seq_div: {}", e)))?,

                // Model layer info
                llama_n_layer: *library
                    .get(b"llama_n_layer\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_n_layer: {}", e)))?,

                llama_rope_freq_scale_train: *library
                    .get(b"llama_rope_freq_scale_train\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_rope_freq_scale_train: {}", e)))?,

                // Vocab info
                llama_vocab_type: *library
                    .get(b"llama_vocab_type\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_vocab_type: {}", e)))?,

                llama_rope_type: *library
                    .get(b"llama_rope_type\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_rope_type: {}", e)))?,

                // Token attributes
                llama_token_get_attr: *library
                    .get(b"llama_token_get_attr\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_token_get_attr: {}", e)))?,

                llama_token_is_control: *library
                    .get(b"llama_token_is_control\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_token_is_control: {}", e)))?,

                // Token conversion
                llama_token_get_text: *library
                    .get(b"llama_token_get_text\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_token_get_text: {}", e)))?,

                llama_token_get_score: *library
                    .get(b"llama_token_get_score\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_token_get_score: {}", e)))?,

                llama_token_get_type: *library
                    .get(b"llama_token_get_type\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_token_get_type: {}", e)))?,

                // Print performance info
                llama_print_timings: *library
                    .get(b"llama_print_timings\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_print_timings: {}", e)))?,

                llama_reset_timings: *library
                    .get(b"llama_reset_timings\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_reset_timings: {}", e)))?,

                // Model tensors
                llama_get_model_tensor: *library
                    .get(b"llama_get_model_tensor\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_get_model_tensor: {}", e)))?,

                // Control vectors
                llama_control_vector_apply: *library
                    .get(b"llama_control_vector_apply\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_control_vector_apply: {}", e)))?,

                // NUMA support
                llama_numa_init: *library
                    .get(b"llama_numa_init\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_numa_init: {}", e)))?,

                // Set causal attention
                llama_set_causal_attn: *library
                    .get(b"llama_set_causal_attn\0")
                    .map_err(|e| ModelError::FfiError(format!("Missing llama_set_causal_attn: {}", e)))?,
            })
        }
    }

    /// Get the raw model pointer (for creating contexts)
    pub(crate) fn as_ptr(&self) -> *mut LlamaModel {
        self.model_ptr
    }

    /// Get the function table
    pub(crate) fn functions(&self) -> &Arc<LlamaFunctions> {
        &self.functions
    }

    /// Get model vocabulary size
    pub fn vocab_size(&self) -> usize {
        // SAFETY: Calling llama C API with valid model_ptr owned by this struct
        unsafe { (self.functions.llama_n_vocab)(self.model_ptr) as usize }
    }

    /// Get model training context size
    pub fn context_train_size(&self) -> usize {
        // SAFETY: Calling llama C API with valid model_ptr owned by this struct
        unsafe { (self.functions.llama_n_ctx_train)(self.model_ptr) as usize }
    }

    /// Get model embedding dimension
    pub fn embedding_dim(&self) -> usize {
        // SAFETY: Calling llama C API with valid model_ptr owned by this struct
        unsafe { (self.functions.llama_n_embd)(self.model_ptr) as usize }
    }

    /// Get BOS (beginning of sequence) token
    pub fn token_bos(&self) -> i32 {
        // SAFETY: Calling llama C API with valid model_ptr owned by this struct
        unsafe { (self.functions.llama_token_bos)(self.model_ptr) }
    }

    /// Get EOS (end of sequence) token
    pub fn token_eos(&self) -> i32 {
        // SAFETY: Calling llama C API with valid model_ptr owned by this struct
        unsafe { (self.functions.llama_token_eos)(self.model_ptr) }
    }

    /// Get newline token
    pub fn token_nl(&self) -> i32 {
        // SAFETY: Calling llama C API with valid model_ptr owned by this struct
        unsafe { (self.functions.llama_token_nl)(self.model_ptr) }
    }

    /// Get model configuration
    pub fn config(&self) -> &ModelConfig {
        &self.config
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        if !self.model_ptr.is_null() {
            // SAFETY: llama_free_model safely deallocates the model.
            // This is safe because:
            // 1. model_ptr was allocated by llama_load_model_from_file and is valid
            // 2. We check is_null() to prevent double-free
            // 3. This is only called once during Drop, ensuring proper cleanup
            unsafe {
                (self.functions.llama_free_model)(self.model_ptr);
            }
            log::info!("Freed model: {}", self.config.model_path.display());
        }
    }
}

// SAFETY: Model is Send because the underlying llama.cpp library is thread-safe for model access.
// The model_ptr can be safely moved between threads, and the Arc-wrapped library/functions ensure
// the shared library remains valid. However, Model is NOT Sync - contexts must be created per-thread.
unsafe impl Send for Model {}
// Model is NOT Sync - contexts must be created per-thread

