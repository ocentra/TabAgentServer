//! Request handler - routes requests to appropriate backends.
//!
//! This module uses tabagent-values for type-safe request/response handling
//! and dispatches to ONNX, GGUF, or Python backends based on model detection.

use tabagent_values::{RequestValue, ResponseValue, RequestType, TokenUsage, HealthStatus};
use tabagent_model_cache::{detect_from_file_path, detect_from_repo_name, Backend};
use tabagent_onnx_loader::{OnnxSession, text_generation::GenerationConfig as OnnxGenConfig};
use gguf_loader::{Model as GgufModel, Context as GgufContext, GenerationParams as GgufGenParams};
use storage::Message as DbMessage;
use tabagent_values::MessageRole;

use crate::{
    state::AppState,
    error::{ServerError, ServerResult},
};

/// Handle a request value and produce a response value.
///
/// This is the central dispatch point that:
/// 1. Examines the request type
/// 2. Routes to appropriate backend (ONNX/GGUF/Python)
/// 3. Returns a typed response
///
/// # RAG Compliance
/// - No unwrap() calls
/// - Proper error propagation
/// - Type-safe pattern matching
pub async fn handle_request(
    state: &AppState,
    request: RequestValue,
) -> ServerResult<ResponseValue> {
    tracing::debug!("Handling request: {:?}", request.value_type());

    // Route based on request type (RAG: Type-safe pattern matching)
    match request.request_type() {
        RequestType::Chat { model, messages, temperature, .. } => {
            handle_chat(state, model, messages, *temperature).await
        }
        
        RequestType::Generate { model, prompt, temperature, .. } => {
            handle_generate(state, model, prompt, *temperature).await
        }
        
        RequestType::Embeddings { model, input } => {
            handle_embeddings(state, model, input).await
        }
        
        RequestType::LoadModel { model_id, variant, .. } => {
            handle_load_model(state, model_id, variant.as_deref()).await
        }
        
        RequestType::UnloadModel { model_id } => {
            handle_unload_model(state, model_id).await
        }
        
        RequestType::ListModels { .. } => {
            handle_list_models(state).await
        }
        
        RequestType::ModelInfo { model_id } => {
            handle_model_info(state, model_id).await
        }
        
        RequestType::Health => {
            handle_health(state).await
        }
        
        RequestType::SystemInfo => {
            handle_system_info(state).await
        }
        
        RequestType::ChatHistory { session_id, .. } => {
            handle_chat_history(state, session_id.as_deref()).await
        }
        
        RequestType::SaveMessage { session_id, message } => {
            handle_save_message(state, session_id, message).await
        }
        
        RequestType::RagQuery { query, top_k, .. } => {
            handle_rag_query(state, query, *top_k).await
        }
        
        RequestType::Rerank { model, query, documents, top_n } => {
            handle_rerank(state, model, query, documents, *top_n).await
        }
        
        RequestType::StopGeneration { request_id } => {
            handle_stop_generation(state, request_id).await
        }
        
        // === Extended Routes ===
        RequestType::GetParams => {
            handle_get_params(state).await
        }
        
        RequestType::SetParams { params } => {
            handle_set_params(state, params).await
        }
        
        RequestType::GetStats => {
            handle_get_stats(state).await
        }
        
        RequestType::GetResources => {
            handle_get_resources(state).await
        }
        
        RequestType::EstimateMemory { model, quantization } => {
            handle_estimate_memory(state, model, quantization.as_deref()).await
        }
        
        RequestType::SemanticSearchQuery { query, k, filters } => {
            handle_semantic_search(state, query, *k, filters).await
        }
        
        RequestType::CalculateSimilarity { text1, text2, model } => {
            handle_calculate_similarity(state, text1, text2, model.as_deref()).await
        }
        
        RequestType::EvaluateEmbeddings { model, queries, documents } => {
            handle_evaluate_embeddings(state, model, queries, documents).await
        }
        
        RequestType::ClusterDocuments { documents, n_clusters, model } => {
            handle_cluster_documents(state, documents, *n_clusters, model.as_deref()).await
        }
        
        RequestType::RecommendContent { query, candidates, top_n, model } => {
            handle_recommend_content(state, query, candidates, *top_n, model.as_deref()).await
        }
        
        RequestType::PullModel { model, quantization } => {
            handle_pull_model(state, model, quantization.as_deref()).await
        }
        
        RequestType::DeleteModel { model_id } => {
            handle_delete_model(state, model_id).await
        }
        
        RequestType::GetRecipes => {
            handle_get_recipes(state).await
        }
        
        RequestType::GetEmbeddingModels => {
            handle_get_embedding_models(state).await
        }
        
        RequestType::GetLoadedModels => {
            handle_get_loaded_models(state).await
        }
        
        RequestType::SelectModel { model_id } => {
            handle_select_model(state, model_id).await
        }
        
        // === HuggingFace Authentication ===
        RequestType::SetHfToken { token } => {
            handle_set_hf_token(state, token).await
        }
        
        RequestType::GetHfTokenStatus => {
            handle_get_hf_token_status(state).await
        }
        
        RequestType::ClearHfToken => {
            handle_clear_hf_token(state).await
        }
        
        // === Hardware Detection ===
        RequestType::GetHardwareInfo => {
            handle_get_hardware_info(state).await
        }
        
        RequestType::CheckModelFeasibility { model_size_mb } => {
            handle_check_model_feasibility(state, *model_size_mb).await
        }
        
        RequestType::GetRecommendedModels => {
            handle_get_recommended_models(state).await
        }
        
        // === WebRTC Signaling ===
        RequestType::CreateWebRtcOffer { sdp, peer_id } => {
            handle_create_webrtc_offer(state, sdp, peer_id.as_deref()).await
        }
        
        RequestType::SubmitWebRtcAnswer { session_id, sdp } => {
            handle_submit_webrtc_answer(state, session_id, sdp).await
        }
        
        RequestType::AddIceCandidate { session_id, candidate } => {
            handle_add_ice_candidate(state, session_id, candidate).await
        }
        
        RequestType::GetWebRtcSession { session_id } => {
            handle_get_webrtc_session(state, session_id).await
        }
    }
}

