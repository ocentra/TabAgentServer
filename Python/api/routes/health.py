"""
Health Check Endpoints
"""

import time
from fastapi import APIRouter

from ..types import HealthStatus
from ..backend_manager import get_backend_manager

router = APIRouter()

# Track server start time
_start_time = time.time()


# [ENDPOINT] GET /api/v1/health - Server health and model status
@router.get(
    "/health",
    response_model=HealthStatus,
    summary="Health Check",
    description="""
    ## Check server health and model status
    
    **Use this to check:**
    - Is the server running?
    - Is a model loaded?
    - Which backend is active?
    - How long has server been running?
    
    ### Common Use Cases:
    - Before making requests (ensure server is ready)
    - Check if model is loaded before chat
    - Monitor server uptime
    - Debugging connection issues
    
    ### Response Fields:
    - `status`: "ok" if server is healthy
    - `model_loaded`: true if model ready for inference
    - `backend`: Which backend is active (ONNX_NPU, LLAMA_CPP_CUDA, etc.)
    - `uptime`: Server uptime in seconds
    """,
    responses={
        200: {
            "description": "Server is healthy",
            "content": {
                "application/json": {
                    "examples": {
                        "model_loaded": {
                            "summary": "Model loaded and ready",
                            "value": {
                                "status": "ok",
                                "model_loaded": True,
                                "backend": "ONNX_NPU",
                                "uptime": 3600.5
                            }
                        },
                        "no_model": {
                            "summary": "Server running but no model loaded",
                            "value": {
                                "status": "ok",
                                "model_loaded": False,
                                "backend": None,
                                "uptime": 120.3
                            }
                        }
                    }
                }
            }
        }
    }
)
async def health_check():
    """
    Health check endpoint
    
    Returns:
        Server health status including model load state
    """
    manager = get_backend_manager()
    
    return HealthStatus(
        status="ok",
        model_loaded=manager.is_model_loaded(),
        backend=manager.get_backend_type(),
        uptime=time.time() - _start_time
    )
