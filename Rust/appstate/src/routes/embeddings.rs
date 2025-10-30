//! Embeddings generation handler.

use anyhow::{Context, Result};
use tabagent_values::{ResponseValue, EmbeddingInput};
use tabagent_model_cache::{detect_from_file_path, detect_from_repo_name, Backend};

use crate::AppState;

/// Handle embeddings generation request.
pub async fn handle(
    state: &AppState,
    model: &str,
    input: &EmbeddingInput,
) -> Result<ResponseValue> {
    tracing::info!("Embeddings request for model: {}", model);

    // Extract texts from input
    let texts: Vec<String> = match input {
        EmbeddingInput::Single(text) => vec![text.clone()],
        EmbeddingInput::Multiple(texts) => texts.clone(),
    };

    // Detect model type
    let model_info = detect_from_file_path(model)
        .or_else(|| detect_from_repo_name(model))
        .with_context(|| format!("Model '{}' not found", model))?;

    // Route based on backend
    match &model_info.backend {
        Backend::Rust { engine } if engine.contains("onnx") => {
            let session = state.get_onnx_model(model)
                .with_context(|| format!("Model '{}' is not loaded", model))?;

            let embeddings = session.generate_embeddings(&texts)
                .context("ONNX embedding generation failed")?;

            Ok(ResponseValue::embeddings(embeddings))
        }
        
        Backend::Python { engine } if engine.contains("transformers") => {
            // Forward to Python ML client for transformers embeddings via gRPC
            use common::grpc::ml::GenerateEmbeddingsRequest;
            
            let request = GenerateEmbeddingsRequest {
                texts: texts.clone(),
                model: model.to_string(),
            };
            
            let response = state.ml_client.generate_embeddings(request).await
                .context("Python embedding generation failed")?;
            
            // Convert from gRPC response to Vec<Vec<f32>>
            let embeddings: Vec<Vec<f32>> = response.embeddings
                .into_iter()
                .map(|emb| emb.values)
                .collect();

            Ok(ResponseValue::embeddings(embeddings))
        }
        
        _ => anyhow::bail!("{:?} does not support embeddings", model_info.backend),
    }
}

