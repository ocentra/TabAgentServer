//! Integration tests for the storage layer.
//!
//! These tests verify the complete lifecycle of database operations including
//! persistence across restarts, batch operations, and complex serialization.

use serde_json::json;
use storage::{
    Attachment, AudioTranscript, Bookmark, Chat, Edge, Embedding, Entity, ImageMetadata, Message,
    ModelInfo, Node, ScrapedPage, StorageManager, Summary, WebSearch,
};
use tempfile::TempDir;

/// Helper function to create a temporary database for testing.
fn create_temp_db() -> (StorageManager, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test_db");
    let storage =
        StorageManager::new(db_path.to_str().unwrap()).expect("Failed to create storage");
    (storage, temp_dir)
}

// --- Basic CRUD Tests ---

#[test]
fn test_chat_crud_lifecycle() {
    let (storage, _temp) = create_temp_db();

    // Create
    let chat = Node::Chat(Chat {
        id: "chat_001".to_string(),
        title: "Planning Meeting".to_string(),
        topic: "Q4 Goals".to_string(),
        created_at: 1697500000000,
        updated_at: 1697500000000,
        message_ids: vec!["msg_1".to_string(), "msg_2".to_string()],
        summary_ids: vec![],
        embedding_id: Some("embed_chat_001".to_string()),
        metadata: json!({"priority": "high", "participants": 5}),
    });

    storage
        .insert_node(&chat)
        .expect("Failed to insert chat node");

    // Read
    let retrieved = storage
        .get_node("chat_001")
        .expect("Failed to get node")
        .expect("Node not found");

    match retrieved {
        Node::Chat(c) => {
            assert_eq!(c.id, "chat_001");
            assert_eq!(c.title, "Planning Meeting");
            assert_eq!(c.topic, "Q4 Goals");
            assert_eq!(c.message_ids.len(), 2);
            assert_eq!(c.metadata["priority"], "high");
        }
        _ => panic!("Expected Chat node"),
    }

    // Update (re-insert with modifications)
    let updated_chat = Node::Chat(Chat {
        id: "chat_001".to_string(),
        title: "Planning Meeting - UPDATED".to_string(),
        topic: "Q4 Goals".to_string(),
        created_at: 1697500000000,
        updated_at: 1697600000000,
        message_ids: vec!["msg_1".to_string(), "msg_2".to_string(), "msg_3".to_string()],
        summary_ids: vec!["sum_1".to_string()],
        embedding_id: Some("embed_chat_001".to_string()),
        metadata: json!({"priority": "urgent", "participants": 7}),
    });

    storage
        .insert_node(&updated_chat)
        .expect("Failed to update chat node");

    let retrieved = storage
        .get_node("chat_001")
        .expect("Failed to get node")
        .expect("Node not found");

    match retrieved {
        Node::Chat(c) => {
            assert_eq!(c.title, "Planning Meeting - UPDATED");
            assert_eq!(c.message_ids.len(), 3);
            assert_eq!(c.summary_ids.len(), 1);
            assert_eq!(c.metadata["priority"], "urgent");
        }
        _ => panic!("Expected Chat node"),
    }

    // Delete
    let deleted = storage
        .delete_node("chat_001")
        .expect("Failed to delete node")
        .expect("Node not found for deletion");

    assert_eq!(deleted.id(), "chat_001");

    let should_be_none = storage.get_node("chat_001").expect("Failed to query node");
    assert!(should_be_none.is_none());
}

#[test]
fn test_message_crud_lifecycle() {
    let (storage, _temp) = create_temp_db();

    let message = Node::Message(Message {
        id: "msg_001".to_string(),
        chat_id: "chat_123".to_string(),
        sender: "user".to_string(),
        timestamp: 1697500000000,
        text_content: "Hello, this is a test message with some content.".to_string(),
        attachment_ids: vec![],
        embedding_id: Some("embed_msg_001".to_string()),
        metadata: json!({"edited": false, "reactions": []}),
    });

    storage
        .insert_node(&message)
        .expect("Failed to insert message");

    let retrieved = storage
        .get_node("msg_001")
        .expect("Failed to get message")
        .expect("Message not found");

    match retrieved {
        Node::Message(m) => {
            assert_eq!(m.id, "msg_001");
            assert_eq!(m.sender, "user");
            assert_eq!(m.text_content.len(), 48);
        }
        _ => panic!("Expected Message node"),
    }
}

