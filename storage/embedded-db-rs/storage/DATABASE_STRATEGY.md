# Database Location & Management Strategy

## Overview

This document defines where databases are stored in different environments and how to manage them properly.

---

## 🧪 **Development & Testing**

### Unit Tests (`src/lib.rs` - `#[cfg(test)]`)
- **Location**: Temporary directories via `tempfile::TempDir`
- **Cleanup**: ✅ Automatic (when `TempDir` drops)
- **Example**: 
  ```rust
  let temp_dir = TempDir::new()?;
  let db_path = temp_dir.path().join("test.db");
  let storage = StorageManager::new(db_path.to_str().unwrap())?;
  // Automatically cleaned up when temp_dir goes out of scope
  ```

### Integration Tests (`tests/`)
- **Location**: Temporary directories via `tempfile::TempDir`
- **Cleanup**: ✅ Automatic
- **Pattern**: All integration tests use `create_temp_db()` helper

### Doc Tests (Documentation Examples)
- **Location**: `my_database/`, `test_db/`, etc. (in crate root during test)
- **Cleanup**: ⚠️ Cargo cleans most, but some may remain
- **Mitigation**: 
  - All patterns in `.gitignore`
  - Run `cargo clean` to remove
  - Doc tests are kept minimal
- **Why**: Doc tests run in a simplified environment where `tempfile` adds complexity

### `.gitignore` Patterns
```gitignore
# Test databases
test_db*/
temp_db*/
my_database/
*_database/
*.db/
*.sled/
```

---

## 🐍 **Python Integration (Future - Phase 4)**

When integrated with Python via PyO3:

### Development Mode
```python
from storage import Database

# Option 1: Explicit path
db = Database("./dev_database")

# Option 2: Use project root
db = Database(os.path.join(os.getcwd(), "tabagent.db"))
```

### Location Strategy
- **Default**: `./tabagent.db` in current working directory
- **Configurable**: Accept path parameter
- **Python facade** will handle path resolution

---

## 🖥️ **Production - Native Desktop App**

When built as a native executable:

### Windows
```
%APPDATA%\TabAgent\data\tabagent.db
```
Example: `C:\Users\YourName\AppData\Roaming\TabAgent\data\tabagent.db`

### macOS
```
~/Library/Application Support/TabAgent/tabagent.db
```
Example: `/Users/YourName/Library/Application Support/TabAgent/tabagent.db`

### Linux
```
~/.local/share/TabAgent/tabagent.db
```
Example: `/home/yourname/.local/share/TabAgent/tabagent.db`

### Implementation Strategy

#### 1. Use `dirs` crate (Rust)
```toml
[dependencies]
dirs = "5.0"
```

```rust
use std::path::PathBuf;

pub fn get_default_db_path() -> PathBuf {
    let data_dir = dirs::data_dir()
        .expect("Cannot determine data directory")
        .join("TabAgent")
        .join("data");
    
    std::fs::create_dir_all(&data_dir)
        .expect("Failed to create data directory");
    
    data_dir.join("tabagent.db")
}
```

#### 2. Python Wrapper
```python
import platform
import os
from pathlib import Path

def get_app_data_dir():
    system = platform.system()
    if system == "Windows":
        base = os.getenv('APPDATA')
    elif system == "Darwin":  # macOS
        base = Path.home() / "Library/Application Support"
    else:  # Linux
        base = Path.home() / ".local/share"
    
    app_dir = Path(base) / "TabAgent" / "data"
    app_dir.mkdir(parents=True, exist_ok=True)
    return app_dir

def get_default_db_path():
    return str(get_app_data_dir() / "tabagent.db")
```

---

## 🌐 **Production - Extension (IndexedDB)**

For browser extension (current):
- **Technology**: IndexedDB (TypeScript)
- **Location**: Browser's profile directory (managed by browser)
- **Migration**: Eventually may sync with native app via IPC

---

## 📦 **Database Initialization Flow**

### Native App Startup

```rust
use storage::StorageManager;

fn init_database(custom_path: Option<&str>) -> Result<StorageManager, DbError> {
    let db_path = custom_path
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            get_default_db_path()
                .to_str()
                .expect("Invalid path")
                .to_string()
        });
    
    StorageManager::new(&db_path)
}
```

