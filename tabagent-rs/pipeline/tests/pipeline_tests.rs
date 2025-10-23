/// Integration tests for pipeline crate - Thin composition layer
use tabagent_pipeline::{
    CacheModelType, PipelineFactory, PipelineType, detect_from_file_path,
};

#[test]
fn test_pipeline_type_enum_no_strings() {
    // Rule 13.5: Ensure we're using enums, not strings
    let pt = PipelineType::ImageToText;
    assert_eq!(pt.to_hf_tag(), "image-to-text");

    // Roundtrip test
    let from_tag = PipelineType::from_hf_tag("image-to-text");
    assert_eq!(from_tag, Some(PipelineType::ImageToText));
}

#[test]
fn test_pipeline_type_from_model_info() {
    // Composable: Use model-cache detection
    let pipeline_type = PipelineType::from_model_info(
        &CacheModelType::ONNX,
        Some("image-to-text"),
    );
    assert_eq!(pipeline_type, PipelineType::ImageToText);

    // Fallback to model type
    let pipeline_type = PipelineType::from_model_info(&CacheModelType::GGUF, None);
    assert_eq!(pipeline_type, PipelineType::TextGeneration);
}

#[test]
fn test_factory_routing_composition() {
    // Composable: Use model-cache detection, just route
    let model_info = detect_from_file_path("models/test.gguf")
        .expect("Should detect GGUF");

    let backend = PipelineFactory::route_backend(&model_info)
        .expect("Should route");
    
    // Just verify we got a backend - actual routing logic is in model-cache
    assert!(matches!(backend, tabagent_pipeline::CacheBackend::Rust { .. }));
}

#[test]
fn test_factory_pipeline_type_extraction() {
    let model_info = detect_from_file_path("models/test.gguf")
        .expect("Should detect");

    let pipeline_type = PipelineFactory::get_pipeline_type(&model_info);
    
    // GGUF defaults to text generation
    assert_eq!(pipeline_type, PipelineType::TextGeneration);
}

#[test]
fn test_specialized_detection() {
    // Generic pipelines
    assert!(!PipelineType::TextGeneration.is_specialized());
    assert!(!PipelineType::FeatureExtraction.is_specialized());

    // Specialized pipelines
    assert!(PipelineType::ImageToText.is_specialized());
    assert!(PipelineType::AutomaticSpeechRecognition.is_specialized());
    assert!(PipelineType::ZeroShotImageClassification.is_specialized());
}

#[test]
fn test_pipeline_type_serialization() {
    let pt = PipelineType::ImageToText;
    let json = serde_json::to_string(&pt).expect("Serialization failed");
    assert_eq!(json, r#""image-to-text""#);

    let deserialized: PipelineType = 
        serde_json::from_str(&json).expect("Deserialization failed");
    assert_eq!(deserialized, pt);
}

