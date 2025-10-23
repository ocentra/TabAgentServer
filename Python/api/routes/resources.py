"""
Resource Management Endpoints
==============================

VRAM/RAM management for agentic multi-model systems.

Provides:
- GET /api/v1/resources - Query available VRAM/RAM
- POST /api/v1/resources/estimate - Estimate model memory
- GET /api/v1/models/loaded - List loaded models
- POST /api/v1/models/select - Select active model

Critical for:
- Orchestrator agents managing multiple worker agents
- Resource allocation before loading
- Multi-model inference scenarios

Related Files:
- core/resource_manager.py - VRAM/RAM tracking
- core/model_tracker.py - Multi-model state
- core/unified_handler.py - Request handling
"""

import logging
from typing import Dict, Any, Optional
from fastapi import APIRouter, HTTPException, status
from pydantic import BaseModel, Field

from Python.core.unified_handler import get_unified_handler
from ..constants import ErrorCode

logger = logging.getLogger(__name__)

router = APIRouter()


class ResourceMessages:
    """Messages for resource operations (no string literals)"""
    RESOURCES_QUERIED = "Resources queried successfully"
    MODEL_ESTIMATED = "Model size estimated"
    MODEL_SELECTED = "Active model selected"
    MODELS_LISTED = "Loaded models listed"


class EstimateRequest(BaseModel):
    """Request to estimate model size"""
    model_path: str = Field(..., description="Path to model file", examples=["C:/models/phi-3.5-mini.gguf"])
    
    model_config = {
        "json_schema_extra": {
            "examples": [
                {"model_path": "C:/models/phi-3.5-mini.gguf"}
            ]
        }
    }


class SelectModelRequest(BaseModel):
    """Request to select active model"""
    model_id: str = Field(..., description="Model ID to activate", examples=["model-1", "model-2"])
    
    model_config = {
        "json_schema_extra": {
            "examples": [
                {"model_id": "model-1"}
            ]
        }
    }


# [ENDPOINT] GET /api/v1/resources - Query available VRAM/RAM
@router.get(
    "/resources",
    summary="Query Resources",
    description="""
    ## Query available VRAM and RAM
    
    **Critical for agentic systems!**
    
    Orchestrator agents should query this BEFORE loading models to:
    - Check if enough VRAM/RAM available
    - See what's already allocated
    - Plan model loading strategy
    
    ### Use Case Example:
    ```
    Orchestrator: "I need 3GB for worker agent"
    1. Query resources → 4GB VRAM available
    2. Estimate model size → 2.8GB needed
    3. Load with full_vram strategy
    4. Orchestrator continues with 1.2GB VRAM left
    ```
    
    ### Response Fields:
    - `resources.vram.available_mb` - Available VRAM
    - `resources.ram.available_mb` - Available RAM
    - `allocations` - Per-model resource usage
    - `loaded_models_count` - How many models loaded
    """,
    responses={
        200: {
            "description": "Resource status",
            "content": {
                "application/json": {
                    "example": {
                        "resources": {
                            "vram": {
                                "total_mb": 8192,
                                "used_mb": 2048,
                                "available_mb": 6144
                            },
                            "ram": {
                                "total_mb": 16384,
                                "used_mb": 8192,
                                "available_mb": 8192
                            },
                            "gpu_count": 1
                        },
                        "allocations": {
                            "model-1": {
                                "vram_mb": 2048,
                                "ram_mb": 0,
                                "backend": "ONNX_NPU"
                            }
                        },
                        "loaded_models_count": 1,
                        "by_backend": {"ONNX_NPU": 1}
                    }
                }
            }
        }
    }
)
async def query_resources():
    """Query available VRAM/RAM resources"""
    try:
        handler = get_unified_handler()
        result = handler.query_resources()
        return result
    
    except Exception as e:
        logger.error(f"Query resources failed: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.BACKEND_ERROR.value,
                }
            }
        )