#[test]
fn test_entity_crud_lifecycle() {
    let (storage, _temp) = create_temp_db();

    let entity = Node::Entity(Entity {
        id: "entity_rust".to_string(),
        label: "Rust Programming Language".to_string(),
        entity_type: "TECHNOLOGY".to_string(),
        embedding_id: Some("embed_entity_rust".to_string()),
        metadata: json!({"mentions": 42, "first_seen": 1697400000000i64}),
    });

    storage
        .insert_node(&entity)
        .expect("Failed to insert entity");

    let retrieved = storage
        .get_node("entity_rust")
        .expect("Failed to get entity")
        .expect("Entity not found");

    match retrieved {
        Node::Entity(e) => {
            assert_eq!(e.label, "Rust Programming Language");
            assert_eq!(e.entity_type, "TECHNOLOGY");
            assert_eq!(e.metadata["mentions"], 42);
        }
        _ => panic!("Expected Entity node"),
    }
}

#[test]
fn test_summary_and_attachment_nodes() {
    let (storage, _temp) = create_temp_db();

    // Summary
    let summary = Node::Summary(Summary {
        id: "sum_001".to_string(),
        chat_id: "chat_123".to_string(),
        created_at: 1697500000000,
        content: "Discussion about Rust database implementation.".to_string(),
        message_ids: vec!["msg_1".to_string(), "msg_2".to_string()],
        embedding_id: Some("embed_sum_001".to_string()),
        metadata: json!({}),
    });

    storage
        .insert_node(&summary)
        .expect("Failed to insert summary");

    // Attachment
    let attachment = Node::Attachment(Attachment {
        id: "attach_001".to_string(),
        message_id: "msg_001".to_string(),
        mime_type: "image/png".to_string(),
        created_at: 1697500000000,
        filename: "screenshot.png".to_string(),
        size_bytes: 1024 * 500, // 500 KB
        storage_path: "/uploads/screenshot.png".to_string(),
        extracted_text: Some("Text from image OCR".to_string()),
        detected_objects: vec!["laptop".to_string(), "desk".to_string()],
        embedding_id: None,
        metadata: json!({"width": 1920, "height": 1080}),
    });

    storage
        .insert_node(&attachment)
        .expect("Failed to insert attachment");

    let retrieved_summary = storage.get_node("sum_001").expect("Failed to get summary");
    assert!(retrieved_summary.is_some());

    let retrieved_attachment = storage
        .get_node("attach_001")
        .expect("Failed to get attachment");
    assert!(retrieved_attachment.is_some());
}

// --- Edge Tests ---

#[test]
fn test_edge_operations() {
    let (storage, _temp) = create_temp_db();

    let edge = Edge {
        id: "edge_mentions_001".to_string(),
        from_node: "msg_001".to_string(),
        to_node: "entity_rust".to_string(),
        edge_type: "MENTIONS".to_string(),
        created_at: 1697500000000,
        metadata: json!({"confidence": 0.95, "position": 15}),
    };

    storage.insert_edge(&edge).expect("Failed to insert edge");

    let retrieved = storage
        .get_edge("edge_mentions_001")
        .expect("Failed to get edge")
        .expect("Edge not found");

    assert_eq!(retrieved.edge_type, "MENTIONS");
    assert_eq!(retrieved.from_node, "msg_001");
    assert_eq!(retrieved.to_node, "entity_rust");
    assert_eq!(retrieved.metadata["confidence"], 0.95);

    let deleted = storage
        .delete_edge("edge_mentions_001")
        .expect("Failed to delete edge");
    assert!(deleted.is_some());

    let should_be_none = storage.get_edge("edge_mentions_001").expect("Query failed");
    assert!(should_be_none.is_none());
}

// --- Embedding Tests ---

