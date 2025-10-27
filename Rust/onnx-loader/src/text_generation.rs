//! Full text generation with autoregressive decoding
//!
//! Implements GPT-2 style text generation for ONNX models

use crate::error::{OnnxError, Result};
use crate::session::OnnxSession;
use ort::{inputs, value::TensorRef};
use std::sync::Arc;
use tabagent_tokenization::Tokenizer;

/// Text generation configuration
#[derive(Debug, Clone)]
pub struct GenerationConfig {
    pub max_new_tokens: usize,
    pub temperature: f32,
    pub top_k: usize,
    pub top_p: f32,
    pub repetition_penalty: f32,
    pub do_sample: bool,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            max_new_tokens: 100,
            temperature: 1.0,
            top_k: 50,
            top_p: 0.9,
            repetition_penalty: 1.0,
            do_sample: true,
        }
    }
}

/// Text generator with autoregressive decoding
pub struct TextGenerator {
    session: OnnxSession,
}

impl TextGenerator {
    pub fn new(session: OnnxSession) -> Self {
        Self { session }
    }
    
    /// Generate text with autoregressive decoding
    /// 
    /// # Arguments
    /// * `prompt` - Input prompt text
    /// * `config` - Generation configuration
    /// 
    /// # Returns
    /// Generated text string
    pub fn generate(&self, prompt: &str, config: &GenerationConfig) -> Result<String> {
        let tokenizer = self.session.tokenizer()
            .ok_or_else(|| OnnxError::TokenizerLoadFailed("Tokenizer required for text generation".to_string()))?;
        
        // Tokenize prompt
        let encoding = tokenizer.encode(prompt, true)
            .map_err(|e| OnnxError::TokenizerLoadFailed(e.to_string()))?;
        
        let mut token_ids: Vec<i64> = encoding.get_ids().iter().map(|i| *i as i64).collect();
        let original_len = token_ids.len();
        
        log::info!("Starting text generation: {} input tokens, max {} new tokens", 
            original_len, config.max_new_tokens);
        
        // Get ort session for inference
        let ort_session = self.session.session();
        
        // Autoregressive generation loop
        for step in 0..config.max_new_tokens {
            // Create input tensor
            let input = TensorRef::from_array_view((
                vec![1, 1, token_ids.len() as i64],
                token_ids.as_slice()
            )).map_err(|e| OnnxError::InferenceFailed(format!("Failed to create input tensor: {}", e)))?;
            
            // Run inference
            let mut session_lock = ort_session.lock()
                .map_err(|e| OnnxError::InferenceFailed(format!("Failed to lock session: {}", e)))?;
            
            let outputs = session_lock.run(inputs![input])
                .map_err(|e| OnnxError::InferenceFailed(format!("Inference failed: {}", e)))?;
            
            // Extract logits (output shape: [B, _, S, V])
            let (dim, logits) = outputs["output1"].try_extract_tensor::<f32>()
                .map_err(|e| OnnxError::InferenceFailed(format!("Failed to extract logits: {}", e)))?;
            
            let (seq_len, vocab_size) = (dim[2] as usize, dim[3] as usize);
            
            // Get logits for the last token
            let last_token_logits = &logits[(seq_len - 1) * vocab_size..seq_len * vocab_size];
            
            // Sample next token
            let next_token = if config.do_sample {
                sample_token(
                    last_token_logits,
                    config.temperature,
                    config.top_k,
                    config.top_p
                )?
            } else {
                greedy_sample(last_token_logits)?
            };
            
            // Check for EOS token (assume EOS is 50256 for GPT-2, or tokenizer.eos_token_id)
            // TODO: Get actual EOS token from tokenizer
            if next_token == 50256 {
                log::info!("EOS token generated at step {}", step);
                break;
            }
            
            // Add to sequence
            token_ids.push(next_token);
            
            if (step + 1) % 10 == 0 {
                log::debug!("Generated {} tokens", step + 1);
            }
        }
        
        // Decode generated tokens (only new tokens, not prompt)
        let new_token_ids: Vec<u32> = token_ids[original_len..]
            .iter()
            .map(|&id| id as u32)
            .collect();
        
        let generated_text = tokenizer.decode(&new_token_ids, true)
            .map_err(|e| OnnxError::TokenizerLoadFailed(format!("Failed to decode: {}", e)))?;
        
        log::info!("Generation complete: {} new tokens generated", new_token_ids.len());
        
        Ok(generated_text)
    }
    