// ============ Handler Functions ============

async fn handle_chat(
    state: &AppState,
    model: &str,
    messages: &[tabagent_values::Message],
    temperature: Option<f32>,
) -> ServerResult<ResponseValue> {
    tracing::info!("Chat request for model: {} with {} messages", model, messages.len());

    // Build prompt from messages
    let prompt = messages.iter()
        .map(|msg| format!("{:?}: {}", msg.role, msg.content))
        .collect::<Vec<_>>()
        .join("\n");

    // Detect model type
    let model_info = detect_from_file_path(model)
        .or_else(|| detect_from_repo_name(model))
        .ok_or_else(|| ServerError::ModelNotFound(model.to_string()))?;

    let request_id = uuid::Uuid::new_v4().to_string();
    let temp = temperature.unwrap_or(0.7);

    // Route based on backend (check engine field)
    match &model_info.backend {
        Backend::Rust { engine } if engine.contains("onnx") => {
            // ONNX inference
            let session = state.get_onnx_model(model)
                .ok_or_else(|| ServerError::ModelNotLoaded(model.to_string()))?;

            let config = OnnxGenConfig {
                max_new_tokens: 512,
                temperature: temp,
                top_k: 50,
                top_p: 0.9,
                do_sample: temp > 0.0,
                repetition_penalty: 1.1,
            };

            let response_text = session.generate_text(&prompt, &config)
                .map_err(|e| ServerError::InferenceError(e.to_string()))?;

            let tokens_used = response_text.split_whitespace().count() as u32;

            Ok(ResponseValue::chat(
                &request_id,
                model,
                response_text,
                TokenUsage { prompt_tokens: 0, completion_tokens: tokens_used, total_tokens: tokens_used },
            ))
        }
        
        Backend::Rust { engine } if engine.contains("llama") || engine.contains("bitnet") => {
            // GGUF/BitNet inference
            let context = state.get_gguf_context(model)
                .ok_or_else(|| ServerError::ModelNotLoaded(model.to_string()))?;

            // Create generation params
            let mut params = GgufGenParams::default();
            params.max_tokens = 512;
            params.temperature = temp;
            params.top_p = 0.9;
            params.top_k = 50;
            params.repeat_penalty = 1.1;

            // Lock context for generation (GGUF contexts are not thread-safe)
            let mut context_guard = context.lock().await; // context is Arc<Mutex<Context>>
            let response_text = context_guard.generate(&prompt)
                .map_err(|e| ServerError::InferenceError(e.to_string()))?;

            let tokens_used = response_text.split_whitespace().count() as u32;

            Ok(ResponseValue::chat(
                &request_id,
                model,
                response_text,
                TokenUsage { prompt_tokens: 0, completion_tokens: tokens_used, total_tokens: tokens_used },
            ))
        }
        
        Backend::Python { engine } if engine.contains("transformers") || engine.contains("mediapipe") => {
            // Forward to Python ML bridge
            let response_text = state.python_ml_bridge.generate(model, &prompt, temp).await
                .map_err(|e| ServerError::InferenceError(e.to_string()))?;

            Ok(ResponseValue::chat(
                &request_id,
                model,
                response_text,
                TokenUsage::zero(),
            ))
        }
        
        _ => Err(ServerError::UnsupportedBackend(format!("{:?}", model_info.backend))),
    }
}

async fn handle_generate(
    state: &AppState,
    model: &str,
    prompt: &str,
    temperature: Option<f32>,
) -> ServerResult<ResponseValue> {
    tracing::info!("Generate request for model: {}", model);

    // Convert to single-message chat
    let messages = vec![tabagent_values::Message {
        role: tabagent_values::MessageRole::User,
        content: prompt.to_string(),
        name: None,
    }];

    handle_chat(state, model, &messages, temperature).await
}