#[test]
fn test_embedding_with_realistic_dimensions() {
    let (storage, _temp) = create_temp_db();

    // 384-dimensional embedding (common for all-MiniLM-L6-v2)
    let vector_384: Vec<f32> = (0..384).map(|i| (i as f32) / 384.0).collect();

    let embedding = Embedding {
        id: "embed_384".to_string(),
        vector: vector_384.clone(),
        model: "all-MiniLM-L6-v2".to_string(),
    };

    storage
        .insert_embedding(&embedding)
        .expect("Failed to insert 384-dim embedding");

    let retrieved = storage
        .get_embedding("embed_384")
        .expect("Failed to get embedding")
        .expect("Embedding not found");

    assert_eq!(retrieved.vector.len(), 384);
    assert_eq!(retrieved.model, "all-MiniLM-L6-v2");
    assert!((retrieved.vector[0] - 0.0).abs() < 0.0001);
    assert!((retrieved.vector[383] - (383.0 / 384.0)).abs() < 0.0001);

    // 768-dimensional embedding (common for larger models)
    let vector_768: Vec<f32> = vec![0.5; 768];

    let embedding_768 = Embedding {
        id: "embed_768".to_string(),
        vector: vector_768,
        model: "large-model".to_string(),
    };

    storage
        .insert_embedding(&embedding_768)
        .expect("Failed to insert 768-dim embedding");

    let retrieved_768 = storage
        .get_embedding("embed_768")
        .expect("Failed to get embedding")
        .expect("Embedding not found");

    assert_eq!(retrieved_768.vector.len(), 768);
}

// --- Persistence Tests ---

#[test]
fn test_data_persists_across_restarts() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("persist_db");
    let db_path_str = db_path.to_str().unwrap();

    // First session: Create and insert data
    {
        let storage = StorageManager::new(db_path_str).expect("Failed to create storage");

        let chat = Node::Chat(Chat {
            id: "persist_chat".to_string(),
            title: "Persistent Chat".to_string(),
            topic: "Testing".to_string(),
            created_at: 1697500000000,
            updated_at: 1697500000000,
            message_ids: vec![],
            summary_ids: vec![],
            embedding_id: None,
            metadata: json!({}),
        });

        storage.insert_node(&chat).expect("Failed to insert node");

        let edge = Edge {
            id: "persist_edge".to_string(),
            from_node: "node_a".to_string(),
            to_node: "node_b".to_string(),
            edge_type: "LINKS_TO".to_string(),
            created_at: 1697500000000,
            metadata: json!({}),
        };

        storage.insert_edge(&edge).expect("Failed to insert edge");

        let embedding = Embedding {
            id: "persist_embed".to_string(),
            vector: vec![1.0, 2.0, 3.0],
            model: "test".to_string(),
        };

        storage
            .insert_embedding(&embedding)
            .expect("Failed to insert embedding");

        // Storage is dropped here, simulating shutdown
    }

    // Second session: Reopen and verify data exists
    {
        let storage = StorageManager::new(db_path_str).expect("Failed to reopen storage");

        let chat = storage
            .get_node("persist_chat")
            .expect("Failed to query node")
            .expect("Node not found after restart");

        match chat {
            Node::Chat(c) => {
                assert_eq!(c.title, "Persistent Chat");
            }
            _ => panic!("Expected Chat node"),
        }

        let edge = storage
            .get_edge("persist_edge")
            .expect("Failed to query edge")
            .expect("Edge not found after restart");

        assert_eq!(edge.edge_type, "LINKS_TO");

        let embedding = storage
            .get_embedding("persist_embed")
            .expect("Failed to query embedding")
            .expect("Embedding not found after restart");

        assert_eq!(embedding.vector.len(), 3);
    }
}

#[test]
fn test_batch_operations() {
    let (storage, _temp) = create_temp_db();

    // Insert 100 nodes
    for i in 0..100 {
        let chat = Node::Chat(Chat {
            id: format!("chat_{:03}", i),
            title: format!("Chat {}", i),
            topic: "Batch Test".to_string(),
            created_at: 1697500000000 + (i as i64),
            updated_at: 1697500000000 + (i as i64),
            message_ids: vec![],
            summary_ids: vec![],
            embedding_id: None,
            metadata: json!({"batch_id": i}),
        });

        storage
            .insert_node(&chat)
            .expect("Failed to insert batch node");
    }

    // Verify all nodes exist
    for i in 0..100 {
        let node = storage
            .get_node(&format!("chat_{:03}", i))
            .expect("Failed to query batch node")
            .expect("Batch node not found");

        match node {
            Node::Chat(c) => {
                assert_eq!(c.metadata["batch_id"], i);
            }
            _ => panic!("Expected Chat node"),
        }
    }

    // Delete half of them
    for i in (0..100).step_by(2) {
        storage
            .delete_node(&format!("chat_{:03}", i))
            .expect("Failed to delete batch node");
    }

    // Verify deletions
    for i in 0..100 {
        let result = storage
            .get_node(&format!("chat_{:03}", i))
            .expect("Failed to query after deletion");

        if i % 2 == 0 {
            assert!(result.is_none(), "Node should be deleted");
        } else {
            assert!(result.is_some(), "Node should still exist");
        }
    }
}

