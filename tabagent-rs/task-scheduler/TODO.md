# Task Scheduler - TODO

## ✅ Completed

### Phase 1.5: Foundation
- [x] ActivityDetector implementation
- [x] Three activity levels (HighActivity, LowActivity, SleepMode)
- [x] Activity level transitions with timeouts
- [x] TaskQueue with priority-based BinaryHeap
- [x] Four priority levels (Urgent, Normal, Low, Batch)
- [x] TaskScheduler orchestration
- [x] Task submission via async channel
- [x] Activity-aware task execution
- [x] Manual activity level setting
- [x] Queue statistics (per-priority counts)
- [x] 9 unit tests passing

### Task Types (Stubbed)
- [x] GenerateEmbedding
- [x] ExtractEntities
- [x] LinkEntities
- [x] GenerateSummary
- [x] CreateAssociativeLinks
- [x] IndexNode
- [x] UpdateVectorIndex

## 🔄 In Progress

_Nothing currently in progress_

## 📋 Pending

### Phase 2/3 Integration
- [ ] Connect IndexNode task to actual indexing layer
- [ ] Connect UpdateVectorIndex to HNSW index
- [ ] Pass storage reference to task executor

### Phase 4: ML Bridge Integration
- [ ] Implement GenerateEmbedding with Python ML models
- [ ] Implement ExtractEntities with NER models
- [ ] Implement GenerateSummary with LLM

### Phase 5: Knowledge Weaver Integration
- [ ] Implement LinkEntities (graph expansion)
- [ ] Implement CreateAssociativeLinks (semantic similarity)
- [ ] Multi-step task pipelines (embedding → NER → linking)

### Features
- [ ] Task dependencies ("Don't run B until A completes")
- [ ] Task cancellation (user deleted the node)
- [ ] Task retry on failure (with exponential backoff)
- [ ] Task timeout limits
- [ ] Rate limiting (max X LLM calls per minute)

### Persistence
- [ ] Save pending tasks to disk (resume after restart)
- [ ] Task execution history/audit log
- [ ] Failed task tracking

### Observability
- [ ] Task execution metrics (duration, success rate)
- [ ] Queue depth monitoring
- [ ] Activity level change logging
- [ ] Prometheus/metrics exporter

### OS Integration
- [ ] Detect actual user input (mouse, keyboard) via OS hooks
- [ ] Detect screen lock/unlock events
- [ ] Detect power state changes (battery vs plugged in)
- [ ] Respect OS low-power mode

## 🚫 Blockers

### ML Execution
- **Issue**: Tasks are currently stubbed (just println)
- **Blocker**: Needs ML bridge (PyO3) to call Python models
- **Timeline**: Phase 4
- **Workaround**: None (tasks queue correctly, just don't do real work yet)

### OS Activity Detection
- **Issue**: Manual activity setting only (no automatic detection)
- **Blocker**: OS-specific input monitoring needs platform-specific code
- **Timeline**: Phase 4 (when UI integration happens)
- **Workaround**: UI can manually call `set_activity()`

## 📊 Progress

- **Phase 1.5 (Foundation)**: ✅ 100% Complete
- **Integration**: 🔴 0% (waiting for Phase 3/4)
- **Task Execution**: 🔴 0% (stubbed, waiting for ML bridge)
- **Overall**: **FOUNDATION READY** - Queue works, needs integration

