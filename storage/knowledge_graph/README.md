# Knowledge Graph Module
======================

This module provides knowledge graph capabilities for TabAgent using **ArangoDB**.

## ✅ Status: FOUNDATION READY

ArangoDB is now the storage backend, which provides:
- **Document Storage**: JSON properties (like IndexedDB)
- **Graph Queries**: Native graph traversal (AQL)
- **Vector Search**: Coming in ArangoDB 3.12+

This perfectly mirrors the IndexedDB architecture from the extension!

## 🏗️ Current Architecture

### Storage Model (Mirrors IndexedDB KnowledgeGraphNode/Edge)

```python
# Everything is a node in the knowledge graph
nodes = {
  "_key": "unique-id",
  "type": "conversation" | "message" | "entity",
  "label": "Human-readable label",
  "properties": {...},  # JSON properties (flexible schema)
  "embedding_id": "optional-embedding-ref",
  "created_at": timestamp,
  "updated_at": timestamp
}

# Edges represent relationships
edges = {
  "_from": "nodes/id1",
  "_to": "nodes/id2",
  "edge_type": "contains" | "mentions" | "relates_to",
  "metadata": {...}
}

# Embeddings for semantic search
embeddings = {
  "_key": "embedding-id",
  "vector_bytes": "...",  # Stored as hex string
  "dimension": 768,
  "model": "sentence-transformers/all-MiniLM-L6-v2",
  "source_type": "message",
  "source_id": "msg-123"
}
```

### Graph Queries (AQL)

```aql
// Find all messages in a conversation (property-based)
FOR msg IN nodes
  FILTER msg.type == "message"
  FILTER msg.properties.conversation_id == @conv_id
  SORT msg.created_at ASC
  RETURN msg

// Traverse graph (edge-based, multi-hop)
FOR v, e, p IN 1..3 OUTBOUND "nodes/conv-123" edges
  FILTER e.edge_type == "contains"
  RETURN {node: v, edge: e, path: p}

// Search by text (simple)
FOR node IN nodes
  FILTER CONTAINS(LOWER(node.label), LOWER(@query))
  RETURN node

// Future: Vector search (ArangoDB 3.12+)
FOR doc IN nodes
  FILTER COSINE_SIMILARITY(doc.embedding_vector, @query_vector) > 0.7
  SORT COSINE_SIMILARITY(doc.embedding_vector, @query_vector) DESC
  RETURN doc
```

## 📋 Roadmap

### Phase 1: Graph Foundation ✅ IMPLEMENTED
- [x] Graph storage (ArangoDB nodes + edges)
- [x] Document properties (JSON like IndexedDB)
- [x] Basic querying (AQL)
- [x] Embedding storage (vector search foundation)
- [ ] Entity Extraction pipelines (NER from messages)
- [ ] Relationship Mapping utilities (link entities)
- [ ] Graph edges for explicit relationships

### Phase 2: Graph RAG (Next)
- [ ] **Context Building**: Build context from graph neighborhoods
- [ ] **Multi-hop Reasoning**: Traverse graph for complex queries  
- [ ] **Semantic + Structural Search**: Combine embeddings with graph structure
- [ ] **Entity Resolution**: Deduplicate entities across conversations

### Phase 3: LangGraph Integration (Agentic)
- [ ] **Agent Orchestration**: Multi-agent workflows via LangGraph
- [ ] **Knowledge Graph as Memory**: Agents query graph for context
- [ ] **Graph Updates from Agents**: Agents add nodes/edges
- [ ] **Collaborative Reasoning**: Multiple agents traverse graph together

## 🔧 Implementation Notes

### Why ArangoDB?

1. **Multi-Model**: Document + Graph + Vector in ONE database
2. **JSON-Native**: Stores exact same structure as IndexedDB
3. **Flexible Schema**: Properties can be any JSON (no rigid schema)
4. **Graph Queries**: AQL is powerful yet easier than Cypher
5. **Open Source**: Apache 2.0 license
6. **Performance**: Written in C++, very fast

### Comparison to Alternatives

| Database   | Document | Graph | Vector | Ease | License |
|------------|----------|-------|--------|------|---------|
| **ArangoDB** | ✅ | ✅ | ✅* | 🟢 Easy | Apache 2.0 |
| Neo4j      | ❌ | ✅ | ❌ | 🟡 Medium | AGPL (not truly open) |
| Chroma     | ❌ | ❌ | ✅ | 🟢 Easy | Apache 2.0 |
| pgvector   | ✅ | ❌ | ✅ | 🟡 Medium | PostgreSQL |
| NetworkX   | ❌ | ✅ | ❌ | 🟢 Easy | BSD |

*Vector search coming in ArangoDB 3.12+. For now, we do Python-side similarity.

### Future: Native Vector Search

When ArangoDB 3.12+ is available:
```aql
// Native vector search (future)
FOR doc IN nodes
  OPTIONS {
    "vectorSearch": {
      "field": "embedding_vector",
      "query": @query_vector,
      "k": 10,
      "metric": "cosine"
    }
  }
  RETURN doc
```

For now: Python-side cosine similarity in `embedding_storage.py`.

## 📚 Resources

- **ArangoDB Docs**: https://www.arangodb.com/docs/
- **AQL Tutorial**: https://www.arangodb.com/docs/stable/aql/
- **Python Driver**: https://github.com/ArangoDB-Community/python-arango
- **LangGraph**: https://langchain-ai.github.io/langgraph/

## 🚀 Getting Started

```bash
# Install ArangoDB (Windows)
# Download from: https://www.arangodb.com/download-major/

# Or use Docker
docker run -p 8529:8529 -e ARANGO_ROOT_PASSWORD="" arangodb/arangodb

# Python dependencies already in requirements.txt
pip install python-arango>=7.9.0

# TabAgent server will auto-create database and collections
python native_host.py
```

The storage layer (`Server/storage/`) handles all ArangoDB operations.
No need to manually create collections - they're auto-created on first run!