### Python Integration

```python
class Database:
    def __init__(self, path: str | None = None):
        if path is None:
            path = get_default_db_path()
        self._storage = _native.StorageManager(path)
```

---

## 🔄 **Migration & Backup Strategy**

### Backup Location
```
<data_dir>/backups/tabagent_YYYYMMDD_HHMMSS.db
```

### Migration Process
1. Detect old database (extension's IndexedDB export?)
2. Create backup of existing data
3. Initialize new Rust database
4. Migrate data (if applicable)
5. Verify integrity

### Export/Import
- **Format**: JSON for portability
- **Use case**: User data export, migration between machines
- **Implementation**: Phase 5+

---

## 🔒 **Security Considerations**

### File Permissions
- **Default**: User-only read/write (`0600` on Unix)
- **Windows**: User's AppData (inherits user permissions)
- **Sensitive data**: Consider encryption at rest (future enhancement)

### Path Validation
```rust
fn validate_db_path(path: &str) -> Result<PathBuf, DbError> {
    let path = PathBuf::from(path);
    
    // Ensure parent directory exists or can be created
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Validate it's not a system directory
    // Add other validation as needed
    
    Ok(path)
}
```

---

## 🧹 **Cleanup & Maintenance**

### Automatic Cleanup
- **Snapshots**: sled creates `.snap.*` files - keep last 3, delete older
- **Blobs**: Automatically managed by sled
- **Temp files**: Cleaned on graceful shutdown

### User-Initiated
- **Compact database**: `sled::Db::compact()`
- **Vacuum**: Remove deleted data
- **Export/Archive**: Save old data before cleanup

### Scheduled Maintenance
```rust
// Weekly maintenance task
async fn maintenance_task(storage: &StorageManager) {
    // Compact database
    storage.db().flush_async().await.ok();
    
    // Clean old snapshots
    cleanup_old_snapshots(&storage.db().path());
    
    // Log stats
    log_database_stats(storage);
}
```

---

## 📊 **Database Size Monitoring**

```rust
use std::fs;

pub fn get_db_size(db_path: &str) -> Result<u64, std::io::Error> {
    let mut total_size = 0;
    for entry in fs::read_dir(db_path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            total_size += metadata.len();
        }
    }
    Ok(total_size)
}
```

---

## 🎯 **Summary**

| Environment | Location | Cleanup | Implementation Phase |
|-------------|----------|---------|---------------------|
| **Unit Tests** | `tempfile` | ✅ Automatic | ✅ Phase 1 (Done) |
| **Integration Tests** | `tempfile` | ✅ Automatic | ✅ Phase 1 (Done) |
| **Doc Tests** | Crate root | ⚠️ Manual | ✅ Phase 1 (gitignored) |
| **Python Dev** | `./` or custom | ⚠️ Manual | 🔮 Phase 4 |
| **Native App** | AppData/Library | ⚠️ Manual | 🔮 Phase 4+ |
| **Extension** | IndexedDB | ✅ Browser | ⏸️ Existing |

---

## 📝 **Action Items**

### Phase 1 (✅ Complete)
- [x] Use `tempfile` in unit tests
- [x] Use `tempfile` in integration tests
- [x] Add comprehensive `.gitignore` patterns

### Phase 4 (PyO3 Integration)
- [ ] Implement `get_default_db_path()` in Rust
- [ ] Add `dirs` crate dependency
- [ ] Expose path configuration via PyO3
- [ ] Create Python helper for platform-specific paths

### Phase 5+ (Production)
- [ ] Implement backup/restore functionality
- [ ] Add database compaction routine
- [ ] Create migration utilities
- [ ] Implement size monitoring
- [ ] Add scheduled maintenance tasks

---

## 🔗 **Related Files**

- `.gitignore` - Test database ignore patterns
- `storage/src/lib.rs` - StorageManager implementation
- `storage/tests/integration_tests.rs` - Test database helpers
- Future: `python-bindings/src/lib.rs` - PyO3 path handling

---

**Last Updated**: October 17, 2025  
**Phase**: 1 (Core Storage) - Complete ✅

