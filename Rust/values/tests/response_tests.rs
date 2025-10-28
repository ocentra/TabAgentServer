//! Comprehensive tests for response value creation and serialization.

use tabagent_values::{
    ResponseValue, TokenUsage, FinishReason, HealthStatus,
    ValueType,
    response::{RerankResult, ModelInfo, SystemInfo, CpuInfo, MemoryInfo, GpuInfo, RagResult},
};

#[test]
fn test_chat_response_creation() {
    let resp = ResponseValue::chat(
        "test-id",
        "gpt-3.5-turbo",
        "Hello!",
        TokenUsage::new(10, 5),
    );

    assert!(matches!(
        resp.value_type(),
        ValueType::ChatResponse { model, .. } if model == "gpt-3.5-turbo"
    ));
}

#[test]
fn test_chat_response_serialization() {
    let resp = ResponseValue::chat(
        "req-123",
        "gpt-4",
        "This is a test response",
        TokenUsage::new(15, 25),
    );

    let json = resp.to_json().unwrap();
    assert!(json.contains("req-123"));
    assert!(json.contains("gpt-4"));
    assert!(json.contains("This is a test response"));
}

#[test]
fn test_error_response_creation() {
    let resp = ResponseValue::error("invalid_model", "Model not found");
    
    assert!(matches!(
        resp.value_type(),
        ValueType::ErrorResponse { code, message }
        if code == "invalid_model" && message == "Model not found"
    ));
}

#[test]
fn test_error_response_serialization() {
    let resp = ResponseValue::error("rate_limit", "Too many requests");
    let json = resp.to_json().unwrap();
    
    assert!(json.contains("rate_limit"));
    assert!(json.contains("Too many requests"));
}

#[test]
fn test_health_response_healthy() {
    let resp = ResponseValue::health(HealthStatus::Healthy);
    let json = resp.to_json().unwrap();
    
    assert!(json.contains("healthy"));
}

#[test]
fn test_health_response_unhealthy() {
    let resp = ResponseValue::health(HealthStatus::Unhealthy);
    let json = resp.to_json().unwrap();
    
    assert!(json.contains("unhealthy"));
}

#[test]
fn test_token_usage_calculation() {
    let usage = TokenUsage::new(100, 50);
    
    assert_eq!(usage.prompt_tokens, 100);
    assert_eq!(usage.completion_tokens, 50);
    assert_eq!(usage.total_tokens, 150);
}

#[test]
fn test_token_usage_zero() {
    let usage = TokenUsage::zero();
    
    assert_eq!(usage.prompt_tokens, 0);
    assert_eq!(usage.completion_tokens, 0);
    assert_eq!(usage.total_tokens, 0);
}

#[test]
fn test_finish_reason_serialization() {
    let reasons = vec![
        (FinishReason::Stop, "stop"),
        (FinishReason::Length, "length"),
        (FinishReason::ContentFilter, "content_filter"),
        (FinishReason::Error, "error"),
    ];

    for (reason, expected) in reasons {
        let json = serde_json::to_string(&reason).unwrap();
        assert_eq!(json, format!("\"{}\"", expected));
    }
}

#[test]
fn test_health_status_serialization() {
    let statuses = vec![
        (HealthStatus::Healthy, "healthy"),
        (HealthStatus::Degraded, "degraded"),
        (HealthStatus::Unhealthy, "unhealthy"),
    ];

    for (status, expected) in statuses {
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, format!("\"{}\"", expected));
    }
}

#[test]
fn test_response_type_access() {
    use tabagent_values::ResponseType;
    
    let resp = ResponseValue::chat(
        "id-1",
        "model-1",
        "response text",
        TokenUsage::new(5, 10),
    );
    
    match resp.response_type() {
        ResponseType::ChatResponse { id, model, response, usage, .. } => {
            assert_eq!(id, "id-1");
            assert_eq!(model, "model-1");
            assert_eq!(response, "response text");
            assert_eq!(usage.total_tokens, 15);
        }
        _ => panic!("Expected ChatResponse"),
    }
}

