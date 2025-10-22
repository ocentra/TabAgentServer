# Python Integration Strategy

## Overview

This document details the **complete Python integration** across all Rust crates. The strategy is: **Rust does everything except pure ML inference.**

---

## Phase 6: ML Bridge (CURRENT - TO BUILD)

### Goal
Create a thin Rust+PyO3 wrapper that calls Python ML functions.

### Crate Structure

```
ml-bridge/
├── Cargo.toml
├── src/
│   └── lib.rs              # Implements MlBridge trait via PyO3
└── python/
    ├── __init__.py
    ├── ml_funcs.py         # Pure stateless ML functions
    └── models.py           # Model loading & caching
```

### Rust Side (ml-bridge/src/lib.rs)

```rust
use pyo3::prelude::*;
use weaver::ml_bridge::{MlBridge, Entity};
use common::DbResult;

pub struct PyMlBridge {
    python: Python,
    module: Py<PyModule>,
}

#[async_trait::async_trait]
impl MlBridge for PyMlBridge {
    async fn generate_embedding(&self, text: &str) -> DbResult<Vec<f32>> {
        // Call Python function
        Python::with_gil(|py| {
            let result = self.module
                .getattr(py, "generate_embedding")?
                .call1(py, (text,))?;
            let vec: Vec<f32> = result.extract(py)?;
            Ok(vec)
        })
    }
    
    // Similar for extract_entities, summarize, health_check
}
```

### Python Side (ml-bridge/python/ml_funcs.py)

```python
from sentence_transformers import SentenceTransformer
import spacy
from transformers import pipeline

# Global model instances (loaded once)
_embed_model = None
_nlp = None
_summarizer = None

def _get_embed_model():
    global _embed_model
    if _embed_model is None:
        _embed_model = SentenceTransformer('all-MiniLM-L6-v2')
    return _embed_model

def _get_nlp():
    global _nlp
    if _nlp is None:
        _nlp = spacy.load('en_core_web_sm')
    return _nlp

def generate_embedding(text: str) -> list[float]:
    """Generate 384-dim embedding for text."""
    model = _get_embed_model()
    embedding = model.encode(text, convert_to_tensor=False)
    return embedding.tolist()

def extract_entities(text: str) -> list[dict]:
    """Extract named entities using spaCy."""
    nlp = _get_nlp()
    doc = nlp(text)
    
    entities = []
    for ent in doc.ents:
        entities.append({
            "text": ent.text,
            "label": ent.label_,
            "start": ent.start_char,
            "end": ent.end_char,
        })
    return entities

def summarize(messages: list[str]) -> str:
    """Summarize a list of messages."""
    global _summarizer
    if _summarizer is None:
        _summarizer = pipeline("summarization", model="facebook/bart-large-cnn")
    
    # Concatenate messages
    full_text = " ".join(messages)
    
    # Truncate if too long (BART max 1024 tokens)
    if len(full_text) > 1024:
        full_text = full_text[:1024]
    
    # Generate summary
    result = _summarizer(full_text, max_length=150, min_length=40, do_sample=False)
    return result[0]['summary_text']
```

---

## Phase 4: Main API Bindings (FUTURE)

### Goal
Expose the entire Rust database to Python with an ergonomic API.

### Crate Structure

```
bindings/
├── Cargo.toml
├── src/
│   └── lib.rs              # PyO3 bindings for DB API
└── python/
    └── embedded_db/
        ├── __init__.py
        ├── database.py     # EmbeddedDB class
        ├── models.py       # Chat, Message, Entity classes
        └── query.py        # Query builder
```

### Python API Design (Active Record Pattern)

```python
from embedded_db import EmbeddedDB, ConvergedQuery

# Open database
db = EmbeddedDB("path/to/db")

# Create chat
chat = db.create_chat(title="Project Discussion", topic="Rust Database")

# Add messages
msg1 = chat.add_message(
    sender="Alice",
    content="We should use Rust for performance",
    role="user"
)

# Query with converged pipeline
results = db.query(
    structural_filters=[
        {"property": "chat_id", "operator": "Equals", "value": chat.id}
    ],
    semantic_query={
        "text": "database performance",  # Auto-generates embedding
        "threshold": 0.7
    },
    limit=10
)

# Access results
for result in results:
    print(f"Node: {result.node.id}, Score: {result.similarity_score}")

# Graph traversal
path = db.find_shortest_path(start_id="msg_1", end_id="entity_5")
```

