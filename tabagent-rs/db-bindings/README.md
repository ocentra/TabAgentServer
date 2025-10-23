# TabAgent Database Bindings (db-bindings)

**Python bindings for the Rust embedded database.**

Python bindings for the TabAgent embedded multi-model database, built with PyO3.

## ✨ Features

- 🔌 **Native Performance**: Direct Rust-to-Python bindings with minimal overhead
- 🗄️ **Multi-Model**: Document, Graph, and Vector data models
- 🔍 **Semantic Search**: Vector embeddings with HNSW indexing
- 🌐 **Knowledge Graph**: Typed relationships and graph traversal
- 🧠 **Knowledge Weaver**: Autonomous background enrichment
- 🔐 **Thread-Safe**: ACID transactions via sled
- 📦 **Zero Config**: Embedded database, no server required

## 📦 Installation

### From Wheel (Recommended)

```bash
# Install the pre-built wheel
pip install path/to/bindings-0.1.0-cp39-abi3-win_amd64.whl
```

### From Source

```bash
# Install maturin
pip install maturin

# Build and install
cd bindings
maturin develop --release
```

## 🚀 Quick Start

```python
import embedded_db

# Create database
db = embedded_db.EmbeddedDB("./my_database")

# Create a chat
chat = {
    "type": "Chat",
    "id": "chat_001",
    "title": "My First Chat",
    "topic": "Python Bindings",
    "created_at": 1697500000000,
    "updated_at": 1697500000000,
    "message_ids": [],
    "summary_ids": [],
    "metadata": "{}"
}
chat_id = db.insert_node(chat)

# Create a message
message = {
    "type": "Message",
    "id": "msg_001",
    "chat_id": chat_id,
    "sender": "user",
    "timestamp": 1697500000000,
    "text_content": "Hello, world!",
    "attachment_ids": [],
    "metadata": "{}"
}
msg_id = db.insert_node(message)

# Create an edge
edge_id = db.insert_edge(
    from_node=chat_id,
    to_node=msg_id,
    edge_type="CONTAINS",
    metadata=None
)

# Retrieve nodes
retrieved_chat = db.get_node(chat_id)
print(f"Chat: {retrieved_chat['title']}")

# Create an embedding
embedding_id = db.insert_embedding(
    embedding_id="emb_001",
    vector=[0.1, 0.2, 0.3] * 128,  # 384-dim vector
    model="sentence-transformers"
)

# Search similar (placeholder)
results = db.search_vectors([0.1, 0.2, 0.3] * 128, top_k=5)

# Get stats
stats = db.stats()
print(stats)
```

## 📚 API Reference

### EmbeddedDB

Main database class.

#### `EmbeddedDB(db_path: str)`

Create a new database instance.

**Parameters:**
- `db_path` (str): Path to database directory

**Example:**
```python
db = embedded_db.EmbeddedDB("./my_db")
```

---

### Node Operations

#### `insert_node(node: dict) -> str`

Insert a new node.

**Parameters:**
- `node` (dict): Node data (must include `type`, `id`, and type-specific fields)

**Returns:**
- `str`: The node ID

**Example:**
```python
chat = {
    "type": "Chat",
    "id": "chat_123",
    "title": "Project Discussion",
    "topic": "Rust Database",
    "created_at": 1697500000000,
    "updated_at": 1697500000000,
    "message_ids": [],
    "summary_ids": [],
    "metadata": "{}"
}
chat_id = db.insert_node(chat)
```

#### `get_node(node_id: str) -> dict | None`

Retrieve a node by ID.

**Parameters:**
- `node_id` (str): The node ID

**Returns:**
- `dict` or `None`: Node data as dictionary, or None if not found

**Example:**
```python
node = db.get_node("chat_123")
if node:
    print(node['title'])
```

#### `delete_node(node_id: str) -> bool`

Delete a node.

**Parameters:**
- `node_id` (str): The node ID

**Returns:**
- `bool`: True if deleted successfully

---

### Edge Operations

