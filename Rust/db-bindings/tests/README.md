# DB Bindings Tests

Comprehensive test suite for the Python FFI layer of the TabAgent Embedded Database.

## Test Structure

### Rust Tests (`db_tests.rs`)
- **Purpose**: Unit tests for the Rust FFI layer
- **Run**: `cargo test --package db-bindings`
- **Coverage**: Module loading, type safety, error handling basics

### Python Integration Tests (`test_python_integration.py`)
- **Purpose**: End-to-end Python API tests with real data
- **Run**: `pytest tests/test_python_integration.py -v`
- **Coverage**: Full CRUD operations, vector search, concurrency, error handling

## Running Tests

### 1. Rust Tests
```bash
cd Server/Rust
cargo test --package db-bindings
```

### 2. Python Integration Tests

**Prerequisites:**
```bash
cd Server/Rust/db-bindings
pip install -e .
pip install pytest
```

**Run all tests:**
```bash
pytest tests/test_python_integration.py -v
```

**Run specific test class:**
```bash
pytest tests/test_python_integration.py::TestNodeOperations -v
```

**Run with coverage:**
```bash
pytest tests/ --cov=embedded_db --cov-report=html
```

## Test Philosophy

Following **Rust Architecture Guidelines (RAG)**:

✅ **Rule 17.4**: Every code addition requires tests  
✅ **Rule 17.5**: Avoid stubs and mocks except when absolutely necessary  
✅ **Rule 17.6**: Tests must validate real functionality with real data  

### Real Data Examples

All tests use **production-realistic data**:
- **Messages**: Real chat messages with timestamps, roles, content
- **Embeddings**: Real 384-dimensional vectors from all-MiniLM-L6-v2
- **Relationships**: Real graph structures (chats → messages → attachments)
- **Error cases**: Real invalid inputs that could occur in production

### Test Categories

1. **Basic Operations** (`TestEmbeddedDBBasics`)
   - Database creation
   - In-memory databases
   - Resource cleanup

2. **CRUD Operations** (`TestNodeOperations`)
   - Insert, read, update, delete
   - Multiple node types (Chat, Message, Attachment)
   - Round-trip data validation

3. **Graph Operations** (`TestEdgeOperations`)
   - Edge creation and traversal
   - Relationship queries
   - Multi-hop navigation

4. **Vector Operations** (`TestVectorOperations`)
   - Embedding insertion
   - Semantic similarity search
   - Top-k retrieval

5. **Query Builder** (`TestQueryBuilder`)
   - Structural filters
   - Graph traversal
   - Combined queries

6. **Error Handling** (`TestErrorHandling`)
   - Invalid paths
   - Missing data
   - Type validation
   - Dimension mismatches

7. **Concurrency** (`TestConcurrency`)
   - Concurrent writes
   - Concurrent reads while writing
   - Thread safety validation

8. **Resource Management** (`TestMemoryAndResources`)
   - Cleanup on deletion
   - Large batch operations
   - Memory efficiency

## CI Integration

Add to your CI pipeline:

```yaml
# .github/workflows/test.yml
- name: Test db-bindings
  run: |
    cd Server/Rust
    cargo test --package db-bindings
    cd db-bindings
    pip install -e .
    pytest tests/ -v
```

## Benchmark Tests

For performance-critical operations:

```bash
pytest tests/test_python_integration.py::TestMemoryAndResources::test_large_batch_insertion --durations=10
```

## Future Test Additions

- [ ] Property-based testing with Hypothesis
- [ ] Fuzz testing for malformed inputs
- [ ] Stress tests for concurrent access
- [ ] Performance benchmarks with pytest-benchmark
- [ ] Memory leak detection with memray

## Troubleshooting

**Module not found error:**
```bash
cd Server/Rust/db-bindings
pip install -e .
```

**Tests fail on Windows:**
- Ensure paths use forward slashes or `Path` objects
- Check temp directory permissions

**Concurrency tests fail:**
- May indicate a real thread-safety issue
- Review Arc/RwLock usage in Rust code

