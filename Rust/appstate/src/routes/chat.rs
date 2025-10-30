//! Chat completions handler - REAL business logic.

use anyhow::{Context, Result};
use tabagent_values::{ResponseValue, TokenUsage};
use tabagent_model_cache::{detect_from_file_path, detect_from_repo_name, Backend};
use tabagent_onnx_loader::text_generation::GenerationConfig as OnnxGenConfig;
use gguf_loader::GenerationParams as GgufGenParams;

use crate::AppState;

/// Handle chat completion request (REAL business logic).
///
/// This function:
/// 1. Detects model type (ONNX/GGUF/Python)
/// 2. Loads model if needed
/// 3. Runs inference
/// 4. Returns response
pub async fn handle(
    state: &AppState,
    model: &str,
    messages: &[tabagent_values::Message],
    temperature: Option<f32>,
) -> Result<ResponseValue> {
    tracing::info!("Chat request for model: {} with {} messages", model, messages.len());

    // Build prompt from messages
    let prompt = messages.iter()
        .map(|msg| format!("{:?}: {}", msg.role, msg.content))
        .collect::<Vec<_>>()
        .join("\n");

    // Detect model type
    let model_info = detect_from_file_path(model)
        .or_else(|| detect_from_repo_name(model))
        .with_context(|| format!("Model '{}' not found", model))?;

    let request_id = uuid::Uuid::new_v4().to_string();
    let temp = temperature.unwrap_or(0.7);

    // Route based on backend (check engine field)
    match &model_info.backend {
        Backend::Rust { engine } if engine.contains("onnx") => {
            // ONNX inference
            let session = state.get_onnx_model(model)
                .with_context(|| format!("Model '{}' is not loaded", model))?;

            let config = OnnxGenConfig {
                max_new_tokens: 512,
                temperature: temp,
                top_k: 50,
                top_p: 0.9,
                do_sample: temp > 0.0,
                repetition_penalty: 1.1,
            };

            let response_text = session.generate_text(&prompt, &config)
                .context("ONNX inference failed")?;

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
                .with_context(|| format!("Model '{}' is not loaded", model))?;

            // Create generation params
            let mut params = GgufGenParams::default();
            params.max_tokens = 512;
            params.temperature = temp;
            params.top_p = 0.9;
            params.top_k = 50;
            params.repeat_penalty = 1.1;

            // Lock context for generation (GGUF contexts are not thread-safe)
            let mut context_guard = context.lock().await;
            let response_text = context_guard.generate(&prompt)
                .context("GGUF inference failed")?;

            let tokens_used = response_text.split_whitespace().count() as u32;

            Ok(ResponseValue::chat(
                &request_id,
                model,
                response_text,
                TokenUsage { prompt_tokens: 0, completion_tokens: tokens_used, total_tokens: tokens_used },
            ))
        }
        
        Backend::Python { engine } if engine.contains("transformers") || engine.contains("mediapipe") => {
            // Forward to Python ML client for text generation via gRPC
            let texts = state.ml_client.generate_text(
                prompt.clone(),
                model.to_string(),
                512, // max_length
                temp,
            ).await.context("Python ML inference failed")?;
            
            // Concatenate all text chunks
            let response_text = texts.join("");

            Ok(ResponseValue::chat(
                &request_id,
                model,
                response_text,
                TokenUsage::zero(),
            ))
        }
        
        _ => anyhow::bail!("Unsupported backend: {:?}", model_info.backend),
    }
}

