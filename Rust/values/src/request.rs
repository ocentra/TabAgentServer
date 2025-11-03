//! Request value types and constructors.
//!
//! All TabAgent API requests are represented as strongly-typed values.

use serde::{Deserialize, Serialize};
use crate::{
    Value, ValueType, ValueData, ValueInner, ValueResult,
    markers::{RequestValueMarker, ChatRequestMarker, GenerateRequestMarker},
    DowncastableTarget,
};

/// Request value type alias for clarity.
pub type RequestValue = Value<RequestValueMarker>;

/// Concrete request data types (RAG: Use enums for type safety, not strings).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum RequestType {
    /// Chat completion request.
    Chat {
        model: String,
        messages: Vec<Message>,
        #[serde(default)]
        temperature: Option<f32>,
        #[serde(default)]
        max_tokens: Option<u32>,
        #[serde(default)]
        top_p: Option<f32>,
        #[serde(default)]
        stream: bool,
    },

    /// Text generation request.
    Generate {
        model: String,
        prompt: String,
        #[serde(default)]
        temperature: Option<f32>,
        #[serde(default)]
        max_tokens: Option<u32>,
    },

    /// Embeddings generation request.
    Embeddings {
        model: String,
        input: EmbeddingInput,
    },

    /// Reranking request.
    Rerank {
        model: String,
        query: String,
        documents: Vec<String>,
        #[serde(default)]
        top_n: Option<usize>,
    },

    /// Load model request.
    LoadModel {
        model_id: String,
        variant: Option<String>,
        #[serde(default)]
        force_reload: bool,
    },

    /// Unload model request.
    UnloadModel {
        model_id: String,
    },

    /// List available models.
    ListModels {
        #[serde(default)]
        filter: Option<String>,
    },

    /// Get model information.
    ModelInfo {
        model_id: String,
    },

    /// RAG query.
    RagQuery {
        query: String,
        #[serde(default)]
        top_k: Option<usize>,
        #[serde(default)]
        filters: Option<serde_json::Value>,
    },

    /// Get chat history.
    ChatHistory {
        session_id: Option<String>,
        #[serde(default)]
        limit: Option<usize>,
    },

    /// Save chat message.
    SaveMessage {
        session_id: String,
        message: Message,
    },

    /// System information request.
    SystemInfo,

    /// Health check.
    Health,

    /// Stop generation.
    StopGeneration {
        request_id: String,
    },

    /// Get generation parameters.
    GetParams,

    /// Set generation parameters.
    SetParams {
        params: serde_json::Value,
    },

    /// Get performance statistics.
    GetStats,

    /// Get resource information.
    GetResources,

    /// Estimate memory for model.
    EstimateMemory {
        model: String,
        quantization: Option<String>,
    },

    /// Semantic search.
    SemanticSearchQuery {
        query: String,
        k: usize,
        filters: Option<serde_json::Value>,
    },

    /// Calculate similarity between two texts.
    CalculateSimilarity {
        text1: String,
        text2: String,
        model: Option<String>,
    },

    /// Evaluate embeddings quality.
    EvaluateEmbeddings {
        model: String,
        queries: Vec<String>,
        documents: Vec<String>,
    },

    /// Cluster documents.
    ClusterDocuments {
        documents: Vec<String>,
        n_clusters: usize,
        model: Option<String>,
    },

    /// Recommend content.
    RecommendContent {
        query: String,
        candidates: Vec<String>,
        top_n: usize,
        model: Option<String>,
    },

    /// Pull/download a model.
    PullModel {
        model: String,
        quantization: Option<String>,
    },

    /// Delete a model.
    DeleteModel {
        model_id: String,
    },

    /// Get available quantization variants for a model (for UI dropdown).
    GetModelQuants {
        repo_id: String,
    },

    /// Get inference settings for a specific model variant.
    GetInferenceSettings {
        repo_id: String,
        variant: String,
    },

    /// Save user-customized inference settings for a model variant.
    SaveInferenceSettings {
        repo_id: String,
        variant: String,
        settings: crate::inference_settings::InferenceSettings,
    },

    /// Get hardware recipes.
    GetRecipes,

    /// Get embedding models list.
    GetEmbeddingModels,

    /// Get loaded models.
    GetLoadedModels,

    /// Select a model as active.
    SelectModel {
        model_id: String,
    },

    // === HUGGINGFACE AUTHENTICATION ===
    /// Set HuggingFace API token.
    SetHfToken {
        token: String,
    },

    /// Get HuggingFace token status.
    GetHfTokenStatus,

    /// Clear HuggingFace token.
    ClearHfToken,

    // === HARDWARE DETECTION ===
    /// Get hardware information.
    GetHardwareInfo,

    /// Check if model can be loaded.
    CheckModelFeasibility {
        model_size_mb: u64,
    },

    /// Get recommended model sizes for current hardware.
    GetRecommendedModels,

    // === WebRTC SIGNALING ===
    /// Create WebRTC offer.
    CreateWebRtcOffer {
        sdp: String,
        peer_id: Option<String>,
    },

    /// Submit WebRTC answer.
    SubmitWebRtcAnswer {
        session_id: String,
        sdp: String,
    },

    /// Add ICE candidate.
    AddIceCandidate {
        session_id: String,
        candidate: String,
    },

    /// Get WebRTC session state.
    GetWebRtcSession {
        session_id: String,
    },

    // === MEDIA STREAMING ===
    /// Configure audio stream.
    AudioStream {
        codec: String,
        sample_rate: u32,
        bitrate: u32,
        channels: u8,
    },

    /// Configure video stream.
    VideoStream {
        codec: String,
        resolution: (u32, u32),
        bitrate: u32,
        framerate: u8,
        hardware_acceleration: bool,
    },
}

