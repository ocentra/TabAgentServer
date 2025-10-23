```markdown
# Specification: API Bindings (`lib.rs` & `python` layer)

## 1. Objective & Core Principles

This specification defines the **public-facing API** of the Rust database engine. It is the bridge between the high-performance Rust core and the external world, primarily the `TabAgentServer`'s Python orchestration layer. The design of this API is critical for making the engine's power accessible, ergonomic, and safe to use.

This component is governed by the following core principles:

*   **Ergonomic & AI-Friendly:** The Python API should be intuitive and high-level. It will implement the **Stateful Facade** pattern, providing "smart" objects that feel like the existing TypeScript `KnowledgeGraphNode` classes, making it easy for both human developers and AI agents to use.
*   **Performance:** The boundary between Python and Rust must be efficient. Data should be transferred in bulk where possible, and heavy computations must remain entirely within the Rust core. The Python layer is for orchestration, not computation.
*   **Safety & Robust Error Handling:** The API must be completely safe. Rust panics must never cross the boundary. All Rust `Result::Err` types must be gracefully converted into specific, catchable Python exceptions.
*   **Clear Separation of Concerns:** The Rust `lib.rs` will expose a minimal, C-compatible, stateless API via `PyO3`. The Python wrapper classes will contain the user-facing logic, stateful caching (for the lifetime of a single operation), and convenience methods.

## 2. Architectural Analysis & Rationale (The "Why")

The design of a language boundary is a critical architectural decision. A poorly designed API can negate all the performance gains of the underlying native code.

### 2.1. Findings from Current System (TypeScript)

*   **Strengths to Preserve:** The existing TypeScript API (`idbChat.ts`, `idbKnowledgeGraph.ts`) is built on the **Active Record pattern**. This is a highly successful and intuitive pattern for application development.
    *   *Example:* `const chat = await Chat.read(...)`, `await chat.addMessage(...)`.
    *   This object-oriented approach, where objects manage their own persistence, is something we must emulate in the Python layer to provide a familiar and productive developer experience.

*   **Critical Limitations to Solve:**
    *   **State Management in a Concurrent World:** As previously identified, the stateful nature of the TypeScript objects (e.g., `myNode.edgesOut`) is a significant risk in a multi-threaded server environment. A simple port of this pattern to Rust would be unsafe.

### 2.2. Findings from Reference Systems

*   **Professional Python Libraries (e.g., `pandas`, `numpy`):** These libraries follow a common pattern: the core data structures and algorithms are implemented in a high-performance language (C++, Rust, Fortran). The Python layer provides a rich, "Pythonic" API that feels natural to Python developers, hiding the complexity of the native core. This is the exact model we will follow.
*   **Database Drivers (e.g., `psycopg2`):** Database drivers separate the concept of a `Connection` or `Engine` object (which is stateless and manages communication) from `Cursor` or `Result` objects (which hold the data from a specific query). This is a sound pattern that reinforces our decision to have a central `EmbeddedDB` object in Rust.

### 2.3. Synthesis & Final Architectural Decision

The definitive architecture is a **two-layer API design**:

1.  **The Rust Core API (`lib.rs`):** A stateless, function-oriented API exposed via `PyO3`. It will consist of a central `EmbeddedDB` class with methods that accept and return simple data structures (structs that can be converted to Python dictionaries). This layer is optimized for safety and raw performance. It knows nothing about "smart objects."

2.  **The Python Facade API (`storage/database.py`):** An object-oriented, stateful-feeling wrapper around the Rust core. This layer will implement the Active Record pattern that developers and AI agents will interact with. It will provide `Node`, `Chat`, and `Message` classes with methods like `.get_edges()` or properties like `.edges_out`. These methods will, under the hood, make efficient, stateless calls to the Rust core.

This hybrid approach gives us the best of both worlds: the **safety and concurrency of a stateless Rust engine** and the **ergonomics and AI-friendliness of a stateful Python API**.

## 3. Detailed Rust Implementation Blueprint (The "What")

### 3.1. Rust `PyO3` Bindings

This is the C-compatible API that Rust will expose.

File: `src/lib.rs`
```rust
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

// Assume the core engine is in a `core` module
use crate::core::{EmbeddedDB, ConvergedQuery, QueryResult, Node, Edge, Path};

/// The main Python-facing class that holds the database instance.
#[pyclass(name = "EmbeddedDB")]
struct PyEmbeddedDB {
    db: EmbeddedDB,
}

#[pymethods]
impl PyEmbeddedDB {
    /// Opens or creates a database at the specified path.
    #[new]
    fn new(db_path: String) -> PyResult<Self> {
        let db = EmbeddedDB::new(&db_path).map_err(to_py_err)?;
        Ok(Self { db })
    }

    /// The primary query method. Accepts a dictionary representing a ConvergedQuery.
    /// Returns a list of dictionaries representing QueryResults.
    fn query(&self, py: Python, query_dict: &PyDict) -> PyResult<PyObject> {
        let query: ConvergedQuery = query_dict.extract()?;
        let results = self.db.query(&query).map_err(to_py_err)?;
        // Convert Vec<QueryResult> to a Python list of dicts
        let py_results = PyList::new(py, results.iter().map(|r| r.to_py_dict(py)));
        Ok(py_results.into())
    }