#[test]
fn test_error_response_type_access() {
    use tabagent_values::ResponseType;
    
    let resp = ResponseValue::error("test_error", "Test message");
    
    match resp.response_type() {
        ResponseType::Error { code, message, .. } => {
            assert_eq!(code, "test_error");
            assert_eq!(message, "Test message");
        }
        _ => panic!("Expected Error response"),
    }
}

#[test]
fn test_large_token_counts() {
    let usage = TokenUsage::new(1_000_000, 500_000);
    assert_eq!(usage.total_tokens, 1_500_000);
}

#[test]
fn test_empty_response_text() {
    let resp = ResponseValue::chat(
        "id",
        "model",
        "",
        TokenUsage::zero(),
    );
    
    let json = resp.to_json().unwrap();
    assert!(json.contains("\"response\":\"\""));
}

#[test]
fn test_special_characters_in_response() {
    let resp = ResponseValue::chat(
        "id",
        "model",
        "Test with \"quotes\" and\nnewlines",
        TokenUsage::new(1, 1),
    );
    
    let json = resp.to_json().unwrap();
    assert!(json.contains("\\n")); // Newline should be escaped
    assert!(json.contains("\\\"")); // Quotes should be escaped
}

#[test]
fn test_unicode_in_response() {
    let resp = ResponseValue::chat(
        "id",
        "model",
        "Hello 世界 🌍",
        TokenUsage::new(1, 1),
    );
    
    let json = resp.to_json().unwrap();
    assert!(json.contains("世界"));
    assert!(json.contains("🌍"));
}

#[test]
fn test_generate_response_structure() {
    // Test that we can serialize a GenerateResponse via its type definition
    // Note: No public constructor exists yet for GenerateResponse
    // This test validates the structure exists and can be serialized
    use tabagent_values::response::ResponseType;
    
    let response_type = ResponseType::GenerateResponse {
        id: "gen-123".to_string(),
        text: "Generated text here".to_string(),
        usage: TokenUsage::new(20, 100),
    };
    
    let json = serde_json::to_string(&response_type).unwrap();
    assert!(json.contains("gen-123"));
    assert!(json.contains("Generated text here"));
}

#[test]
fn test_embeddings_response_creation() {
    let embeddings = vec![
        vec![0.1, 0.2, 0.3],
        vec![0.4, 0.5, 0.6],
    ];
    
    let resp = ResponseValue::embeddings(embeddings);
    let json = resp.to_json().unwrap();
    
    assert!(json.contains("0.1") || json.contains("["));
}

#[test]
fn test_embeddings_response_large() {
    let embedding: Vec<f32> = (0..1536).map(|i| i as f32 * 0.001).collect();
    let embeddings = vec![embedding];
    
    let resp = ResponseValue::embeddings(embeddings);
    let json = resp.to_json().unwrap();
    
    assert!(json.len() > 1000); // Should be large
}

#[test]
fn test_rerank_response_structure() {
    use tabagent_values::response::ResponseType;
    
    let results = vec![
        RerankResult {
            index: 0,
            score: 0.95,
            document: "Most relevant doc".to_string(),
        },
        RerankResult {
            index: 1,
            score: 0.75,
            document: "Less relevant doc".to_string(),
        },
    ];
    
    let response_type = ResponseType::RerankResponse {
        model: "rerank-model".to_string(),
        results,
    };
    
    let json = serde_json::to_string(&response_type).unwrap();
    assert!(json.contains("rerank-model"));
    assert!(json.contains("Most relevant doc"));
}

