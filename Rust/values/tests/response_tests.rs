//! Comprehensive tests for response value creation and serialization.

use tabagent_values::{
    ResponseValue, TokenUsage, FinishReason, HealthStatus,
    ValueType,
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

