"""
Generation Control Endpoints

Provides control over in-progress generation (halt, pause, resume).
"""

import logging
from typing import Dict, Any
from fastapi import APIRouter, HTTPException, status

from ..backend_manager import get_backend_manager
from ..constants import ErrorCode

logger = logging.getLogger(__name__)

router = APIRouter()


class GenerationControlMessages:
    """Messages for generation control operations (no string literals)"""
    GENERATION_HALTED = "Generation halted successfully"
    NO_GENERATION_IN_PROGRESS = "No generation in progress"
    HALT_FAILED = "Failed to halt generation"


@router.get("/halt")
async def halt_generation() -> Dict[str, Any]:
    """
    Halt in-progress generation.
    
    Stops the current generation and returns any partial output.
    Works across all backends (BitNet, ONNX, llama.cpp, MediaPipe).
    
    Returns:
        Status response with any partial output
        
    Raises:
        HTTPException: If halt operation fails
    """
    logger.info("Halt generation requested")
    
    try:
        manager = get_backend_manager()
        
        # Check if generation is in progress
        if not manager.is_generating():
            logger.warning("Halt requested but no generation in progress")
            return {
                "status": "success",
                "message": GenerationControlMessages.NO_GENERATION_IN_PROGRESS,
                "was_generating": False
            }
        
        # Halt generation
        result = await manager.halt_generation()
        
        logger.info("Generation halted successfully")
        
        return {
            "status": "success",
            "message": GenerationControlMessages.GENERATION_HALTED,
            "was_generating": True,
            "partial_output": result.get("partial_output"),
            "tokens_generated": result.get("tokens_generated", 0)
        }
    
    except Exception as e:
        logger.error(f"Error halting generation: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.BACKEND_ERROR.value,
                }
            }
        )


@router.post("/halt")
async def halt_generation_post() -> Dict[str, Any]:
    """
    Halt generation (POST variant).
    
    Same as GET /halt, provided for client compatibility.
    """
    return await halt_generation()