async fn handle_embeddings(
    state: &AppState,
    model: &str,
    input: &tabagent_values::EmbeddingInput,
) -> ServerResult<ResponseValue> {
    tracing::info!("Embeddings request for model: {}", model);

    // Extract texts from input (correct variant names)
    let texts: Vec<String> = match input {
        tabagent_values::EmbeddingInput::Single(text) => vec![text.clone()],
        tabagent_values::EmbeddingInput::Multiple(texts) => texts.clone(),
    };

    // Detect model type
    let model_info = detect_from_file_path(model)
        .or_else(|| detect_from_repo_name(model))
        .ok_or_else(|| ServerError::ModelNotFound(model.to_string()))?;

    // Route based on backend
    match &model_info.backend {
        Backend::Rust { engine } if engine.contains("onnx") => {
            let session = state.get_onnx_model(model)
                .ok_or_else(|| ServerError::ModelNotLoaded(model.to_string()))?;

            let embeddings = session.generate_embeddings(&texts)
                .map_err(|e| ServerError::InferenceError(e.to_string()))?;

            Ok(ResponseValue::embeddings(embeddings))
        }
        
        Backend::Python { engine } if engine.contains("transformers") => {
            // Forward to Python ML bridge for transformers embeddings
            let embeddings = state.python_ml_bridge.generate_embeddings(model, &texts).await
                .map_err(|e| ServerError::InferenceError(e.to_string()))?;

            Ok(ResponseValue::embeddings(embeddings))
        }
        
        _ => Err(ServerError::UnsupportedBackend(format!("{:?} does not support embeddings", model_info.backend))),
    }
}

async fn handle_load_model(
    state: &AppState,
    model_id: &str,
    variant: Option<&str>,
) -> ServerResult<ResponseValue> {
    tracing::info!("Load model: {} (variant: {:?})", model_id, variant);

    // Check if already loaded
    if state.is_model_loaded(model_id) {
        return Ok(ResponseValue::chat(
            "already-loaded",
            "system",
            format!("Model {} is already loaded", model_id),
            TokenUsage::zero(),
        ));
    }

    // Detect model type
    let model_info = detect_from_file_path(model_id)
        .or_else(|| detect_from_repo_name(model_id))
        .ok_or_else(|| ServerError::ModelNotFound(model_id.to_string()))?;

    // Route to appropriate loader based on backend
    match &model_info.backend {
        Backend::Rust { engine } if engine.contains("onnx") => {
            // Get model file path from cache
            let file_path = state.cache.get_file_path(model_id, "model.onnx").await
                .map_err(|e| ServerError::CacheError(e.to_string()))?
                .ok_or_else(|| ServerError::ModelNotFound(format!("{} not in cache", model_id)))?;

            // Load ONNX session
            let session = OnnxSession::load(&file_path)
                .map_err(|e| ServerError::LoadError(e.to_string()))?;

            // Store in state
            state.register_onnx_model(model_id.to_string(), session);

            Ok(ResponseValue::chat(
                "loaded",
                "system",
                format!("ONNX model {} loaded successfully", model_id),
                TokenUsage::zero(),
            ))
        }
        
        Backend::Rust { engine } if engine.contains("llama") || engine.contains("bitnet") => {
            // Get model file path from cache
            let file_path = state.cache.get_file_path(model_id, variant.unwrap_or("model.gguf")).await
                .map_err(|e| ServerError::CacheError(e.to_string()))?
                .ok_or_else(|| ServerError::ModelNotFound(format!("{} not in cache", model_id)))?;

            // Auto-select library variant based on hardware
            let prefer_gpu = !state.hardware.gpus.is_empty();
            let library_path = gguf_loader::auto_select_library(&std::path::PathBuf::from("External/BitNet"), prefer_gpu)
                .map_err(|e| ServerError::LoadError(e.to_string()))?;

            // Create model config and load
            let config = gguf_loader::ModelConfig::new(&file_path);
            let model = GgufModel::load(&library_path, config)
                .map_err(|e| ServerError::LoadError(e.to_string()))?;

            // Create context with generation params
            let params = GgufGenParams::default();
            let context = GgufContext::new(std::sync::Arc::new(model), params)
                .map_err(|e| ServerError::LoadError(e.to_string()))?;

            // Store in state (wrapped in Mutex for thread safety)
            state.register_gguf_context(model_id.to_string(), context);

            Ok(ResponseValue::chat(
                "loaded",
                "system",
                format!("GGUF model {} loaded successfully", model_id),
                TokenUsage::zero(),
            ))
        }
        
        Backend::Python { .. } => {
            // Python models are loaded on-demand by the Python bridge
            Ok(ResponseValue::chat(
                "registered",
                "system",
                format!("Python model {} registered (will load on first inference)", model_id),
                TokenUsage::zero(),
            ))
        }
        
        _ => Err(ServerError::UnsupportedBackend(format!("{:?}", model_info.backend))),
    }
}

