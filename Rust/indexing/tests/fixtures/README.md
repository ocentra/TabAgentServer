# Test Fixtures

This directory contains **permanent mock databases** used by tests.

## Why Permanent?

- ✅ **Fast tests** - No DB creation overhead on every test run
- ✅ **Realistic data** - Pre-populated with actual vector distributions
- ✅ **Reusable** - 233 tests share the same fixtures
- ✅ **Consistent** - Same data across test runs

## Fixtures

### `mock_vectors.mdbx/`
- 100 nodes with 384D embeddings
- Used for vector search tests
- **Reused across tests** - NOT deleted

### `mock_graph.mdbx/`
- 50 nodes with 100 edges
- Used for graph traversal tests
- **Reused across tests** - NOT deleted

## When to Use Which?

### Use Permanent Fixtures:
```rust
#[test]
fn test_vector_search() {
    let storage = get_or_create_mock_vectors_db()?;
    let index = create_index_from_storage(&storage, false)?;
    // Test on pre-populated data
}
```

### Use Temporary DB:
```rust
#[test]
fn test_index_insertion() {
    let (_temp, storage) = create_temp_db()?;
    let index = create_index_from_storage(&storage, false)?;
    // Test with clean state - auto-deletes
}
```

## Regenerating Fixtures

If you need to regenerate the fixtures:
```bash
rm -rf tests/fixtures/mock_*.mdbx
cargo test  # Will recreate on first test run
```

