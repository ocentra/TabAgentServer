"""
Model Management Endpoints
"""

from fastapi import APIRouter
import time

from ..constants import OpenAIObject
from ..types import ModelListResponse, ModelInfo
from ..backend_manager import get_backend_manager

router = APIRouter()


@router.get("/models", response_model=ModelListResponse)
async def list_models():
    """
    List available models
    
    OpenAI-compatible endpoint that returns available models.
    Uses existing ModelLibrary from models/ directory.
    """
    from models import ModelLibrary
    
    library = ModelLibrary()
    available_models = library.list_models()
    
    # Convert to OpenAI format
    model_list = [
        ModelInfo(
            id=model.id,
            created=int(time.time()),
            owned_by="tabagent"
        )
        for model in available_models
    ]
    
    return ModelListResponse(data=model_list)


@router.get("/models/{model_id}", response_model=ModelInfo)
async def get_model(model_id: str):
    """
    Get specific model information
    
    Args:
        model_id: Model identifier
        
    Returns:
        Model information
    """
    from models import ModelLibrary
    
    library = ModelLibrary()
    model = library.get_model(model_id)
    
    if not model:
        # Return generic info if not found
        return ModelInfo(
            id=model_id,
            created=int(time.time()),
            owned_by="tabagent"
        )
    
    return ModelInfo(
        id=model.id,
        created=int(time.time()),
        owned_by="tabagent"
    )
