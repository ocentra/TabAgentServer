# TabAgent Testing Guide

## Quick Start

**Run ALL tests:**
```powershell
cd Server
.\scripts\testing\run_tests.ps1
```

## Rust Test Granularity

### 1. Run ALL Rust tests
```powershell
cd Server/tabagent-rs
cargo test --workspace
```

### 2. Run SINGLE CRATE tests
```powershell
cargo test --package tabagent-model-cache
cargo test --package tabagent-hardware
cargo test --package storage
cargo test --package query
cargo test --package tabagent-native-handler
```

### 3. Run SINGLE TEST FILE
```powershell
cargo test --package tabagent-model-cache --test integration_tests
cargo test --package storage --test storage_tests
cargo test --package query --test query_tests
```

### 4. Run SINGLE TEST FUNCTION
```powershell
cargo test --package tabagent-model-cache test_real_download_and_cache
cargo test --package storage test_real_node_crud
cargo test --package query test_graph_traversal
```

### 5. Run with OUTPUT (see println!)
```powershell
cargo test --package storage test_real_node_crud -- --nocapture
```

### 6. Run IGNORED tests (large downloads)
```powershell
cargo test --package tabagent-model-cache -- --ignored
```

### 7. Run specific pattern
```powershell
cargo test --package storage real  # Runs all tests with "real" in name
```

## Python Test Granularity

### Run all Python tests
```powershell
cd Server
python -m pytest tests/
```

### Run single test file
```powershell
python tests/test_secrets.py
python tests/test_model_pipeline.py
```

### Run single test function
```powershell
python -m pytest tests/test_secrets.py::test_env_file_loading
```

## Test Coverage by Crate

### ✅ Fully Tested
- `model-cache` - Download, storage, chunking, catalog
- `hardware` - CPU/GPU detection, memory
- `storage` - Node/edge CRUD, multi-DB, concurrency
- `query` - Graph traversal, filtering, threading
- `native-handler` - Message routing, all actions

### 🔶 Partially Tested  
- `indexing` - Basic tests (needs more)
- `weaver` - Basic tests (needs more)

### ❌ Not Tested (Low Priority)
- `common` - Just constants
- `model-loader` - FFI, hard to unit test
- `python-ml-bridge` - Integration only
- `*-bindings` - Tested via integration

## Mock Data & Fixtures

### Using Realistic Fixtures

Tests use REAL data where possible, but for DB-heavy tests we have fixtures:

```rust
use storage::tests::fixtures::*;

#[test]
fn my_test() {
    let fixture = ConversationFixture::create_realistic_conversation();
    // fixture contains 10 messages, edges, embeddings
    
    fixture.load_into_db(&storage)?;
    // Now test against realistic data
}
```

**Fixture contents:**
- 10 messages (user/assistant alternating)
- 6+ edges (replies, continuations)
- 10 embeddings
- Realistic timestamps, token counts
- Parent-child relationships

### Multi-conversation fixture
```rust
let conversations = create_multi_conversation_fixture();
// Returns 3 complete conversations
```

## Test Philosophy

🔥 **NO MOCKS for I/O operations:**
- Model downloads use REAL HuggingFace API
- Database tests use REAL sled DB (with tempfile cleanup)
- Hardware tests query REAL system resources

✅ **Fixtures for complex state:**
- Chat history uses realistic mock conversations
- Embeddings use mock vectors (testing logic, not models)

❌ **NEVER:**
- Stub tests just to pass
- Mock critical functionality
- Skip cleanup (always use tempfile)

## Common Test Commands

### Fast iteration on one function
```powershell
cargo watch -x 'test --package storage test_real_node_crud -- --nocapture'
```

### Run tests on file save
```powershell
cargo watch -x 'test --package storage'
```

### Check before commit
```powershell
cd Server
.\scripts\testing\run_tests.ps1
```

### Run only fast tests (skip large downloads)
```powershell
cargo test --workspace -- --skip test_large
```

## Test Model Downloads

Tests use these small models (<500MB):
- `SmolLM-135M` (~100MB) - Fastest
- `microsoft/bitnet-b1.58-2B-4T` (~300MB) - BitNet
- `TinyLlama-1.1B` (~600MB) - Larger, run with `--ignored`

First run downloads models, subsequent runs use cache.

## Continuous Integration

GitHub Actions runs:
```yaml
- cargo test --workspace
- python -m pytest tests/
```

All tests must pass before merge.

