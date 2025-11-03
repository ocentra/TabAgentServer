//! Inference context for text generation
//!
//! Provides safe wrappers for creating inference contexts and generating text

use crate::error::{ModelError, Result};
use crate::ffi::{LlamaContext, LlamaContextParams, LlamaToken};
use crate::model::Model;
use serde::{Deserialize, Serialize};
use std::ffi::CString;
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

        // SAFETY: llama_new_context_with_model creates a new inference context.
        // This is safe because:
        // 1. model.as_ptr() is a valid model pointer from a loaded Model
        // 2. ctx_params is a valid struct matching the C API layout
        // 3. We check for null return value to handle creation failures
        // 4. The returned pointer's lifetime is managed by our Drop implementation
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
        // SAFETY: Calling llama_tokenize with null output buffer to get token count.
        // This is safe because:
        // 1. text_cstr is a valid null-terminated C string
        // 2. Passing null_mut() with size 0 is explicitly supported by llama.cpp for counting
        // 3. model.as_ptr() is valid and owned by self.model
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
        // SAFETY: Second call to llama_tokenize with properly sized output buffer.
        // This is safe because:
        // 1. tokens buffer was allocated with exact size n_tokens from first call
        // 2. as_mut_ptr() provides a valid mutable pointer to the buffer
        // 3. All parameters match the first call that determined buffer size
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

        // SAFETY: Calling llama_token_to_piece to convert token ID to text.
        // Safe because buf is a valid mutable buffer and we handle buffer resize if needed.
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
            // SAFETY: Second call with correctly sized buffer based on first call's return value.
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
        use crate::ffi::{LlamaTokenData, LlamaTokenDataArray};
        
        // Tokenize the prompt
        let mut tokens = self.tokenize(prompt, true)?;
        log::debug!("Tokenized prompt: {} tokens", tokens.len());

        if tokens.is_empty() {
            return Err(ModelError::InvalidParameter(
                "Empty prompt after tokenization".to_string(),
            ));
        }

        // Check if we have enough context
        let n_ctx = self.context_size();
        let n_predict = self.params.max_tokens.min((n_ctx as usize).saturating_sub(tokens.len()));
        
        if tokens.len() >= n_ctx as usize {
            return Err(ModelError::InferenceError(
                format!("Prompt too long: {} tokens exceeds context size {}", tokens.len(), n_ctx)
            ));
        }

        log::info!("Starting generation: {} prompt tokens, max {} new tokens", tokens.len(), n_predict);

        // SAFETY: Initialize batch structure for inference.
        // Safe because we're calling the llama.cpp batch API as documented.
        let batch = unsafe {
            (self.model.functions().llama_batch_init)(
                512,  // max tokens in batch
                0,    // embeddings mode (0 = tokens)
                1,    // max sequences
            )
        };

        // Clear KV cache before generation
        // SAFETY: Clearing KV cache with valid context pointer owned by this struct
        unsafe {
            (self.model.functions().llama_kv_cache_clear)(self.context_ptr);
        }

        // Decode the prompt tokens
        // SAFETY: Processing prompt through batch API.
        // We use llama_batch_get_one as a simple helper for single-sequence inference.
        unsafe {
            let prompt_batch = (self.model.functions().llama_batch_get_one)(
                tokens.as_mut_ptr(),
                tokens.len() as i32,
                0,  // starting position
                0,  // sequence ID
            );

            let decode_result = (self.model.functions().llama_decode)(self.context_ptr, prompt_batch);
            if decode_result != 0 {
                (self.model.functions().llama_batch_free)(batch);
                return Err(ModelError::InferenceError(
                    "Failed to decode prompt".to_string()
                ));
            }
        }

        let mut output = String::new();
        let n_vocab = self.model.vocab_size() as i32;
        let mut n_cur = tokens.len() as i32;

        // Generation loop - same pattern as simple.cpp
        for _i in 0..n_predict {
            // SAFETY: Get logits from the last token position.
            // Safe because we just successfully decoded tokens.
            let logits = unsafe {
                (self.model.functions().llama_get_logits_ith)(
                    self.context_ptr,
                    -1,  // last token
                )
            };

            if logits.is_null() {
                // SAFETY: Free batch before returning error - batch was initialized above
                unsafe { (self.model.functions().llama_batch_free)(batch); }
                return Err(ModelError::InferenceError(
                    "Failed to get logits".to_string()
                ));
            }

            // Build candidates array for sampling
            // SAFETY: Reading logits and building token candidates.
            // Safe because logits is a valid pointer from llama.cpp.
            let mut candidates: Vec<LlamaTokenData> = (0..n_vocab)
                .map(|token_id| unsafe {
                    LlamaTokenData {
                        id: token_id,
                        logit: *logits.offset(token_id as isize),
                        p: 0.0,
                    }
                })
                .collect();

            let mut candidates_p = LlamaTokenDataArray {
                data: candidates.as_mut_ptr(),
                size: candidates.len(),
                sorted: false,
            };

            // Sample next token (greedy for now)
            // SAFETY: Calling sampling function with valid candidates array.
            let new_token_id = unsafe {
                (self.model.functions().llama_sample_token_greedy)(
                    self.context_ptr,
                    &mut candidates_p,
                )
            };

            // Check for end of generation
            // SAFETY: Checking if token is end-of-generation marker.
            let is_eog = unsafe {
                (self.model.functions().llama_token_is_eog)(
                    self.model.as_ptr(),
                    new_token_id,
                )
            };

            if is_eog {
                log::debug!("Hit EOG token, stopping generation");
                break;
            }

            // Convert token to text and append
            let token_text = self.token_to_text(new_token_id)?;
            output.push_str(&token_text);

            // Decode the new token for next iteration
            // SAFETY: Decoding single new token through batch API.
            unsafe {
                let mut new_token = new_token_id;
                let next_batch = (self.model.functions().llama_batch_get_one)(
                    &mut new_token,
                    1,      // one token
                    n_cur,  // current position
                    0,      // sequence ID
                );

                let decode_result = (self.model.functions().llama_decode)(self.context_ptr, next_batch);
                if decode_result != 0 {
                    (self.model.functions().llama_batch_free)(batch);
                    return Err(ModelError::InferenceError(
                        "Failed to decode new token".to_string()
                    ));
                }
            }

            n_cur += 1;
        }

        // Free batch resources
        unsafe {
            (self.model.functions().llama_batch_free)(batch);
        }

        log::info!("Generation complete: {} tokens generated", output.split_whitespace().count());
        Ok(output)
    }

    /// Get the generation parameters
    pub fn params(&self) -> &GenerationParams {
        &self.params
    }

    /// Get the context size
    pub fn context_size(&self) -> u32 {
        // SAFETY: Calling llama C API with valid context_ptr owned by this struct
        unsafe { (self.model.functions().llama_n_ctx)(self.context_ptr) }
    }
    
    /// Generate embeddings for input text
    ///
    /// This enables embedding mode, processes the input tokens, and returns
    /// the normalized embedding vector.
    ///
    /// # Arguments
    /// * `text` - Input text to embed
    ///
    /// # Returns
    /// Normalized embedding vector (dimension = model.embedding_dim())
    ///
    /// # Example
    /// ```no_run
    /// # use gguf_loader::{Model, ModelConfig, Context, GenerationParams};
    /// # use std::path::Path;
    /// # use std::sync::Arc;
    /// # let config = ModelConfig::new("models/llama-7b-q4.gguf");
    /// # let model = Arc::new(Model::load(Path::new("llama.dll"), config)?);
    /// # let mut context = Context::new(model.clone(), GenerationParams::default())?;
    /// let embedding = context.generate_embeddings("Hello, world!")?;
    /// println!("Embedding dimension: {}", embedding.len());
    /// # Ok::<(), gguf_loader::ModelError>(())
    /// ```
    pub fn generate_embeddings(&mut self, text: &str) -> Result<Vec<f32>> {
        // Enable embedding mode
        // SAFETY: Calling llama_set_embeddings with valid context pointer
        unsafe {
            (self.model.functions().llama_set_embeddings)(self.context_ptr, true);
        }
        
        // Tokenize the input
        let tokens = self.tokenize(text, true)?;
        log::debug!("Tokenized {} tokens for embedding", tokens.len());
        
        if tokens.is_empty() {
            return Err(ModelError::InvalidParameter(
                "Empty input after tokenization".to_string(),
            ));
        }
        
        // Check context size
        let n_ctx = self.context_size();
        if tokens.len() > n_ctx as usize {
            return Err(ModelError::InferenceError(format!(
                "Input too long: {} tokens exceeds context size {}",
                tokens.len(),
                n_ctx
            )));
        }
        
        // Create batch for processing
        // SAFETY: Initializing batch structure for embedding inference
        let batch = unsafe {
            (self.model.functions().llama_batch_get_one)(
                tokens.as_ptr() as *mut i32,
                tokens.len() as i32,
                0, // position
                0, // sequence ID
            )
        };
        
        // Clear KV cache
        unsafe {
            (self.model.functions().llama_kv_cache_clear)(self.context_ptr);
        }
        
        // Decode tokens
        // SAFETY: Processing input tokens through the model for embeddings
        let decode_result = unsafe {
            (self.model.functions().llama_decode)(self.context_ptr, batch)
        };
        
        if decode_result != 0 {
            // SAFETY: Free batch before returning error
            unsafe {
                (self.model.functions().llama_batch_free)(batch);
            }
            return Err(ModelError::InferenceError(format!(
                "Failed to decode tokens for embedding: error code {}",
                decode_result
            )));
        }
        
        // Get embeddings
        // SAFETY: Getting embeddings pointer from successfully decoded context
        let embeddings_ptr = unsafe {
            (self.model.functions().llama_get_embeddings)(self.context_ptr)
        };
        
        // Free batch
        unsafe {
            (self.model.functions().llama_batch_free)(batch);
        }
        
        // Disable embedding mode
        unsafe {
            (self.model.functions().llama_set_embeddings)(self.context_ptr, false);
        }
        
        if embeddings_ptr.is_null() {
            return Err(ModelError::InferenceError(
                "Failed to get embeddings (null pointer returned)".to_string(),
            ));
        }
        
        // Copy embeddings to Vec
        let embd_dim = self.model.embedding_dim();
        let mut embeddings = Vec::with_capacity(embd_dim);
        
        // SAFETY: Reading embedding values from valid pointer
        // The pointer is valid for embd_dim elements as guaranteed by llama.cpp
        unsafe {
            for i in 0..embd_dim {
                embeddings.push(*embeddings_ptr.add(i));
            }
        }
        
        // Normalize the embeddings (L2 normalization - standard for embedding models)
        let norm: f32 = embeddings.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embeddings {
                *val /= norm;
            }
        }
        
        log::debug!("Generated embedding with dimension {}", embd_dim);
        Ok(embeddings)
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        if !self.context_ptr.is_null() {
            // SAFETY: llama_free safely deallocates the context.
            // This is safe because:
            // 1. context_ptr was allocated by llama_new_context_with_model and is valid
            // 2. We check is_null() to prevent double-free
            // 3. This is only called once during Drop, ensuring proper cleanup
            unsafe {
                (self.model.functions().llama_free)(self.context_ptr);
            }
            log::info!("Freed inference context");
        }
    }
}

// SAFETY: Context is Send because each context is independent and can be safely moved between threads.
// The underlying llama.cpp library supports per-thread contexts. However, Context is NOT Sync -
// it must not be accessed from multiple threads simultaneously.
unsafe impl Send for Context {}
// Context is NOT Sync - must not be accessed from multiple threads simultaneously

