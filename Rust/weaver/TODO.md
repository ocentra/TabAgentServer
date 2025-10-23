# Weaver - TODO

## ✅ Phase 5 Complete (Current State)

- [x] Event system with 5 event types
- [x] MlBridge trait abstraction
- [x] MockMlBridge for testing
- [x] Async dispatcher with concurrent task spawning
- [x] Semantic indexer module
- [x] Entity linker module
- [x] Associative linker module
- [x] Summarizer module
- [x] Integration tests
- [x] Documentation

## 🔄 Phase 5.5: Optimizations (Optional)

- [ ] **Batch Processing**: Optimize `BatchMessagesAdded` event
  - Current: Individual processing
  - Target: Batch embed multiple messages at once
  - Benefit: ~5x faster for bulk imports

- [ ] **Deduplication Logic**: Prevent duplicate entities
  - Current: Always creates new Entity nodes
  - Target: Check for existing entities by (label, type)
  - Method: Add indexed query in entity_linker

- [ ] **Configurable Thresholds**:
  - Similarity threshold for associative links (currently hardcoded 0.85)
  - Max links per node (currently hardcoded 3)
  - Summary trigger threshold (currently hardcoded 20 messages)

## 📋 Phase 6: Activity Integration (Pending)

- [ ] **Integrate with TaskScheduler**:
  - Listen to activity level changes
  - Pause intensive tasks during HighActivity
  - Resume/batch during LowActivity and SleepMode
  - Priority: Urgent vs Normal vs Low events

## 🚀 Future Enhancements

### Performance
- [ ] Metrics collection (events/sec, avg processing time)
- [ ] Performance benchmarking suite
- [ ] Profiling integration
- [ ] Memory usage monitoring

### Features
- [ ] Event replay for failed processing
- [ ] Event persistence (save to disk for crash recovery)
- [ ] Configurable worker pool size
- [ ] Rate limiting for ML calls
- [ ] Health monitoring dashboard

### ML Improvements
- [ ] Model warm-up on startup
- [ ] Model versioning
- [ ] A/B testing for different models
- [ ] Fallback strategies for ML failures

## 🐛 Known Issues

- None currently

## 📝 Notes

- Weaver uses `tokio::spawn` for unlimited concurrency - consider semaphore for rate limiting
- Python ML calls are CPU-bound - consider dedicated thread pool
- Event queue is unbounded - could add backpressure if needed

