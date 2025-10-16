"""
Generation Parameters Endpoint
==============================

Dedicated endpoint for setting/getting generation parameters.
Inspired by Lemonade's /api/v1/params endpoint.

Provides:
- POST /api/v1/params - Set generation parameters (persistent)
- GET /api/v1/params - Get current parameters

Benefits:
- Set parameters once, use across multiple requests
- Separate parameter configuration from inference
- Test different configurations easily

Related Files:
- api/backend_manager.py - Manages global settings
- core/message_types.py - InferenceSettings definition
- api/types.py - Request/response types

Usage Example:
    # Set parameters
    POST /api/v1/params
    {
      "temperature": 0.8,
      "top_p": 0.95,
      "max_length": 1000
    }
    
    # Then use in completions
    POST /api/v1/chat/completions
    {
      "messages": [...]
      # Uses parameters set above
    }
"""

import logging
from typing import Dict, Any, Optional
from fastapi import APIRouter, HTTPException, status
from pydantic import BaseModel, Field

from ..backend_manager import get_backend_manager
from ..constants import ErrorCode
from core.message_types import InferenceSettings

logger = logging.getLogger(__name__)

router = APIRouter()


class ParamsMessages:
    """Messages for params operations (no string literals)"""
    PARAMS_SET = "Generation parameters set successfully"
    PARAMS_GET = "Current generation parameters"
    NO_BACKEND = "No backend available"


class ParamsRequest(BaseModel):
    """
    Request to set generation parameters.
    
    All parameters are optional - only specified params will be updated.
    """
    temperature: Optional[float] = Field(
        None,
        ge=0.0,
        le=2.0,
        description="Sampling temperature (0=deterministic, 2=very random)",
        examples=[0.7, 0.8, 1.0]
    )
    top_p: Optional[float] = Field(
        None,
        ge=0.0,
        le=1.0,
        description="Nucleus sampling threshold",
        examples=[0.9, 0.95, 1.0]
    )
    top_k: Optional[int] = Field(
        None,
        ge=1,
        description="Top-K sampling (limit to K most likely tokens)",
        examples=[40, 50, 100]
    )
    min_length: Optional[int] = Field(
        None,
        ge=0,
        description="Minimum output length in tokens",
        examples=[0, 10, 50]
    )
    max_length: Optional[int] = Field(
        None,
        ge=1,
        description="Maximum output length in tokens",
        examples=[512, 1024, 2048]
    )
    max_new_tokens: Optional[int] = Field(
        None,
        ge=1,
        description="Maximum new tokens to generate",
        examples=[512, 1024, 2048]
    )
    repetition_penalty: Optional[float] = Field(
        None,
        ge=1.0,
        le=2.0,
        description="Penalty for token repetition (1.0=no penalty)",
        examples=[1.0, 1.1, 1.2]
    )
    do_sample: Optional[bool] = Field(
        None,
        description="Use sampling (true) or greedy decoding (false)",
        examples=[True, False]
    )
    
    model_config = {
        "json_schema_extra": {
            "examples": [
                {
                    "temperature": 0.8,
                    "top_p": 0.95,
                    "top_k": 40,
                    "max_length": 1000,
                    "do_sample": True
                }
            ]
        }
    }


class ParamsResponse(BaseModel):
    """Response after setting parameters"""
    status: str = Field(..., description="Operation status", examples=["success"])
    message: str = Field(..., description="Status message")
    params: Dict[str, Any] = Field(..., description="Current parameter values")
    
    model_config = {
        "json_schema_extra": {
            "examples": [
                {
                    "status": "success",
                    "message": "Generation parameters set successfully",
                    "params": {
                        "temperature": 0.8,
                        "top_p": 0.95,
                        "top_k": 40,
                        "max_length": 1000,
                        "do_sample": True
                    }
                }
            ]
        }
    }


# [ENDPOINT] POST /api/v1/params - Set persistent generation parameters
@router.post(
    "/params",
    response_model=ParamsResponse,
    summary="Set generation parameters",
    description="""
    Set generation parameters that will persist across requests.
    
    This is useful for:
    - Setting parameters once and making multiple completion requests
    - Separating parameter configuration from inference
    - Testing different parameter configurations
    
    Only specified parameters will be updated. Others remain unchanged.
    """,
    responses={
        200: {
            "description": "Parameters set successfully",
            "content": {
                "application/json": {
                    "example": {
                        "status": "success",
                        "message": "Generation parameters set successfully",
                        "params": {
                            "temperature": 0.8,
                            "top_p": 0.95,
                            "max_length": 1000
                        }
                    }
                }
            }
        },
        503: {
            "description": "No backend available",
            "content": {
                "application/json": {
                    "example": {
                        "error": {
                            "message": "No backend available",
                            "type": "backend_error"
                        }
                    }
                }
            }
        }
    }
)
async def set_params(request: ParamsRequest):
    """
    Set generation parameters.
    
    Parameters will persist across requests until changed.
    """
    manager = get_backend_manager()
    
    logger.info(f"Setting generation parameters: {request.model_dump(exclude_none=True)}")
    
    try:
        # Build InferenceSettings from request
        settings_dict = request.model_dump(exclude_none=True)
        
        # Get current settings
        current_settings = manager.get_current_settings()
        
        # Update with new values
        for key, value in settings_dict.items():
            if hasattr(current_settings, key):
                setattr(current_settings, key, value)
        
        # Update backend
        manager.update_global_settings(current_settings)
        
        # Build response with current params
        params_dict = {
            "temperature": current_settings.temperature,
            "top_p": current_settings.top_p,
            "top_k": current_settings.top_k,
            "max_new_tokens": current_settings.max_new_tokens,
            "repetition_penalty": current_settings.repetition_penalty,
            "do_sample": current_settings.do_sample,
        }
        
        return ParamsResponse(
            status="success",
            message=ParamsMessages.PARAMS_SET,
            params=params_dict
        )
    
    except Exception as e:
        logger.error(f"Failed to set parameters: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.BACKEND_ERROR.value,
                }
            }
        )


# [ENDPOINT] GET /api/v1/params - Get current generation parameters
@router.get(
    "/params",
    response_model=ParamsResponse,
    summary="Get current generation parameters",
    description="""
    Get the current generation parameters.
    
    Returns the parameters that will be used for the next inference request.
    """,
    responses={
        200: {
            "description": "Current parameters",
            "content": {
                "application/json": {
                    "example": {
                        "status": "success",
                        "message": "Current generation parameters",
                        "params": {
                            "temperature": 0.7,
                            "top_p": 0.9,
                            "top_k": 40,
                            "max_new_tokens": 512,
                            "do_sample": True
                        }
                    }
                }
            }
        }
    }
)
async def get_params():
    """
    Get current generation parameters.
    
    Returns the parameters that are currently configured.
    """
    manager = get_backend_manager()
    
    logger.info("Getting current generation parameters")
    
    try:
        current_settings = manager.get_current_settings()
        
        params_dict = {
            "temperature": current_settings.temperature,
            "top_p": current_settings.top_p,
            "top_k": current_settings.top_k,
            "max_new_tokens": current_settings.max_new_tokens,
            "repetition_penalty": current_settings.repetition_penalty,
            "do_sample": current_settings.do_sample,
        }
        
        return ParamsResponse(
            status="success",
            message=ParamsMessages.PARAMS_GET,
            params=params_dict
        )
    
    except Exception as e:
        logger.error(f"Failed to get parameters: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.BACKEND_ERROR.value,
                }
            }
        )