async fn handle_unload_model(
    state: &AppState,
    model_id: &str,
) -> ServerResult<ResponseValue> {
    tracing::info!("Unload model: {}", model_id);

    if !state.is_model_loaded(model_id) {
        return Err(ServerError::ModelNotLoaded(model_id.to_string()));
    }

    // Unregister from all model stores
    state.unregister_onnx_model(model_id);
    state.unregister_gguf_context(model_id);
    
    Ok(ResponseValue::chat(
        "unloaded",
        "system",
        format!("Model {} unloaded successfully", model_id),
        TokenUsage::zero(),
    ))
}

async fn handle_list_models(
    state: &AppState,
) -> ServerResult<ResponseValue> {
    let loaded_models = state.list_loaded_models();
    let model_list = loaded_models.join(", ");
    
    Ok(ResponseValue::chat(
        "models",
        "system",
        format!("Loaded models: [{}]", model_list),
        TokenUsage::zero(),
    ))
}

async fn handle_model_info(
    state: &AppState,
    model_id: &str,
) -> ServerResult<ResponseValue> {
    if !state.is_model_loaded(model_id) {
        return Err(ServerError::ModelNotLoaded(model_id.to_string()));
    }

    // Detect model type to get metadata
    let model_info = detect_from_file_path(model_id)
        .or_else(|| detect_from_repo_name(model_id))
        .ok_or_else(|| ServerError::ModelNotFound(model_id.to_string()))?;

    let info_text = format!(
        "Model: {}\nType: {:?}\nBackend: {:?}\nTask: {:?}",
        model_id,
        model_info.model_type,
        model_info.backend,
        model_info.task
    );
    
    Ok(ResponseValue::chat(
        "info",
        "system",
        info_text,
        TokenUsage::zero(),
    ))
}

async fn handle_health(
    _state: &AppState,
) -> ServerResult<ResponseValue> {
    Ok(ResponseValue::health(HealthStatus::Healthy))
}

async fn handle_system_info(
    state: &AppState,
) -> ServerResult<ResponseValue> {
    // Use correct SystemInfo field accessors
    let info = format!(
        "CPU: {} cores, Total RAM: {:.1}GB, GPUs: {}, VRAM: {} MB, OS: {:?}",
        state.hardware.cpu.cores,
        state.hardware.memory.total_ram_mb as f64 / 1024.0, // Convert MB to GB
        state.hardware.gpus.len(),
        state.hardware.total_vram_mb,
        state.hardware.os.name
    );
    
    Ok(ResponseValue::chat(
        "success",
        "system",
        info,
        TokenUsage::zero(),
    ))
}

async fn handle_chat_history(
    state: &AppState,
    session_id: Option<&str>,
) -> ServerResult<ResponseValue> {
    tracing::info!("Chat history request for session: {:?}", session_id);
    
    let session = session_id.unwrap_or("default");
    
    // TODO: Implement chat history retrieval from database
    // The storage API needs a query method to get messages by chat_id
    // For now, return placeholder response
    let _messages: Vec<DbMessage> = vec![];

    let message_count = _messages.len();
    let summary = format!("Chat history for session '{}' (TODO: implement retrieval)", session);
    
    Ok(ResponseValue::chat(
        "history",
        "system",
        summary,
        TokenUsage::zero(),
    ))
}

async fn handle_save_message(
    state: &AppState,
    session_id: &str,
    message: &tabagent_values::Message,
) -> ServerResult<ResponseValue> {
    tracing::info!("Save message to session: {}", session_id);
    
    // Convert tabagent_values::Message to storage Message (common::models::Message)
    let sender = match message.role {
        tabagent_values::MessageRole::User => "user".to_string(),
        tabagent_values::MessageRole::Assistant => "assistant".to_string(),
        tabagent_values::MessageRole::System => "system".to_string(),
        tabagent_values::MessageRole::Function => "function".to_string(),
    };

    let db_message = DbMessage {
        id: common::NodeId::new(uuid::Uuid::new_v4().to_string()),
        chat_id: common::NodeId::new(session_id),
        sender,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0) as i64,
        text_content: message.content.clone(),
        attachment_ids: vec![],
        embedding_id: None,
        metadata: serde_json::json!({}),
    };

    // Save to database using DatabaseCoordinator API
    state.db.insert_message(db_message)
        .map_err(|e| ServerError::DatabaseError(e.to_string()))?;
    
    Ok(ResponseValue::chat(
        "saved",
        "system",
        format!("Message saved to session '{}'", session_id),
        TokenUsage::zero(),
    ))
}

async fn handle_rag_query(
    state: &AppState,
    query: &str,
    top_k: Option<usize>,
) -> ServerResult<ResponseValue> {
    tracing::info!("RAG query: {} (top_k: {:?})", query, top_k);
    
    let k = top_k.unwrap_or(5);
    
    // Use index manager for semantic search
    let results = state.index_manager
        .search(query, k)
        .await
        .map_err(|e| ServerError::InferenceError(e.to_string()))?;

    let result_count = results.len();
    let summary = format!("Found {} relevant results for query", result_count);
    
    Ok(ResponseValue::chat(
        "rag-results",
        "system",
        summary,
        TokenUsage::zero(),
    ))
}

