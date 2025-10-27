//! Shared tokenization layer for all inference backends
//!
//! Wraps HuggingFace's fast tokenizers for consistent, high-performance tokenization
//! across ONNX, GGUF/BitNet, and future inference engines.

pub mod error;

use std::path::Path;
pub use tokenizers::{Encoding, Tokenizer as HfTokenizer};

pub use error::{Result, TokenizationError};

/// Tokenizer wrapper providing consistent interface
pub struct Tokenizer {
    inner: HfTokenizer,
}

impl Tokenizer {
    /// Load tokenizer from file (tokenizer.json)
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let inner = HfTokenizer::from_file(path)
            .map_err(|e| TokenizationError::LoadFailed(e.to_string()))?;
        
        Ok(Self { inner })
    }
    
    /// Load tokenizer from pretrained model (HuggingFace Hub)
    /// 
    /// Note: For HuggingFace models, download tokenizer.json first and use from_file()
    /// This version of tokenizers doesn't support direct HF Hub downloads
    pub fn from_pretrained(_identifier: &str, _auth_token: Option<&str>) -> Result<Self> {
        Err(TokenizationError::LoadFailed(
            "from_pretrained not supported in tokenizers 0.19. Download tokenizer.json and use from_file() instead".to_string()
        ))
    }
    
    /// Encode text to token IDs
    pub fn encode(&self, text: &str, add_special_tokens: bool) -> Result<Encoding> {
        self.inner
            .encode(text, add_special_tokens)
            .map_err(|e| TokenizationError::EncodeFailed(e.to_string()))
    }
    
    /// Encode batch of texts
    pub fn encode_batch(
        &self,
        texts: Vec<&str>,
        add_special_tokens: bool,
    ) -> Result<Vec<Encoding>> {
        self.inner
            .encode_batch(texts, add_special_tokens)
            .map_err(|e| TokenizationError::EncodeFailed(e.to_string()))
    }
    
    /// Decode token IDs to text
    pub fn decode(&self, ids: &[u32], skip_special_tokens: bool) -> Result<String> {
        self.inner
            .decode(ids, skip_special_tokens)
            .map_err(|e| TokenizationError::DecodeFailed(e.to_string()))
    }
    
    /// Decode batch of token ID sequences
    pub fn decode_batch(
        &self,
        sequences: &[&[u32]],
        skip_special_tokens: bool,
    ) -> Result<Vec<String>> {
        self.inner
            .decode_batch(sequences, skip_special_tokens)
            .map_err(|e| TokenizationError::DecodeFailed(e.to_string()))
    }
    
    /// Get vocabulary size
    pub fn vocab_size(&self) -> usize {
        self.inner.get_vocab_size(false)
    }
    
    /// Get special token IDs
    pub fn bos_token_id(&self) -> Option<u32> {
        self.inner.token_to_id("<s>")
            .or_else(|| self.inner.token_to_id("<bos>"))
            .or_else(|| self.inner.token_to_id("[CLS]"))
    }
    
    pub fn eos_token_id(&self) -> Option<u32> {
        self.inner.token_to_id("</s>")
            .or_else(|| self.inner.token_to_id("<eos>"))
            .or_else(|| self.inner.token_to_id("[SEP]"))
    }
    
    pub fn pad_token_id(&self) -> Option<u32> {
        self.inner.token_to_id("<pad>")
            .or_else(|| self.inner.token_to_id("[PAD]"))
    }
    
    pub fn unk_token_id(&self) -> Option<u32> {
        self.inner.token_to_id("<unk>")
            .or_else(|| self.inner.token_to_id("[UNK]"))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_tokenizer_basic() {
        // This test requires a real tokenizer file
        // In practice, use a fixture or mock
    }
}

