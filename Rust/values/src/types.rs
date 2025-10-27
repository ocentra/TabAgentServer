//! Runtime type information for values.
//!
//! This mirrors ort's ValueType but for TabAgent's domain.

use serde::{Deserialize, Serialize};

/// Runtime type information for a value.
///
/// # RAG: Type Safety
///
/// This enum represents the runtime type, while the marker traits
/// provide compile-time type safety. Both work together for maximum safety.
///
/// # Example
///
/// ```rust
/// use tabagent_values::ValueType;
///
/// let dtype = ValueType::ChatRequest {
///     model: "gpt-3.5-turbo".to_string(),
/// };
///
/// match dtype {
///     ValueType::ChatRequest { model } => {
///         println!("Chat with model: {}", model);
///     }
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ValueType {
    // ============ Request Types ============
    
    /// Chat completion request.
    ChatRequest {
        model: String,
    },

    /// Text generation request.
    GenerateRequest {
        model: String,
    },

    /// Embeddings generation request.
    EmbeddingsRequest {
        model: String,
    },

    /// Reranking request.
    RerankRequest {
        model: String,
    },

    /// Load model request.
    LoadModel {
        model_id: String,
        variant: Option<String>,
    },

    /// Unload model request.
    UnloadModel {
        model_id: String,
    },

    /// List models request.
    ListModels,

    /// Get model info request.
    ModelInfo {
        model_id: String,
    },

    /// RAG query request.
    RagQuery {
        query: String,
    },

    /// Chat history request.
    ChatHistory {
        session_id: Option<String>,
    },

    /// System info request.
    SystemInfo,

    /// Health check request.
    Health,

    // ============ Response Types ============
    
    /// Chat completion response.
    ChatResponse {
        id: String,
        model: String,
    },

    /// Text generation response.
    GenerateResponse {
        id: String,
    },

    /// Embeddings response.
    EmbeddingsResponse {
        model: String,
        dimensions: usize,
    },

    /// Rerank response.
    RerankResponse {
        model: String,
    },

    /// Model list response.
    ModelListResponse {
        count: usize,
    },

    /// Error response.
    ErrorResponse {
        code: String,
        message: String,
    },

    // ============ Model Data Types ============
    
    /// Tensor data (for ONNX, GGUF, etc.).
    Tensor {
        dtype: TensorDataType,
        shape: Vec<i64>,
    },

    /// Embedding vector.
    Embedding {
        dimensions: usize,
    },

    /// Model parameters.
    ModelParameters,

    /// Tokenizer data.
    TokenizerData,
}

/// Data type for tensor elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TensorDataType {
    Float16,
    Float32,
    Float64,
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Bool,
    String,
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::ChatRequest { model } => write!(f, "ChatRequest({})", model),
            ValueType::GenerateRequest { model } => write!(f, "GenerateRequest({})", model),
            ValueType::EmbeddingsRequest { model } => write!(f, "EmbeddingsRequest({})", model),
            ValueType::ChatResponse { id, model } => write!(f, "ChatResponse({}, {})", id, model),
            ValueType::Tensor { dtype, shape } => write!(f, "Tensor({:?}, {:?})", dtype, shape),
            _ => write!(f, "{:?}", self),
        }
    }
}