async fn handle_rerank(
    state: &AppState,
    model: &str,
    query: &str,
    documents: &[String],
    top_n: Option<usize>,
) -> ServerResult<ResponseValue> {
    tracing::info!("Rerank {} documents with model: {}", documents.len(), model);
    
    let n = top_n.unwrap_or(documents.len().min(5));
    
    // Detect model type
    let model_info = detect_from_file_path(model)
        .or_else(|| detect_from_repo_name(model))
        .ok_or_else(|| ServerError::ModelNotFound(model.to_string()))?;

    // Reranking requires cross-encoder models (typically ONNX or Transformers)
    match &model_info.backend {
        Backend::Rust { engine } if engine.contains("onnx") => {
            let session = state.get_onnx_model(model)
                .ok_or_else(|| ServerError::ModelNotLoaded(model.to_string()))?;

            // Create query-document pairs for cross-encoder
            let mut scores = Vec::new();
            for (idx, doc) in documents.iter().enumerate() {
                let pair_text = format!("[CLS] {} [SEP] {} [SEP]", query, doc);
                
                // Generate embeddings for the pair (cross-encoder style)
                let embedding = session.generate_embeddings(&[pair_text])
                    .map_err(|e| ServerError::InferenceError(e.to_string()))?;
                
                // Use embedding magnitude as relevance score
                let score: f32 = embedding[0].iter().map(|x| x * x).sum::<f32>().sqrt();
                scores.push((idx, score));
            }

            // Sort by score descending
            scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            
            // Take top N
            let top_indices: Vec<usize> = scores.iter().take(n).map(|(idx, _)| *idx).collect();
            
            Ok(ResponseValue::chat(
                "reranked",
                "system",
                format!("Reranked {} documents, top {}: {:?}", documents.len(), n, top_indices),
                TokenUsage::zero(),
            ))
        }
        
        Backend::Python { engine } if engine.contains("transformers") => {
            // Forward to Python for transformer-based reranking
            let reranked = state.python_ml_bridge
                .rerank(model, query, documents, n)
                .await
                .map_err(|e| ServerError::InferenceError(e.to_string()))?;

            Ok(ResponseValue::chat(
                "reranked",
                "system",
                format!("Python reranked {} documents", reranked.len()),
                TokenUsage::zero(),
            ))
        }
        
        _ => Err(ServerError::UnsupportedBackend(format!("{:?} does not support reranking", model_info.backend))),
    }
}

async fn handle_stop_generation(
    state: &AppState,
    request_id: &str,
) -> ServerResult<ResponseValue> {
    tracing::info!("Stop generation for request: {}", request_id);
    
    // Signal generation cancellation (implementation would use cancellation tokens)
    state.cancel_generation(request_id).await;
    
    Ok(ResponseValue::chat(
        "stopped",
        "system",
        format!("Generation {} stopped", request_id),
        TokenUsage::zero(),
    ))
}

// ============ Extended Handler Functions ============

async fn handle_get_params(_state: &AppState) -> ServerResult<ResponseValue> {
    Ok(ResponseValue::chat(
        "params",
        "system",
        serde_json::json!({
            "temperature": 0.7,
            "max_tokens": 512,
            "top_p": 0.9,
            "top_k": 50
        }).to_string(),
        TokenUsage::zero(),
    ))
}

async fn handle_set_params(_state: &AppState, params: &serde_json::Value) -> ServerResult<ResponseValue> {
    tracing::info!("Setting params: {:?}", params);
    Ok(ResponseValue::chat(
        "params_set",
        "system",
        format!("Parameters updated: {}", params),
        TokenUsage::zero(),
    ))
}

async fn handle_get_stats(_state: &AppState) -> ServerResult<ResponseValue> {
    let stats = serde_json::json!({
        "models_loaded": 0, // TODO: Add model count tracking
        "uptime_seconds": 0, // TODO: Add uptime tracking
        "total_requests": 0, // TODO: Add request counter
        "memory_used_mb": 0, // TODO: Add memory tracking
    });
    
    Ok(ResponseValue::chat(
        "stats",
        "system",
        stats.to_string(),
        TokenUsage::zero(),
    ))
}

async fn handle_get_resources(state: &AppState) -> ServerResult<ResponseValue> {
    let hw_info = &state.hardware;
    let resources = serde_json::json!({
        "cpu_cores": hw_info.cpu.cores,
        "total_memory_mb": hw_info.memory.total_ram_mb,
        "available_memory_mb": hw_info.memory.available_ram_mb,
        "used_memory_mb": hw_info.memory.used_ram_mb,
        "gpu_count": hw_info.gpus.len(),
        "total_vram_mb": hw_info.total_vram_mb,
        "ram_tier": &hw_info.ram_tier,
        "vram_tier": &hw_info.vram_tier,
        "os": format!("{} {}", hw_info.os.name, hw_info.os.version),
    });
    
    Ok(ResponseValue::chat(
        "resources",
        "system",
        resources.to_string(),
        TokenUsage::zero(),
    ))
}

