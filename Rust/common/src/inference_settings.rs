/// Inference Settings - Full parity with extension
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct InferenceSettings {
    pub temperature: f32,
    pub max_length: u32,
    pub max_new_tokens: u32,
    pub min_length: u32,
    pub min_new_tokens: u32,
    pub top_k: u32,
    pub top_p: f32,
    pub typical_p: f32,
    pub epsilon_cutoff: f32,
    pub eta_cutoff: f32,
    pub repetition_penalty: f32,
    pub encoder_repetition_penalty: f32,
    pub do_sample: bool,
    pub num_beams: u32,
    pub num_beam_groups: u32,
    pub diversity_penalty: f32,
    pub early_stopping: bool,
    pub length_penalty: f32,
    pub penalty_alpha: f32,
    pub no_repeat_ngram_size: u32,
    pub encoder_no_repeat_ngram_size: u32,
    pub decoder_start_token_id: Option<u32>,
    pub forced_bos_token_id: Option<u32>,
    pub forced_eos_token_id: Option<u32>,
    pub bad_words_ids: Option<Vec<Vec<u32>>>,
    pub force_words_ids: Option<Vec<Vec<u32>>>,
    pub suppress_tokens: Option<Vec<u32>>,
    pub begin_suppress_tokens: Option<Vec<u32>>,
    pub num_return_sequences: u32,
    pub output_attentions: bool,
    pub output_hidden_states: bool,
    pub output_scores: bool,
    pub return_dict_in_generate: bool,
    pub use_cache: bool,
    pub remove_invalid_values: bool,
    pub renormalize_logits: bool,
    pub guidance_scale: f32,
    pub max_time: Option<f32>,
    pub system_prompt: String,
    pub json_mode: bool,
}

impl Default for InferenceSettings {
    fn default() -> Self {
        Self {
            temperature: 0.3,
            max_length: 8192,
            max_new_tokens: 1024,
            min_length: 0,
            min_new_tokens: 0,
            top_k: 50,
            top_p: 0.9,
            typical_p: 1.0,
            epsilon_cutoff: 0.0,
            eta_cutoff: 0.0,
            repetition_penalty: 1.2,
            encoder_repetition_penalty: 1.0,
            do_sample: true,
            num_beams: 1,
            num_beam_groups: 1,
            diversity_penalty: 0.0,
            early_stopping: true,
            length_penalty: 0.8,
            penalty_alpha: 0.0,
            no_repeat_ngram_size: 3,
            encoder_no_repeat_ngram_size: 0,
            decoder_start_token_id: None,
            forced_bos_token_id: None,
            forced_eos_token_id: None,
            bad_words_ids: None,
            force_words_ids: None,
            suppress_tokens: None,
            begin_suppress_tokens: None,
            num_return_sequences: 1,
            output_attentions: false,
            output_hidden_states: false,
            output_scores: false,
            return_dict_in_generate: false,
            use_cache: true,
            remove_invalid_values: false,
            renormalize_logits: false,
            guidance_scale: 1.0,
            max_time: None,
            system_prompt: "You are a helpful AI assistant.".to_string(),
            json_mode: false,
        }
    }
}

impl InferenceSettings {
    /// Get settings for a model by pattern matching on repo_id
    pub fn for_model(repo_id: &str) -> Self {
        let lower = repo_id.to_lowercase();
        
        // SmolLM models
        if lower.contains("smollm") {
            return Self {
                temperature: 0.7,
                top_k: 40,
                top_p: 0.95,
                repetition_penalty: 1.1,
                max_new_tokens: 512,
                ..Default::default()
            };
        }
        
        // Phi models
        if lower.contains("phi") {
            return Self {
                temperature: 0.6,
                top_k: 50,
                top_p: 0.9,
                repetition_penalty: 1.15,
                max_new_tokens: 1024,
                ..Default::default()
            };
        }
        
        // Qwen models
        if lower.contains("qwen") {
            return Self {
                temperature: 0.7,
                top_k: 50,
                top_p: 0.8,
                repetition_penalty: 1.1,
                max_new_tokens: 2048,
                ..Default::default()
            };
        }
        
        // Llama models
        if lower.contains("llama") {
            return Self {
                temperature: 0.6,
                top_k: 40,
                top_p: 0.9,
                repetition_penalty: 1.15,
                max_new_tokens: 2048,
                ..Default::default()
            };
        }
        
        // Default for unknown models
        Self::default()
    }
}