# [ENDPOINT] POST /api/v1/resources/estimate - Estimate model memory requirements
@router.post(
    "/resources/estimate",
    summary="Estimate Model Size",
    description="""
    ## Estimate memory requirements and get offload options
    
    **Query this BEFORE loading a model!**
    
    Returns:
    - Estimated model size
    - Available VRAM/RAM
    - Suggested offload strategies
    
    ### Offload Strategies:
    1. **full_vram**: All layers on GPU (fastest)
    2. **hybrid**: Split between GPU and RAM (balanced)
    3. **full_ram**: All layers on RAM (slowest, no VRAM)
    
    ### Agentic Workflow:
    ```
    1. Estimate model → Get options
    2. Agent picks strategy based on needs
    3. Load with chosen strategy
    4. Agent gets exactly what it requested
    ```
    
    ### Response Example:
    ```json
    {
      "can_load": true,
      "options": [
        {
          "strategy": "hybrid",
          "vram_layers": 20,
          "ram_layers": 12,
          "vram_mb": 2000,
          "ram_mb": 1000,
          "speed": "medium",
          "description": "20 layers GPU, 12 layers RAM"
        }
      ]
    }
    ```
    """,
    responses={
        200: {
            "description": "Model size estimate with offload options"
        },
        404: {
            "description": "Model file not found"
        }
    }
)
async def estimate_model_size(request: EstimateRequest):
    """Estimate model memory requirements"""
    try:
        handler = get_unified_handler()
        result = handler.estimate_model_size(request.model_path)
        
        if "error" in result:
            raise HTTPException(
                status_code=status.HTTP_404_NOT_FOUND,
                detail={
                    "error": {
                        "message": result["error"],
                        "type": ErrorCode.INVALID_MODEL.value,
                    }
                }
            )
        
        return result
    
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Estimate model size failed: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.BACKEND_ERROR.value,
                }
            }
        )


# [ENDPOINT] GET /api/v1/models/loaded - List all loaded models
@router.get(
    "/models/loaded",
    summary="List Loaded Models",
    description="""
    ## List all currently loaded models
    
    **Multi-model support for agentic systems!**
    
    Shows:
    - All loaded models with IDs
    - Which model is active
    - Resource allocation per model
    - Backend per model
    
    ### Use Cases:
    - Orchestrator sees what workers are running
    - Check which model to use for task
    - Monitor resource usage
    - Manage multiple loaded models
    """,
    responses={
        200: {
            "description": "List of loaded models",
            "content": {
                "application/json": {
                    "example": {
                        "models": [
                            {
                                "model_id": "model-1",
                                "model_path": "C:/models/phi-3.gguf",
                                "backend": "ONNX_NPU",
                                "state": "loaded",
                                "resources": {
                                    "vram_mb": 2048,
                                    "ram_mb": 0,
                                    "vram_layers": 32,
                                    "ram_layers": 0
                                }
                            }
                        ],
                        "total": 1,
                        "active_model_id": "model-1",
                        "by_backend": {"ONNX_NPU": 1}
                    }
                }
            }
        }
    }
)
async def list_loaded_models():
    """List all currently loaded models"""
    try:
        handler = get_unified_handler()
        result = handler.list_loaded_models()
        return result
    
    except Exception as e:
        logger.error(f"List loaded models failed: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.BACKEND_ERROR.value,
                }
            }
        )


# [ENDPOINT] POST /api/v1/models/select - Select active model for inference
@router.post(
    "/models/select",
    summary="Select Active Model",
    description="""
    ## Switch between loaded models
    
    When multiple models are loaded, use this to select which one to use.
    
    ### Multi-Model Scenario:
    ```
    Loaded models:
    - model-1: Orchestrator (Phi-3.5, ONNX NPU)
    - model-2: Code Agent (CodeLlama, llama.cpp CUDA)
    - model-3: Chat Agent (Llama-3.2, BitNet GPU)
    
    Switch to model-2 for code generation:
    POST /api/v1/models/select {"model_id": "model-2"}
    
    Now /chat/completions uses model-2
    ```
    """,
    responses={
        200: {
            "description": "Model selected",
            "content": {
                "application/json": {
                    "example": {
                        "active_model_id": "model-2",
                        "backend": "LLAMA_CPP_CUDA",
                        "message": "Active model set"
                    }
                }
            }
        },
        404: {
            "description": "Model not found"
        }
    }
)
async def select_active_model(request: SelectModelRequest):
    """Select which model to use for inference"""
    try:
        handler = get_unified_handler()
        result = handler.select_active_model(request.model_id)
        result["message"] = f"Active model set to: {request.model_id}"
        return result
    
    except ValueError as e:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.INVALID_MODEL.value,
                }
            }
        )
    except Exception as e:
        logger.error(f"Select active model failed: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.BACKEND_ERROR.value,
                }
            }
        )

