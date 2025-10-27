//! Response value types.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::{
    Value, ValueType, ValueData, ValueInner, ValueResult,
    markers::ResponseValueMarker,
    DowncastableTarget,
};

/// Response value type alias.
pub type ResponseValue = Value<ResponseValueMarker>;

/// Concrete response data types (RAG: Use enums for type safety).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseType {
    /// Chat completion response.
    ChatResponse {
        id: String,
        model: String,
        response: String,
        usage: TokenUsage,
        #[serde(default)]
        finish_reason: Option<FinishReason>,
    },

    /// Text generation response.
    GenerateResponse {
        id: String,
        text: String,
        usage: TokenUsage,
    },

    /// Embeddings response.
    EmbeddingsResponse {
        model: String,
        embeddings: Vec<Vec<f32>>,
        usage: TokenUsage,
    },

    /// Rerank response.
    RerankResponse {
        model: String,
        results: Vec<RerankResult>,
    },

    /// Model list response.
    ModelListResponse {
        models: Vec<ModelInfo>,
    },

    /// Model info response.
    ModelInfoResponse {
        info: ModelInfo,
    },

    /// RAG query response.
    RagResponse {
        results: Vec<RagResult>,
        query_time_ms: u64,
    },

    /// Chat history response.
    ChatHistoryResponse {
        session_id: String,
        messages: Vec<crate::request::Message>,
    },

    /// System info response.
    SystemInfoResponse {
        system: SystemInfo,
    },

    /// Health check response.
    HealthResponse {
        status: HealthStatus,
        timestamp: DateTime<Utc>,
    },

    /// Success response (generic).
    Success {
        message: String,
    },

    /// Error response.
    Error {
        code: String,
        message: String,
        #[serde(default)]
        details: Option<serde_json::Value>,
    },

    // === WebRTC SIGNALING ===
    /// WebRTC session created response.
    WebRtcSessionCreated {
        session_id: String,
        created_at: String,
    },

    /// WebRTC session info response.
    WebRtcSessionInfo {
        session_id: String,
        state: String,
        offer: Option<String>,
        answer: Option<String>,
        ice_candidates: Vec<String>,
    },
}

/// Token usage information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

impl TokenUsage {
    pub fn new(prompt: u32, completion: u32) -> Self {
        Self {
            prompt_tokens: prompt,
            completion_tokens: completion,
            total_tokens: prompt + completion,
        }
    }

    pub fn zero() -> Self {
        Self {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        }
    }
}

/// Reason generation finished (RAG: Enum for type safety).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    Error,
}

/// Rerank result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResult {
    pub index: usize,
    pub score: f32,
    pub document: String,
}

/// Model information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub backend: String,
    pub loaded: bool,
    #[serde(default)]
    pub size_bytes: Option<u64>,
    #[serde(default)]
    pub parameters: Option<u64>,
}

/// RAG search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagResult {
    pub id: String,
    pub score: f32,
    pub content: String,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// System information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub gpu: Option<GpuInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model: String,
    pub cores: usize,
    pub threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub memory_bytes: u64,
    pub vendor: String,
}