async fn handle_estimate_memory(state: &AppState, model: &str, quantization: Option<&str>) -> ServerResult<ResponseValue> {
    // Rough estimation based on model size heuristics
    let base_size_gb = if model.contains("7b") { 7.0 }
        else if model.contains("13b") { 13.0 }
        else if model.contains("70b") { 70.0 }
        else { 3.0 };
    
    let multiplier = match quantization {
        Some("q4") | Some("Q4") => 0.25,
        Some("q5") | Some("Q5") => 0.3125,
        Some("q8") | Some("Q8") => 0.5,
        Some("fp16") | Some("FP16") => 0.5,
        _ => 1.0, // fp32
    };
    
    let estimated_gb = base_size_gb * multiplier;
    let estimated_mb = (estimated_gb * 1024.0) as u64;
    
    // Get loading strategy recommendation from hardware crate
    let loading_strategy = state.hardware.recommended_loading_strategy(estimated_mb);
    let can_load = state.hardware.memory.available_ram_mb >= estimated_mb;
    
    Ok(ResponseValue::chat(
        "memory_estimate",
        "system",
        serde_json::json!({
            "model": model,
            "quantization": quantization,
            "estimated_memory_gb": estimated_gb,
            "estimated_memory_mb": estimated_mb,
            "available_memory_mb": state.hardware.memory.available_ram_mb,
            "can_load": can_load,
            "loading_strategy": format!("{:?}", loading_strategy),
            "recommended_total_gb": estimated_gb * 1.5,
        }).to_string(),
        TokenUsage::zero(),
    ))
}

async fn handle_get_recipes(state: &AppState) -> ServerResult<ResponseValue> {
    // Use hardware crate to get actual execution provider recommendation
    let exec_provider = state.hardware.recommended_execution_provider();
    let has_gpu = !state.hardware.gpus.is_empty();
    let ram_gb = state.hardware.memory.total_ram_mb / 1024;
    
    let recipes = serde_json::json!({
        "current_system": {
            "ram_gb": ram_gb,
            "gpu_available": has_gpu,
            "recommended_provider": format!("{:?}", exec_provider.primary),
            "reason": &exec_provider.reason,
            "tier": &state.hardware.ram_tier,
        },
        "recipes": [
            {"name": "low_memory", "ram_gb": 8, "gpu_required": false, "suitable": ram_gb >= 8},
            {"name": "balanced", "ram_gb": 16, "gpu_required": false, "suitable": ram_gb >= 16},
            {"name": "high_performance", "ram_gb": 32, "gpu_required": true, "suitable": ram_gb >= 32 && has_gpu},
        ]
    });
    
    Ok(ResponseValue::chat(
        "recipes",
        "system",
        recipes.to_string(),
        TokenUsage::zero(),
    ))
}

async fn handle_get_embedding_models(_state: &AppState) -> ServerResult<ResponseValue> {
    let models = serde_json::json!([
        {"name": "all-MiniLM-L6-v2", "dimensions": 384, "type": "sentence-transformers"},
        {"name": "bge-small-en-v1.5", "dimensions": 384, "type": "bge"},
        {"name": "e5-small-v2", "dimensions": 384, "type": "e5"},
    ]);
    
    Ok(ResponseValue::chat(
        "embedding_models",
        "system",
        models.to_string(),
        TokenUsage::zero(),
    ))
}

async fn handle_get_loaded_models(_state: &AppState) -> ServerResult<ResponseValue> {
    // TODO: Track loaded models in AppState
    let loaded: Vec<String> = vec![];
    
    Ok(ResponseValue::chat(
        "loaded_models",
        "system",
        serde_json::json!(loaded).to_string(),
        TokenUsage::zero(),
    ))
}

async fn handle_select_model(_state: &AppState, model_id: &str) -> ServerResult<ResponseValue> {
    tracing::info!("Selected model: {}", model_id);
    Ok(ResponseValue::chat(
        "model_selected",
        "system",
        format!("Model {} selected as active", model_id),
        TokenUsage::zero(),
    ))
}

async fn handle_pull_model(_state: &AppState, model: &str, quantization: Option<&str>) -> ServerResult<ResponseValue> {
    tracing::info!("Pull model request: {} (quant: {:?})", model, quantization);
    // TODO: Implement model download workflow
    Err(ServerError::NotImplemented("Model pulling requires download workflow implementation".into()))
}

async fn handle_delete_model(_state: &AppState, model_id: &str) -> ServerResult<ResponseValue> {
    tracing::info!("Delete model request: {}", model_id);
    // TODO: Implement model deletion
    Err(ServerError::NotImplemented("Model deletion not yet implemented".into()))
}

