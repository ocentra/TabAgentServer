//! Low-level FFI bindings to llama.cpp C API
//!
//! This module provides unsafe bindings to the llama.cpp library.
//! All function signatures match the C API exactly.

use std::os::raw::{c_char, c_float, c_int, c_void};

/// Opaque pointer to llama_model
#[repr(C)]
pub struct LlamaModel {
    _private: [u8; 0],
}

/// Opaque pointer to llama_context
#[repr(C)]
pub struct LlamaContext {
    _private: [u8; 0],
}

/// Llama model parameters (matches llama_model_params in C)
#[repr(C)]
#[derive(Debug, Clone)]
pub struct LlamaModelParams {
    pub n_gpu_layers: c_int,
    pub main_gpu: c_int,
    pub tensor_split: *const c_float,
    pub vocab_only: bool,
    pub use_mmap: bool,
    pub use_mlock: bool,
}

impl Default for LlamaModelParams {
    fn default() -> Self {
        Self {
            n_gpu_layers: 0,
            main_gpu: 0,
            tensor_split: std::ptr::null(),
            vocab_only: false,
            use_mmap: true,
            use_mlock: false,
        }
    }
}

/// Llama context parameters (matches llama_context_params in C)
#[repr(C)]
#[derive(Debug, Clone)]
pub struct LlamaContextParams {
    pub n_ctx: u32,
    pub n_batch: u32,
    pub n_threads: c_int,
    pub n_threads_batch: c_int,
    pub rope_scaling_type: c_int,
    pub rope_freq_base: c_float,
    pub rope_freq_scale: c_float,
    pub yarn_ext_factor: c_float,
    pub yarn_attn_factor: c_float,
    pub yarn_beta_fast: c_float,
    pub yarn_beta_slow: c_float,
    pub yarn_orig_ctx: u32,
    pub defrag_thold: c_float,
    pub type_k: c_int,
    pub type_v: c_int,
    pub logits_all: bool,
    pub embedding: bool,
    pub offload_kqv: bool,
}

impl Default for LlamaContextParams {
    fn default() -> Self {
        Self {
            n_ctx: 512,
            n_batch: 512,
            n_threads: 4,
            n_threads_batch: 4,
            rope_scaling_type: -1,
            rope_freq_base: 0.0,
            rope_freq_scale: 0.0,
            yarn_ext_factor: -1.0,
            yarn_attn_factor: 1.0,
            yarn_beta_fast: 32.0,
            yarn_beta_slow: 1.0,
            yarn_orig_ctx: 0,
            defrag_thold: -1.0,
            type_k: 0,
            type_v: 0,
            logits_all: false,
            embedding: false,
            offload_kqv: true,
        }
    }
}

/// Token type
pub type LlamaToken = i32;

/// FFI function signatures for llama.cpp
/// These are loaded dynamically from llama.dll
pub struct LlamaFunctions {
    // Backend initialization
    pub llama_backend_init: unsafe extern "C" fn(),
    pub llama_backend_free: unsafe extern "C" fn(),

    // Model loading
    pub llama_load_model_from_file: unsafe extern "C" fn(
        path: *const c_char,
        params: LlamaModelParams,
    ) -> *mut LlamaModel,
    pub llama_free_model: unsafe extern "C" fn(model: *mut LlamaModel),

    // Context creation
    pub llama_new_context_with_model: unsafe extern "C" fn(
        model: *mut LlamaModel,
        params: LlamaContextParams,
    ) -> *mut LlamaContext,
    pub llama_free: unsafe extern "C" fn(ctx: *mut LlamaContext),

    // Tokenization
    pub llama_tokenize: unsafe extern "C" fn(
        model: *const LlamaModel,
        text: *const c_char,
        text_len: c_int,
        tokens: *mut LlamaToken,
        n_max_tokens: c_int,
        add_bos: bool,
        special: bool,
    ) -> c_int,

    pub llama_token_to_piece: unsafe extern "C" fn(
        model: *const LlamaModel,
        token: LlamaToken,
        buf: *mut c_char,
        length: c_int,
    ) -> c_int,