    /// Generate text with streaming callback
    pub fn generate_stream<F>(
        &self,
        prompt: &str,
        config: &GenerationConfig,
        mut callback: F,
    ) -> Result<String>
    where
        F: FnMut(&str) -> Result<()>,
    {
        let tokenizer = self.session.tokenizer()
            .ok_or_else(|| OnnxError::TokenizerLoadFailed("Tokenizer required".to_string()))?;
        
        let encoding = tokenizer.encode(prompt, true)
            .map_err(|e| OnnxError::TokenizerLoadFailed(e.to_string()))?;
        
        let mut token_ids: Vec<i64> = encoding.get_ids().iter().map(|i| *i as i64).collect();
        let original_len = token_ids.len();
        
        let ort_session = self.session.session();
        let mut full_generated = String::new();
        
        for step in 0..config.max_new_tokens {
            let input = TensorRef::from_array_view((
                vec![1, 1, token_ids.len() as i64],
                token_ids.as_slice()
            )).map_err(|e| OnnxError::InferenceFailed(e.to_string()))?;
            
            let mut session_lock = ort_session.lock()
                .map_err(|e| OnnxError::InferenceFailed(e.to_string()))?;
            let outputs = session_lock.run(inputs![input])
                .map_err(|e| OnnxError::InferenceFailed(e.to_string()))?;
            
            let (dim, logits) = outputs["output1"].try_extract_tensor::<f32>()
                .map_err(|e| OnnxError::InferenceFailed(e.to_string()))?;
            
            let (seq_len, vocab_size) = (dim[2] as usize, dim[3] as usize);
            let last_token_logits = &logits[(seq_len - 1) * vocab_size..];
            
            let next_token = if config.do_sample {
                sample_token(last_token_logits, config.temperature, config.top_k, config.top_p)?
            } else {
                greedy_sample(last_token_logits)?
            };
            
            if next_token == 50256 {
                break;
            }
            
            token_ids.push(next_token);
            
            // Decode and stream the new token
            let token_text = tokenizer.decode(&[next_token as u32], true)
                .map_err(|e| OnnxError::TokenizerLoadFailed(e.to_string()))?;
            
            full_generated.push_str(&token_text);
            callback(&token_text)?;
        }
        
        Ok(full_generated)
    }
}

/// Greedy sampling (argmax)
fn greedy_sample(logits: &[f32]) -> Result<i64> {
    let max_idx = logits
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(idx, _)| idx)
        .ok_or_else(|| OnnxError::InferenceFailed("Empty logits".to_string()))?;
    
    Ok(max_idx as i64)
}

/// Sample token with temperature, top-k, and top-p
fn sample_token(
    logits: &[f32],
    temperature: f32,
    top_k: usize,
    top_p: f32,
) -> Result<i64> {
    use rand::distributions::WeightedIndex;
    use rand::prelude::*;
    
    // Apply temperature
    let probs: Vec<f32> = logits.iter().map(|&x| (x / temperature).exp()).collect();
    
    // Sort by probability (descending)
    let mut indexed_probs: Vec<(usize, f32)> = probs.iter().copied().enumerate().collect();
    indexed_probs.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    // Apply top-k
    if top_k > 0 && top_k < indexed_probs.len() {
        indexed_probs.truncate(top_k);
    }
    
    // Apply top-p (nucleus sampling)
    if top_p < 1.0 {
        let total: f32 = indexed_probs.iter().map(|(_, p)| p).sum();
        let mut cumsum = 0.0;
        let mut cutoff_idx = indexed_probs.len();
        
        for (i, (_, prob)) in indexed_probs.iter().enumerate() {
            cumsum += prob / total;
            if cumsum >= top_p {
                cutoff_idx = i + 1;
                break;
            }
        }
        
        indexed_probs.truncate(cutoff_idx);
    }
    
    // Normalize probabilities
    let total: f32 = indexed_probs.iter().map(|(_, p)| p).sum();
    for (_, p) in indexed_probs.iter_mut() {
        *p /= total;
    }
    
    // Sample from distribution
    let indices: Vec<usize> = indexed_probs.iter().map(|(idx, _)| *idx).collect();
    let weights: Vec<f32> = indexed_probs.iter().map(|(_, p)| *p).collect();
    
    let dist = WeightedIndex::new(&weights)
        .map_err(|e| OnnxError::InferenceFailed(format!("Failed to create distribution: {}", e)))?;
    
    let mut rng = thread_rng();
    let selected_idx = indices[dist.sample(&mut rng)];
    
    Ok(selected_idx as i64)
}

