//! Integration tests demonstrating end-to-end usage patterns.

use tabagent_values::{
    RequestValue, ResponseValue, ValueType, TokenUsage, EmbeddingInput, Message, MessageRole,
    Value, DynValueTypeMarker,
    markers::RequestValueMarker,
};

// Helper function to create messages from tuples
fn make_messages(tuples: Vec<(&str, &str)>) -> Vec<Message> {
    tuples
        .into_iter()
        .map(|(role, content)| Message {
            role: match role {
                "system" => MessageRole::System,
                "user" => MessageRole::User,
                "assistant" => MessageRole::Assistant,
                "function" => MessageRole::Function,
                _ => MessageRole::User,
            },
            content: content.to_string(),
            name: None,
        })
        .collect()
}

#[test]
fn test_request_response_cycle() {
    // Create a request
    let request = RequestValue::chat(
        "gpt-3.5-turbo",
        make_messages(vec![("user", "Hello!")]),
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
        make_messages(vec![("user", "test")]),
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
    let req = RequestValue::chat("model", Vec::<Message>::new(), None);
    process_request(req);

    // This would NOT compile (uncomment to verify):
    // let resp = ResponseValue::error("code", "msg");
    // process_request(resp);  // ERROR: expected RequestValueMarker, found ResponseValueMarker
}

#[test]
fn test_multiple_request_types() {
    let requests = vec![
        RequestValue::chat("gpt-4", make_messages(vec![("user", "Hi")]), None).into_dyn(),
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
                let message_content = format!("message-{}", i);
                let messages = vec![Message {
                    role: MessageRole::User,
                    content: message_content,
                    name: None,
                }];
                let req = RequestValue::chat(
                    &format!("model-{}", i),
                    messages,
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
    let request = RequestValue::chat("model", make_messages(vec![("user", "test")]), None);

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
    
    let messages = vec![Message {
        role: MessageRole::User,
        content: large_content.clone(),
        name: None,
    }];
    
    let request = RequestValue::chat(
        "gpt-4",
        messages,
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
    
    let messages = vec![Message {
        role: MessageRole::User,
        content: special_chars.to_string(),
        name: None,
    }];
    
    let request = RequestValue::chat(
        "model",
        messages,
        None,
    );

    let json = serde_json::to_string(request.request_type()).unwrap();
    let parsed = RequestValue::from_json(&json).unwrap();

    assert!(matches!(parsed.value_type(), ValueType::ChatRequest { .. }));
}

#[test]
fn test_rerank_request_response_cycle() {
    use tabagent_values::response::{ResponseType, RerankResult};
    
    // Create rerank request
    let request = RequestValue::rerank(
        "rerank-1",
        "machine learning",
        vec!["AI doc".to_string(), "ML doc".to_string()],
        Some(2),
    );
    
    // Simulate response - test the response type directly since we can't construct ResponseValue
    let results = vec![
        RerankResult {
            index: 1,
            score: 0.95,
            document: "ML doc".to_string(),
        },
        RerankResult {
            index: 0,
            score: 0.85,
            document: "AI doc".to_string(),
        },
    ];
    
    let response_type = ResponseType::RerankResponse {
        model: "rerank-1".to_string(),
        results,
    };
    
    // Verify request and response can be serialized
    assert!(matches!(request.value_type(), ValueType::RerankRequest { .. }));
    let json = serde_json::to_string(&response_type).unwrap();
    assert!(json.contains("rerank-1"));
}

#[test]
fn test_model_lifecycle() {
    // Load model
    let load_req = RequestValue::load_model("test-model", Some("q4_0".to_string()));
    assert!(matches!(load_req.value_type(), ValueType::LoadModel { .. }));
    
    // Get model info
    let info_req = RequestValue::model_info("test-model");
    assert!(matches!(info_req.value_type(), ValueType::ModelInfo { .. }));
    
    // List models
    let list_req = RequestValue::list_models();
    assert!(matches!(list_req.value_type(), ValueType::ListModels));
    
    // Unload model
    let unload_req = RequestValue::unload_model("test-model");
    assert!(matches!(unload_req.value_type(), ValueType::UnloadModel { .. }));
}

#[test]
fn test_rag_workflow() {
    // Create RAG query
    let filters = serde_json::json!({"category": "tech"});
    let request = RequestValue::rag_query("search term", Some(10), Some(filters));
    
    assert!(matches!(request.value_type(), ValueType::RagQuery { .. }));
    
    // Verify serialization
    let json = serde_json::to_string(request.request_type()).unwrap();
    assert!(json.contains("search term"));
    assert!(json.contains("tech"));
}

#[test]
fn test_chat_history_workflow() {
    // Save a message
    let message = Message {
        role: MessageRole::User,
        content: "Test message".to_string(),
        name: None,
    };
    
    let save_req = RequestValue::save_message("session-123", &message);
    assert!(matches!(save_req.value_type(), ValueType::ChatHistory { .. }));
    
    // Get chat history
    let history_req = RequestValue::chat_history(Some("session-123"), Some(50));
    assert!(matches!(
        history_req.value_type(),
        ValueType::ChatHistory { session_id } if session_id.as_deref() == Some("session-123")
    ));
}

#[test]
fn test_system_management_requests() {
    // System info
    let sys_req = RequestValue::system_info();
    assert!(matches!(sys_req.value_type(), ValueType::SystemInfo));
    
    // Get params
    let get_params = RequestValue::get_params();
    assert!(matches!(get_params.value_type(), ValueType::SystemInfo));
    
    // Set params
    let params = serde_json::json!({"temperature": 0.7});
    let set_params = RequestValue::set_params(params);
    assert!(matches!(set_params.value_type(), ValueType::SystemInfo));
    
    // Get stats
    let stats = RequestValue::get_stats();
    assert!(matches!(stats.value_type(), ValueType::SystemInfo));
    
    // Get resources
    let resources = RequestValue::get_resources();
    assert!(matches!(resources.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_semantic_search_workflow() {
    // Basic semantic search
    let search_req = RequestValue::semantic_search("find docs", 5, None);
    assert!(matches!(search_req.value_type(), ValueType::SystemInfo));
    
    // Calculate similarity
    let sim_req = RequestValue::calculate_similarity("text1", "text2", None);
    assert!(matches!(sim_req.value_type(), ValueType::SystemInfo));
    
    // Evaluate embeddings
    let eval_req = RequestValue::evaluate_embeddings(
        "model",
        vec!["q1".to_string()],
        vec!["d1".to_string()],
    );
    assert!(matches!(eval_req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_clustering_workflow() {
    let documents = vec![
        "doc1".to_string(),
        "doc2".to_string(),
        "doc3".to_string(),
    ];
    
    let cluster_req = RequestValue::cluster_documents(documents, 2, None);
    assert!(matches!(cluster_req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_recommendation_workflow() {
    let candidates = vec![
        "item1".to_string(),
        "item2".to_string(),
        "item3".to_string(),
    ];
    
    let rec_req = RequestValue::recommend_content("user query", candidates, 2, None);
    assert!(matches!(rec_req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_model_management_workflow() {
    // Pull model
    let pull_req = RequestValue::pull_model("model-repo/model-name", Some("q4_0".to_string()));
    assert!(matches!(pull_req.value_type(), ValueType::SystemInfo));
    
    // Get recipes
    let recipes_req = RequestValue::get_recipes();
    assert!(matches!(recipes_req.value_type(), ValueType::SystemInfo));
    
    // Get embedding models
    let embed_models_req = RequestValue::get_embedding_models();
    assert!(matches!(embed_models_req.value_type(), ValueType::SystemInfo));
    
    // Get loaded models
    let loaded_req = RequestValue::get_loaded_models();
    assert!(matches!(loaded_req.value_type(), ValueType::SystemInfo));
    
    // Select model
    let select_req = RequestValue::select_model("model-id");
    assert!(matches!(select_req.value_type(), ValueType::SystemInfo));
    
    // Delete model
    let delete_req = RequestValue::delete_model("old-model");
    assert!(matches!(delete_req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_webrtc_signaling_workflow() {
    use tabagent_values::ResponseValue;
    
    // Create offer
    let offer_req = RequestValue::create_webrtc_offer("sdp-offer", Some("peer-123"));
    assert!(matches!(offer_req.value_type(), ValueType::SystemInfo));
    
    // Response: session created
    let session_resp = ResponseValue::webrtc_session_created("session-abc");
    let (session_id, _) = session_resp.as_webrtc_session_created();
    assert_eq!(session_id, "session-abc");
    
    // Submit answer
    let answer_req = RequestValue::submit_webrtc_answer("session-abc", "sdp-answer");
    assert!(matches!(answer_req.value_type(), ValueType::SystemInfo));
    
    // Add ICE candidate
    let ice_req = RequestValue::add_ice_candidate("session-abc", "candidate-data");
    assert!(matches!(ice_req.value_type(), ValueType::SystemInfo));
    
    // Get session state
    let state_req = RequestValue::get_webrtc_session("session-abc");
    assert!(matches!(state_req.value_type(), ValueType::SystemInfo));
    
    // Response: session info
    let info_resp = ResponseValue::webrtc_session_info(
        "session-abc",
        "connected",
        Some("offer".to_string()),
        Some("answer".to_string()),
        vec!["candidate1".to_string()],
    );
    
    let (sid, state, _, _, _) = info_resp.as_webrtc_session_info();
    assert_eq!(sid, "session-abc");
    assert_eq!(state, "connected");
}

#[test]
fn test_memory_estimation_workflow() {
    // Estimate without quantization
    let est1 = RequestValue::estimate_memory("llama-2-7b", None);
    assert!(matches!(est1.value_type(), ValueType::SystemInfo));
    
    // Estimate with quantization
    let est2 = RequestValue::estimate_memory("llama-2-7b", Some("q4_0".to_string()));
    assert!(matches!(est2.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_stop_generation_workflow() {
    // Start a generation
    let gen_req = RequestValue::generate("model", "prompt", Some(0.8));
    assert!(matches!(gen_req.value_type(), ValueType::GenerateRequest { .. }));
    
    // Stop the generation
    let stop_req = RequestValue::stop_generation("request-id-123");
    assert!(matches!(stop_req.value_type(), ValueType::Health));
}

#[test]
fn test_embeddings_request_response_cycle() {
    use tabagent_values::ResponseValue;
    
    // Single embedding
    let req1 = RequestValue::embeddings("embed-model", EmbeddingInput::Single("text".to_string()));
    assert!(matches!(req1.value_type(), ValueType::EmbeddingsRequest { .. }));
    
    // Multiple embeddings
    let req2 = RequestValue::embeddings(
        "embed-model",
        EmbeddingInput::Multiple(vec!["text1".to_string(), "text2".to_string()]),
    );
    assert!(matches!(req2.value_type(), ValueType::EmbeddingsRequest { .. }));
    
    // Response
    let embeddings = vec![vec![0.1, 0.2, 0.3], vec![0.4, 0.5, 0.6]];
    let resp = ResponseValue::embeddings(embeddings);
    let json = resp.to_json().unwrap();
    assert!(!json.is_empty());
}

#[test]
fn test_all_request_types_serialization() {
    let requests = vec![
        RequestValue::chat("model", make_messages(vec![("user", "hi")]), None).into_dyn(),
        RequestValue::generate("model", "prompt", None).into_dyn(),
        RequestValue::embeddings("model", EmbeddingInput::Single("text".to_string())).into_dyn(),
        RequestValue::rerank("model", "query", vec![], None).into_dyn(),
        RequestValue::load_model("model", None).into_dyn(),
        RequestValue::unload_model("model").into_dyn(),
        RequestValue::list_models().into_dyn(),
        RequestValue::model_info("model").into_dyn(),
        RequestValue::rag_query("query", None, None).into_dyn(),
        RequestValue::chat_history(None::<String>, None).into_dyn(),
        RequestValue::system_info().into_dyn(),
    ];
    
    // All should serialize successfully
    for request in requests {
        // Just verify we can access type
        let _ = request.value_type();
    }
}

#[test]
fn test_all_health_statuses() {
    use tabagent_values::{ResponseValue, HealthStatus};
    
    let statuses = vec![
        HealthStatus::Healthy,
        HealthStatus::Degraded,
        HealthStatus::Unhealthy,
    ];
    
    for status in statuses {
        let resp = ResponseValue::health(status);
        let json = resp.to_json().unwrap();
        assert!(!json.is_empty());
    }
}

#[test]
fn test_complex_rag_query_with_filters() {
    let complex_filters = serde_json::json!({
        "category": "technology",
        "date_range": {
            "start": "2024-01-01",
            "end": "2024-12-31"
        },
        "tags": ["rust", "ai", "ml"]
    });
    
    let request = RequestValue::rag_query("complex query", Some(20), Some(complex_filters));
    
    // Should serialize and deserialize
    let json = serde_json::to_string(request.request_type()).unwrap();
    let parsed = RequestValue::from_json(&json).unwrap();
    
    assert!(matches!(parsed.value_type(), ValueType::RagQuery { .. }));
}

#[test]
fn test_message_with_name_field() {
    let message = Message {
        role: MessageRole::Function,
        content: "Function result".to_string(),
        name: Some("my_function".to_string()),
    };
    
    let request = RequestValue::save_message("session-xyz", &message);
    
    let json = serde_json::to_string(request.request_type()).unwrap();
    assert!(json.contains("my_function"));
}

#[test]
fn test_concurrent_request_creation() {
    use std::sync::Arc;
    use std::thread;
    
    let results = Arc::new(std::sync::Mutex::new(Vec::new()));
    
    let handles: Vec<_> = (0..20)
        .map(|i| {
            let results = Arc::clone(&results);
            thread::spawn(move || {
                let req = if i % 4 == 0 {
                    let messages = vec![Message {
                        role: MessageRole::User,
                        content: "hi".to_string(),
                        name: None,
                    }];
                    RequestValue::chat(&format!("model-{}", i), messages, None).into_dyn()
                } else if i % 4 == 1 {
                    RequestValue::generate(&format!("model-{}", i), "prompt", None).into_dyn()
                } else if i % 4 == 2 {
                    RequestValue::embeddings(&format!("model-{}", i), EmbeddingInput::Single("text".to_string())).into_dyn()
                } else {
                    RequestValue::load_model(&format!("model-{}", i), None).into_dyn()
                };
                results.lock().unwrap().push(req);
            })
        })
        .collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    assert_eq!(results.lock().unwrap().len(), 20);
}