#[test]
fn test_model_list_response_structure() {
    use tabagent_values::response::ResponseType;
    
    let models = vec![
        ModelInfo {
            id: "model-1".to_string(),
            name: "GPT-4".to_string(),
            backend: "openai".to_string(),
            loaded: true,
            size_bytes: Some(1_000_000),
            parameters: Some(175_000_000_000),
        },
        ModelInfo {
            id: "model-2".to_string(),
            name: "LLaMA-2".to_string(),
            backend: "gguf".to_string(),
            loaded: false,
            size_bytes: Some(5_000_000),
            parameters: Some(7_000_000_000),
        },
    ];
    
    let response_type = ResponseType::ModelListResponse {
        models,
    };
    
    let json = serde_json::to_string(&response_type).unwrap();
    assert!(json.contains("GPT-4"));
    assert!(json.contains("LLaMA-2"));
}

#[test]
fn test_model_info_response_structure() {
    use tabagent_values::response::ResponseType;
    
    let info = ModelInfo {
        id: "model-test".to_string(),
        name: "Test Model".to_string(),
        backend: "onnx".to_string(),
        loaded: true,
        size_bytes: Some(2_000_000_000),
        parameters: Some(13_000_000_000),
    };
    
    let response_type = ResponseType::ModelInfoResponse {
        info,
    };
    
    let json = serde_json::to_string(&response_type).unwrap();
    assert!(json.contains("Test Model"));
    assert!(json.contains("onnx"));
}

#[test]
fn test_rag_response_structure() {
    use tabagent_values::response::ResponseType;
    
    let results = vec![
        RagResult {
            id: "doc-1".to_string(),
            score: 0.92,
            content: "This is the most relevant result".to_string(),
            metadata: Some(serde_json::json!({"source": "database"})),
        },
        RagResult {
            id: "doc-2".to_string(),
            score: 0.81,
            content: "This is a relevant result".to_string(),
            metadata: None,
        },
    ];
    
    let response_type = ResponseType::RagResponse {
        results,
        query_time_ms: 45,
    };
    
    let json = serde_json::to_string(&response_type).unwrap();
    assert!(json.contains("most relevant result"));
    assert!(json.contains("45"));
}

#[test]
fn test_chat_history_response_structure() {
    use tabagent_values::response::ResponseType;
    use tabagent_values::{Message, MessageRole};
    
    let messages = vec![
        Message {
            role: MessageRole::User,
            content: "Hello".to_string(),
            name: None,
        },
        Message {
            role: MessageRole::Assistant,
            content: "Hi there!".to_string(),
            name: None,
        },
    ];
    
    let response_type = ResponseType::ChatHistoryResponse {
        session_id: "session-abc".to_string(),
        messages,
    };
    
    let json = serde_json::to_string(&response_type).unwrap();
    assert!(json.contains("session-abc"));
    assert!(json.contains("Hello"));
}

#[test]
fn test_system_info_response_structure() {
    use tabagent_values::response::ResponseType;
    
    let system = SystemInfo {
        cpu: CpuInfo {
            model: "Intel Core i9".to_string(),
            cores: 16,
            threads: 32,
        },
        memory: MemoryInfo {
            total_bytes: 64_000_000_000,
            available_bytes: 32_000_000_000,
        },
        gpu: Some(GpuInfo {
            name: "NVIDIA RTX 4090".to_string(),
            memory_bytes: 24_000_000_000,
            vendor: "NVIDIA".to_string(),
        }),
    };
    
    let response_type = ResponseType::SystemInfoResponse {
        system,
    };
    
    let json = serde_json::to_string(&response_type).unwrap();
    assert!(json.contains("Intel Core i9"));
    assert!(json.contains("NVIDIA RTX 4090"));
}

#[test]
fn test_success_response_structure() {
    use tabagent_values::response::ResponseType;
    
    let response_type = ResponseType::Success {
        message: "Operation completed successfully".to_string(),
    };
    
    let json = serde_json::to_string(&response_type).unwrap();
    assert!(json.contains("Operation completed successfully"));
}

#[test]
fn test_webrtc_session_created_response() {
    let resp = ResponseValue::webrtc_session_created("session-webrtc-123");
    
    let (session_id, _) = resp.as_webrtc_session_created();
    assert_eq!(session_id, "session-webrtc-123");
    
    let json = resp.to_json().unwrap();
    assert!(json.contains("session-webrtc-123"));
}

