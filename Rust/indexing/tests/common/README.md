# Common Test Utilities

**REAL database setup, no fake bullshit!**

## Usage

```rust
use common::{setup_real_db, test_edge};

#[test]
fn my_test() {
    // Get REAL MDBX database in temp directory
    let (manager, _temp) = setup_real_db();
    
    // Create test data
    let edge = test_edge("e1", "from", "to");
    manager.index_edge(&edge).unwrap();
    
    // Verify with REAL zero-copy MDBX access
    let guard = manager.graph().get_outgoing("from").unwrap().unwrap();
    assert!(guard.contains_edge("e1"));
}
```

## Functions

- `setup_real_db()` - Creates IndexManager with REAL MDBX
- `setup_real_db_with_hybrid()` - Creates with hot/warm/cold tiers
- `test_edge(id, from, to)` - Helper to create Edge struct
- `test_edges(&[...])` - Create multiple edges
- `assert_uses_real_mdbx(manager)` - Verifies MDBX is working

**NO MOCKS. NO FAKES. REAL DATABASE TESTING.**