/// Health status (RAG: Enum for type safety).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl ResponseValue {
    /// Create a chat response.
    pub fn chat(id: impl Into<String>, model: impl Into<String>, response: impl Into<String>, usage: TokenUsage) -> Self {
        let id = id.into();
        let model = model.into();
        Value {
            inner: ValueInner {
                data: ValueData::Response(Box::new(ResponseType::ChatResponse {
                    id: id.clone(),
                    model: model.clone(),
                    response: response.into(),
                    usage,
                    finish_reason: Some(FinishReason::Stop),
                })),
                dtype: ValueType::ChatResponse { id, model },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create an error response.
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        let code = code.into();
        let message = message.into();
        Value {
            inner: ValueInner {
                data: ValueData::Response(Box::new(ResponseType::Error {
                    code: code.clone(),
                    message: message.clone(),
                    details: None,
                })),
                dtype: ValueType::ErrorResponse { code, message },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a health response.
    pub fn health(status: HealthStatus) -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Response(Box::new(ResponseType::HealthResponse {
                    status,
                    timestamp: Utc::now(),
                })),
                dtype: ValueType::ChatResponse {
                    id: "health".to_string(),
                    model: "system".to_string(),
                },
            },
            _marker: std::marker::PhantomData,
        }
    }

    // === WebRTC SIGNALING CONSTRUCTORS ===

    /// Create a WebRTC session created response.
    pub fn webrtc_session_created(session_id: impl Into<String>) -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Response(Box::new(ResponseType::WebRtcSessionCreated {
                    session_id: session_id.into(),
                    created_at: Utc::now().to_rfc3339(),
                })),
                dtype: ValueType::ChatResponse {
                    id: "webrtc".to_string(),
                    model: "system".to_string(),
                },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create a WebRTC session info response.
    pub fn webrtc_session_info(
        session_id: impl Into<String>,
        state: impl Into<String>,
        offer: Option<String>,
        answer: Option<String>,
        ice_candidates: Vec<String>,
    ) -> Self {
        Value {
            inner: ValueInner {
                data: ValueData::Response(Box::new(ResponseType::WebRtcSessionInfo {
                    session_id: session_id.into(),
                    state: state.into(),
                    offer,
                    answer,
                    ice_candidates,
                })),
                dtype: ValueType::ChatResponse {
                    id: "webrtc".to_string(),
                    model: "system".to_string(),
                },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Create an embeddings response.
    /// 
    /// For now, we encode embeddings as JSON in a chat response.
    /// This is a temporary solution until we add a proper Embeddings response type.
    pub fn embeddings(embeddings: Vec<Vec<f32>>) -> Self {
        let embedding_json = serde_json::to_string(&embeddings)
            .unwrap_or_else(|_| "[]".to_string());
        
        Value {
            inner: ValueInner {
                data: ValueData::Response(Box::new(ResponseType::ChatResponse {
                    id: uuid::Uuid::new_v4().to_string(),
                    model: "embedding-model".to_string(),
                    response: embedding_json,
                    finish_reason: Some(FinishReason::Stop),
                    usage: TokenUsage::zero(),
                })),
                dtype: ValueType::ChatResponse {
                    id: "embeddings".to_string(),
                    model: "embedding-model".to_string(),
                },
            },
            _marker: std::marker::PhantomData,
        }
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> ValueResult<String> {
        match &self.inner.data {
            ValueData::Response(resp) => Ok(serde_json::to_string(resp)?),
            _ => unreachable!("ResponseValue always contains Response data"),
        }
    }

    /// Get the underlying response type.
    pub fn response_type(&self) -> &ResponseType {
        match &self.inner.data {
            ValueData::Response(resp) => resp,
            _ => unreachable!("ResponseValue always contains Response data"),
        }
    }

    /// Extract chat response data.
    pub fn as_chat(&self) -> Option<(&str, &str, &TokenUsage)> {
        match self.response_type() {
            ResponseType::ChatResponse { response, model, usage, .. } => {
                Some((response.as_str(), model.as_str(), usage))
            }
            _ => None,
        }
    }

    /// Extract generate response data.
    pub fn as_generate(&self) -> Option<(&str, &TokenUsage)> {
        match self.response_type() {
            ResponseType::GenerateResponse { text, usage, .. } => {
                Some((text.as_str(), usage))
            }
            _ => None,
        }
    }

    /// Extract embeddings response data.
    pub fn as_embeddings(&self) -> Option<&[Vec<f32>]> {
        match self.response_type() {
            ResponseType::EmbeddingsResponse { embeddings, .. } => {
                Some(embeddings.as_slice())
            }
            _ => None,
        }
    }

    /// Extract rerank response data.
    pub fn as_rerank(&self) -> Option<&[RerankResult]> {
        match self.response_type() {
            ResponseType::RerankResponse { results, .. } => {
                Some(results.as_slice())
            }
            _ => None,
        }
    }

    /// Extract model list response data.
    pub fn as_model_list(&self) -> Option<&[ModelInfo]> {
        match self.response_type() {
            ResponseType::ModelListResponse { models } => {
                Some(models.as_slice())
            }
            _ => None,
        }
    }

    /// Extract RAG response data.
    pub fn as_rag(&self) -> Option<(&[RagResult], u64)> {
        match self.response_type() {
            ResponseType::RagResponse { results, query_time_ms } => {
                Some((results.as_slice(), *query_time_ms))
            }
            _ => None,
        }
    }

    /// Extract chat history response data.
    pub fn as_chat_history(&self) -> Option<(&str, &[crate::request::Message])> {
        match self.response_type() {
            ResponseType::ChatHistoryResponse { session_id, messages } => {
                Some((session_id.as_str(), messages.as_slice()))
            }
            _ => None,
        }
    }

    /// Extract system info response data.
    pub fn as_system_info(&self) -> Option<&SystemInfo> {
        match self.response_type() {
            ResponseType::SystemInfoResponse { system } => {
                Some(system)
            }
            _ => None,
        }
    }

    /// Extract health response data.
    pub fn as_health(&self) -> Option<(HealthStatus, &DateTime<Utc>)> {
        match self.response_type() {
            ResponseType::HealthResponse { status, timestamp } => {
                Some((*status, timestamp))
            }
            _ => None,
        }
    }

    /// Extract error response data.
    pub fn as_error(&self) -> Option<(&str, &str)> {
        match self.response_type() {
            ResponseType::Error { code, message, .. } => {
                Some((code.as_str(), message.as_str()))
            }
            _ => None,
        }
    }

    /// Convert to JSON value (not debug string).
    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::to_value(self.response_type())
            .unwrap_or_else(|_| serde_json::json!({"error": "Failed to serialize response"}))
    }
}

impl DowncastableTarget for ResponseValueMarker {
    fn can_downcast(dtype: &ValueType) -> bool {
        matches!(
            dtype,
            ValueType::ChatResponse { .. }
                | ValueType::GenerateResponse { .. }
                | ValueType::EmbeddingsResponse { .. }
                | ValueType::RerankResponse { .. }
                | ValueType::ModelListResponse { .. }
                | ValueType::ErrorResponse { .. }
        )
    }

    fn type_name() -> &'static str {
        "ResponseValue"
    }
}

/// Extraction methods for ResponseValue
impl ResponseValue {
    /// Extract WebRTC session created info: (session_id, created_at)
    pub fn as_webrtc_session_created(&self) -> (&str, &str) {
        match self.response_type() {
            ResponseType::WebRtcSessionCreated { session_id, created_at } => {
                (session_id.as_str(), created_at.as_str())
            }
            _ => ("unknown", "unknown"),
        }
    }

    /// Extract WebRTC session info: (session_id, state, offer, answer, ice_candidates)
    pub fn as_webrtc_session_info(&self) -> (&str, &str, Option<&str>, Option<&str>, &[String]) {
        match self.response_type() {
            ResponseType::WebRtcSessionInfo { session_id, state, offer, answer, ice_candidates } => {
                (
                    session_id.as_str(),
                    state.as_str(),
                    offer.as_deref(),
                    answer.as_deref(),
                    ice_candidates.as_slice(),
                )
            }
            _ => ("unknown", "unknown", None, None, &[]),
        }
    }
}