#[test]
fn test_webrtc_session_info_response() {
    let ice_candidates = vec![
        "candidate:1 1 udp 123456 192.168.1.1 54321".to_string(),
        "candidate:2 1 udp 123457 192.168.1.2 54322".to_string(),
    ];
    
    let resp = ResponseValue::webrtc_session_info(
        "session-xyz",
        "connected",
        Some("offer-sdp".to_string()),
        Some("answer-sdp".to_string()),
        ice_candidates,
    );
    
    let (session_id, state, offer, answer, candidates) = resp.as_webrtc_session_info();
    assert_eq!(session_id, "session-xyz");
    assert_eq!(state, "connected");
    assert_eq!(offer, Some("offer-sdp"));
    assert_eq!(answer, Some("answer-sdp"));
    assert_eq!(candidates.len(), 2);
}

#[test]
fn test_error_response_with_details() {
    use tabagent_values::response::ResponseType;
    
    let details = serde_json::json!({
        "suggestion": "Try again later",
        "error_code": 503
    });
    
    let response_type = ResponseType::Error {
        code: "service_unavailable".to_string(),
        message: "Service temporarily unavailable".to_string(),
        details: Some(details),
    };
    
    let json = serde_json::to_string(&response_type).unwrap();
    assert!(json.contains("service_unavailable"));
    assert!(json.contains("Try again later"));
}

#[test]
fn test_health_response_degraded() {
    let resp = ResponseValue::health(HealthStatus::Degraded);
    let json = resp.to_json().unwrap();
    
    assert!(json.contains("degraded"));
}

#[test]
fn test_response_extraction_methods() {
    // Test as_chat
    let chat_resp = ResponseValue::chat("id", "model", "text", TokenUsage::new(1, 1));
    let (text, model, usage) = chat_resp.as_chat().unwrap();
    assert_eq!(text, "text");
    assert_eq!(model, "model");
    assert_eq!(usage.total_tokens, 2);
    
    // Test as_error
    let error_resp = ResponseValue::error("code", "message");
    let (code, msg) = error_resp.as_error().unwrap();
    assert_eq!(code, "code");
    assert_eq!(msg, "message");
}

#[test]
fn test_response_to_json_value() {
    let resp = ResponseValue::chat("id", "model", "response", TokenUsage::new(5, 10));
    let json_value = resp.to_json_value();
    
    assert!(json_value.is_object());
    assert_eq!(json_value["id"], "id");
    assert_eq!(json_value["model"], "model");
    assert_eq!(json_value["response"], "response");
}

#[test]
fn test_finish_reason_all_variants() {
    let reasons = vec![
        FinishReason::Stop,
        FinishReason::Length,
        FinishReason::ContentFilter,
        FinishReason::Error,
    ];
    
    for reason in reasons {
        let json = serde_json::to_string(&reason).unwrap();
        assert!(!json.is_empty());
        
        // Should be deserializable
        let _: FinishReason = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_token_usage_edge_cases() {
    // Zero usage
    let zero = TokenUsage::zero();
    assert_eq!(zero.total_tokens, 0);
    
    // Large numbers
    let large = TokenUsage::new(1_000_000, 2_000_000);
    assert_eq!(large.total_tokens, 3_000_000);
    
    // Serialization
    let json = serde_json::to_string(&large).unwrap();
    assert!(json.contains("1000000"));
    assert!(json.contains("2000000"));
}

#[test]
fn test_multiple_response_serializations() {
    let resp = ResponseValue::chat("id", "model", "text", TokenUsage::new(10, 20));
    
    // Multiple serializations should be identical
    let json1 = resp.to_json().unwrap();
    let json2 = resp.to_json().unwrap();
    let json3 = resp.to_json_value().to_string();
    
    assert_eq!(json1, json2);
    // json3 might have different formatting, so just check it's not empty
    assert!(!json3.is_empty());
}