// --- Error Handling Tests ---

#[test]
fn test_nonexistent_entities_return_none() {
    let (storage, _temp) = create_temp_db();

    let node = storage
        .get_node("nonexistent_node")
        .expect("Query should succeed");
    assert!(node.is_none());

    let edge = storage
        .get_edge("nonexistent_edge")
        .expect("Query should succeed");
    assert!(edge.is_none());

    let embedding = storage
        .get_embedding("nonexistent_embedding")
        .expect("Query should succeed");
    assert!(embedding.is_none());

    let deleted_node = storage
        .delete_node("nonexistent_node")
        .expect("Delete should succeed");
    assert!(deleted_node.is_none());
}

// --- Serialization Tests ---

#[test]
fn test_complex_metadata_serialization() {
    let (storage, _temp) = create_temp_db();

    let complex_metadata = json!({
        "nested": {
            "array": [1, 2, 3, 4, 5],
            "object": {
                "key1": "value1",
                "key2": 42,
                "key3": true
            }
        },
        "unicode": "Hello 世界 🦀",
        "null_value": null,
        "boolean": true,
        "number": 3.14159,
        "large_string": "x".repeat(1000)
    });

    let message = Node::Message(Message {
        id: "msg_complex".to_string(),
        chat_id: "chat_123".to_string(),
        sender: "user".to_string(),
        timestamp: 1697500000000,
        text_content: "Test message".to_string(),
        attachment_ids: vec![],
        embedding_id: None,
        metadata: complex_metadata.clone(),
    });

    storage
        .insert_node(&message)
        .expect("Failed to insert complex node");

    let retrieved = storage
        .get_node("msg_complex")
        .expect("Failed to get complex node")
        .expect("Complex node not found");

    match retrieved {
        Node::Message(m) => {
            assert_eq!(m.metadata["nested"]["array"][2], 3);
            assert_eq!(m.metadata["nested"]["object"]["key2"], 42);
            assert_eq!(m.metadata["unicode"], "Hello 世界 🦀");
            assert_eq!(m.metadata["large_string"].as_str().unwrap().len(), 1000);
        }
        _ => panic!("Expected Message node"),
    }
}

#[test]
fn test_node_id_extraction() {
    let chat = Node::Chat(Chat {
        id: "chat_test".to_string(),
        title: "Test".to_string(),
        topic: "Test".to_string(),
        created_at: 0,
        updated_at: 0,
        message_ids: vec![],
        summary_ids: vec![],
        embedding_id: None,
        metadata: json!({}),
    });

    assert_eq!(chat.id(), "chat_test");

    let message = Node::Message(Message {
        id: "msg_test".to_string(),
        chat_id: "chat_123".to_string(),
        sender: "user".to_string(),
        timestamp: 0,
        text_content: "Test".to_string(),
        attachment_ids: vec![],
        embedding_id: None,
        metadata: json!({}),
    });

    assert_eq!(message.id(), "msg_test");
}

// --- MIA-Specific Node Type Tests ---

#[test]
fn test_web_search_node() {
    let (storage, _temp) = create_temp_db();

    let web_search = Node::WebSearch(WebSearch {
        id: "search_001".to_string(),
        query: "rust embedded database".to_string(),
        timestamp: 1697500000000,
        results_urls: vec![
            "https://example.com/result1".to_string(),
            "https://example.com/result2".to_string(),
        ],
        embedding_id: Some("embed_search_001".to_string()),
        metadata: json!({"engine": "google", "result_count": 10}),
    });

    storage
        .insert_node(&web_search)
        .expect("Failed to insert web search");

    let retrieved = storage
        .get_node("search_001")
        .expect("Failed to get web search")
        .expect("Web search not found");

    match retrieved {
        Node::WebSearch(ws) => {
            assert_eq!(ws.query, "rust embedded database");
            assert_eq!(ws.results_urls.len(), 2);
            assert_eq!(ws.metadata["engine"], "google");
        }
        _ => panic!("Expected WebSearch node"),
    }
}

