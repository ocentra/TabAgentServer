# Embedding Support in TabAgent

Comprehensive embedding capabilities for semantic search, recommendations, clustering, and RAG applications.

## Overview

TabAgent provides state-of-the-art embedding support across multiple modalities:
- **Text embeddings** - Semantic search, classification, retrieval
- **Image embeddings** - Visual search, similarity
- **Multi-modal embeddings** - CLIP-style text-image embeddings
- **Audio/Video** - Coming soon via MediaPipe

## Embedding Models

### Curated Model Registry

TabAgent includes a registry of popular, production-ready embedding models:

#### Text Models
- **all-MiniLM-L6-v2** (384d) - Fast, lightweight, general purpose
- **all-MPNet-Base-v2** (768d) - Best overall performance
- **BGE Small/Base** (384d/768d) - Excellent retrieval
- **E5 Small** (384d) - Microsoft embeddings
- **GTE Small** (384d) - Alibaba model

#### Multi-modal Models
- **CLIP ViT-B/32** (512d) - Text-image embeddings
- **CLIP ViT-L/14** (768d) - Higher quality CLIP

#### Specialized Models
- **BGE Reranker** - Cross-encoder for reranking

### Model Selection Guide

**For Semantic Search:**
- Small corpus (<100K docs): `all-MiniLM-L6-v2`
- Large corpus: `bge-base-en-v1.5`
- Best quality: `all-mpnet-base-v2`

**For Classification:**
- Fast: `all-MiniLM-L6-v2`
- Accurate: `all-mpnet-base-v2`

**For Multi-modal:**
- General: `clip-vit-base-patch32`
- High-end: `clip-vit-large-patch14`

## API Endpoints

### 1. Generate Embeddings
```http
POST /api/v1/embeddings
```

Generate embeddings for texts.

**Request:**
```json
{
  "input": ["text1", "text2"],
  "model": "all-MiniLM-L6-v2"
}
```

**Response:**
```json
{
  "embeddings": [[0.1, 0.2, ...], [0.3, 0.4, ...]],
  "model": "all-MiniLM-L6-v2",
  "dimensions": 384,
  "total_tokens": 100
}
```

### 2. Semantic Search
```http
POST /api/v1/semantic-search
```

Find most relevant documents for a query.

**Request:**
```json
{
  "query": "What is machine learning?",
  "documents": ["doc1", "doc2", "doc3"],
  "model": "bge-small-en-v1.5",
  "k": 5,
  "score_threshold": 0.7
}
```

**Response:**
```json
{
  "query": "What is machine learning?",
  "results": [
    {
      "index": 0,
      "document": "doc1",
      "score": 0.92
    }
  ],
  "total_documents": 3,
  "metric": "cosine"
}
```

### 3. Text Similarity
```http
POST /api/v1/similarity
```

Compute similarity between two texts.

**Request:**
```json
{
  "text1": "I love machine learning",
  "text2": "AI is fascinating",
  "model": "all-mpnet-base-v2",
  "metric": "cosine"
}
```

**Response:**
```json
{
  "similarity": 0.87,
  "metric": "cosine",
  "text1_length": 25,
  "text2_length": 18
}
```

### 4. Document Reranking
```http
POST /api/v1/reranking
```

Rerank documents by relevance to query.

**Request:**
```json
{
  "query": "Python programming",
  "documents": ["doc1", "doc2", "doc3"],
  "model": "bge-reranker-base",
  "top_k": 5
}
```

**Response:**
```json
{
  "results": [
    {
      "index": 0,
      "document": "doc1",
      "score": 0.95
    }
  ],
  "total_tokens": 150
}
```

### 5. Clustering
```http
POST /api/v1/cluster
```

Cluster texts using embeddings.

**Request:**
```json
{
  "texts": ["text1", "text2", "text3"],
  "model": "all-MiniLM-L6-v2",
  "n_clusters": 2,
  "algorithm": "kmeans"
}
```

**Response:**
```json
{
  "labels": [0, 0, 1],
  "n_clusters": 2,
  "silhouette_score": 0.73,
  "algorithm": "kmeans"
}
```