/// Chat message (RAG: Use enums for role, not strings).
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    #[serde(default)]
    pub name: Option<String>,
}

/// Message role (RAG: Type-safe enum instead of strings).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Function,
}

/// Embedding input can be single string or array.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbeddingInput {
    Single(String),
    Multiple(Vec<String>),
}

impl RequestValue {
    /// Create a chat request with full OpenAI-compatible parameters.
    pub fn chat_full(
        model: impl Into<String>,
        messages: Vec<Message>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        top_p: Option<f32>,
        stream: bool,
    ) -> Self {
        let model = model.into();
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::Chat {
                    model: model.clone(),
                    messages,
                    temperature,
                    max_tokens,
                    top_p,
                    stream,
                })),
                dtype: ValueType::ChatRequest { model },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a simple chat request (convenience method).
    pub fn chat(
        model: impl Into<String>,
        messages: Vec<Message>,
        temperature: Option<f32>,
    ) -> Self {
        Self::chat_full(model, messages, temperature, None, None, false)
    }

    /// Create a generate request with full parameters.
    pub fn generate_full(
        model: impl Into<String>,
        prompt: impl Into<String>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Self {
        let model = model.into();
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::Generate {
                    model: model.clone(),
                    prompt: prompt.into(),
                    temperature,
                    max_tokens,
                })),
                dtype: ValueType::GenerateRequest { model },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a generate request (convenience method).
    pub fn generate(
        model: impl Into<String>,
        prompt: impl Into<String>,
        temperature: Option<f32>,
    ) -> Self {
        Self::generate_full(model, prompt, temperature, None)
    }

    /// Create an embeddings request.
    pub fn embeddings(model: impl Into<String>, input: EmbeddingInput) -> Self {
        let model = model.into();
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::Embeddings {
                    model: model.clone(),
                    input,
                })),
                dtype: ValueType::EmbeddingsRequest { model },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a load model request.
    pub fn load_model(model_id: impl Into<String>, variant: Option<String>) -> Self {
        let model_id = model_id.into();
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::LoadModel {
                    model_id: model_id.clone(),
                    variant: variant.clone(),
                    force_reload: false,
                })),
                dtype: ValueType::LoadModel { model_id, variant },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create an unload model request.
    pub fn unload_model(model_id: impl Into<String>) -> Self {
        let model_id = model_id.into();
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::UnloadModel {
                    model_id: model_id.clone(),
                })),
                dtype: ValueType::UnloadModel { model_id },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a list models request.
    pub fn list_models() -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::ListModels { filter: None })),
                dtype: ValueType::ListModels,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a model info request.
    pub fn model_info(model_id: impl Into<String>) -> Self {
        let model_id = model_id.into();
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::ModelInfo {
                    model_id: model_id.clone(),
                })),
                dtype: ValueType::ModelInfo { model_id },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a chat history request.
    pub fn chat_history(session_id: Option<impl Into<String>>, limit: Option<usize>) -> Self {
        let session_id = session_id.map(|s| s.into());
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::ChatHistory {
                    session_id: session_id.clone(),
                    limit,
                })),
                dtype: ValueType::ChatHistory { session_id },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a save message request.
    pub fn save_message(session_id: impl Into<String>, message: &Message) -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::SaveMessage {
                    session_id: session_id.into(),
                    message: message.clone(),
                })),
                dtype: ValueType::ChatHistory { session_id: None },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a RAG query request.
    pub fn rag_query(query: impl Into<String>, top_k: Option<usize>, filters: Option<serde_json::Value>) -> Self {
        let query = query.into();
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::RagQuery {
                    query: query.clone(),
                    top_k,
                    filters,
                })),
                dtype: ValueType::RagQuery { query },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a rerank request.
    pub fn rerank(
        model: impl Into<String>,
        query: impl Into<String>,
        documents: Vec<String>,
        top_n: Option<usize>,
    ) -> Self {
        let model = model.into();
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::Rerank {
                    model: model.clone(),
                    query: query.into(),
                    documents,
                    top_n,
                })),
                dtype: ValueType::RerankRequest { model },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a system info request.
    pub fn system_info() -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::SystemInfo)),
                dtype: ValueType::SystemInfo,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a health check request.
    pub fn health() -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::Health)),
                dtype: ValueType::Health,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a stop generation request.
    pub fn stop_generation(request_id: impl Into<String>) -> Self {
        let request_id_str = request_id.into();
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::StopGeneration {
                    request_id: request_id_str.clone(),
                })),
                dtype: ValueType::StopGeneration {
                    request_id: request_id_str,
                },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a get params request.
    pub fn get_params() -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::GetParams)),
                dtype: ValueType::GetParams,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a set params request.
    pub fn set_params(params: impl serde::Serialize) -> Self {
        let params_json = serde_json::to_value(params)
            .unwrap_or_else(|_| serde_json::Value::Null);
        
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::SetParams {
                    params: params_json.clone(),
                })),
                dtype: ValueType::SetParams {
                    params: params_json,
                },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a get stats request.
    pub fn get_stats() -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::GetStats)),
                dtype: ValueType::GetStats,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a get resources request.
    pub fn get_resources() -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::GetResources)),
                dtype: ValueType::GetResources,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create an estimate memory request.
    pub fn estimate_memory(model: impl Into<String>, quantization: Option<String>) -> Self {
        let model_str = model.into();
        let quantization_clone = quantization.clone();
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::EstimateMemory {
                    model: model_str.clone(),
                    quantization: quantization_clone.clone(),
                })),
                dtype: ValueType::EstimateMemory {
                    model: model_str,
                    quantization: quantization_clone,
                },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a semantic search request.
    pub fn semantic_search(query: impl Into<String>, k: usize, filters: Option<serde_json::Value>) -> Self {
        let query_str = query.into();
        let filters_clone = filters.clone();
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::SemanticSearchQuery {
                    query: query_str.clone(),
                    k,
                    filters: filters_clone.clone(),
                })),
                dtype: ValueType::SemanticSearch {
                    query: query_str,
                    k,
                    filters: filters_clone,
                },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a calculate similarity request.
    pub fn calculate_similarity(text1: impl Into<String>, text2: impl Into<String>, model: Option<String>) -> Self {
        let text1_str = text1.into();
        let text2_str = text2.into();
        let model_clone = model.clone();
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::CalculateSimilarity {
                    text1: text1_str.clone(),
                    text2: text2_str.clone(),
                    model: model_clone.clone(),
                })),
                dtype: ValueType::CalculateSimilarity {
                    text1: text1_str,
                    text2: text2_str,
                    model: model_clone,
                },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create an evaluate embeddings request.
    pub fn evaluate_embeddings(model: impl Into<String>, queries: Vec<String>, documents: Vec<String>) -> Self {
        let model_str = model.into();
        let queries_clone = queries.clone();
        let documents_clone = documents.clone();
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::EvaluateEmbeddings {
                    model: model_str.clone(),
                    queries: queries_clone.clone(),
                    documents: documents_clone.clone(),
                })),
                dtype: ValueType::EvaluateEmbeddings {
                    model: model_str,
                    queries: queries_clone,
                    documents: documents_clone,
                },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a cluster documents request.
    pub fn cluster_documents(documents: Vec<String>, n_clusters: usize, model: Option<String>) -> Self {
        let documents_clone = documents.clone();
        let model_clone = model.clone();
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::ClusterDocuments {
                    documents: documents_clone.clone(),
                    n_clusters,
                    model: model_clone.clone(),
                })),
                dtype: ValueType::ClusterDocuments {
                    documents: documents_clone,
                    n_clusters,
                    model: model_clone,
                },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a recommend content request.
    pub fn recommend_content(query: impl Into<String>, candidates: Vec<String>, top_n: usize, model: Option<String>) -> Self {
        let query_str = query.into();
        let candidates_clone = candidates.clone();
        let model_clone = model.clone();
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::RecommendContent {
                    query: query_str.clone(),
                    candidates: candidates_clone.clone(),
                    top_n,
                    model: model_clone.clone(),
                })),
                dtype: ValueType::RecommendContent {
                    query: query_str,
                    candidates: candidates_clone,
                    top_n,
                    model: model_clone,
                },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a pull model request.
    pub fn pull_model(model: impl Into<String>, quantization: Option<String>) -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::PullModel {
                    model: model.into(),
                    quantization,
                })),
                dtype: ValueType::SystemInfo,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a delete model request.
    pub fn delete_model(model_id: impl Into<String>) -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::DeleteModel {
                    model_id: model_id.into(),
                })),
                dtype: ValueType::SystemInfo,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a get model quants request (for UI dropdown).
    pub fn get_model_quants(repo_id: impl Into<String>) -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::GetModelQuants {
                    repo_id: repo_id.into(),
                })),
                dtype: ValueType::SystemInfo,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a get inference settings request.
    pub fn get_inference_settings(repo_id: impl Into<String>, variant: impl Into<String>) -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::GetInferenceSettings {
                    repo_id: repo_id.into(),
                    variant: variant.into(),
                })),
                dtype: ValueType::SystemInfo,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a save inference settings request.
    pub fn save_inference_settings(
        repo_id: impl Into<String>,
        variant: impl Into<String>,
        settings: crate::inference_settings::InferenceSettings,
    ) -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::SaveInferenceSettings {
                    repo_id: repo_id.into(),
                    variant: variant.into(),
                    settings,
                })),
                dtype: ValueType::SystemInfo,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a get recipes request.
    pub fn get_recipes() -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::GetRecipes)),
                dtype: ValueType::SystemInfo,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a get embedding models request.
    pub fn get_embedding_models() -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::GetEmbeddingModels)),
                dtype: ValueType::SystemInfo,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a get loaded models request.
    pub fn get_loaded_models() -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::GetLoadedModels)),
                dtype: ValueType::SystemInfo,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a select model request.
    pub fn select_model(model_id: impl Into<String>) -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::SelectModel {
                    model_id: model_id.into(),
                })),
                dtype: ValueType::SystemInfo,
            },
            _marker: std::marker::PhantomData,
        }
    }

    // === HARDWARE DETECTION CONSTRUCTORS ===

    /// Create a get hardware info request.
    pub fn get_hardware_info() -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::GetHardwareInfo)),
                dtype: ValueType::SystemInfo,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a check model feasibility request.
    pub fn check_model_feasibility(model_size_mb: u64) -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::CheckModelFeasibility { model_size_mb })),
                dtype: ValueType::SystemInfo,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a get recommended models request.
    pub fn get_recommended_models() -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::GetRecommendedModels)),
                dtype: ValueType::SystemInfo,
            },
            _marker: std::marker::PhantomData,
        }
    }

    // === WebRTC SIGNALING CONSTRUCTORS ===

    /// Create a WebRTC offer request.
    pub fn create_webrtc_offer(sdp: impl Into<String>, peer_id: Option<impl Into<String>>) -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::CreateWebRtcOffer {
                    sdp: sdp.into(),
                    peer_id: peer_id.map(|p| p.into()),
                })),
                dtype: ValueType::SystemInfo,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a WebRTC answer request.
    pub fn submit_webrtc_answer(session_id: impl Into<String>, sdp: impl Into<String>) -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::SubmitWebRtcAnswer {
                    session_id: session_id.into(),
                    sdp: sdp.into(),
                })),
                dtype: ValueType::SystemInfo,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Add ICE candidate request.
    pub fn add_ice_candidate(session_id: impl Into<String>, candidate: impl Into<String>) -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::AddIceCandidate {
                    session_id: session_id.into(),
                    candidate: candidate.into(),
                })),
                dtype: ValueType::SystemInfo,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Get WebRTC session state request.
    pub fn get_webrtc_session(session_id: impl Into<String>) -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(RequestType::GetWebRtcSession {
                    session_id: session_id.into(),
                })),
                dtype: ValueType::SystemInfo,
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Parse a request from JSON.
    pub fn from_json(json: &str) -> ValueResult<Self> {
        let req_type: RequestType = serde_json::from_str(json)?;
        let dtype = match &req_type {
            RequestType::Chat { model, .. } => ValueType::ChatRequest { model: model.clone() },
            RequestType::Generate { model, .. } => ValueType::GenerateRequest { model: model.clone() },
            RequestType::Embeddings { model, .. } => ValueType::EmbeddingsRequest { model: model.clone() },
            RequestType::LoadModel { model_id, variant, .. } => {
                ValueType::LoadModel { model_id: model_id.clone(), variant: variant.clone() }
            }
            RequestType::UnloadModel { model_id } => {
                ValueType::UnloadModel { model_id: model_id.clone() }
            }
            RequestType::ListModels { .. } => ValueType::ListModels,
            RequestType::ModelInfo { model_id } => ValueType::ModelInfo { model_id: model_id.clone() },
            RequestType::RagQuery { query, .. } => ValueType::RagQuery { query: query.clone() },
            RequestType::ChatHistory { session_id, .. } => ValueType::ChatHistory { session_id: session_id.clone() },
            RequestType::SystemInfo => ValueType::SystemInfo,
            RequestType::Health => ValueType::Health,
            RequestType::Rerank { model, .. } => ValueType::RerankRequest { model: model.clone() },
            RequestType::SaveMessage { .. } => ValueType::ChatHistory { session_id: None },
            RequestType::StopGeneration { request_id } => ValueType::StopGeneration {
                request_id: request_id.clone(),
            },
            RequestType::GetParams => ValueType::GetParams,
            RequestType::SetParams { params } => ValueType::SetParams {
                params: params.clone(),
            },
            RequestType::GetStats => ValueType::GetStats,
            RequestType::GetResources => ValueType::GetResources,
            RequestType::EstimateMemory { model, quantization } => ValueType::EstimateMemory {
                model: model.clone(),
                quantization: quantization.clone(),
            },
            RequestType::SemanticSearchQuery { query, k, filters } => ValueType::SemanticSearch {
                query: query.clone(),
                k: *k,
                filters: filters.clone(),
            },
            RequestType::CalculateSimilarity { text1, text2, model } => ValueType::CalculateSimilarity {
                text1: text1.clone(),
                text2: text2.clone(),
                model: model.clone(),
            },
            RequestType::EvaluateEmbeddings { model, queries, documents } => ValueType::EvaluateEmbeddings {
                model: model.clone(),
                queries: queries.clone(),
                documents: documents.clone(),
            },
            RequestType::ClusterDocuments { documents, n_clusters, model } => ValueType::ClusterDocuments {
                documents: documents.clone(),
                n_clusters: *n_clusters,
                model: model.clone(),
            },
            RequestType::RecommendContent { query, candidates, top_n, model } => ValueType::RecommendContent {
                query: query.clone(),
                candidates: candidates.clone(),
                top_n: *top_n,
                model: model.clone(),
            },
            RequestType::PullModel { .. } => ValueType::SystemInfo,
            RequestType::DeleteModel { .. } => ValueType::SystemInfo,
            RequestType::GetModelQuants { .. } => ValueType::SystemInfo,
            RequestType::GetInferenceSettings { .. } => ValueType::SystemInfo,
            RequestType::SaveInferenceSettings { .. } => ValueType::SystemInfo,
            RequestType::GetRecipes => ValueType::SystemInfo,
            RequestType::GetEmbeddingModels => ValueType::SystemInfo,
            RequestType::GetLoadedModels => ValueType::SystemInfo,
            RequestType::SelectModel { .. } => ValueType::SystemInfo,
            RequestType::CreateWebRtcOffer { .. } => ValueType::SystemInfo,
            RequestType::SubmitWebRtcAnswer { .. } => ValueType::SystemInfo,
            RequestType::AddIceCandidate { .. } => ValueType::SystemInfo,
            RequestType::GetWebRtcSession { .. } => ValueType::SystemInfo,
            RequestType::AudioStream { .. } => ValueType::AudioStream,
            RequestType::VideoStream { .. } => ValueType::VideoStream,
            RequestType::SetHfToken { .. } => ValueType::SystemInfo,
            RequestType::GetHfTokenStatus => ValueType::SystemInfo,
            RequestType::ClearHfToken => ValueType::SystemInfo,
            RequestType::GetHardwareInfo => ValueType::SystemInfo,
            RequestType::CheckModelFeasibility { .. } => ValueType::SystemInfo,
            RequestType::GetRecommendedModels => ValueType::SystemInfo,
        };

        Ok(Value {
            inner: ValueInner {
                data: ValueData::Request(Box::new(req_type)),
                dtype,
            },
            _marker: std::marker::PhantomData,
        })
    }

    /// Get the underlying request type.
    pub fn request_type(&self) -> &RequestType {
        match &self.inner.data {
            ValueData::Request(req) => req,
            _ => unreachable!("RequestValue always contains Request data"),
        }
    }
}