#[test]
fn test_scraped_page_node() {
    let (storage, _temp) = create_temp_db();

    let scraped_page = Node::ScrapedPage(ScrapedPage {
        id: "page_001".to_string(),
        url: "https://example.com/article".to_string(),
        scraped_at: 1697500000000,
        content_hash: "abc123def456".to_string(),
        title: Some("Interesting Article".to_string()),
        text_content: "This is the extracted text from the page...".to_string(),
        embedding_id: Some("embed_page_001".to_string()),
        storage_path: "/scraped/page_001.html".to_string(),
        metadata: json!({"word_count": 500, "language": "en"}),
    });

    storage
        .insert_node(&scraped_page)
        .expect("Failed to insert scraped page");

    let retrieved = storage
        .get_node("page_001")
        .expect("Failed to get scraped page")
        .expect("Scraped page not found");

    match retrieved {
        Node::ScrapedPage(sp) => {
            assert_eq!(sp.url, "https://example.com/article");
            assert_eq!(sp.title, Some("Interesting Article".to_string()));
            assert_eq!(sp.metadata["word_count"], 500);
        }
        _ => panic!("Expected ScrapedPage node"),
    }
}

#[test]
fn test_bookmark_node() {
    let (storage, _temp) = create_temp_db();

    let bookmark = Node::Bookmark(Bookmark {
        id: "bookmark_001".to_string(),
        url: "https://rust-lang.org".to_string(),
        title: "Rust Programming Language".to_string(),
        description: Some("Official Rust website".to_string()),
        created_at: 1697500000000,
        tags: vec!["programming".to_string(), "rust".to_string()],
        embedding_id: Some("embed_bookmark_001".to_string()),
        metadata: json!({"folder": "development", "favorite": true}),
    });

    storage
        .insert_node(&bookmark)
        .expect("Failed to insert bookmark");

    let retrieved = storage
        .get_node("bookmark_001")
        .expect("Failed to get bookmark")
        .expect("Bookmark not found");

    match retrieved {
        Node::Bookmark(b) => {
            assert_eq!(b.url, "https://rust-lang.org");
            assert_eq!(b.tags.len(), 2);
            assert!(b.tags.contains(&"rust".to_string()));
        }
        _ => panic!("Expected Bookmark node"),
    }
}

#[test]
fn test_image_metadata_node() {
    let (storage, _temp) = create_temp_db();

    let image_meta = Node::ImageMetadata(ImageMetadata {
        id: "img_meta_001".to_string(),
        file_path: "/photos/vacation.jpg".to_string(),
        detected_objects: vec!["person".to_string(), "beach".to_string(), "sunset".to_string()],
        detected_faces: vec!["person_1".to_string(), "person_2".to_string()],
        ocr_text: Some("Welcome to Paradise Beach".to_string()),
        embedding_id: Some("embed_img_001".to_string()),
        metadata: json!({"resolution": "4K", "date_taken": "2024-01-15"}),
    });

    storage
        .insert_node(&image_meta)
        .expect("Failed to insert image metadata");

    let retrieved = storage
        .get_node("img_meta_001")
        .expect("Failed to get image metadata")
        .expect("Image metadata not found");

    match retrieved {
        Node::ImageMetadata(im) => {
            assert_eq!(im.detected_objects.len(), 3);
            assert_eq!(im.detected_faces.len(), 2);
            assert_eq!(im.ocr_text, Some("Welcome to Paradise Beach".to_string()));
        }
        _ => panic!("Expected ImageMetadata node"),
    }
}

