//! ONNX Session management with integrated tokenization

use crate::error::{OnnxError, Result};
use crate::providers_bridge;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tabagent_tokenization::Tokenizer;
use tabagent_execution_providers::ExecutionProvider;
use common::InferenceSettings;
use ort::session::{Session, builder::GraphOptimizationLevel};
use ort::value::Value;
use ort::tensor::Shape;
use ort::inputs;

/// ONNX inference session with smart defaults + full `ort` API access
/// 
/// # Architecture
/// This is a **thin smart wrapper** around `ort::Session` that:
/// - ✅ Adds: Automatic execution provider selection (via @hardware)
/// - ✅ Adds: Sensible defaults (Level3 optimization, 4/2 threads)
/// - ✅ Adds: Integrated tokenization (optional convenience)
/// - ✅ Exposes: **Full `ort::Session` API** via `.session()` method
/// 
/// # Philosophy
/// We don't limit you! If `ort` can do it, you can do it.
/// Use our convenience methods OR access raw `ort::Session` directly.
/// 
/// # Examples
/// ```ignore
/// // High-level (our convenience API)
/// let emb = session.generate_embedding("text")?;
/// 
/// // Low-level (full ort power)
/// let sess = session.session().lock().unwrap();
/// let outputs = sess.run(ort::inputs![my_tensor])?;
/// ```
#[derive(Clone)]
pub struct OnnxSession {
    model_path: PathBuf,
    tokenizer: Option<Arc<Tokenizer>>,
    session: Arc<Mutex<Session>>,
}

impl OnnxSession {
    /// Load ONNX model with auto-selected execution providers
    pub fn load<P: AsRef<Path>>(model_path: P) -> Result<Self> {
        let ort_providers = providers_bridge::auto_select_providers()?;
        Self::load_with_ort_providers(model_path, ort_providers)
    }
    
    /// Load ONNX model with specific execution providers
    pub fn load_with_providers<P: AsRef<Path>>(
        model_path: P,
        providers: Vec<std::sync::Arc<dyn ExecutionProvider>>,
    ) -> Result<Self> {
        log::info!("Converting {} tabagent providers to ort format", providers.len());
        let ort_providers = providers_bridge::bridge_to_ort(&providers)?;
        Self::load_with_ort_providers(model_path, ort_providers)
    }
    
    /// Internal: Load ONNX model with ort execution providers
    fn load_with_ort_providers<P: AsRef<Path>>(
        model_path: P,
        ort_providers: Vec<ort::execution_providers::ExecutionProviderDispatch>,
    ) -> Result<Self> {
        let model_path = model_path.as_ref();
        log::info!("Loading ONNX model from: {:?}", model_path);
        
        // Verify model file exists
        if !model_path.exists() {
            return Err(OnnxError::ModelLoadFailed(format!(
                "Model file not found: {:?}",
                model_path
            )));
        }
        
        log::info!("Configuring session with {} execution providers", ort_providers.len());
        
        // Create ONNX Runtime session with full configuration
        let session = Session::builder()
            .map_err(|e| OnnxError::SessionCreationFailed(e.to_string()))?
            
            // Hardware acceleration
            .with_execution_providers(&ort_providers)
            .map_err(|e| OnnxError::SessionCreationFailed(
                format!("Failed to set execution providers: {}", e)
            ))?
            
            // Graph optimization (Level3 = all optimizations)
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| OnnxError::SessionCreationFailed(
                format!("Failed to set optimization level: {}", e)
            ))?
            
            // Threading configuration
            .with_intra_threads(4)  // Threads within ops
            .map_err(|e| OnnxError::SessionCreationFailed(
                format!("Failed to set intra threads: {}", e)
            ))?
            
            .with_inter_threads(2)  // Threads between ops
            .map_err(|e| OnnxError::SessionCreationFailed(
                format!("Failed to set inter threads: {}", e)
            ))?
            
            // Enable parallel execution for multi-branch models
            .with_parallel_execution(true)
            .map_err(|e| OnnxError::SessionCreationFailed(
                format!("Failed to enable parallel execution: {}", e)
            ))?
            
            // Memory optimization
            .with_memory_pattern(true)
            .map_err(|e| OnnxError::SessionCreationFailed(
                format!("Failed to enable memory pattern: {}", e)
            ))?
            
