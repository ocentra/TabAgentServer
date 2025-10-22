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
    library: Arc<Library>,
    functions: Arc<LlamaFunctions>,
    config: ModelConfig,
}

impl Model {
    /// Load a model from a GGUF file
    ///
    /// # Arguments
    /// * `library_path` - Path to llama.dll or llama.so
    /// * `config` - Model configuration
    ///
    /// # Returns
    /// A loaded Model instance
    ///
    /// # Example
    /// ```no_run
    /// use model_loader::{Model, ModelConfig};
    /// use std::path::Path;
    ///
    /// let config = ModelConfig::new("models/llama-7b-q4.gguf");
    /// let model = Model::load(Path::new("llama.dll"), config)?;
    /// # Ok::<(), model_loader::ModelError>(())
    /// ```
    pub fn load<P: AsRef<Path>>(library_path: P, config: ModelConfig) -> Result<Self> {
        // Validate model path
        if !config.model_path.exists() {
            return Err(ModelError::ModelNotFound(
                config.model_path.display().to_string(),
            ));
        }

        // Load the shared library
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
            library,
            functions,
            config,
        })
    }

    /// Load all required function symbols from the library
    fn load_functions(library: &Library) -> Result<LlamaFunctions> {
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

    /// Get the library handle
    pub(crate) fn library(&self) -> &Arc<Library> {
        &self.library
    }

    /// Get model vocabulary size
    pub fn vocab_size(&self) -> usize {
        unsafe { (self.functions.llama_n_vocab)(self.model_ptr) as usize }
    }

    /// Get model training context size
    pub fn context_train_size(&self) -> usize {
        unsafe { (self.functions.llama_n_ctx_train)(self.model_ptr) as usize }
    }

    /// Get model embedding dimension
    pub fn embedding_dim(&self) -> usize {
        unsafe { (self.functions.llama_n_embd)(self.model_ptr) as usize }
    }

    /// Get BOS (beginning of sequence) token
    pub fn token_bos(&self) -> i32 {
        unsafe { (self.functions.llama_token_bos)(self.model_ptr) }
    }

    /// Get EOS (end of sequence) token
    pub fn token_eos(&self) -> i32 {
        unsafe { (self.functions.llama_token_eos)(self.model_ptr) }
    }

    /// Get newline token
    pub fn token_nl(&self) -> i32 {
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
            unsafe {
                (self.functions.llama_free_model)(self.model_ptr);
            }
            log::info!("Freed model: {}", self.config.model_path.display());
        }
    }
}

// Model is Send because the underlying C library is thread-safe for model access
unsafe impl Send for Model {}
// Model is NOT Sync - contexts must be created per-thread