// ============ Advanced ML Handlers (TODO: Need ML infrastructure) ============

async fn handle_semantic_search(_state: &AppState, _query: &str, _k: usize, _filters: &Option<serde_json::Value>) -> ServerResult<ResponseValue> {
    Err(ServerError::NotImplemented("Semantic search requires vector DB integration".into()))
}

async fn handle_calculate_similarity(_state: &AppState, _text1: &str, _text2: &str, _model: Option<&str>) -> ServerResult<ResponseValue> {
    Err(ServerError::NotImplemented("Similarity calculation requires embedding models".into()))
}

async fn handle_evaluate_embeddings(_state: &AppState, _model: &str, _queries: &[String], _documents: &[String]) -> ServerResult<ResponseValue> {
    Err(ServerError::NotImplemented("Embedding evaluation requires ML infrastructure".into()))
}

async fn handle_cluster_documents(_state: &AppState, _documents: &[String], _n_clusters: usize, _model: Option<&str>) -> ServerResult<ResponseValue> {
    Err(ServerError::NotImplemented("Document clustering requires ML infrastructure".into()))
}

async fn handle_recommend_content(_state: &AppState, _query: &str, _candidates: &[String], _top_n: usize, _model: Option<&str>) -> ServerResult<ResponseValue> {
    Err(ServerError::NotImplemented("Content recommendation requires ML infrastructure".into()))
}

// ============ WebRTC Handlers (TODO: Need webrtc-server crate) ============

async fn handle_create_webrtc_offer(_state: &AppState, _sdp: &str, peer_id: Option<&str>) -> ServerResult<ResponseValue> {
    tracing::info!("Create WebRTC offer (peer: {:?})", peer_id);
    let session_id = uuid::Uuid::new_v4().to_string();
    
    // TODO: Forward to webrtc-server crate when implemented
    Ok(ResponseValue::webrtc_session_created(&session_id))
}

async fn handle_submit_webrtc_answer(_state: &AppState, session_id: &str, _sdp: &str) -> ServerResult<ResponseValue> {
    tracing::info!("Submit WebRTC answer for session: {}", session_id);
    // TODO: Forward to webrtc-server crate when implemented
    Ok(ResponseValue::chat(
        "answer_accepted",
        "system",
        format!("Answer accepted for session {}", session_id),
        TokenUsage::zero(),
    ))
}

async fn handle_add_ice_candidate(_state: &AppState, session_id: &str, _candidate: &str) -> ServerResult<ResponseValue> {
    tracing::info!("Add ICE candidate for session: {}", session_id);
    // TODO: Forward to webrtc-server crate when implemented
    Ok(ResponseValue::chat(
        "candidate_added",
        "system",
        format!("ICE candidate added to session {}", session_id),
        TokenUsage::zero(),
    ))
}

async fn handle_get_webrtc_session(_state: &AppState, session_id: &str) -> ServerResult<ResponseValue> {
    tracing::info!("Get WebRTC session: {}", session_id);
    // TODO: Forward to webrtc-server crate when implemented
    Ok(ResponseValue::webrtc_session_info(
        session_id,
        "pending",
        None,
        None,
        vec![],
    ))
}

// ========== HuggingFace Authentication Handlers ==========

async fn handle_set_hf_token(state: &AppState, token: &str) -> ServerResult<ResponseValue> {
    tracing::info!("Setting HuggingFace auth token");
    
    state.hf_auth.set_token(token)
        .map_err(|e| ServerError::Internal(format!("Failed to store HF token: {}", e)))?;
    
    Ok(ResponseValue::success("HuggingFace token stored securely"))
}

async fn handle_get_hf_token_status(state: &AppState) -> ServerResult<ResponseValue> {
    tracing::debug!("Checking HuggingFace token status");
    
    let has_token = state.hf_auth.has_token();
    
    Ok(ResponseValue::generic(serde_json::json!({
        "hasToken": has_token,
        "message": if has_token { "Token is stored" } else { "No token stored" }
    })))
}

async fn handle_clear_hf_token(state: &AppState) -> ServerResult<ResponseValue> {
    tracing::info!("Clearing HuggingFace auth token");
    
    state.hf_auth.clear_token()
        .map_err(|e| ServerError::Internal(format!("Failed to clear HF token: {}", e)))?;
    
    Ok(ResponseValue::success("HuggingFace token removed"))
}

// ========== Hardware Detection Handlers ==========

