# Test Migration Guide

## Old Pattern (Deprecated)
```rust
let temp_dir = TempDir::new().unwrap();
let index = IndexManager::new(temp_dir.path()).unwrap();
```

## New Pattern (Correct)
```rust
use fixtures::{create_temp_db, create_index_from_storage};

// For clean-state tests (auto-deletes)
let (_temp, storage) = create_temp_db().unwrap();
let index = create_index_from_storage(&storage, false).unwrap();

// For tests needing realistic data (permanent, reused)
use fixtures::get_or_create_mock_vectors_db;
let storage = get_or_create_mock_vectors_db().unwrap();
let index = create_index_from_storage(&storage, false).unwrap();
```

## Migration Steps for Each Test File

1. Add `mod fixtures;` at top
2. Add `use fixtures::{create_temp_db, create_index_from_storage};`
3. Replace `IndexManager::new()` with pattern above
4. Add `#[allow(deprecated)]` if keeping old pattern temporarily

