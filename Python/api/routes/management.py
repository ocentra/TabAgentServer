"""
Model Management Endpoints
==========================

Model lifecycle management with recipe-based configuration.

Provides:
- POST /api/v1/models/pull (or /pull) - Download from HuggingFace with recipe registration
- POST /api/v1/models/load (or /load) - Load model into memory
- POST /api/v1/models/unload (or /unload) - Unload from memory
- POST /api/v1/models/delete (or /delete) - Delete from disk
- GET /api/v1/recipes - List available recipes
- GET /api/v1/models/registered - List registered models

Recipe System:
- User-friendly model configurations (onnx-npu, llama-cuda, bitnet-gpu, etc.)
- Auto-maps to correct backend and acceleration
- Inspired by Lemonade SDK

Related Files:
- models/model_manager.py - Core model operations
- models/model_registry.py - Model registration with recipes and capabilities
- core/recipe_types.py - Recipe definitions and backend mapping
- core/inference_service.py - Model loading logic
"""

from fastapi import APIRouter, HTTPException, status
import logging
from typing import Dict, Any

from ..types import (
    ModelPullRequest,
    ModelLoadRequest,
    ModelUnloadRequest,
    ModelDeleteRequest,
    ModelOperationResponse,
)
from ..constants import ErrorCode
from Python.core.inference_service import get_inference_service
from Python.core.recipe_types import ModelCapabilities as ModelCapabilitiesDataclass
from Python.models import ModelManager

logger = logging.getLogger(__name__)

router = APIRouter()


class ManagementMessages:
    """Messages for model management operations (no string literals)"""
    MODEL_DOWNLOADED = "Model downloaded successfully"
    MODEL_LOADED = "Model loaded successfully"
    MODEL_UNLOADED = "Model unloaded successfully"
    MODEL_DELETED = "Model deleted successfully"
    NO_MODEL_LOADED = "No model loaded"
    DOWNLOAD_FAILED = "Failed to download model"
    LOAD_FAILED = "Failed to load model"
    DELETE_FAILED = "Failed to delete model"
    MODEL_NOT_FOUND = "Model not found"


# [ENDPOINT] POST /api/v1/models/pull - Download and register model with recipe
# [FEATURE] Recipe System - User-friendly model configuration
# [FEATURE] Model Registration - Register custom models with metadata
@router.post("/models/pull", response_model=ModelOperationResponse)
async def pull_model(request: ModelPullRequest):
    """
    Download and register a model from HuggingFace.
    
    Supports recipe-based model configuration (inspired by Lemonade).
    
    Args:
        request: Model pull request with optional recipe and capabilities
        
    Returns:
        Success message with model info
        
    Raises:
        HTTPException: If download or registration fails
    """
    logger.info(f"Pull request for model: {request.model}, recipe: {request.recipe}, name: {request.model_name}")
    
    try:
        # Download the model
        manager = ModelManager()
        success = manager.download_model(
            model_name=request.model,
            variant=request.variant
        )
        
        if not success:
            raise HTTPException(
                status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
                detail={
                    "error": {
                        "message": f"Failed to download model: {request.model}",
                        "type": ErrorCode.BACKEND_ERROR.value,
                    }
                }
            )
        
        # Model downloaded successfully
        # (Legacy registry system removed - models are auto-detected by Rust)
        message = f"Model {request.model} downloaded successfully"
        
        return ModelOperationResponse(
            success=True,
            message=message,
            model=request.model_name or request.model
        )
    
    except ValueError as e:
        logger.error(f"Model not found: {e}")
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
        logger.error(f"Pull error: {e}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.BACKEND_ERROR.value,
                }
            }
        )


# [ENDPOINT] POST /api/v1/models/load - Load model with optional recipe
# [FEATURE] Recipe System - Load registered models by name or path
@router.post(
    "/models/load",
    response_model=ModelOperationResponse,
    summary="Load Model",
    description="""
    ## Load a model for inference
    
    **This is the FIRST step before using chat/completions!**
    
    ### Three Ways to Load:
    
    1️⃣ **By Registered Name** (easiest):
    ```json
    {
      "model": "Phi-3.5-Mini-NPU"
    }
    ```
    See registered models: `GET /api/v1/models/registered`
    
    2️⃣ **By File Path** (auto-detects backend):
    ```json
    {
      "model": "C:/models/my-model.gguf"
    }
    ```
    Supports: `.gguf`, `.onnx`, `.task` files
    
    3️⃣ **By File Path + Recipe** (explicit control):
    ```json
    {
      "model": "C:/models/my-model.onnx",
      "recipe": "onnx-npu"
    }
    ```
    See available recipes: `GET /api/v1/recipes`
    
    ### What Happens:
    - Detects model format (.gguf, .onnx, .task)
    - Selects best backend (ONNX, llama.cpp, BitNet, MediaPipe)
    - Selects best acceleration (GPU > NPU > CPU)
    - Loads model into memory
    - Ready for `/chat/completions`!
    
    ### After Loading:
    You can use any generation endpoint:
    - `POST /api/v1/chat/completions`
    - `POST /api/v1/completions`
    - `POST /api/v1/embeddings` (if model supports it)
    """,
    responses={
        200: {
            "description": "Model loaded successfully",
            "content": {
                "application/json": {
                    "example": {
                        "success": True,
                        "message": "Model loaded successfully",
                        "model": "Phi-3.5-Mini-NPU",
                        "backend": "ONNX_NPU"
                    }
                }
            }
        },
        404: {
            "description": "Model not found",
            "content": {
                "application/json": {
                    "example": {
                        "error": {
                            "message": "Model file not found: path/to/model.gguf",
                            "type": "invalid_model",
                            "hint": "Check the model path or pull it first: POST /api/v1/pull"
                        }
                    }
                }
            }
        },
        500: {
            "description": "Load failed",
            "content": {
                "application/json": {
                    "example": {
                        "error": {
                            "message": "Failed to load model: Unsupported format",
                            "type": "backend_error",
                            "hint": "Supported formats: .gguf, .onnx, .task"
                        }
                    }
                }
            }
        }
    }
)
async def load_model(request: ModelLoadRequest):
    """
    Load model into inference service.
    
    Supports:
    - Loading by registered name (e.g., "Phi-3.5-Mini-NPU")
    - Loading by file path with optional recipe
    - Auto-detection if no recipe specified
    
    Args:
        request: Model load request with optional recipe
        
    Returns:
        Success message with backend info
    """
    logger.info(f"Load request for model: {request.model}, recipe: {request.recipe}")
    
    try:
        service = get_inference_service()
        model_path = request.model
        
        # Load model (uses Rust detection + pipelines, no registry needed)
        result = service.load_model(model_path)
        
        if result["status"] == "success":
            return ModelOperationResponse(
                success=True,
                message=f"Model loaded successfully",
                model=request.model,
                backend=service.get_backend_type()
            )
        else:
            raise HTTPException(
                status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
                detail={
                    "error": {
                        "message": result.get("message", "Unknown error"),
                        "type": ErrorCode.BACKEND_ERROR.value,
                    }
                }
            )
    
    except Exception as e:
        logger.error(f"Load error: {e}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.BACKEND_ERROR.value,
                }
            }
        )