**Supported Algorithms:**
- `kmeans` - Fast, requires knowing number of clusters
- `hierarchical` - Builds cluster tree
- `dbscan` - Density-based, finds outliers

### 6. Recommendations
```http
POST /api/v1/recommend
```

Content-based recommendations.

**Request:**
```json
{
  "items": ["item1", "item2", "item3"],
  "query_item_index": 0,
  "model": "all-mpnet-base-v2",
  "k": 5
}
```

**Response:**
```json
{
  "query_item": "item1",
  "recommendations": [
    {
      "item_id": "1",
      "item_index": 1,
      "similarity_score": 0.89,
      "metadata": {}
    }
  ]
}
```

### 7. List Embedding Models
```http
GET /api/v1/embedding-models?modality=text&use_case=semantic_search
```

Get curated list of embedding models.

**Response:**
```json
{
  "models": {
    "all-minilm-l6-v2": {
      "name": "All-MiniLM-L6-v2",
      "modality": "text",
      "dimension": 384,
      "use_cases": ["semantic_search", "similarity"],
      "size": "small",
      "backend": "onnx",
      "repo_id": "sentence-transformers/all-MiniLM-L6-v2",
      "description": "Fast, lightweight text embeddings"
    }
  },
  "total": 1,
  "filtered_by": "text"
}
```

### 8. Evaluate Embeddings
```http
POST /api/v1/evaluate-embeddings
```

Compute quality metrics for embeddings.

**Request:**
```json
{
  "embeddings1": [[0.1, 0.2], [0.3, 0.4]],
  "embeddings2": [[0.5, 0.6], [0.7, 0.8]],
  "metric_type": "average_similarity"
}
```

**Response:**
```json
{
  "metric": "average_similarity",
  "score": 0.82,
  "embedding_count": 2
}
```

## Code Modules

### 1. `core/embedding_eval.py`
- **Similarity metrics**: Cosine, dot product, Euclidean
- **RAG retrieval**: Semantic search with thresholds
- **Evaluation**: Quality metrics for embeddings

### 2. `core/embedding_models.py`
- **Model registry**: Curated list of top models
- **Model info**: Dimensions, use cases, backends
- **Recommendations**: Auto-select best model for task

### 3. `core/embedding_clustering.py`
- **K-Means**: Fast partitioning
- **Hierarchical**: Build cluster trees
- **DBSCAN**: Density-based, find outliers
- **Recommendation engine**: Content-based filtering

## Use Cases

### 1. Semantic Search
```python
# Generate embeddings
POST /api/v1/embeddings
{
  "input": ["query", "doc1", "doc2", "doc3"],
  "model": "bge-small-en-v1.5"
}

# Or use semantic search directly
POST /api/v1/semantic-search
{
  "query": "query",
  "documents": ["doc1", "doc2", "doc3"],
  "model": "bge-small-en-v1.5",
  "k": 2
}
```

### 2. RAG Pipeline
```python
# 1. Semantic search to retrieve
POST /api/v1/semantic-search
{
  "query": "user question",
  "documents": [...],
  "k": 10
}

# 2. Rerank for precision
POST /api/v1/reranking
{
  "query": "user question",
  "documents": [top 10 from search],
  "top_k": 3
}

# 3. Generate answer with context
POST /api/v1/chat/completions
{
  "messages": [
    {"role": "system", "content": "Use these docs: ..."},
    {"role": "user", "content": "user question"}
  ]
}
```

### 3. Document Clustering
```python
# Cluster documents
POST /api/v1/cluster
{
  "texts": [all documents],
  "model": "all-MiniLM-L6-v2",
  "n_clusters": 5,
  "algorithm": "kmeans"
}

# Result: cluster labels for each document
# Use for topic discovery, organization
```

### 4. Recommendation System
```python
# Item-based recommendations
POST /api/v1/recommend
{
  "items": [product descriptions],
  "query_item_index": 42,  # Product user is viewing
  "k": 10
}

# Returns: 10 most similar products
```

### 5. Duplicate Detection
```python
# Check if two texts are duplicates
POST /api/v1/similarity
{
  "text1": "first text",
  "text2": "second text",
  "metric": "cosine"
}

# If similarity > 0.95, likely duplicates
```

## Performance Guide

