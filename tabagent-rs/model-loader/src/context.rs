//! Inference context for text generation
//!
//! Provides safe wrappers for creating inference contexts and generating text

use crate::error::{ModelError, Result};
use crate::ffi::{LlamaBatch, LlamaContext, LlamaContextParams, LlamaToken, LlamaTokenData, LlamaTokenDataArray};
use crate::model::Model;
use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::sync::Arc;

/// Parameters for text generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationParams {
    /// Maximum context size
    pub n_ctx: u32,
    
    /// Batch size for prompt processing
    pub n_batch: u32,
    
    /// Number of threads to use
    pub n_threads: i32,
    
    /// Maximum tokens to generate
    pub max_tokens: usize,
    
    /// Temperature for sampling (0.0 = greedy, higher = more random)
    pub temperature: f32,
    
    /// Top-p sampling threshold
    pub top_p: f32,
    
    /// Top-k sampling threshold
    pub top_k: i32,
    
    /// Repetition penalty
    pub repeat_penalty: f32,
}

impl Default for GenerationParams {
    fn default() -> Self {
        Self {
            n_ctx: 2048,
            n_batch: 512,
            n_threads: 4,
            max_tokens: 512,
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            repeat_penalty: 1.1,
        }
    }
}

/// An inference context for a loaded model
pub struct Context {
    context_ptr: *mut LlamaContext,
    model: Arc<Model>,
    params: GenerationParams,
}

impl Context {
    /// Create a new inference context for the given model
    ///
    /// # Arguments
    /// * `model` - The loaded model to use
    /// * `params` - Generation parameters
    ///
    /// # Returns
    /// A new Context instance
    ///
    /// # Example
    /// ```no_run
    /// use model_loader::{Model, ModelConfig, Context, GenerationParams};
    /// use std::path::Path;
    /// use std::sync::Arc;
    ///
    /// let config = ModelConfig::new("models/llama-7b-q4.gguf");
    /// let model = Arc::new(Model::load(Path::new("llama.dll"), config)?);
    /// let params = GenerationParams::default();
    /// let context = Context::new(model, params)?;
    /// # Ok::<(), model_loader::ModelError>(())
    /// ```
    pub fn new(model: Arc<Model>, params: GenerationParams) -> Result<Self> {
        let ctx_params = LlamaContextParams {
            n_ctx: params.n_ctx,
            n_batch: params.n_batch,
            n_threads: params.n_threads,
            n_threads_batch: params.n_threads,
            ..Default::default()
        };

        let context_ptr = unsafe {
            (model.functions().llama_new_context_with_model)(
                model.as_ptr(),
                ctx_params,
            )
        };

        if context_ptr.is_null() {
            return Err(ModelError::ContextCreationError(
                "Failed to create llama context".to_string(),
            ));
        }

        log::info!("Created inference context with n_ctx={}", params.n_ctx);

        Ok(Self {
            context_ptr,
            model,
            params,
        })
    }

    /// Tokenize a string
    ///
    /// # Arguments
    /// * `text` - The text to tokenize
    /// * `add_bos` - Whether to add BOS token
    ///
    /// # Returns
    /// Vector of token IDs
    pub fn tokenize(&self, text: &str, add_bos: bool) -> Result<Vec<LlamaToken>> {
        let text_cstr = CString::new(text)
            .map_err(|e| ModelError::InvalidParameter(format!("Invalid text: {}", e)))?;

        // First pass: get token count
        let n_tokens = unsafe {
            (self.model.functions().llama_tokenize)(
                self.model.as_ptr(),
                text_cstr.as_ptr(),
                text.len() as i32,
                std::ptr::null_mut(),
                0,
                add_bos,
                true, // parse special tokens
            )
        };

        if n_tokens < 0 {
            return Err(ModelError::InferenceError(
                "Tokenization failed".to_string(),
            ));
        }

        // Second pass: get actual tokens
        let mut tokens = vec![0i32; n_tokens as usize];
        let result = unsafe {
            (self.model.functions().llama_tokenize)(
                self.model.as_ptr(),
                text_cstr.as_ptr(),
                text.len() as i32,
                tokens.as_mut_ptr(),
                n_tokens,
                add_bos,
                true,
            )
        };

        if result < 0 {
            return Err(ModelError::InferenceError(
                "Tokenization failed".to_string(),
            ));
        }

        tokens.truncate(result as usize);
        Ok(tokens)
    }

