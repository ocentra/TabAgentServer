//! Integration tests demonstrating end-to-end usage patterns.

use tabagent_values::{
    RequestValue, ResponseValue, ValueType, TokenUsage, EmbeddingInput,
    Value, DynValueTypeMarker,
    markers::RequestValueMarker,
};

#[test]
fn test_request_response_cycle() {
    // Create a request
    let request = RequestValue::chat(
        "gpt-3.5-turbo",
        vec![("user", "Hello!")],
        Some(0.7),
    );

    // Simulate processing
    let model_id = if let ValueType::ChatRequest { model } = request.value_type() {
        model.clone()
    } else {
        panic!("Expected chat request");
    };

    // Create response
    let response = ResponseValue::chat(
        "req-123",
        &model_id,
        "Hello! How can I help you?",
        TokenUsage::new(5, 12),
    );

    // Verify response
    assert!(matches!(response.value_type(), ValueType::ChatResponse { .. }));
}

#[test]
fn test_json_round_trip() {
    // Create request
    let request = RequestValue::generate(
        "llama-2-7b",
        "Once upon a time",
        Some(0.9),
    );

    // Serialize
    let json = serde_json::to_string(request.request_type()).unwrap();

    // Deserialize
    let parsed = RequestValue::from_json(&json).unwrap();

    // Verify
    assert!(matches!(parsed.value_type(), ValueType::GenerateRequest { .. }));
}

#[test]
fn test_dynamic_dispatch() {
    // Start with concrete type
    let request: Value<RequestValueMarker> = RequestValue::chat(
        "model",
        vec![("user", "test")],
        None,
    );

    // Convert to dynamic
    let dynamic: Value<DynValueTypeMarker> = request.into_dyn();

    // Dispatch based on runtime type
    match dynamic.value_type() {
        ValueType::ChatRequest { .. } => {
            // Success - correctly identified
        }
        _ => panic!("Should have been chat request"),
    }
}

#[test]
fn test_type_safety_compile_time() {
    // This function only accepts requests
    fn process_request(_req: Value<RequestValueMarker>) {
        // Do something
    }

    // This compiles
    let req = RequestValue::chat("model", Vec::<(&str, &str)>::new(), None);
    process_request(req);

    // This would NOT compile (uncomment to verify):
    // let resp = ResponseValue::error("code", "msg");
    // process_request(resp);  // ERROR: expected RequestValueMarker, found ResponseValueMarker
}

#[test]
fn test_multiple_request_types() {
    let requests = vec![
        RequestValue::chat("gpt-4", vec![("user", "Hi")], None).into_dyn(),
        RequestValue::generate("llama-2", "Test prompt", Some(0.8)).into_dyn(),
        RequestValue::embeddings("embed-model", EmbeddingInput::Single("text".into())).into_dyn(),
        RequestValue::load_model("model-id", Some("q4_0".into())).into_dyn(),
    ];

    // Process all requests dynamically
    for (i, request) in requests.iter().enumerate() {
        match request.value_type() {
            ValueType::ChatRequest { .. } => assert_eq!(i, 0),
            ValueType::GenerateRequest { .. } => assert_eq!(i, 1),
            ValueType::EmbeddingsRequest { .. } => assert_eq!(i, 2),
            ValueType::LoadModel { .. } => assert_eq!(i, 3),
            _ => panic!("Unexpected request type"),
        }
    }
}

#[test]
fn test_error_handling_chain() {
    // Simulate error chain
    let json = r#"{"invalid":"request"}"#;
    let result = RequestValue::from_json(json);
    
    assert!(result.is_err());
    
    // Create error response
    let error = ResponseValue::error(
        "invalid_json",
        &format!("Failed to parse request: {}", result.unwrap_err()),
    );
    
    // Verify error response serializes correctly
    let json = error.to_json().unwrap();
    assert!(json.contains("invalid_json"));
}

#[test]
fn test_batch_processing() {
    // Simulate batch of requests
    let batch = vec![
        r#"{"action":"chat","model":"gpt-4","messages":[{"role":"user","content":"1"}]}"#,
        r#"{"action":"generate","model":"llama-2","prompt":"2"}"#,
        r#"{"action":"embeddings","model":"embed","input":"3"}"#,
    ];

    let parsed: Vec<_> = batch
        .iter()
        .filter_map(|json| RequestValue::from_json(json).ok())
        .collect();

    assert_eq!(parsed.len(), 3);
}

#[test]
fn test_response_serialization_consistency() {
    let response = ResponseValue::chat(
        "id",
        "model",
        "response text",
        TokenUsage::new(10, 20),
    );

    // Serialize multiple times
    let json1 = response.to_json().unwrap();
    let json2 = response.to_json().unwrap();

    // Should be identical
    assert_eq!(json1, json2);
}

#[test]
fn test_concurrent_value_creation() {
    use std::sync::Arc;
    use std::thread;

    let results = Arc::new(std::sync::Mutex::new(Vec::new()));

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let results = Arc::clone(&results);
            thread::spawn(move || {
                let req = RequestValue::chat(
                    &format!("model-{}", i),
                    vec![("user", &format!("message-{}", i))],
                    Some(0.5 + i as f32 * 0.01),
                );
                results.lock().unwrap().push(req);
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(results.lock().unwrap().len(), 10);
}

#[test]
fn test_downcast_chain() {
    // Start with specific type
    let request = RequestValue::chat("model", vec![("user", "test")], None);

    // Up to dynamic
    let dynamic = request.into_dyn();

    // Down to request
    let as_request: Value<RequestValueMarker> = dynamic.downcast().unwrap();

    // Verify
    assert!(matches!(as_request.value_type(), ValueType::ChatRequest { .. }));
}

#[test]
fn test_large_message_handling() {
    // Test with large message content
    let large_content = "x".repeat(10_000);
    
    let request = RequestValue::chat(
        "gpt-4",
        vec![("user", large_content.as_str())],
        None,
    );

    // Should handle serialization
    let json = serde_json::to_string(request.request_type()).unwrap();
    assert!(json.len() > 10_000);

    // Should be able to parse back
    let parsed = RequestValue::from_json(&json).unwrap();
    assert!(matches!(parsed.value_type(), ValueType::ChatRequest { .. }));
}

#[test]
fn test_special_characters_handling() {
    let special_chars = "Test with: \"quotes\", 'apostrophes', \n newlines, \t tabs, and émojis: 🚀";
    
    let request = RequestValue::chat(
        "model",
        vec![("user", special_chars)],
        None,
    );

    let json = serde_json::to_string(request.request_type()).unwrap();
    let parsed = RequestValue::from_json(&json).unwrap();

    assert!(matches!(parsed.value_type(), ValueType::ChatRequest { .. }));
}