### Embedding Dimensions vs Speed/Quality

| Dimension | Speed | Quality | Use Case |
|-----------|-------|---------|----------|
| 384 | Fast | Good | Mobile, real-time |
| 512 | Fast | Very Good | Multimodal (CLIP) |
| 768 | Medium | Excellent | Production systems |
| 1024 | Slow | State-of-art | Research, high-end |

### Memory Estimation

For 1M vectors:
- 384d: ~1.5 GB
- 512d: ~2.0 GB
- 768d: ~3.0 GB
- 1024d: ~4.0 GB

Formula: `dimension × num_vectors × 4 bytes` (float32)

### Backend Support

| Backend | Text | Image | Audio |
|---------|------|-------|-------|
| ONNX Runtime | ✅ | ✅ | ⚠️ |
| llama.cpp | ✅ | ❌ | ❌ |
| MediaPipe | ✅ | ✅ | ✅ |
| BitNet | ✅ | ❌ | ❌ |

## Best Practices

### 1. Model Selection
- Start with `all-MiniLM-L6-v2` for prototyping (fast)
- Upgrade to `all-mpnet-base-v2` for production
- Use `bge-*` models for retrieval-heavy tasks
- Use CLIP for multi-modal

### 2. Similarity Thresholds
- **Cosine similarity:**
  - > 0.9: Very similar (likely duplicates)
  - 0.7-0.9: Related
  - 0.5-0.7: Somewhat related
  - < 0.5: Not related

### 3. Clustering
- Use `kmeans` when you know cluster count
- Use `dbscan` to find outliers/anomalies
- Use `hierarchical` to explore cluster structure

### 4. RAG Pipeline
1. **Retrieval**: Use fast model (384d) with `semantic-search`
2. **Reranking**: Use cross-encoder (`bge-reranker`) on top-K
3. **Generation**: Use retrieved docs as context

### 5. Batch Processing
- Batch embeddings for better throughput
- Max batch size: 32-64 texts (depending on GPU memory)
- Use streaming for large corpora

## Examples

### Python Client
```python
import requests

# Generate embeddings
response = requests.post("http://localhost:8000/api/v1/embeddings", json={
    "input": ["Hello world", "Good morning"],
    "model": "all-MiniLM-L6-v2"
})
embeddings = response.json()["embeddings"]

# Semantic search
response = requests.post("http://localhost:8000/api/v1/semantic-search", json={
    "query": "AI and ML",
    "documents": ["AI is cool", "I like pizza", "Machine learning rocks"],
    "model": "all-MiniLM-L6-v2",
    "k": 2
})
results = response.json()["results"]
print(f"Top result: {results[0]['document']} (score: {results[0]['score']:.3f})")
```

### JavaScript Client
```javascript
// Generate embeddings
const response = await fetch("http://localhost:8000/api/v1/embeddings", {
  method: "POST",
  headers: {"Content-Type": "application/json"},
  body: JSON.stringify({
    input: ["Hello world", "Good morning"],
    model: "all-MiniLM-L6-v2"
  })
});
const {embeddings} = await response.json();

// Recommendations
const recResponse = await fetch("http://localhost:8000/api/v1/recommend", {
  method: "POST",
  headers: {"Content-Type": "application/json"},
  body: JSON.stringify({
    items: ["product1", "product2", "product3"],
    query_item_index: 0,
    k: 5
  })
});
const {recommendations} = await recResponse.json();
```

## Roadmap

- [x] Text embeddings (ONNX, llama.cpp, MediaPipe)
- [x] Semantic search
- [x] Clustering (K-Means, Hierarchical, DBSCAN)
- [x] Recommendation engine
- [x] Reranking
- [x] Multi-modal (CLIP)
- [ ] Vector database integration
- [ ] Batch processing optimization
- [ ] Audio/video embeddings
- [ ] Fine-tuning support
- [ ] Quantized models (INT8)

## References

- [Sentence Transformers](https://www.sbert.net/)
- [BGE Models](https://github.com/FlagOpen/FlagEmbedding)
- [CLIP](https://github.com/openai/CLIP)
- [MTEB Leaderboard](https://huggingface.co/spaces/mteb/leaderboard)

