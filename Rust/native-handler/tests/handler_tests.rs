/// Native handler tests
/// 
/// REAL TESTS - NO MOCKS:
/// - Tests actual message routing
/// - Uses real JSON parsing
/// - Tests real action handling
/// - Cleans up with tempfile

use tabagent_native_handler::{initialize_handler, handle_message};
use tempfile::TempDir;
use serde_json::json;

#[test]
fn test_initialization() {
    println!("\n🧪 Testing handler initialization...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_dir = temp_dir.path().to_str().unwrap();
    
    let response = initialize_handler(cache_dir).expect("Initialization failed");
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("Invalid JSON response");
    
    assert_eq!(parsed["status"], "success", "Initialization should succeed");
    assert!(parsed["message"].as_str().unwrap().contains("initialized"));
    
    println!("✅ Handler initialized");
    println!("   Cache dir: {}", cache_dir);
}

#[test]
fn test_get_available_models() {
    println!("\n🧪 Testing get_available_models action...");
    
    // Initialize first
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let cache_dir = temp_dir.path().to_str().unwrap();
    initialize_handler(cache_dir).expect("Initialization failed");
    
    // Call handler
    let message = json!({
        "action": "get_available_models"
    });
    
    let response = handle_message(message.to_string().as_str()).expect("Handler failed");
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("Invalid JSON");
    
    assert_eq!(parsed["status"], "success");
    
    let models = parsed["payload"]["models"].as_array().expect("Should return models array");
    assert!(!models.is_empty(), "Should have models in catalog");
    
    // Check for test models
    let test_models: Vec<_> = models.iter()
        .filter(|m| m["source"].as_str() == Some("test"))
        .collect();
    
    assert!(!test_models.is_empty(), "Should have test models");
    println!("✅ Found {} models ({} test models)", models.len(), test_models.len());
    
    for model in test_models {
        println!("   - {} ({})", model["name"], model["model_type"]);
    }
}

#[test]
fn test_get_models_by_type() {
    println!("\n🧪 Testing get_models_by_type action...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    initialize_handler(temp_dir.path().to_str().unwrap()).expect("Init failed");
    
    // Query GGUF models
    let message = json!({
        "action": "get_models_by_type",
        "modelType": "gguf"
    });
    
    let response = handle_message(message.to_string().as_str()).expect("Handler failed");
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("Invalid JSON");
    
    assert_eq!(parsed["status"], "success");
    
    let models = parsed["payload"]["models"].as_array().expect("Should return models");
    assert!(!models.is_empty(), "Should have GGUF models");
    
    // Verify all are GGUF
    for model in models {
        assert_eq!(model["model_type"], "gguf");
    }
    
    println!("✅ Found {} GGUF models", models.len());
    
    // Query BitNet models
    let message = json!({
        "action": "get_models_by_type",
        "modelType": "bitnet"
    });
    
    let response = handle_message(message.to_string().as_str()).expect("Handler failed");
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("Invalid JSON");
    
    let bitnet_models = parsed["payload"]["models"].as_array().expect("Should return models");
    assert!(!bitnet_models.is_empty(), "Should have BitNet models");
    
    println!("✅ Found {} BitNet models", bitnet_models.len());
}

#[test]
fn test_get_system_resources() {
    println!("\n🧪 Testing get_system_resources action...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    initialize_handler(temp_dir.path().to_str().unwrap()).expect("Init failed");
    
    let message = json!({
        "action": "get_system_resources"
    });
    
    let response = handle_message(message.to_string().as_str()).expect("Handler failed");
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("Invalid JSON");
    
    assert_eq!(parsed["status"], "success");
    
    let payload = &parsed["payload"];
    
    // Check RAM info
    assert!(payload["total_ram"].as_u64().is_some(), "Should have total_ram");
    assert!(payload["available_ram"].as_u64().is_some(), "Should have available_ram");
    
    let total_ram_gb = payload["total_ram"].as_u64().unwrap() / (1024 * 1024 * 1024);
    let available_ram_gb = payload["available_ram"].as_u64().unwrap() / (1024 * 1024 * 1024);
    
    println!("✅ System resources:");
    println!("   Total RAM: {} GB", total_ram_gb);
    println!("   Available RAM: {} GB", available_ram_gb);
    
    // GPU might be present
    if let Some(gpu) = payload["gpu"].as_object() {
        if gpu["available"].as_bool() == Some(true) {
            let vram_gb = gpu["total_vram"].as_u64().unwrap_or(0) / (1024 * 1024 * 1024);
            println!("   GPU: {} ({} GB VRAM)", gpu["name"], vram_gb);
        }
    }
}

#[test]
fn test_get_memory_usage() {
    println!("\n🧪 Testing get_memory_usage action...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    initialize_handler(temp_dir.path().to_str().unwrap()).expect("Init failed");
    
    let message = json!({
        "action": "get_memory_usage"
    });
    
    let response = handle_message(message.to_string().as_str()).expect("Handler failed");
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("Invalid JSON");
    
    assert_eq!(parsed["status"], "success");
    
    let payload = &parsed["payload"];
    
    // Check RAM structure
    assert!(payload["ram"].is_object(), "RAM should be object");
    let ram = &payload["ram"];
    assert!(ram["total"].as_u64().is_some());
    assert!(ram["used"].as_u64().is_some());
    assert!(ram["available"].as_u64().is_some());
    
    println!("✅ Memory usage:");
    println!("   RAM: {} GB / {} GB", 
        ram["used"].as_u64().unwrap() / (1024 * 1024 * 1024),
        ram["total"].as_u64().unwrap() / (1024 * 1024 * 1024)
    );
    
    // VRAM might be null if no GPU
    if !payload["vram"].is_null() {
        let vram = &payload["vram"];
        println!("   VRAM: {} GB / {} GB",
            vram["used"].as_u64().unwrap_or(0) / (1024 * 1024 * 1024),
            vram["total"].as_u64().unwrap_or(0) / (1024 * 1024 * 1024)
        );
    }
    
    assert_eq!(payload["loadedModelsCount"], 0, "No models should be loaded initially");
}

#[test]
fn test_get_current_model() {
    println!("\n🧪 Testing get_current_model action...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    initialize_handler(temp_dir.path().to_str().unwrap()).expect("Init failed");
    
    let message = json!({
        "action": "get_current_model"
    });
    
    let response = handle_message(message.to_string().as_str()).expect("Handler failed");
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("Invalid JSON");
    
    assert_eq!(parsed["status"], "success");
    
    // Initially should be null
    assert!(parsed["payload"]["currentModel"].is_null(), "No model should be active initially");
    
    println!("✅ Current model: None (as expected)");
}

#[test]
fn test_get_default_model() {
    println!("\n🧪 Testing get_default_model action...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    initialize_handler(temp_dir.path().to_str().unwrap()).expect("Init failed");
    
    // Get default GGUF model
    let message = json!({
        "action": "get_default_model",
        "modelType": "gguf"
    });
    
    let response = handle_message(message.to_string().as_str()).expect("Handler failed");
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("Invalid JSON");
    
    assert_eq!(parsed["status"], "success");
    
    let default_model = &parsed["payload"]["defaultModel"];
    assert!(!default_model.is_null(), "Should have default GGUF model");
    assert_eq!(default_model["model_type"], "gguf");
    
    println!("✅ Default GGUF model: {}", default_model["name"]);
    
    // Get default BitNet model
    let message = json!({
        "action": "get_default_model",
        "modelType": "bitnet"
    });
    
    let response = handle_message(message.to_string().as_str()).expect("Handler failed");
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("Invalid JSON");
    
    if parsed["status"] == "success" {
        let default_bitnet = &parsed["payload"]["defaultModel"];
        println!("✅ Default BitNet model: {}", default_bitnet["name"]);
    }
}

#[test]
fn test_invalid_action() {
    println!("\n🧪 Testing invalid action handling...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    initialize_handler(temp_dir.path().to_str().unwrap()).expect("Init failed");
    
    let message = json!({
        "action": "nonexistent_action_xyz"
    });
    
    let response = handle_message(message.to_string().as_str()).expect("Handler should still return");
    let parsed: serde_json::Value = serde_json::from_str(&response).expect("Invalid JSON");
    
    assert_eq!(parsed["status"], "error");
    assert!(parsed["message"].as_str().unwrap().contains("Unknown action"));
    
    println!("✅ Invalid action handled correctly");
}

#[test]
fn test_malformed_json() {
    println!("\n🧪 Testing malformed JSON handling...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    initialize_handler(temp_dir.path().to_str().unwrap()).expect("Init failed");
    
    let malformed = "{ invalid json here !!!";
    
    let result = handle_message(malformed);
    assert!(result.is_err(), "Should fail on malformed JSON");
    
    println!("✅ Malformed JSON rejected");
}

#[test]
fn test_message_routing_consistency() {
    println!("\n🧪 Testing message routing consistency...");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    initialize_handler(temp_dir.path().to_str().unwrap()).expect("Init failed");
    
    // Call same action multiple times
    let message = json!({
        "action": "get_system_resources"
    });
    
    let response1 = handle_message(message.to_string().as_str()).expect("Handler failed");
    let response2 = handle_message(message.to_string().as_str()).expect("Handler failed");
    let response3 = handle_message(message.to_string().as_str()).expect("Handler failed");
    
    let parsed1: serde_json::Value = serde_json::from_str(&response1).expect("Invalid JSON");
    let parsed2: serde_json::Value = serde_json::from_str(&response2).expect("Invalid JSON");
    let parsed3: serde_json::Value = serde_json::from_str(&response3).expect("Invalid JSON");
    
    // All should succeed
    assert_eq!(parsed1["status"], "success");
    assert_eq!(parsed2["status"], "success");
    assert_eq!(parsed3["status"], "success");
    
    // Total RAM should be consistent
    let ram1 = parsed1["payload"]["total_ram"].as_u64().unwrap();
    let ram2 = parsed2["payload"]["total_ram"].as_u64().unwrap();
    let ram3 = parsed3["payload"]["total_ram"].as_u64().unwrap();
    
    assert_eq!(ram1, ram2);
    assert_eq!(ram2, ram3);
    
    println!("✅ Message routing is consistent across calls");
}

// ============================================================================
// TDD: INFERENCE HANDLER TESTS (WILL FAIL - NOT IMPLEMENTED YET)
// ============================================================================

#[test]
#[ignore] // Remove when inference is implemented
fn test_handle_infer_simple() {
    println!("\n🧪 TDD: Testing simple infer handler (WILL FAIL)...");
    
    // What we NEED in native-handler:
    // 1. "infer" action handler
    // 2. Extract model ID, prompt, settings
    // 3. Call model-loader inference
    // 4. Return text response
    
    let message = json!({
        "action": "infer",
        "modelId": "test/model.gguf",
        "prompt": "Say hello",
        "maxTokens": 50
    });
    
    // TODO: Implement handle_infer in native-handler
    // let response = handle_message(&message.to_string());
    // let parsed: serde_json::Value = serde_json::from_str(&response).unwrap();
    // assert_eq!(parsed["status"], "success");
    // assert!(parsed["payload"]["text"].as_str().unwrap().len() > 0);
    
    println!("⚠️  Infer handler not implemented yet - TDD placeholder");
}

#[test]
#[ignore] // Remove when inference is implemented
fn test_handle_infer_streaming() {
    println!("\n🧪 TDD: Testing streaming infer handler (WILL FAIL)...");
    
    // What we NEED:
    // 1. "infer_stream" action handler
    // 2. Token-by-token callbacks
    // 3. Progress events
    // 4. Cancellation support
    
    let message = json!({
        "action": "infer_stream",
        "modelId": "test/model.gguf",
        "prompt": "Write a story",
        "onToken": true
    });
    
    // TODO: Implement handle_infer_stream
    // Should emit multiple events:
    // - { "event": "token", "data": "Hello" }
    // - { "event": "token", "data": " world" }
    // - { "event": "done", "data": { "text": "Hello world", "tokens": 2 } }
    
    println!("⚠️  Streaming infer not implemented yet - TDD placeholder");
}

#[test]
#[ignore] // Remove when inference is implemented
fn test_handle_infer_with_chat_history() {
    println!("\n🧪 TDD: Testing infer with chat history (WILL FAIL)...");
    
    // What we NEED:
    // 1. Accept messages array
    // 2. Build context from history
    // 3. Apply chat template
    // 4. Return response
    
    let message = json!({
        "action": "infer",
        "modelId": "test/model.gguf",
        "messages": [
            { "role": "user", "content": "What is 2+2?" },
            { "role": "assistant", "content": "4" },
            { "role": "user", "content": "What about 3+3?" }
        ]
    });
    
    // TODO: Implement context management in handle_infer
    // let response = handle_message(&message.to_string());
    // let parsed: serde_json::Value = serde_json::from_str(&response).unwrap();
    // assert!(parsed["payload"]["text"].as_str().unwrap().contains("6"));
    
    println!("⚠️  Chat history not implemented yet - TDD placeholder");
}

#[test]
#[ignore] // Remove when inference is implemented
fn test_handle_infer_with_settings() {
    println!("\n🧪 TDD: Testing infer with custom settings (WILL FAIL)...");
    
    // What we NEED:
    // 1. Parse inference settings from message
    // 2. Pass to model-loader
    // 3. Respect temperature, top_p, etc.
    
    let message = json!({
        "action": "infer",
        "modelId": "test/model.gguf",
        "prompt": "Hello",
        "settings": {
            "temperature": 0.7,
            "topP": 0.9,
            "topK": 40,
            "maxTokens": 100,
            "stop": ["</s>", "\n\n"]
        }
    });
    
    // TODO: Implement settings extraction and passing
    // let response = handle_message(&message.to_string());
    // let parsed: serde_json::Value = serde_json::from_str(&response).unwrap();
    // assert_eq!(parsed["status"], "success");
    
    println!("⚠️  Inference settings not implemented yet - TDD placeholder");
}

#[test]
#[ignore] // Remove when inference is implemented
fn test_handle_cancel_inference() {
    println!("\n🧪 TDD: Testing cancel inference (WILL FAIL)...");
    
    // What we NEED:
    // 1. "cancel_inference" action
    // 2. Track active inference sessions
    // 3. Ability to stop mid-generation
    
    let message = json!({
        "action": "cancel_inference",
        "sessionId": "test-session-123"
    });
    
    // TODO: Implement inference session tracking and cancellation
    // let response = handle_message(&message.to_string());
    // let parsed: serde_json::Value = serde_json::from_str(&response).unwrap();
    // assert_eq!(parsed["status"], "success");
    // assert_eq!(parsed["message"], "Inference cancelled");
    
    println!("⚠️  Cancel inference not implemented yet - TDD placeholder");
}