// Implement downcast support (RAG: Use traits for polymorphism)
impl DowncastableTarget for RequestValueMarker {
    fn can_downcast(dtype: &ValueType) -> bool {
        matches!(
            dtype,
            ValueType::ChatRequest { .. }
                | ValueType::GenerateRequest { .. }
                | ValueType::EmbeddingsRequest { .. }
                | ValueType::LoadModel { .. }
                | ValueType::UnloadModel { .. }
                | ValueType::ListModels
                | ValueType::ModelInfo { .. }
                | ValueType::RagQuery { .. }
                | ValueType::ChatHistory { .. }
                | ValueType::SystemInfo
                | ValueType::Health
                | ValueType::RerankRequest { .. }
        )
    }

    fn type_name() -> &'static str {
        "RequestValue"
    }
}

impl DowncastableTarget for ChatRequestMarker {
    fn can_downcast(dtype: &ValueType) -> bool {
        matches!(dtype, ValueType::ChatRequest { .. })
    }

    fn type_name() -> &'static str {
        "ChatRequest"
    }
}

impl DowncastableTarget for GenerateRequestMarker {
    fn can_downcast(dtype: &ValueType) -> bool {
        matches!(dtype, ValueType::GenerateRequest { .. })
    }

    fn type_name() -> &'static str {
        "GenerateRequest"
    }
}