    // Inference
    pub llama_decode: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        batch: LlamaBatch,
    ) -> c_int,

    pub llama_get_logits: unsafe extern "C" fn(ctx: *mut LlamaContext) -> *const c_float,
    
    pub llama_get_logits_ith: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        i: c_int,
    ) -> *mut c_float,

    // Batch API
    pub llama_batch_init: unsafe extern "C" fn(
        n_tokens: c_int,
        embd: c_int,
        n_seq_max: c_int,
    ) -> LlamaBatch,
    
    pub llama_batch_free: unsafe extern "C" fn(batch: LlamaBatch),
    
    pub llama_batch_get_one: unsafe extern "C" fn(
        tokens: *mut LlamaToken,
        n_tokens: c_int,
        pos_0: c_int,
        seq_id: c_int,
    ) -> LlamaBatch,

    // Sampling
    pub llama_sample_token_greedy: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        candidates: *mut LlamaTokenDataArray,
    ) -> LlamaToken,

    // Model info
    pub llama_n_vocab: unsafe extern "C" fn(model: *const LlamaModel) -> c_int,
    pub llama_n_ctx_train: unsafe extern "C" fn(model: *const LlamaModel) -> c_int,
    pub llama_n_embd: unsafe extern "C" fn(model: *const LlamaModel) -> c_int,

    // Context info
    pub llama_n_ctx: unsafe extern "C" fn(ctx: *const LlamaContext) -> u32,

    // Special tokens
    pub llama_token_bos: unsafe extern "C" fn(model: *const LlamaModel) -> LlamaToken,
    pub llama_token_eos: unsafe extern "C" fn(model: *const LlamaModel) -> LlamaToken,
    pub llama_token_nl: unsafe extern "C" fn(model: *const LlamaModel) -> LlamaToken,
    
    pub llama_token_is_eog: unsafe extern "C" fn(
        model: *const LlamaModel,
        token: LlamaToken,
    ) -> bool,
    
    // Embeddings API (for llama-embedding.exe functionality)
    pub llama_get_embeddings: unsafe extern "C" fn(ctx: *mut LlamaContext) -> *mut c_float,
    
    pub llama_get_embeddings_ith: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        i: c_int,
    ) -> *mut c_float,
    
    pub llama_get_embeddings_seq: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        seq_id: c_int,
    ) -> *mut c_float,
    
    pub llama_set_embeddings: unsafe extern "C" fn(ctx: *mut LlamaContext, embeddings: bool),
    
    // KV cache management
    pub llama_kv_cache_clear: unsafe extern "C" fn(ctx: *mut LlamaContext),
    
    pub llama_kv_cache_seq_rm: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        seq_id: c_int,
        p0: c_int,
        p1: c_int,
    ) -> bool,
    
    pub llama_kv_cache_seq_cp: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        seq_id_src: c_int,
        seq_id_dst: c_int,
        p0: c_int,
        p1: c_int,
    ),
    
    pub llama_kv_cache_seq_keep: unsafe extern "C" fn(ctx: *mut LlamaContext, seq_id: c_int),
    
    pub llama_kv_cache_seq_add: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        seq_id: c_int,
        p0: c_int,
        p1: c_int,
        delta: c_int,
    ),
    
    // Encoder/decoder support
    pub llama_encode: unsafe extern "C" fn(ctx: *mut LlamaContext, batch: LlamaBatch) -> c_int,
    
    pub llama_model_has_encoder: unsafe extern "C" fn(model: *const LlamaModel) -> bool,
    
    pub llama_model_has_decoder: unsafe extern "C" fn(model: *const LlamaModel) -> bool,
    
    // Pooling type (for embeddings)
    pub llama_pooling_type: unsafe extern "C" fn(ctx: *const LlamaContext) -> c_int,
    
    // Model info
    pub llama_get_model: unsafe extern "C" fn(ctx: *const LlamaContext) -> *const LlamaModel,
    
    // Advanced sampling (for temperature, top-p, top-k, etc.)
    pub llama_sample_token: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        candidates: *mut LlamaTokenDataArray,
    ) -> LlamaToken,
    
    // Model quantization (for llama-quantize.exe functionality)
    pub llama_model_quantize: unsafe extern "C" fn(
        fname_inp: *const c_char,
        fname_out: *const c_char,
        params: *const c_void,  // llama_model_quantize_params
    ) -> u32,
    
    // LoRA adapters (for fine-tuning)
    pub llama_lora_adapter_init: unsafe extern "C" fn(
        model: *mut LlamaModel,
        path_lora: *const c_char,
    ) -> *mut c_void,  // llama_lora_adapter*
    
    pub llama_lora_adapter_set: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        adapter: *mut c_void,
        scale: c_float,
    ) -> c_int,
    
    pub llama_lora_adapter_remove: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        adapter: *mut c_void,
    ) -> c_int,
    
    pub llama_lora_adapter_clear: unsafe extern "C" fn(ctx: *mut LlamaContext),
    
    pub llama_lora_adapter_free: unsafe extern "C" fn(adapter: *mut c_void),
    
    // State save/load (for session persistence)
    pub llama_state_get_size: unsafe extern "C" fn(ctx: *mut LlamaContext) -> usize,
    
    pub llama_state_get_data: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        dst: *mut u8,
        size: usize,
    ) -> usize,
    
    pub llama_state_set_data: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        src: *const u8,
        size: usize,
    ) -> usize,
    
    pub llama_state_load_file: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        path_session: *const c_char,
        tokens_out: *mut LlamaToken,
        n_token_capacity: usize,
        n_token_count_out: *mut usize,
    ) -> bool,
    
    pub llama_state_save_file: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        path_session: *const c_char,
        tokens: *const LlamaToken,
        n_token_count: usize,
    ) -> bool,
    
    // Sequence state operations
    pub llama_state_seq_get_size: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        seq_id: c_int,
    ) -> usize,
    
    pub llama_state_seq_get_data: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        dst: *mut u8,
        size: usize,
        seq_id: c_int,
    ) -> usize,
    
    pub llama_state_seq_set_data: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        src: *const u8,
        size: usize,
        dest_seq_id: c_int,
    ) -> usize,
    
    pub llama_state_seq_save_file: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        filepath: *const c_char,
        seq_id: c_int,
        tokens: *const LlamaToken,
        n_token_count: usize,
    ) -> usize,
    
    pub llama_state_seq_load_file: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        filepath: *const c_char,
        dest_seq_id: c_int,
        tokens_out: *mut LlamaToken,
        n_token_capacity: usize,
        n_token_count_out: *mut usize,
    ) -> usize,
    
    // Model metadata inspection
    pub llama_model_meta_val_str: unsafe extern "C" fn(
        model: *const LlamaModel,
        key: *const c_char,
        buf: *mut c_char,
        buf_size: usize,
    ) -> c_int,
    
    pub llama_model_meta_count: unsafe extern "C" fn(model: *const LlamaModel) -> c_int,
    
    pub llama_model_meta_key_by_index: unsafe extern "C" fn(
        model: *const LlamaModel,
        i: c_int,
        buf: *mut c_char,
        buf_size: usize,
    ) -> c_int,
    
    pub llama_model_meta_val_str_by_index: unsafe extern "C" fn(
        model: *const LlamaModel,
        i: c_int,
        buf: *mut c_char,
        buf_size: usize,
    ) -> c_int,
    
    pub llama_model_desc: unsafe extern "C" fn(
        model: *const LlamaModel,
        buf: *mut c_char,
        buf_size: usize,
    ) -> c_int,
    
    pub llama_model_size: unsafe extern "C" fn(model: *const LlamaModel) -> u64,
    
    pub llama_model_n_params: unsafe extern "C" fn(model: *const LlamaModel) -> u64,
    
    // Model capabilities
    pub llama_model_is_recurrent: unsafe extern "C" fn(model: *const LlamaModel) -> bool,
    
    pub llama_model_decoder_start_token: unsafe extern "C" fn(model: *const LlamaModel) -> LlamaToken,
    
    // Hardware support queries
    pub llama_supports_mmap: unsafe extern "C" fn() -> bool,
    
    pub llama_supports_mlock: unsafe extern "C" fn() -> bool,
    
    pub llama_supports_gpu_offload: unsafe extern "C" fn() -> bool,
    
    pub llama_max_devices: unsafe extern "C" fn() -> usize,
    
    // Timing
    pub llama_time_us: unsafe extern "C" fn() -> i64,
    
    // Context info (additional)
    pub llama_n_batch: unsafe extern "C" fn(ctx: *const LlamaContext) -> u32,
    
    pub llama_n_ubatch: unsafe extern "C" fn(ctx: *const LlamaContext) -> u32,
    
    pub llama_n_seq_max: unsafe extern "C" fn(ctx: *const LlamaContext) -> u32,
    
    // KV cache inspection
    pub llama_get_kv_cache_token_count: unsafe extern "C" fn(ctx: *const LlamaContext) -> c_int,
    
    pub llama_get_kv_cache_used_cells: unsafe extern "C" fn(ctx: *const LlamaContext) -> c_int,
    
    pub llama_kv_cache_seq_pos_max: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        seq_id: c_int,
    ) -> c_int,
    
    pub llama_kv_cache_defrag: unsafe extern "C" fn(ctx: *mut LlamaContext),
    
    pub llama_kv_cache_update: unsafe extern "C" fn(ctx: *mut LlamaContext),
    
    pub llama_kv_cache_seq_div: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        seq_id: c_int,
        p0: c_int,
        p1: c_int,
        d: c_int,
    ),
    
    // Model layer info
    pub llama_n_layer: unsafe extern "C" fn(model: *const LlamaModel) -> c_int,
    
    pub llama_rope_freq_scale_train: unsafe extern "C" fn(model: *const LlamaModel) -> c_float,
    
    // Vocab info
    pub llama_vocab_type: unsafe extern "C" fn(model: *const LlamaModel) -> c_int,
    
    pub llama_rope_type: unsafe extern "C" fn(model: *const LlamaModel) -> c_int,
    
    // Token attributes (for detailed tokenization)
    pub llama_token_get_attr: unsafe extern "C" fn(
        model: *const LlamaModel,
        token: LlamaToken,
    ) -> c_int,
    
    pub llama_token_is_control: unsafe extern "C" fn(
        model: *const LlamaModel,
        token: LlamaToken,
    ) -> bool,
    
    // Token conversion (char-level)
    pub llama_token_get_text: unsafe extern "C" fn(
        model: *const LlamaModel,
        token: LlamaToken,
    ) -> *const c_char,
    
    pub llama_token_get_score: unsafe extern "C" fn(
        model: *const LlamaModel,
        token: LlamaToken,
    ) -> c_float,
    
    pub llama_token_get_type: unsafe extern "C" fn(
        model: *const LlamaModel,
        token: LlamaToken,
    ) -> c_int,
    
    // Print performance info
    pub llama_print_timings: unsafe extern "C" fn(ctx: *mut LlamaContext),
    
    pub llama_reset_timings: unsafe extern "C" fn(ctx: *mut LlamaContext),
    
    // Model tensors (for inspection/debugging)
    pub llama_get_model_tensor: unsafe extern "C" fn(
        model: *mut LlamaModel,
        name: *const c_char,
    ) -> *mut c_void,  // ggml_tensor*
    
    // Control vectors (for model steering)
    pub llama_control_vector_apply: unsafe extern "C" fn(
        ctx: *mut LlamaContext,
        data: *const c_float,
        len: usize,
        n_embd: c_int,
        il_start: c_int,
        il_end: c_int,
    ) -> c_int,
    
    // NUMA support
    pub llama_numa_init: unsafe extern "C" fn(numa: c_int),
    
    // Set causal attention vs non-causal
    pub llama_set_causal_attn: unsafe extern "C" fn(ctx: *mut LlamaContext, causal_attn: bool),
}

/// Batch structure for efficient inference
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LlamaBatch {
    pub n_tokens: c_int,
    pub token: *mut LlamaToken,
    pub embd: *const c_float,
    pub pos: *mut c_int,
    pub n_seq_id: *mut c_int,
    pub seq_id: *mut *mut c_int,
    pub logits: *mut i8,
    pub all_pos_0: c_int,
    pub all_pos_1: c_int,
    pub all_seq_id: c_int,
}

/// Token data for sampling
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LlamaTokenData {
    pub id: LlamaToken,
    pub logit: c_float,
    pub p: c_float,
}

/// Token data array for sampling
#[repr(C)]
#[derive(Debug)]
pub struct LlamaTokenDataArray {
    pub data: *mut LlamaTokenData,
    pub size: usize,
    pub sorted: bool,
}