    /// High-level convenience method for finding the shortest path.
    fn find_shortest_path(&self, py: Python, start_node_id: &str, end_node_id: &str) -> PyResult<Option<PyObject>> {
        let path_result = self.db.find_shortest_path(start_node_id, end_node_id).map_err(to_py_err)?;
        if let Some(path) = path_result {
            Ok(Some(path.to_py_dict(py).into()))
        } else {
            Ok(None)
        }
    }

    // --- Direct, Stateless CRUD for the Python Facade ---
    
    fn get_node(&self, py: Python, node_id: &str) -> PyResult<Option<PyObject>> {
        // ... implementation to fetch and convert Node to PyDict ...
    }

    fn get_edges(&self, py: Python, node_id: &str, direction: &str) -> PyResult<PyObject> {
        // ... implementation to fetch and convert Vec<Edge> to PyList ...
    }
    
    // ... other necessary primitive methods for the facade ...
}

/// The Python module definition.
#[pymodule]
fn embedded_db(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEmbeddedDB>()?;
    Ok(())
}

// Helper trait to convert Rust structs to PyDicts
trait ToPyDict {
    fn to_py_dict(&self, py: Python) -> &PyDict;
}

// Helper function to convert our custom Rust errors into Python exceptions
fn to_py_err(err: core::DbError) -> PyErr {
    // ... implementation to create specific Python exception types ...
}
```

### 3.2. Python Facade Layer

This is the high-level, object-oriented API that the `TabAgentServer` application code will use.

File: `TabAgentServer/storage/database.py`
```python
from typing import List, Optional, Dict, Any
import embedded_db # This is the compiled Rust library

class Database:
    """The main connection to the embedded Rust database."""
    def __init__(self, db_path: str):
        self._db = embedded_db.EmbeddedDB(db_path)

    def query(self, query_dict: Dict[str, Any]) -> List['QueryResult']:
        """Executes a raw Converged Query."""
        results = self._db.query(query_dict)
        return [QueryResult(res) for res in results]

    def get_node(self, node_id: str) -> Optional['Node']:
        """Retrieves a node and wraps it in a smart object."""
        node_dict = self._db.get_node(node_id)
        if node_dict:
            # Pass the connection so the object can make future calls
            return Node(node_dict, self)
        return None

    # Expose other high-level Rust functions directly
    def find_shortest_path(self, start_node_id: str, end_node_id: str) -> Optional['Path']:
        # ...
        pass

class Node:
    """A smart wrapper for a node dictionary returned from Rust."""
    def __init__(self, node_dict: Dict[str, Any], db_conn: Database):
        self._data = node_dict
        self._db = db_conn
        self._edges_out_cache: Optional[List['Edge']] = None

    @property
    def id(self) -> str:
        return self._data['id']

    # ... other properties to access self._data ...

    @property
    def edges_out(self) -> List['Edge']:
        """
        Lazy-loads outgoing edges on first access.
        This mimics the stateful feel of the TypeScript implementation.
        """
        if self._edges_out_cache is None:
            edge_dicts = self._db._db.get_edges(self.id, "outbound")
            self._edges_out_cache = [Edge(e, self._db) for e in edge_dicts]
        return self._edges_out_cache

    def add_edge(self, to_node: 'Node', edge_type: str):
        """Convenience method that calls the underlying Rust function."""
        # self._db._db.create_edge(...)
        pass
```

## 4. Implementation Plan & Checklist

*   [ ] **Project Setup:**
    *   [ ] Configure the root `Cargo.toml` to produce a `cdylib`.
    *   [ ] Add `pyo3` with the `extension-module` feature to dependencies.
*   [ ] **Rust `lib.rs` Implementation:**
    *   [ ] Implement the `PyEmbeddedDB` struct and its `new` method.
    *   [ ] Implement the `ToPyDict` trait for all core model structs (`Node`, `Edge`, `Path`, `QueryResult`).
    *   [ ] Implement the main `query` binding, handling the conversion from `PyDict` to `ConvergedQuery`.
    *   [ ] Implement the primitive bindings needed by the Python facade (`get_node`, `get_edges`, etc.).
    *   [ ] Implement the `to_py_err` function to map Rust errors to custom Python exceptions (e.g., `DbError` -> `embedded_db.DatabaseError`).
*   [ ] **Python Facade Implementation:**
    *   [ ] Implement the `Database` connection class in Python.
    *   [ ] Implement the `Node`, `Edge`, and other "smart" wrapper classes.
    *   [ ] Implement the lazy-loading properties (like `.edges_out`) that call back into the Rust core.
*   [ ] **Build & Integration:**
    *   [ ] Set up `maturin` or a similar tool to build the Rust library into a Python wheel.
    *   [ ] Write a Python integration test (`tests/test_integration.py`) that:
        *   Initializes the `Database`.
        *   Creates several nodes and edges.
        *   Calls `db.get_node()` and uses the lazy-loading `.edges_out` property.
        *   Executes a complex `db.query()` and verifies the results.
        *   Tests that a Rust error correctly raises a Python exception.

## 5. Open Questions & Decisions Needed

*   **Python Exception Hierarchy:** A clear hierarchy for Python exceptions needs to be designed. For example, `embedded_db.DatabaseError` as a base class, with subclasses like `embedded_db.NodeNotFoundError`, `embedded_db.InvalidQueryError`, etc.
*   **Asynchronous API:** The initial API will be synchronous. A future version could expose an `async` Python API by wrapping the Rust calls in a thread pool (`run_in_executor`), which would integrate more smoothly with FastAPI's async endpoints. This is a Phase 2 consideration for the API layer.
```