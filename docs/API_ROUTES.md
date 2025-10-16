# TabAgent API Routes Reference

Comprehensive list of all API endpoints for easy discovery and exploration.

## 🔍 Quick Navigation

- [Health & Status](#health--status)
- [Chat & Completions](#chat--completions)
- [Model Management](#model-management)
- [Generation Parameters](#generation-parameters)
- [Embeddings & RAG](#embeddings--rag)
- [System & Control](#system--control)

---

## Health & Status

### GET `/api/v1/health`
**File:** `api/routes/health.py`  
**Description:** Server health check and model status  
**Returns:** Server status, loaded model info

---

## Chat & Completions

### POST `/api/v1/chat/completions`
**File:** `api/routes/chat.py` → `chat_completions()`  
**Description:** OpenAI-compatible chat completion  
**Streaming:** ✅ Supported  
**Request:**
```json
{
  "model": "current",
  "messages": [{"role": "user", "content": "Hello"}],
  "stream": false
}
```

### POST `/api/v1/completions`
**File:** `api/routes/chat.py` → `completions()`  
**Description:** OpenAI-compatible text completion  
**Streaming:** ✅ Supported  
**Request:**
```json
{
  "model": "current",
  "prompt": "Hello world",
  "stream": false
}
```

### POST `/api/v1/responses`
**File:** `api/routes/chat.py` → `responses()`  
**Description:** Alternative completion format (Lemonade-style)  
**Streaming:** ✅ Supported

---

## Model Management

### POST `/api/v1/models/pull` or `/api/v1/pull`
**File:** `api/routes/management.py` → `pull_model()`  
**Description:** Download model from HuggingFace with recipe registration  
**Recipe Support:** ✅  
**Request:**
```json
{
  "model": "microsoft/Phi-3.5-mini-instruct",
  "recipe": "onnx-npu",
  "model_name": "Phi-3.5-Mini-NPU",
  "capabilities": {
    "reasoning": false,
    "vision": false
  }
}
```

### POST `/api/v1/models/load` or `/api/v1/load`
**File:** `api/routes/management.py` → `load_model()`  
**Description:** Load model into inference service  
**Recipe Support:** ✅  
**Request:**
```json
{
  "model": "Phi-3.5-Mini-NPU",
  "recipe": "onnx-npu"
}
```

### POST `/api/v1/models/unload` or `/api/v1/unload`
**File:** `api/routes/management.py` → `unload_model()`  
**Description:** Unload current model from memory  
**Request:** None

### POST `/api/v1/models/delete` or `/api/v1/delete`
**File:** `api/routes/management.py` → `delete_model()`  
**Description:** Delete model from disk  
**Request:**
```json
{
  "model_id": "model-name-to-delete"
}
```

### GET `/api/v1/models`
**File:** `api/routes/models.py` → `list_models()`  
**Description:** List available models  
**Returns:** All loaded and available models

### GET `/api/v1/recipes`
**File:** `api/routes/management.py` → `list_recipes()`  
**Description:** List available recipes (onnx-npu, llama-cuda, bitnet-gpu, etc.)  
**Returns:** All recipes with backend/hardware requirements

### GET `/api/v1/models/registered`
**File:** `api/routes/management.py` → `list_registered_models()`  
**Description:** List registered models (system + user)  
**Returns:** Models with recipes and capabilities

---

## Generation Parameters

### POST `/api/v1/params`
**File:** `api/routes/params.py` → `set_params()`  
**Description:** Set generation parameters (persistent across requests)  
**Inspired by:** Lemonade `/api/v1/params`  
**Request:**
```json
{
  "temperature": 0.8,
  "top_p": 0.95,
  "max_length": 1000
}
```

### GET `/api/v1/params`
**File:** `api/routes/params.py` → `get_params()`  
**Description:** Get current generation parameters  
**Returns:** Current temperature, top_p, max_tokens, etc.

---

## Embeddings & RAG

### POST `/api/v1/embeddings`
**File:** `api/routes/embeddings.py` → `generate_embeddings()`  
**Description:** Generate embeddings for text/images  
**OpenAI Compatible:** ✅  
**Request:**
```json
{
  "input": ["text1", "text2"],
  "model": "all-MiniLM-L6-v2"
}
```

### POST `/api/v1/semantic-search`
**File:** `api/routes/rag.py` → `semantic_search()`  
**Description:** Semantic search for RAG retrieval  
**Request:**
```json
{
  "query": "search query",
  "documents": ["doc1", "doc2"],
  "model": "bge-small-en-v1.5",
  "k": 5
}
```

### POST `/api/v1/reranking` or `/api/v1/rerank`
**File:** `api/routes/reranking.py` → `rerank_documents()`  
**Description:** Rerank documents by relevance  
**Request:**
```json
{
  "query": "query",
  "documents": ["doc1", "doc2"],
  "top_k": 5
}
```

### POST `/api/v1/cluster`
**File:** `api/routes/rag.py` → `cluster_embeddings()`  
**Description:** Cluster documents using embeddings  
**Algorithms:** kmeans, hierarchical, dbscan  
**Request:**
```json
{
  "texts": ["text1", "text2"],
  "model": "all-MiniLM-L6-v2",
  "n_clusters": 3,
  "algorithm": "kmeans"
}
```

### POST `/api/v1/recommend`
**File:** `api/routes/rag.py` → `recommend_items()`  
**Description:** Content-based recommendations  
**Request:**
```json
{
  "items": ["item1", "item2"],
  "query_item_index": 0,
  "k": 5
}
```

### POST `/api/v1/similarity`
**File:** `api/routes/rag.py` → `compute_similarity()`  
**Description:** Compute similarity between two texts  
**Request:**
```json
{
  "text1": "first text",
  "text2": "second text",
  "metric": "cosine"
}
```

### GET `/api/v1/embedding-models`
**File:** `api/routes/rag.py` → `list_embedding_models()`  
**Description:** List curated embedding models  
**Query Params:** `modality`, `use_case`

### POST `/api/v1/evaluate-embeddings`
**File:** `api/routes/rag.py` → `evaluate_embeddings()`  
**Description:** Compute quality metrics for embeddings

---

## System & Control

### GET `/api/v1/system-info`
**File:** `api/routes/system_info.py` → `get_system_info()`  
**Description:** System hardware and available inference engines  
**Returns:** OS, CPU, RAM, GPU, VRAM, available backends

### GET `/api/v1/stats`
**File:** `api/routes/stats.py` → `get_stats()`  
**Description:** Performance statistics (TTFT, TPS, token counts)  
**Returns:** Last generation metrics

### GET `/api/v1/halt` or POST `/api/v1/halt`
**File:** `api/routes/generation_control.py` → `halt_generation()`  
**Description:** Stop in-progress generation  
**Returns:** Halt status

---

## 📁 File Organization Map

```
Server/api/
├── main.py                          # FastAPI app, router registration
├── types.py                         # Request/response Pydantic models
├── constants.py                     # Enums and constants
├── backend_manager.py               # Backend coordination
├── backend_adapter.py               # Backend adapter
│
└── routes/
    ├── __init__.py                  # Export all route modules
    ├── chat.py                      # Chat, completions, responses
    ├── models.py                    # List models
    ├── management.py                # Pull, load, unload, delete, recipes, registered models
    ├── params.py                    # Generation parameter configuration ⭐ NEW
    ├── health.py                    # Health check
    ├── stats.py                     # Performance stats
    ├── system_info.py               # System information
    ├── generation_control.py        # Halt generation
    ├── embeddings.py                # Embeddings generation
    ├── reranking.py                 # Document reranking
    └── rag.py                       # RAG: semantic search, clustering, recommendations
```

---

## 🔍 Search Patterns for Discovery

To find specific functionality, search for:

### Find Endpoint Implementations:
```bash
grep -r "@router.post\|@router.get" Server/api/routes/
```

### Find Request Types:
```bash
grep -r "class.*Request.*BaseModel" Server/api/types.py
```

### Find Response Types:
```bash
grep -r "class.*Response.*BaseModel" Server/api/types.py
```

### Find Features by Keyword:
```bash
# Find embedding-related code
grep -r "embedding" Server/

# Find recipe-related code  
grep -r "recipe" Server/

# Find model management
grep -r "pull_model\|load_model\|delete_model" Server/
```

---

## 🎯 Related Files Guide

| Feature | Main File | Related Files |
|---------|-----------|---------------|
| Chat Completions | `routes/chat.py` | `types.py`, `backend_manager.py`, `backend_adapter.py` |
| Model Management | `routes/management.py` | `models/model_manager.py`, `models/model_registry.py` |
| Recipes | `routes/management.py` | `core/recipe_types.py`, `models/model_registry.py` |
| Embeddings | `routes/embeddings.py` | `core/embedding_eval.py`, `core/embedding_models.py` |
| RAG | `routes/rag.py` | `core/embedding_eval.py`, `core/embedding_clustering.py` |
| Parameters | `routes/params.py` | `backend_manager.py`, `core/message_types.py` |
| System Info | `routes/system_info.py` | `hardware/system_info_builder.py`, `hardware/engine_detection.py` |

---

## 🚀 Quick Start for Developers

### 1. Explore API via Swagger:
```
http://localhost:8000/docs         # Interactive Swagger UI
http://localhost:8000/redoc        # Alternative documentation
http://localhost:8000/openapi.json # Machine-readable spec
```

### 2. Test an Endpoint:
```bash
# List all endpoints
curl http://localhost:8000/

# Get available recipes
curl http://localhost:8000/api/v1/recipes

# Get system info
curl http://localhost:8000/api/v1/system-info
```

### 3. Find Code for Feature:
```bash
# Where is semantic search implemented?
grep -r "semantic.search" Server/

# Where are recipes defined?
grep -r "RecipeType" Server/

# Where is model pulling handled?
grep -r "def pull_model" Server/
```

---

## 📊 Endpoint Grouping by Tag

### health
- GET `/api/v1/health`

### models
- GET `/api/v1/models`

### chat
- POST `/api/v1/chat/completions`
- POST `/api/v1/completions`
- POST `/api/v1/responses`

### management
- POST `/api/v1/models/pull` or `/pull`
- POST `/api/v1/models/load` or `/load`
- POST `/api/v1/models/unload` or `/unload`
- POST `/api/v1/models/delete` or `/delete`
- GET `/api/v1/recipes`
- GET `/api/v1/models/registered`

### params
- POST `/api/v1/params`
- GET `/api/v1/params`

### stats
- GET `/api/v1/stats`

### system
- GET `/api/v1/system-info`

### control
- GET `/api/v1/halt`
- POST `/api/v1/halt`

### embeddings
- POST `/api/v1/embeddings`

### reranking
- POST `/api/v1/reranking`
- POST `/api/v1/rerank`

### rag
- POST `/api/v1/semantic-search`
- POST `/api/v1/similarity`
- POST `/api/v1/cluster`
- POST `/api/v1/recommend`
- GET `/api/v1/embedding-models`
- POST `/api/v1/evaluate-embeddings`

---

## 💡 Implementation Notes

### Recipe System
- **What:** User-friendly model loading configurations
- **File:** `core/recipe_types.py`
- **Examples:** `onnx-npu`, `llama-cuda`, `bitnet-gpu`
- **Inspired by:** Lemonade SDK

### Model Registry
- **What:** Pre-configured and user-registered models
- **File:** `models/model_registry.py`
- **System models:** Built-in, optimized configurations
- **User models:** Custom registrations with `user.` prefix

### Performance Tracking
- **What:** TTFT, TPS, token count metrics
- **File:** `core/performance_tracker.py`
- **Used in:** All backend managers

### Embeddings Framework
- **What:** Complete RAG and embedding support
- **Files:** `core/embedding_eval.py`, `core/embedding_models.py`, `core/embedding_clustering.py`
- **Features:** Semantic search, clustering, recommendations

---

**Last Updated:** 2025-10-15  
**Version:** 1.0.0  
**Auto-generated:** No - Manually maintained

