//! Comprehensive tests for request value creation and manipulation.

use tabagent_values::{
    RequestValue, RequestType, ValueType, EmbeddingInput, MessageRole, Message,
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
fn test_chat_request_creation() {
    let req = RequestValue::chat(
        "gpt-3.5-turbo",
        make_messages(vec![("user", "Hello!")]),
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
        make_messages(vec![
            ("system", "You are a helpful assistant"),
            ("user", "Hello!"),
            ("assistant", "Hi! How can I help?"),
            ("user", "What's the weather?"),
        ]),
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
    let req = RequestValue::chat("model", make_messages(vec![("user", "test")]), None);
    let dyn_req: Value<DynValueTypeMarker> = req.into_dyn();
    
    // Should be able to downcast back
    let specific: Value<RequestValueMarker> = dyn_req.downcast().unwrap();
    assert!(matches!(specific.value_type(), ValueType::ChatRequest { .. }));
}

#[test]
fn test_downcast_wrong_type() {
    use tabagent_values::markers::ResponseValueMarker;
    
    let req = RequestValue::chat("model", make_messages(vec![("user", "test")]), None);
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
    
    let req = RequestValue::chat("model", make_messages(vec![("user", "test")]), Some(0.7));
    
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
    let req = RequestValue::chat("model", Vec::<Message>::new(), None);
    
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
        let req = RequestValue::chat("model", make_messages(vec![("user", "test")]), Some(temp));
        assert!(matches!(req.value_type(), ValueType::ChatRequest { .. }));
    }
}

#[test]
fn test_rerank_request() {
    let req = RequestValue::rerank(
        "rerank-model",
        "What is machine learning?",
        vec![
            "ML is a subset of AI".to_string(),
            "Deep learning uses neural networks".to_string(),
        ],
        Some(5),
    );

    assert!(matches!(
        req.value_type(),
        ValueType::RerankRequest { model } if model == "rerank-model"
    ));
}

#[test]
fn test_unload_model_request() {
    let req = RequestValue::unload_model("model-to-unload");
    
    assert!(matches!(
        req.value_type(),
        ValueType::UnloadModel { model_id } if model_id == "model-to-unload"
    ));
}

#[test]
fn test_list_models_request() {
    let req = RequestValue::list_models();
    
    assert!(matches!(req.value_type(), ValueType::ListModels));
}

#[test]
fn test_model_info_request() {
    let req = RequestValue::model_info("gpt-4");
    
    assert!(matches!(
        req.value_type(),
        ValueType::ModelInfo { model_id } if model_id == "gpt-4"
    ));
}

#[test]
fn test_rag_query_request() {
    let req = RequestValue::rag_query("search query", Some(10), None);
    
    assert!(matches!(
        req.value_type(),
        ValueType::RagQuery { query } if query == "search query"
    ));
}

#[test]
fn test_rag_query_with_filters() {
    let filters = serde_json::json!({"category": "tech"});
    let req = RequestValue::rag_query("test", Some(5), Some(filters));
    
    assert!(matches!(req.value_type(), ValueType::RagQuery { .. }));
}

#[test]
fn test_chat_history_request_no_session() {
    let req = RequestValue::chat_history(None::<String>, Some(50));
    
    assert!(matches!(
        req.value_type(),
        ValueType::ChatHistory { session_id } if session_id.is_none()
    ));
}

#[test]
fn test_chat_history_request_with_session() {
    let req = RequestValue::chat_history(Some("session-123"), Some(100));
    
    assert!(matches!(
        req.value_type(),
        ValueType::ChatHistory { session_id } if session_id == &Some("session-123".to_string())
    ));
}

#[test]
fn test_save_message_request() {
    use tabagent_values::{Message, MessageRole};
    
    let message = Message {
        role: MessageRole::User,
        content: "Test message".to_string(),
        name: None,
    };
    
    let req = RequestValue::save_message("session-456", &message);
    assert!(matches!(req.value_type(), ValueType::ChatHistory { .. }));
}

#[test]
fn test_system_info_request() {
    let req = RequestValue::system_info();
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_stop_generation_request() {
    let req = RequestValue::stop_generation("request-789");
    assert!(matches!(req.value_type(), ValueType::Health));
}

#[test]
fn test_get_params_request() {
    let req = RequestValue::get_params();
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_set_params_request() {
    let params = serde_json::json!({"temperature": 0.8, "max_tokens": 500});
    let req = RequestValue::set_params(params);
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_get_stats_request() {
    let req = RequestValue::get_stats();
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_get_resources_request() {
    let req = RequestValue::get_resources();
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_estimate_memory_request() {
    let req = RequestValue::estimate_memory("llama-2-7b", Some("q4_0".to_string()));
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_estimate_memory_no_quantization() {
    let req = RequestValue::estimate_memory("gpt2", None);
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_semantic_search_request() {
    let req = RequestValue::semantic_search("find similar docs", 10, None);
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_semantic_search_with_filters() {
    let filters = serde_json::json!({"tag": "rust"});
    let req = RequestValue::semantic_search("rust tutorials", 5, Some(filters));
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_calculate_similarity_request() {
    let req = RequestValue::calculate_similarity("hello world", "hi there", None);
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_calculate_similarity_with_model() {
    let req = RequestValue::calculate_similarity("text1", "text2", Some("embed-model".to_string()));
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_evaluate_embeddings_request() {
    let queries = vec!["query1".to_string(), "query2".to_string()];
    let documents = vec!["doc1".to_string(), "doc2".to_string(), "doc3".to_string()];
    let req = RequestValue::evaluate_embeddings("eval-model", queries, documents);
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_cluster_documents_request() {
    let documents = vec!["doc1".to_string(), "doc2".to_string(), "doc3".to_string()];
    let req = RequestValue::cluster_documents(documents, 3, None);
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_cluster_documents_with_model() {
    let documents = vec!["a".to_string(), "b".to_string()];
    let req = RequestValue::cluster_documents(documents, 2, Some("cluster-model".to_string()));
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_recommend_content_request() {
    let candidates = vec!["item1".to_string(), "item2".to_string(), "item3".to_string()];
    let req = RequestValue::recommend_content("user query", candidates, 2, None);
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_recommend_content_with_model() {
    let candidates = vec!["a".to_string(), "b".to_string()];
    let req = RequestValue::recommend_content("query", candidates, 1, Some("rec-model".to_string()));
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_pull_model_request() {
    let req = RequestValue::pull_model("HuggingFaceH4/zephyr-7b-beta", Some("q4_k_m".to_string()));
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_pull_model_no_quantization() {
    let req = RequestValue::pull_model("model-name", None);
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_delete_model_request() {
    let req = RequestValue::delete_model("old-model-id");
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_get_recipes_request() {
    let req = RequestValue::get_recipes();
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_get_embedding_models_request() {
    let req = RequestValue::get_embedding_models();
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_get_loaded_models_request() {
    let req = RequestValue::get_loaded_models();
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_select_model_request() {
    let req = RequestValue::select_model("selected-model-id");
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_create_webrtc_offer_request() {
    let req = RequestValue::create_webrtc_offer("sdp-offer-data", Some("peer-123"));
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_create_webrtc_offer_no_peer() {
    let req = RequestValue::create_webrtc_offer("sdp-data", None::<String>);
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_submit_webrtc_answer_request() {
    let req = RequestValue::submit_webrtc_answer("session-abc", "sdp-answer-data");
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_add_ice_candidate_request() {
    let req = RequestValue::add_ice_candidate("session-xyz", "candidate-data");
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_get_webrtc_session_request() {
    let req = RequestValue::get_webrtc_session("session-999");
    assert!(matches!(req.value_type(), ValueType::SystemInfo));
}

#[test]
fn test_chat_full_parameters() {
    let messages = make_messages(vec![("user", "Test")]);
    
    let req = RequestValue::chat_full(
        "gpt-4-turbo",
        messages,
        Some(0.9),
        Some(2000),
        Some(0.95),
        true,
    );
    
    assert!(matches!(
        req.value_type(),
        ValueType::ChatRequest { model } if model == "gpt-4-turbo"
    ));
}

#[test]
fn test_generate_full_parameters() {
    let req = RequestValue::generate_full(
        "llama-3-8b",
        "Generate text",
        Some(0.7),
        Some(1024),
    );
    
    assert!(matches!(
        req.value_type(),
        ValueType::GenerateRequest { model } if model == "llama-3-8b"
    ));
}

#[test]
fn test_request_serialization_consistency() {
    let req = RequestValue::chat("model", make_messages(vec![("user", "hi")]), Some(0.5));
    
    let json1 = serde_json::to_string(req.request_type()).unwrap();
    let json2 = serde_json::to_string(req.request_type()).unwrap();
    
    assert_eq!(json1, json2);
}

#[test]
fn test_all_request_types_from_json() {
    let test_cases = vec![
        (r#"{"action":"chat","model":"gpt-4","messages":[]}"#, "ChatRequest"),
        (r#"{"action":"generate","model":"llama","prompt":"test"}"#, "GenerateRequest"),
        (r#"{"action":"embeddings","model":"embed","input":"text"}"#, "EmbeddingsRequest"),
        (r#"{"action":"load_model","model_id":"model","force_reload":false}"#, "LoadModel"),
        (r#"{"action":"unload_model","model_id":"model"}"#, "UnloadModel"),
        (r#"{"action":"list_models"}"#, "ListModels"),
        (r#"{"action":"model_info","model_id":"model"}"#, "ModelInfo"),
        (r#"{"action":"health"}"#, "Health"),
        (r#"{"action":"system_info"}"#, "SystemInfo"),
    ];
    
    for (json, expected_type) in test_cases {
        let result = RequestValue::from_json(json);
        assert!(result.is_ok(), "Failed to parse {}: {:?}", expected_type, result.err());
    }
}

