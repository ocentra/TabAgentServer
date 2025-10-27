//! Comprehensive tests for request value creation and manipulation.

use tabagent_values::{
    RequestValue, RequestType, ValueType, EmbeddingInput, MessageRole,
    Value, DynValueTypeMarker,
    markers::RequestValueMarker,
};

#[test]
fn test_chat_request_creation() {
    let req = RequestValue::chat(
        "gpt-3.5-turbo",
        vec![("user", "Hello!")],
        Some(0.7),
    );

    assert!(matches!(
        req.value_type(),
        ValueType::ChatRequest { model } if model == "gpt-3.5-turbo"
    ));
}

#[test]
fn test_chat_request_multiple_messages() {
    let req = RequestValue::chat(
        "gpt-4",
        vec![
            ("system", "You are a helpful assistant"),
            ("user", "Hello!"),
            ("assistant", "Hi! How can I help?"),
            ("user", "What's the weather?"),
        ],
        Some(0.8),
    );

    assert!(matches!(
        req.value_type(),
        ValueType::ChatRequest { model } if model == "gpt-4"
    ));
}

#[test]
fn test_generate_request_creation() {
    let req = RequestValue::generate(
        "llama-2-7b",
        "Once upon a time",
        Some(0.9),
    );

    assert!(matches!(
        req.value_type(),
        ValueType::GenerateRequest { model } if model == "llama-2-7b"
    ));
}

#[test]
fn test_embeddings_request_single() {
    let req = RequestValue::embeddings(
        "sentence-transformers/all-MiniLM-L6-v2",
        EmbeddingInput::Single("Embed this text".to_string()),
    );

    assert!(matches!(
        req.value_type(),
        ValueType::EmbeddingsRequest { model } if model == "sentence-transformers/all-MiniLM-L6-v2"
    ));
}

#[test]
fn test_embeddings_request_multiple() {
    let req = RequestValue::embeddings(
        "text-embedding-ada-002",
        EmbeddingInput::Multiple(vec![
            "First text".to_string(),
            "Second text".to_string(),
            "Third text".to_string(),
        ]),
    );

    assert!(matches!(
        req.value_type(),
        ValueType::EmbeddingsRequest { .. }
    ));
}

#[test]
fn test_load_model_request() {
    let req = RequestValue::load_model(
        "Qwen/Qwen2.5-1.5B-Instruct-GGUF",
        Some("q4_0".to_string()),
    );

    assert!(matches!(
        req.value_type(),
        ValueType::LoadModel { model_id, variant }
        if model_id == "Qwen/Qwen2.5-1.5B-Instruct-GGUF" && variant.as_deref() == Some("q4_0")
    ));
}

#[test]
fn test_load_model_request_no_variant() {
    let req = RequestValue::load_model(
        "gpt2",
        None,
    );

    assert!(matches!(
        req.value_type(),
        ValueType::LoadModel { model_id, variant }
        if model_id == "gpt2" && variant.is_none()
    ));
}

#[test]
fn test_request_from_json_chat() {
    let json = r#"{"action":"chat","model":"gpt-4","messages":[{"role":"user","content":"Hi"}]}"#;
    let req = RequestValue::from_json(json).unwrap();

    assert!(matches!(req.value_type(), ValueType::ChatRequest { .. }));
}

#[test]
fn test_request_from_json_generate() {
    let json = r#"{"action":"generate","model":"llama-2","prompt":"Hello"}"#;
    let req = RequestValue::from_json(json).unwrap();

    assert!(matches!(req.value_type(), ValueType::GenerateRequest { .. }));
}

#[test]
fn test_request_from_json_load_model() {
    let json = r#"{"action":"load_model","model_id":"test-model","variant":"q4_0","force_reload":false}"#;
    let req = RequestValue::from_json(json).unwrap();

    assert!(matches!(req.value_type(), ValueType::LoadModel { .. }));
}

#[test]
fn test_request_from_json_invalid() {
    let json = r#"{"invalid":"json"}"#;
    let result = RequestValue::from_json(json);

    assert!(result.is_err());
}

#[test]
fn test_downcast_to_specific() {
    let req = RequestValue::chat("model", vec![("user", "test")], None);
    let dyn_req: Value<DynValueTypeMarker> = req.into_dyn();
    
    // Should be able to downcast back
    let specific: Value<RequestValueMarker> = dyn_req.downcast().unwrap();
    assert!(matches!(specific.value_type(), ValueType::ChatRequest { .. }));
}

#[test]
fn test_downcast_wrong_type() {
    use tabagent_values::markers::ResponseValueMarker;
    
    let req = RequestValue::chat("model", vec![("user", "test")], None);
    let dyn_req: Value<DynValueTypeMarker> = req.into_dyn();
    
    // Should fail to downcast to response
    let result: Result<Value<ResponseValueMarker>, _> = dyn_req.downcast();
    assert!(result.is_err());
}

#[test]
fn test_message_role_serialization() {
    let roles = vec![
        (MessageRole::System, "system"),
        (MessageRole::User, "user"),
        (MessageRole::Assistant, "assistant"),
        (MessageRole::Function, "function"),
    ];

    for (role, expected) in roles {
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, format!("\"{}\"", expected));
    }
}

#[test]
fn test_embedding_input_serialization() {
    // Single input
    let single = EmbeddingInput::Single("test".to_string());
    let json = serde_json::to_string(&single).unwrap();
    assert_eq!(json, "\"test\"");

    // Multiple input
    let multiple = EmbeddingInput::Multiple(vec!["a".to_string(), "b".to_string()]);
    let json = serde_json::to_string(&multiple).unwrap();
    assert_eq!(json, "[\"a\",\"b\"]");
}

#[test]
fn test_request_type_access() {
    use tabagent_values::RequestType;
    
    let req = RequestValue::chat("model", vec![("user", "test")], Some(0.7));
    
    match req.request_type() {
        RequestType::Chat { model, messages, temperature, .. } => {
            assert_eq!(model, "model");
            assert_eq!(messages.len(), 1);
            assert_eq!(*temperature, Some(0.7));
        }
        _ => panic!("Expected Chat request"),
    }
}

#[test]
fn test_empty_messages() {
    let req = RequestValue::chat("model", Vec::<(&str, &str)>::new(), None);
    
    match req.request_type() {
        RequestType::Chat { messages, .. } => {
            assert_eq!(messages.len(), 0);
        }
        _ => panic!("Expected Chat request"),
    }
}

#[test]
fn test_temperature_bounds() {
    // Test various temperature values (validation would be in a future PR)
    let temps = vec![0.0, 0.5, 1.0, 1.5, 2.0];
    
    for temp in temps {
        let req = RequestValue::chat("model", vec![("user", "test")], Some(temp));
        assert!(matches!(req.value_type(), ValueType::ChatRequest { .. }));
    }
}

