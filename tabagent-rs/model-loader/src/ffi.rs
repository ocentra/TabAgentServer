//! Low-level FFI bindings to llama.cpp C API
//!
//! This module provides unsafe bindings to the llama.cpp library.
//! All function signatures match the C API exactly.

use std::os::raw::{c_char, c_float, c_int};

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
}

/// Batch structure for efficient inference
#[repr(C)]
#[derive(Debug, Clone)]
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