#[test]
fn test_audio_transcript_node() {
    let (storage, _temp) = create_temp_db();

    let audio_transcript = Node::AudioTranscript(AudioTranscript {
        id: "audio_001".to_string(),
        file_path: "/recordings/meeting.mp3".to_string(),
        transcribed_at: 1697500000000,
        transcript: "This is a transcription of the audio content.".to_string(),
        speaker_diarization: Some(r#"{"speakers":[{"id":"speaker_1","segments":[[0,10],[20,30]]},{"id":"speaker_2","segments":[[10,20]]}]}"#.to_string()),
        embedding_id: Some("embed_audio_001".to_string()),
        metadata: json!({"duration_seconds": 180, "language": "en"}),
    });

    storage
        .insert_node(&audio_transcript)
        .expect("Failed to insert audio transcript");

    let retrieved = storage
        .get_node("audio_001")
        .expect("Failed to get audio transcript")
        .expect("Audio transcript not found");

    match retrieved {
        Node::AudioTranscript(at) => {
            assert_eq!(at.file_path, "/recordings/meeting.mp3");
            assert!(at.speaker_diarization.is_some());
            assert_eq!(at.metadata["duration_seconds"], 180);
        }
        _ => panic!("Expected AudioTranscript node"),
    }
}

#[test]
fn test_model_info_node() {
    let (storage, _temp) = create_temp_db();

    let model_info = Node::ModelInfo(ModelInfo {
        id: "model_001".to_string(),
        name: "Llama-3.2-1B".to_string(),
        path: "/models/llama-3.2-1b.gguf".to_string(),
        size_bytes: 1_500_000_000, // 1.5 GB
        sha256: "abc123...".to_string(),
        format: "GGUF".to_string(),
        loaded_at: Some(1697500000000),
        metadata: json!({"quantization": "Q4_K_M", "context_length": 8192}),
    });

    storage
        .insert_node(&model_info)
        .expect("Failed to insert model info");

    let retrieved = storage
        .get_node("model_001")
        .expect("Failed to get model info")
        .expect("Model info not found");

    match retrieved {
        Node::ModelInfo(mi) => {
            assert_eq!(mi.name, "Llama-3.2-1B");
            assert_eq!(mi.format, "GGUF");
            assert_eq!(mi.size_bytes, 1_500_000_000);
        }
        _ => panic!("Expected ModelInfo node"),
    }
}

#[test]
fn test_all_new_node_types_id_extraction() {
    let web_search = Node::WebSearch(WebSearch {
        id: "ws_test".to_string(),
        query: "test".to_string(),
        timestamp: 0,
        results_urls: vec![],
        embedding_id: None,
        metadata: json!({}),
    });
    assert_eq!(web_search.id(), "ws_test");

    let scraped_page = Node::ScrapedPage(ScrapedPage {
        id: "sp_test".to_string(),
        url: "http://test.com".to_string(),
        scraped_at: 0,
        content_hash: "hash".to_string(),
        title: None,
        text_content: "content".to_string(),
        embedding_id: None,
        storage_path: "/path".to_string(),
        metadata: json!({}),
    });
    assert_eq!(scraped_page.id(), "sp_test");

    let bookmark = Node::Bookmark(Bookmark {
        id: "bm_test".to_string(),
        url: "http://test.com".to_string(),
        title: "Test".to_string(),
        description: None,
        created_at: 0,
        tags: vec![],
        embedding_id: None,
        metadata: json!({}),
    });
    assert_eq!(bookmark.id(), "bm_test");

    let image_meta = Node::ImageMetadata(ImageMetadata {
        id: "im_test".to_string(),
        file_path: "/path".to_string(),
        detected_objects: vec![],
        detected_faces: vec![],
        ocr_text: None,
        embedding_id: None,
        metadata: json!({}),
    });
    assert_eq!(image_meta.id(), "im_test");

    let audio_transcript = Node::AudioTranscript(AudioTranscript {
        id: "at_test".to_string(),
        file_path: "/path".to_string(),
        transcribed_at: 0,
        transcript: "test".to_string(),
        speaker_diarization: None,
        embedding_id: None,
        metadata: json!({}),
    });
    assert_eq!(audio_transcript.id(), "at_test");

    let model_info = Node::ModelInfo(ModelInfo {
        id: "mi_test".to_string(),
        name: "test".to_string(),
        path: "/path".to_string(),
        size_bytes: 0,
        sha256: "hash".to_string(),
        format: "GGUF".to_string(),
        loaded_at: None,
        metadata: json!({}),
    });
    assert_eq!(model_info.id(), "mi_test");
}

