//! RAG (Retrieval-Augmented Generation) handlers.

use anyhow::{Context, Result};
use tabagent_values::{ResponseValue, TokenUsage};
use tabagent_model_cache::{detect_from_file_path, detect_from_repo_name, Backend};

use crate::AppState;

/// Handle RAG query request.
pub async fn handle_query(
    _state: &AppState,
    query: &str,
    top_k: Option<usize>,
) -> Result<ResponseValue> {
    tracing::info!("RAG query: {} (top_k: {:?})", query, top_k);
    
    let _k = top_k.unwrap_or(5);
    
    anyhow::bail!("RAG query requires vector DB integration (not yet implemented)")
}

/// Handle rerank request.
pub async fn handle_rerank(
    state: &AppState,
    model: &str,
    query: &str,
    documents: &[String],
    top_n: Option<usize>,
) -> Result<ResponseValue> {
    tracing::info!("Rerank {} documents with model: {}", documents.len(), model);
    
    let n = top_n.unwrap_or(documents.len().min(5));
    
    // Detect model type
    let model_info = detect_from_file_path(model)
        .or_else(|| detect_from_repo_name(model))
        .with_context(|| format!("Model '{}' not found", model))?;

    // Reranking requires cross-encoder models
    match &model_info.backend {
        Backend::Rust { engine } if engine.contains("onnx") => {
            let session = state.get_onnx_model(model)
                .with_context(|| format!("Model '{}' is not loaded", model))?;

            // Create query-document pairs for cross-encoder
            let mut scores = Vec::new();
            for (idx, doc) in documents.iter().enumerate() {
                let pair_text = format!("[CLS] {} [SEP] {} [SEP]", query, doc);
                
                let embedding = session.generate_embeddings(&[pair_text])
                    .context("Embedding generation failed")?;
                
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
                .context("Python reranking failed")?;

            Ok(ResponseValue::chat(
                "reranked",
                "system",
                format!("Python reranked {} documents", reranked.len()),
                TokenUsage::zero(),
            ))
        }
        
        _ => anyhow::bail!("{:?} does not support reranking", model_info.backend),
    }
}

/// Handle stop generation request.
pub async fn handle_stop_generation(
    state: &AppState,
    request_id: &str,
) -> Result<ResponseValue> {
    tracing::info!("Stop generation for request: {}", request_id);
    
    // Signal generation cancellation
    state.cancel_generation(request_id).await;
    
    Ok(ResponseValue::chat(
        "stopped",
        "system",
        format!("Generation {} stopped", request_id),
        TokenUsage::zero(),
    ))
}