#### `insert_edge(from_node: str, to_node: str, edge_type: str, metadata: str | None = None) -> str`

Create a relationship between two nodes.

**Parameters:**
- `from_node` (str): Source node ID
- `to_node` (str): Target node ID
- `edge_type` (str): Type of relationship (e.g., "CONTAINS", "MENTIONS")
- `metadata` (str, optional): JSON metadata string

**Returns:**
- `str`: The edge ID

**Example:**
```python
edge_id = db.insert_edge(
    from_node="chat_123",
    to_node="msg_456",
    edge_type="CONTAINS",
    metadata='{"weight": 1.0}'
)
```

#### `get_edge(edge_id: str) -> dict | None`

Retrieve an edge by ID.

#### `delete_edge(edge_id: str) -> bool`

Delete an edge.

---

### Embedding Operations

#### `insert_embedding(embedding_id: str, vector: list[float], model: str) -> str`

Store a vector embedding.

**Parameters:**
- `embedding_id` (str): Unique ID for the embedding
- `vector` (list[float]): Embedding vector (384/768/1536 dimensions)
- `model` (str): Model name used to generate the embedding

**Returns:**
- `str`: The embedding ID

**Example:**
```python
embedding_id = db.insert_embedding(
    embedding_id="emb_001",
    vector=[0.1, 0.2, 0.3] * 128,  # 384-dim
    model="sentence-transformers/all-MiniLM-L6-v2"
)
```

#### `get_embedding(embedding_id: str) -> dict | None`

Retrieve an embedding by ID.

#### `search_vectors(query_vector: list[float], top_k: int) -> list[tuple[str, float]]`

Find similar embeddings (placeholder - not yet fully implemented).

**Parameters:**
- `query_vector` (list[float]): Query embedding
- `top_k` (int): Number of results

**Returns:**
- `list[tuple[str, float]]`: List of (node_id, score) tuples

---

### Utility Methods

#### `stats() -> dict`

Get database statistics.

**Returns:**
- `dict`: Statistics dictionary

## 🧪 Testing

Run the test suite:

```bash
python test_python.py
```

Expected output:
```
🧪 Testing TabAgent Embedded Database Python Bindings
============================================================
✅ ALL TESTS PASSED!
```

## 🏗️ Architecture

```
Python Application
        ↓
   PyO3 Bindings (this crate)
        ↓
   ┌─────────────────────────┐
   │  Rust Database Core     │
   ├─────────────────────────┤
   │  • storage (sled)       │
   │  • indexing (HNSW)      │
   │  • query (converged)    │
   │  • weaver (async)       │
   └─────────────────────────┘
        ↓
   Persistent Storage (sled)
```

## 📝 Node Types

### Chat
```python
{
    "type": "Chat",
    "id": str,
    "title": str,
    "topic": str,
    "created_at": int,  # Unix timestamp (ms)
    "updated_at": int,
    "message_ids": list[str],
    "summary_ids": list[str],
    "metadata": str  # JSON string
}
```

### Message
```python
{
    "type": "Message",
    "id": str,
    "chat_id": str,
    "sender": str,
    "timestamp": int,
    "text_content": str,
    "attachment_ids": list[str],
    "metadata": str  # JSON string
}
```

### Entity
```python
{
    "type": "Entity",
    "id": str,
    "label": str,
    "entity_type": str,  # e.g., "PERSON", "PROJECT"
    "metadata": str  # JSON string
}
```

### Summary
```python
{
    "type": "Summary",
    "id": str,
    "chat_id": str,
    "content": str,
    "created_at": int,
    "message_ids": list[str],
    "metadata": str  # JSON string
}
```

## 🔮 Future Enhancements

- [ ] Query builder API
- [ ] Graph traversal methods
- [ ] Full Knowledge Weaver integration
- [ ] Async Python API
- [ ] Context managers for transactions
- [ ] Streaming results for large queries
- [ ] Python-friendly type hints

## 📄 License

Same as parent project (see root LICENSE)

## 🤝 Contributing

This is part of the TabAgent project. See main README for contribution guidelines.