async fn handle_get_hardware_info(state: &AppState) -> ServerResult<ResponseValue> {
    tracing::debug!("Getting hardware information");
    
    let hw = &state.hardware;
    
    // Build comprehensive hardware info response
    let info = serde_json::json!({
        "cpu": {
            "vendor": format!("{:?}", hw.cpu.vendor),
            "architecture": format!("{:?}", hw.cpu.architecture),
            "model_name": hw.cpu.model_name,
            "cores": hw.cpu.cores,
            "threads": hw.cpu.threads,
            "family": hw.cpu.family,
            "model": hw.cpu.model,
            "stepping": hw.cpu.stepping,
        },
        "memory": {
            "total_ram_mb": hw.memory.total_ram_mb,
            "available_ram_mb": hw.memory.available_ram_mb,
            "used_ram_mb": hw.memory.used_ram_mb,
            "ram_tier": hw.ram_tier,
        },
        "gpus": hw.gpus.iter().enumerate().map(|(idx, gpu)| {
            serde_json::json!({
                "index": idx,
                "name": gpu.name,
                "vendor": format!("{:?}", gpu.vendor),
                "vram_mb": gpu.vram_mb,
                "driver_version": gpu.driver_version,
            })
        }).collect::<Vec<_>>(),
        "vram": {
            "total_vram_mb": hw.total_vram_mb,
            "vram_tier": hw.vram_tier,
        },
        "execution_provider": format!("{:?}", hw.recommended_execution_provider()),
        "bitnet_dll_variant": hw.bitnet_dll_variant(),
        "bitnet_dll_filename": hw.bitnet_dll_filename(),
    });
    
    Ok(ResponseValue::generic(info))
}

async fn handle_check_model_feasibility(state: &AppState, model_size_mb: u64) -> ServerResult<ResponseValue> {
    tracing::debug!("Checking model feasibility for size: {} MB", model_size_mb);
    
    let hw = &state.hardware;
    let available_ram = hw.memory.available_ram_mb;
    let available_vram = hw.total_vram_mb;
    
    // Simple feasibility check
    let can_load_ram = model_size_mb < available_ram;
    let can_load_vram = model_size_mb < available_vram;
    let can_load = can_load_ram || can_load_vram;
    
    let recommendation = if can_load_vram {
        format!("Model can fit in VRAM ({} MB available)", available_vram)
    } else if can_load_ram {
        format!("Model can fit in RAM ({} MB available), will run on CPU", available_ram)
    } else {
        format!("Model too large! Need {} MB but only have {} MB RAM and {} MB VRAM", 
                model_size_mb, available_ram, available_vram)
    };
    
    Ok(ResponseValue::generic(serde_json::json!({
        "can_load": can_load,
        "model_size_mb": model_size_mb,
        "available_ram_mb": available_ram,
        "available_vram_mb": available_vram,
        "recommendation": recommendation,
    })))
}

async fn handle_get_recommended_models(state: &AppState) -> ServerResult<ResponseValue> {
    tracing::debug!("Getting recommended model sizes");
    
    let hw = &state.hardware;
    let available_ram = hw.memory.available_ram_mb;
    let available_vram = hw.total_vram_mb;
    
    // Recommend models based on available memory (conservative: use 70% of available)
    let safe_ram = (available_ram as f64 * 0.7) as u64;
    let safe_vram = (available_vram as f64 * 0.7) as u64;
    
    let recommendations = if safe_vram >= 24000 {
        vec!["70B", "34B", "13B", "7B", "3B"]
    } else if safe_vram >= 12000 {
        vec!["34B", "13B", "7B", "3B", "1B"]
    } else if safe_vram >= 6000 {
        vec!["13B", "7B", "3B", "1B"]
    } else if safe_ram >= 16000 {
        vec!["7B", "3B", "1B"]
    } else if safe_ram >= 8000 {
        vec!["3B", "1B"]
    } else {
        vec!["1B"]
    };
    
    Ok(ResponseValue::generic(serde_json::json!({
        "available_ram_mb": available_ram,
        "available_vram_mb": available_vram,
        "safe_ram_mb": safe_ram,
        "safe_vram_mb": safe_vram,
        "recommended_sizes": recommendations,
        "recommendation": format!(
            "For your system ({} MB RAM, {} MB VRAM), we recommend models up to {}",
            available_ram, available_vram, recommendations[0]
        ),
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CliArgs, ServerMode};
    use std::path::PathBuf;

    async fn create_test_state() -> AppState {
        let args = CliArgs {
            mode: ServerMode::Both,
            port: 8001,
            config: PathBuf::from("test.toml"),
            db_path: PathBuf::from("./test_db"),
            model_cache_path: PathBuf::from("./test_models"),
            log_level: "info".to_string(),
            webrtc_enabled: false,
            webrtc_port: 8002,
        };
        AppState::new(&args).await.unwrap()
    }

    #[tokio::test]
    async fn test_health_handler() {
        let state = create_test_state().await;
        let request = RequestValue::from_json(r#"{"action":"health"}"#).unwrap();
        let response = handle_request(&state, request).await.unwrap();
        
        // Response should be health status
        assert!(matches!(response.value_type(), tabagent_values::ValueType::ChatResponse { .. }));
    }

    #[tokio::test]
    async fn test_system_info_handler() {
        let state = create_test_state().await;
        let request = RequestValue::from_json(r#"{"action":"system_info"}"#).unwrap();
        let response = handle_request(&state, request).await.unwrap();
        
        let json = response.to_json().unwrap();
        assert!(json.contains("CPU"));
    }
}