    /// Convert a token to text
    ///
    /// # Arguments
    /// * `token` - The token ID to convert
    ///
    /// # Returns
    /// The text representation of the token
    pub fn token_to_text(&self, token: LlamaToken) -> Result<String> {
        let mut buf = vec![0u8; 32]; // Most tokens are < 32 bytes

        let n_bytes = unsafe {
            (self.model.functions().llama_token_to_piece)(
                self.model.as_ptr(),
                token,
                buf.as_mut_ptr() as *mut i8,
                buf.len() as i32,
            )
        };

        if n_bytes < 0 {
            return Err(ModelError::InferenceError(format!(
                "Failed to convert token {} to text",
                token
            )));
        }

        // If buffer was too small, retry with correct size
        if n_bytes as usize > buf.len() {
            buf.resize(n_bytes as usize, 0);
            let result = unsafe {
                (self.model.functions().llama_token_to_piece)(
                    self.model.as_ptr(),
                    token,
                    buf.as_mut_ptr() as *mut i8,
                    buf.len() as i32,
                )
            };

            if result < 0 {
                return Err(ModelError::InferenceError(format!(
                    "Failed to convert token {} to text",
                    token
                )));
            }
        }

        buf.truncate(n_bytes as usize);
        String::from_utf8(buf).map_err(|e| {
            ModelError::InferenceError(format!("Invalid UTF-8 in token: {}", e))
        })
    }

    /// Generate text from a prompt
    ///
    /// # Arguments
    /// * `prompt` - The input prompt
    ///
    /// # Returns
    /// Generated text
    ///
    /// # Example
    /// ```no_run
    /// # use model_loader::{Model, ModelConfig, Context, GenerationParams};
    /// # use std::path::Path;
    /// # use std::sync::Arc;
    /// # let config = ModelConfig::new("models/llama-7b-q4.gguf");
    /// # let model = Arc::new(Model::load(Path::new("llama.dll"), config)?);
    /// # let context = Context::new(model, GenerationParams::default())?;
    /// let response = context.generate("Hello, world!")?;
    /// println!("Response: {}", response);
    /// # Ok::<(), model_loader::ModelError>(())
    /// ```
    pub fn generate(&mut self, prompt: &str) -> Result<String> {
        // Tokenize the prompt
        let tokens = self.tokenize(prompt, true)?;
        log::debug!("Tokenized prompt: {} tokens", tokens.len());

        if tokens.is_empty() {
            return Err(ModelError::InvalidParameter(
                "Empty prompt after tokenization".to_string(),
            ));
        }

        // TODO: Implement full inference loop with batching
        // This is a simplified version - full implementation would include:
        // - Batch processing with llama_decode
        // - Logits extraction with llama_get_logits
        // - Sampling (greedy, top-k, top-p, temperature)
        // - EOS detection
        // - KV cache management

        log::warn!("Full inference not yet implemented - this is a stub");
        
        Ok("(Inference not yet implemented - FFI bindings are ready)".to_string())
    }

    /// Get the generation parameters
    pub fn params(&self) -> &GenerationParams {
        &self.params
    }

    /// Get the context size
    pub fn context_size(&self) -> u32 {
        unsafe { (self.model.functions().llama_n_ctx)(self.context_ptr) }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        if !self.context_ptr.is_null() {
            unsafe {
                (self.model.functions().llama_free)(self.context_ptr);
            }
            log::info!("Freed inference context");
        }
    }
}

// Context is Send because each context is independent
unsafe impl Send for Context {}
// Context is NOT Sync - must not be accessed from multiple threads simultaneously