# [ENDPOINT] POST /api/v1/models/unload - Unload model from memory
@router.post("/models/unload", response_model=ModelOperationResponse)
async def unload_model(request: ModelUnloadRequest):
    """
    Unload current model.
    Uses existing manager.unload_model()
    """
    logger.info("Unload request")
    
    try:
        service = get_inference_service()
        manager = service.get_active_manager()
        
        if manager and hasattr(manager, 'unload_model'):
            manager.unload_model()
            
            return ModelOperationResponse(
                success=True,
                message="Model unloaded successfully"
            )
        else:
            return ModelOperationResponse(
                success=True,
                message="No model loaded"
            )
    
    except Exception as e:
        logger.error(f"Unload error: {e}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.BACKEND_ERROR.value,
                }
            }
        )


@router.post("/models/delete", response_model=ModelOperationResponse)
async def delete_model(request: ModelDeleteRequest):
    """
    Delete a downloaded model.
    
    Removes model files from local storage.
    
    Args:
        request: Model delete request with model identifier
        
    Returns:
        Operation response
    """
    logger.info(f"Delete request for model: {request.model}")
    
    try:
        manager = ModelManager()
        
        # Check if model exists
        if not manager.is_model_downloaded(request.model):
            raise HTTPException(
                status_code=status.HTTP_404_NOT_FOUND,
                detail={
                    "error": {
                        "message": ManagementMessages.MODEL_NOT_FOUND,
                        "type": ErrorCode.INVALID_MODEL.value,
                    }
                }
            )
        
        # Delete model
        success = manager.delete_model(request.model)
        
        if success:
            return ModelOperationResponse(
                success=True,
                message=ManagementMessages.MODEL_DELETED,
                model=request.model
            )
        else:
            raise HTTPException(
                status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
                detail={
                    "error": {
                        "message": ManagementMessages.DELETE_FAILED,
                        "type": ErrorCode.BACKEND_ERROR.value,
                    }
                }
            )
    
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Delete error: {e}")
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.BACKEND_ERROR.value,
                }
            }
        )


# Simplified route aliases (without /models/ prefix)

@router.post("/pull", response_model=ModelOperationResponse)
async def pull_model_simple(request: ModelPullRequest):
    """Pull model (simplified route)"""
    return await pull_model(request)


@router.post("/load", response_model=ModelOperationResponse)
async def load_model_simple(request: ModelLoadRequest):
    """Load model (simplified route)"""
    return await load_model(request)


@router.post("/unload", response_model=ModelOperationResponse)
async def unload_model_simple(request: ModelUnloadRequest):
    """Unload model (simplified route)"""
    return await unload_model(request)


@router.post("/delete", response_model=ModelOperationResponse)
async def delete_model_simple(request: ModelDeleteRequest):
    """Delete model (simplified route)"""
    return await delete_model(request)


@router.get("/recipes")
async def list_recipes():
    """
    List available recipes.
    
    Recipes define how models are loaded (backend + acceleration).
    Inspired by Lemonade's recipe system.
    
    Returns:
        Dictionary of available recipes with descriptions
    """
    from core.recipe_types import RecipeRegistry
    
    recipes = RecipeRegistry.get_all_recipes()
    
    return {
        "recipes": [
            {
                "recipe": info.recipe.value,
                "backend": info.backend.value,
                "acceleration": info.acceleration.value,
                "file_format": info.file_format,
                "description": info.description,
                "hardware_required": info.hardware_required,
                "os_support": info.os_support
            }
            for info in recipes
        ],
        "total": len(recipes)
    }


@router.get("/models/registered")
async def list_registered_models():
    """
    List all available models from catalog.
    
    Returns models from curated library (replaces legacy registry).
    
    Returns:
        Dictionary of available models
    """
    from models import ModelLibrary
    
    library = ModelLibrary()
    all_models = library.list_models()
    
    return {
        "models": {
            model.name: {
                "repo": model.repo,
                "type": model.model_type.value,
                "description": model.description,
                "size_gb": model.size_gb,
                "context_length": model.context_length,
                "recommended": model.recommended,
                "variants": model.variants,
                "license": model.license.value,
                "use_cases": [uc.value for uc in model.use_cases]
            }
            for model in all_models
        },
        "total": len(all_models)
    }