---

## Integration Map by Crate

### common
**Python Needs**: Type definitions
```python
# Python equivalent types
NodeId = str
EdgeId = str
EmbeddingId = str

class DbError(Exception): pass
```

### storage
**Python Needs**: CRUD operations
```python
db.create_node(node_dict)
db.get_node(node_id)
db.update_node(node_id, updates)
db.delete_node(node_id)
# Same for edges, embeddings
```

### indexing
**Python Needs**: Transparent (auto-managed by storage)
- No direct Python API needed
- Indexes update automatically

### query
**Python Needs**: Query builder + execution
```python
query = ConvergedQuery(
    structural_filters=[...],
    graph_filter={...},
    semantic_query={...}
)
results = db.execute_query(query)
```

### task-scheduler
**Python Needs**: Activity level control
```python
# Set activity level from Python
db.set_activity_level("HighActivity")  # User is chatting
db.set_activity_level("SleepMode")     # User idle for 30 min
```

### weaver
**Python Needs**: Event emission + status
```python
# Automatically emits events on CRUD
# Python can query status:
status = db.weaver_status()
# {"active_workers": 4, "queue_size": 12, "last_event": "..."}
```

### ml-bridge
**Python Needs**: ML functions (detailed above)
- generate_embedding
- extract_entities
- summarize

---

## Python Package Structure

```
embedded_db_python/
├── setup.py
├── embedded_db/
│   ├── __init__.py
│   ├── database.py         # Main EmbeddedDB class
│   ├── models.py           # Node types (Chat, Message, etc.)
│   ├── query.py            # Query builder
│   ├── errors.py           # Exception types
│   └── _native.so          # Compiled Rust binary (PyO3)
└── tests/
    ├── test_crud.py
    ├── test_query.py
    └── test_weaver.py
```

---

## Build & Installation

### Development
```bash
# Build Rust with Python bindings
cd Server/storage/embedded-db-rs
maturin develop --release

# Python can now import
python
>>> import embedded_db
>>> db = embedded_db.EmbeddedDB("test.db")
```

### Production
```bash
# Build wheel
maturin build --release

# Install
pip install target/wheels/embedded_db-*.whl
```

---

## Testing Strategy

### Unit Tests
- Rust: 93 tests ✅ DONE
- Python: Test Python API layer

### Integration Tests
- Python → Rust → Python roundtrip
- Event emission → Weaver → ML → Database
- Full query pipeline

### Example Test
```python
def test_message_enrichment():
    db = EmbeddedDB(":memory:")
    
    # Create message
    msg = db.create_message(
        chat_id="chat_1",
        content="Alice met Bob in Paris"
    )
    
    # Wait for weaver to process
    time.sleep(0.5)
    
    # Check enrichment
    embedding = db.get_embedding_for_node(msg.id)
    assert embedding is not None
    assert len(embedding.vector) == 384
    
    # Check entity extraction
    entities = db.get_entities_mentioned_in(msg.id)
    assert len(entities) >= 3  # Alice, Bob, Paris
    assert any(e.label == "Paris" for e in entities)
```

---

## Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Python→Rust call overhead | < 1μs | PyO3 is very fast |
| Type conversion (dict→struct) | < 10μs | Serde JSON |
| ML embedding generation | ~50ms | Dominated by model inference |
| Full message insert + enrich | ~200ms | Includes ML calls |
| Query (with ML) | ~100ms | Embed query + search |

---

## Next Implementation Steps

1. **Build ml-bridge crate** (Rust+PyO3)
2. **Implement Python ML functions** (sentence-transformers, spaCy)
3. **Test ml-bridge integration** with weaver
4. **Document Python API requirements** for Phase 4
5. **Build main PyO3 bindings** (Phase 4)
6. **Implement Python facade** (Active Record pattern)
7. **Integration testing**
8. **Performance benchmarking**

---

**Status**: Phase 6 in progress (ml-bridge)  
**Next**: Build ml-bridge Rust crate