            // Finally, load the model
            .commit_from_file(model_path)
            .map_err(|e| OnnxError::ModelLoadFailed(e.to_string()))?;
        
        log::info!("ONNX model loaded successfully");
        log::debug!("Session configuration:");
        log::debug!("  - Optimization: Level 3 (all optimizations)");
        log::debug!("  - Intra-op threads: 4");
        log::debug!("  - Inter-op threads: 2");
        log::debug!("  - Parallel execution: enabled");
        log::debug!("  - Memory pattern: enabled");
        
        Ok(Self {
            model_path: model_path.to_path_buf(),
            tokenizer: None,
            session: Arc::new(Mutex::new(session)),
        })
    }
    
    /// Load tokenizer from file
    pub fn load_tokenizer<P: AsRef<Path>>(&mut self, tokenizer_path: P) -> Result<()> {
        log::info!("Loading tokenizer from: {:?}", tokenizer_path.as_ref());
        let tokenizer = Tokenizer::from_file(tokenizer_path)?;
        self.tokenizer = Some(Arc::new(tokenizer));
        Ok(())
    }
    
    /// Load tokenizer from HuggingFace Hub
    pub fn load_tokenizer_from_hub(
        &mut self,
        identifier: &str,
        auth_token: Option<&str>,
    ) -> Result<()> {
        log::info!("Loading tokenizer from HuggingFace Hub: {}", identifier);
        let tokenizer = Tokenizer::from_pretrained(identifier, auth_token)?;
        self.tokenizer = Some(Arc::new(tokenizer));
        Ok(())
    }
    
    /// Get tokenizer reference
    pub fn tokenizer(&self) -> Option<Arc<Tokenizer>> {
        self.tokenizer.clone()
    }
    
    /// Get model path
    pub fn model_path(&self) -> &Path {
        &self.model_path
    }
    
    /// Check if tokenizer is loaded
    pub fn has_tokenizer(&self) -> bool {
        self.tokenizer.is_some()
    }
    
    /// Get direct access to the underlying `ort::Session`
    /// 
    /// # Full Power
    /// This gives you **complete access** to everything `ort` can do:
    /// - GPT-2 autoregressive generation
    /// - YOLOv8 object detection  
    /// - Custom tensor operations
    /// - Async inference
    /// - Training (if enabled)
    /// - ANYTHING `ort` supports!
    /// 
    /// # Example
    /// ```ignore
    /// let session = onnx_session.session().lock().unwrap();
    /// let outputs = session.run(ort::inputs![
    ///     "input_ids" => my_ids_tensor,
    ///     "attention_mask" => my_mask_tensor
    /// ])?;
    /// ```
    pub fn session(&self) -> Arc<Mutex<Session>> {
        Arc::clone(&self.session)
    }
    
    
    /// Generate text with full autoregressive decoding
    /// 
    /// # High-Level Convenience API
    /// This is the easy way to do text generation!
    /// 
    /// # Arguments
    /// * `prompt` - Input text prompt
    /// * `config` - Generation configuration (temperature, top_k, etc.)
    /// 
    /// # Returns
    /// Generated text string
    /// 
    /// # Example
    /// ```ignore
    /// use tabagent_onnx_loader::{OnnxSession, GenerationConfig};
    /// 
    /// let mut session = OnnxSession::load("gpt2.onnx")?;
    /// session.load_tokenizer("tokenizer.json")?;
    /// 
    /// let config = GenerationConfig {
    ///     max_new_tokens: 100,
    ///     temperature: 0.7,
    ///     top_k: 50,
    ///     ..Default::default()
    /// };
    /// 
    /// let text = session.generate_text("Once upon a time", &config)?;
    /// println!("{}", text);
    /// ```
    pub fn generate_text(&self, prompt: &str, config: &crate::text_generation::GenerationConfig) -> Result<String> {
        let generator = crate::text_generation::TextGenerator::new(self.clone());
        generator.generate(prompt, config)
    }
    
    /// Generate text with streaming (token-by-token callback)
    /// 
    /// # Arguments
    /// * `prompt` - Input text
    /// * `config` - Generation config
    /// * `callback` - Called for each generated token
    /// 
    /// # Example
    /// ```ignore
    /// session.generate_text_stream("Hello", &config, |token| {
    ///     print!("{}", token);
    ///     Ok(())
    /// })?;
    /// ```
    pub fn generate_text_stream<F>(
        &self,
        prompt: &str,
        config: &crate::text_generation::GenerationConfig,
        callback: F,
    ) -> Result<String>
    where
        F: FnMut(&str) -> Result<()>,
    {
        let generator = crate::text_generation::TextGenerator::new(self.clone());
        generator.generate_stream(prompt, config, callback)
    }
    
    /// Run text generation inference
    /// 
    /// # Arguments
    /// * `input` - Input text prompt
    /// 
    /// # Returns
    /// Generated text output
    /// 
    /// # Note
    /// Text generation models require more complex decoding logic.
    /// For now, this validates the pipeline is working.
    pub fn generate(&self, input: &str) -> Result<String> {
        log::info!("ONNX generate called with input length: {}", input.len());
        
        let tokenizer = self.tokenizer.as_ref()
            .ok_or_else(|| OnnxError::TokenizerLoadFailed("Tokenizer required for text generation".to_string()))?;
        
        // Tokenize input to validate pipeline
        let encoding = tokenizer.encode(input, true)
            .map_err(|e| OnnxError::TokenizerLoadFailed(e.to_string()))?;
        
        let token_ids = encoding.get_ids();
        log::debug!("Tokenized to {} tokens", token_ids.len());
        
        // Text generation requires autoregressive decoding which is complex
        // For embedding models (our current focus), this isn't needed
        Ok(format!("[ONNX] Tokenized {} tokens. Full generation requires autoregressive decoding.", 
            token_ids.len()
        ))
    }
    
    /// Generate embedding vector for text
    /// 
    /// # Arguments
    /// * `text` - Input text to embed
    /// 
    /// # Returns
    /// Embedding vector (typically 384, 768, or 1536 dimensions)
    pub fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        log::info!("ONNX generate_embedding called");
        
        let tokenizer = self.tokenizer.as_ref()
            .ok_or_else(|| OnnxError::TokenizerLoadFailed("Tokenizer required for embeddings".to_string()))?;
        
        // Step 1: Tokenize input
        let encoding = tokenizer.encode(text, true)
            .map_err(|e| OnnxError::TokenizerLoadFailed(e.to_string()))?;
        
        let token_ids = encoding.get_ids();
        log::debug!("Tokenized to {} tokens for embedding", token_ids.len());
        
        // Step 2: Dynamically detect expected input type from model metadata
        let mut session = self.session.lock()
            .map_err(|e| OnnxError::ModelLoadFailed(format!("Session lock failed: {}", e)))?;
        
        let inputs_metadata = &session.inputs;
        let input_type = &inputs_metadata.first()
            .ok_or_else(|| OnnxError::ModelLoadFailed("Model has no inputs".to_string()))?
            .input_type;
        
        log::debug!("Model expects input type: {:?}", input_type);
        
        // Step 3: Create input tensors with the correct type
        let seq_len = token_ids.len();
        let shape = (1, seq_len);
        
        // Check if model expects int32 or int64
        let type_string = format!("{:?}", input_type);
        let outputs = if type_string.contains("Int32") {
            // Model expects i32
            log::debug!("Using i32 for token IDs");
            let input_ids: Vec<i32> = token_ids.iter().map(|&id| id as i32).collect();
            let attention_mask: Vec<i32> = vec![1i32; seq_len];
            let token_type_ids: Vec<i32> = vec![0i32; seq_len]; // All zeros for single sentence
            
            let input_ids_array = ndarray::Array2::from_shape_vec(shape, input_ids)
                .map_err(|e| OnnxError::ModelLoadFailed(format!("Failed to create input tensor: {}", e)))?;
            let attention_mask_array = ndarray::Array2::from_shape_vec(shape, attention_mask)
                .map_err(|e| OnnxError::ModelLoadFailed(format!("Failed to create attention mask: {}", e)))?;
            let token_type_ids_array = ndarray::Array2::from_shape_vec(shape, token_type_ids)
                .map_err(|e| OnnxError::ModelLoadFailed(format!("Failed to create token_type_ids: {}", e)))?;
            
            let input_ids_value = Value::from_array(input_ids_array)
                .map_err(|e| OnnxError::ModelLoadFailed(format!("Failed to create input_ids Value: {}", e)))?;
            let attention_mask_value = Value::from_array(attention_mask_array)
                .map_err(|e| OnnxError::ModelLoadFailed(format!("Failed to create attention_mask Value: {}", e)))?;
            let token_type_ids_value = Value::from_array(token_type_ids_array)
                .map_err(|e| OnnxError::ModelLoadFailed(format!("Failed to create token_type_ids Value: {}", e)))?;
            
            session.run(inputs![
                "input_ids" => input_ids_value,
                "attention_mask" => attention_mask_value,
                "token_type_ids" => token_type_ids_value
            ])
            .map_err(|e| OnnxError::ModelLoadFailed(format!("Inference failed: {}", e)))?
        } else {
            // Default to i64 (most HuggingFace models)
            log::debug!("Using i64 for token IDs");
            let input_ids: Vec<i64> = token_ids.iter().map(|&id| id as i64).collect();
            let attention_mask: Vec<i64> = vec![1i64; seq_len];
            let token_type_ids: Vec<i64> = vec![0i64; seq_len]; // All zeros for single sentence
            
            let input_ids_array = ndarray::Array2::from_shape_vec(shape, input_ids)
                .map_err(|e| OnnxError::ModelLoadFailed(format!("Failed to create input tensor: {}", e)))?;
            let attention_mask_array = ndarray::Array2::from_shape_vec(shape, attention_mask)
                .map_err(|e| OnnxError::ModelLoadFailed(format!("Failed to create attention mask: {}", e)))?;
            let token_type_ids_array = ndarray::Array2::from_shape_vec(shape, token_type_ids)
                .map_err(|e| OnnxError::ModelLoadFailed(format!("Failed to create token_type_ids: {}", e)))?;
            
            let input_ids_value = Value::from_array(input_ids_array)
                .map_err(|e| OnnxError::ModelLoadFailed(format!("Failed to create input_ids Value: {}", e)))?;
            let attention_mask_value = Value::from_array(attention_mask_array)
                .map_err(|e| OnnxError::ModelLoadFailed(format!("Failed to create attention_mask Value: {}", e)))?;
            let token_type_ids_value = Value::from_array(token_type_ids_array)
                .map_err(|e| OnnxError::ModelLoadFailed(format!("Failed to create token_type_ids Value: {}", e)))?;
            
            session.run(inputs![
                "input_ids" => input_ids_value,
                "attention_mask" => attention_mask_value,
                "token_type_ids" => token_type_ids_value
            ])
            .map_err(|e| OnnxError::ModelLoadFailed(format!("Inference failed: {}", e)))?
        };
        
        // Step 4: Extract embedding from output
        // For sentence transformers, we need to mean pool the last_hidden_state
        let last_hidden_state = outputs[0].try_extract_tensor::<f32>()
            .map_err(|e| OnnxError::ModelLoadFailed(format!("Failed to extract output tensor: {}", e)))?;
        
        let embedding = mean_pool_embedding(last_hidden_state, token_ids.len());
        
        log::debug!("Generated embedding with {} dimensions", embedding.len());
        Ok(embedding)
    }
    
    /// Generate embeddings for multiple texts (batch processing)
    /// 
    /// # Arguments
    /// * `texts` - Slice of text strings to embed
    /// 
    /// # Returns
    /// Vector of embedding vectors, one per input text
    /// 
    /// # Note
    /// Currently processes texts individually. Batching optimization can be added later.
    pub fn generate_embeddings(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        log::info!("ONNX generate_embeddings batch: {} texts", texts.len());
        
        texts.iter()
            .map(|text| self.generate_embedding(text))
            .collect()
    }
    
}

/// Mean pool the token embeddings to get sentence embedding
fn mean_pool_embedding(hidden_state: (&Shape, &[f32]), seq_len: usize) -> Vec<f32> {
    let (shape, data) = hidden_state;
    if shape.len() < 3 {
        log::warn!("Unexpected hidden state shape (len: {})", shape.len());
        return vec![];
    }
    
    let _batch_size = shape[0] as usize;
    let seq_length = shape[1] as usize;
    let embedding_dim = shape[2] as usize;
    let mut pooled = vec![0.0f32; embedding_dim];
    
    // Mean pool across sequence length (dim 1)
    for i in 0..embedding_dim {
        let mut sum = 0.0;
        for j in 0..seq_len.min(seq_length) {
            // Access data in row-major order: [batch, seq, embed]
            let idx = j * embedding_dim + i;
            sum += data[idx];
        }
        pooled[i] = sum / (seq_len as f32);
    }
    
    pooled
}
