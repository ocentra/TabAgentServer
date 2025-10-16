"""
TabAgent Server - Main FastAPI Application

OpenAI-compatible API server for local AI inference with hardware-aware backend selection.
"""

import logging
from contextlib import asynccontextmanager
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from .routes import (
    chat,
    models,
    health,
    management,
    stats,
    system_info,
    generation_control,
    embeddings,
    reranking,
    rag,
    params,
    resources,
    chat_history,
)
from .backend_adapter import get_inference_adapter
from .backend_manager import get_backend_manager

logger = logging.getLogger(__name__)


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan manager - startup and shutdown"""
    # Startup
    logger.info("TabAgent Server starting...")
    
    # Initialize adapter (uses shared InferenceService)
    adapter = get_inference_adapter()
    backend_mgr = get_backend_manager()
    backend_mgr.set_backend(adapter, "tabagent")
    logger.info("Backend adapter initialized (shares logic with native_host.py)")
    
    yield
    
    # Shutdown
    logger.info("TabAgent Server shutting down...")


# Create FastAPI application
app = FastAPI(
    title="TabAgent Server",
    description="""
    ## 🚀 Hardware-Aware Inference Platform
    
    TabAgent provides an OpenAI-compatible API with multi-backend support for local AI inference.
    
    ### Key Features
    - **Multi-Backend Support**: BitNet, ONNX Runtime, llama.cpp, MediaPipe
    - **Hardware Auto-Detection**: Auto-selects best backend (GPU → NPU → CPU)
    - **Recipe System**: User-friendly model loading (onnx-npu, llama-cuda, bitnet-gpu, etc.)
    - **Embeddings & RAG**: Semantic search, clustering, recommendations
    - **Performance Tracking**: TTFT, TPS, token counts
    - **Native Messaging**: Browser extension integration
    
    ### Supported Backends
    - **BitNet** (1.58-bit) - Ultra-efficient CPU/GPU inference
    - **ONNX Runtime** - DirectML (AMD/Intel/NVIDIA), CUDA, NPU, CPU
    - **llama.cpp** - GGUF models with CUDA, Vulkan, Metal, ROCm, CPU
    - **MediaPipe** - Multimodal AI (vision, text, audio)
    
    ### OpenAI-Compatible
    - ✅ `/api/v1/chat/completions` - Chat API
    - ✅ `/api/v1/completions` - Text completion
    - ✅ `/api/v1/embeddings` - Embeddings generation
    
    ### Extended Features (Beyond OpenAI)
    - ✅ `/api/v1/semantic-search` - RAG retrieval
    - ✅ `/api/v1/reranking` - Document reranking
    - ✅ `/api/v1/cluster` - Document clustering
    - ✅ `/api/v1/recommend` - Content recommendations
    - ✅ `/api/v1/params` - Persistent parameter configuration
    - ✅ `/api/v1/system-info` - Hardware detection
    
    ### Quick Start
    ```bash
    # 1. Load a model
    curl -X POST http://localhost:8000/api/v1/load \\
      -H "Content-Type: application/json" \\
      -d '{"model_path": "path/to/model.gguf"}'
    
    # 2. Chat completion
    curl -X POST http://localhost:8000/api/v1/chat/completions \\
      -H "Content-Type: application/json" \\
      -d '{
        "model": "current",
        "messages": [{"role": "user", "content": "Hello!"}]
      }'
    ```
    """,
    version="1.0.0",
    lifespan=lifespan,
    terms_of_service="https://github.com/ocentra/TabAgent",
    contact={
        "name": "TabAgent Team",
        "url": "https://github.com/ocentra/TabAgent",
    },
    license_info={
        "name": "MIT License",
        "url": "https://opensource.org/licenses/MIT",
    },
    openapi_tags=[
        {
            "name": "health",
            "description": "Health check and server status"
        },
        {
            "name": "models",
            "description": "List available models and get model information"
        },
        {
            "name": "chat",
            "description": "OpenAI-compatible chat and text completions with streaming support"
        },
        {
            "name": "management",
            "description": "Model lifecycle management: pull, load, unload, delete"
        },
        {
            "name": "params",
            "description": "Generation parameter configuration (persistent across requests)"
        },
        {
            "name": "stats",
            "description": "Performance statistics: TTFT, TPS, token counts"
        },
        {
            "name": "system",
            "description": "System information: hardware detection, available engines"
        },
        {
            "name": "control",
            "description": "Generation control: halt in-progress generation"
        },
        {
            "name": "embeddings",
            "description": "Generate embeddings for text and images"
        },
        {
            "name": "reranking",
            "description": "Rerank documents by relevance to query"
        },
        {
            "name": "rag",
            "description": "RAG utilities: semantic search, clustering, recommendations"
        },
        {
            "name": "resources",
            "description": "Resource management: VRAM/RAM queries, multi-model support for agentic systems"
        },
    ],
)

# Add CORS middleware - allow all origins for local development
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Include routers
app.include_router(health.router, prefix="/api/v1", tags=["health"])
app.include_router(models.router, prefix="/api/v1", tags=["models"])
app.include_router(chat.router, prefix="/api/v1", tags=["chat"])
app.include_router(management.router, prefix="/api/v1", tags=["management"])
app.include_router(params.router, prefix="/api/v1", tags=["params"])
app.include_router(resources.router, prefix="/api/v1", tags=["resources"])
app.include_router(chat_history.router, prefix="/api/v1", tags=["chat-history", "sync"])
app.include_router(stats.router, prefix="/api/v1", tags=["stats"])
app.include_router(system_info.router, prefix="/api/v1", tags=["system"])
app.include_router(generation_control.router, prefix="/api/v1", tags=["control"])
app.include_router(embeddings.router, prefix="/api/v1", tags=["embeddings"])
app.include_router(reranking.router, prefix="/api/v1", tags=["reranking"])
app.include_router(rag.router, prefix="/api/v1", tags=["rag"])


@app.get("/")
async def root():
    """
    Root endpoint - Server info and getting started guide
    
    Shows server status and all available endpoints.
    Perfect starting point for exploring the API!
    """
    manager = get_backend_manager()
    model_loaded = manager.is_model_loaded()
    
    return {
        "name": "TabAgent Server",
        "version": "1.0.0",
        "status": "running",
        "description": "Hardware-aware inference platform with OpenAI-compatible API",
        "model_loaded": model_loaded,
        "getting_started": {
            "step_1": {
                "action": "Check available recipes",
                "endpoint": "GET /api/v1/recipes",
                "description": "See what hardware acceleration is available"
            },
            "step_2": {
                "action": "Load a model",
                "endpoint": "POST /api/v1/load",
                "example": {
                    "model": "path/to/your/model.gguf"
                },
                "description": "Required before chat/completions"
            },
            "step_3": {
                "action": "Chat with AI",
                "endpoint": "POST /api/v1/chat/completions",
                "example": {
                    "model": "current",
                    "messages": [{"role": "user", "content": "Hello!"}]
                },
                "description": "Start chatting!"
            }
        },
        "documentation": {
            "swagger_ui": "http://localhost:8000/docs",
            "redoc": "http://localhost:8000/redoc",
            "openapi_json": "http://localhost:8000/openapi.json"
        },
        "endpoints": {
            "health": "/api/v1/health",
            "system_info": "/api/v1/system-info",
            "models": "/api/v1/models",
            "chat": "/api/v1/chat/completions",
            "completions": "/api/v1/completions",
            "responses": "/api/v1/responses",
            "embeddings": "/api/v1/embeddings",
            "reranking": "/api/v1/reranking",
            "semantic_search": "/api/v1/semantic-search",
            "similarity": "/api/v1/similarity",
            "evaluate_embeddings": "/api/v1/evaluate-embeddings",
            "cluster": "/api/v1/cluster",
            "recommend": "/api/v1/recommend",
            "embedding_models": "/api/v1/embedding-models",
            "pull": "/api/v1/pull",
            "load": "/api/v1/load",
            "unload": "/api/v1/unload",
            "delete": "/api/v1/delete",
            "params": "/api/v1/params",
            "halt": "/api/v1/halt",
            "stats": "/api/v1/stats",
        }
    }


if __name__ == "__main__":
    import uvicorn
    
    # Configure logging
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s [%(levelname)s] %(message)s"
    )
    
    # Run server
    uvicorn.run(
        app,
        host="0.0.0.0",
        port=8000,
        log_level="info"
    )

