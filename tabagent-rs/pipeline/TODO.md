# Pipeline Crate TODO

## Core Architecture
- [ ] Define PipelineType enum (Rule 13.5 - NO strings!)
- [ ] Create BasePipeline trait
- [ ] Create PipelineConfig trait
- [ ] Implement PipelineError type
- [ ] Create PipelineFactory

## Specialized Types
- [ ] Florence2Config
- [ ] WhisperConfig
- [ ] TextGenerationConfig
- [ ] ClipConfig

## Testing
- [ ] Unit tests for factory routing
- [ ] Integration tests with mock pipelines
- [ ] Validate no string literals (cargo clippy)

## Integration
- [ ] Expose via PyO3 in native-handler
- [ ] Update detection.rs to return PipelineType
- [ ] Wire up to native_host.py

## Migration
- [ ] Document Python → Rust migration path
- [ ] Add migration flags (like ONNX_USE_RUST)

