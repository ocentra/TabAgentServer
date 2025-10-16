"""
Embeddings Endpoints

OpenAI-compatible embeddings generation using ONNX Runtime or llama.cpp.
"""

import logging
from typing import List, Union, Dict, Any
from fastapi import APIRouter, HTTPException, status
from pydantic import BaseModel, Field

from ..backend_manager import get_backend_manager
from ..constants import ErrorCode

logger = logging.getLogger(__name__)

router = APIRouter()


class EmbeddingFormat(str):
    """Supported embedding formats (no string literal enum)"""
    FLOAT = "float"
    BASE64 = "base64"


class EmbeddingsMessages:
    """Messages for embeddings operations (no string literals)"""
    NO_MODEL_LOADED = "No model loaded"
    EMBEDDINGS_FAILED = "Failed to generate embeddings"
    INVALID_INPUT = "Invalid input format"
    UNSUPPORTED_MODEL = "Model does not support embeddings"


class EmbeddingsRequest(BaseModel):
    """Request for embeddings generation"""
    input: Union[str, List[str]] = Field(..., description="Text(s) to embed", examples=["Hello world", ["text1", "text2"]])
    model: str = Field(..., description="Model identifier", examples=["all-MiniLM-L6-v2", "bge-small-en-v1.5"])
    encoding_format: str = Field(
        EmbeddingFormat.FLOAT,
        description="Format for embeddings (float or base64)",
        examples=["float"]
    )
    
    model_config = {
        "json_schema_extra": {
            "examples": [
                {
                    "input": ["Hello world", "Good morning"],
                    "model": "all-MiniLM-L6-v2",
                    "encoding_format": "float"
                }
            ]
        }
    }


class EmbeddingObject(BaseModel):
    """Single embedding result"""
    object: str = "embedding"
    embedding: List[float]
    index: int


class EmbeddingsUsage(BaseModel):
    """Token usage for embeddings"""
    prompt_tokens: int
    total_tokens: int


class EmbeddingsResponse(BaseModel):
    """Embeddings API response"""
    object: str = "list"
    data: List[EmbeddingObject]
    model: str
    usage: EmbeddingsUsage


@router.post("/embeddings", response_model=EmbeddingsResponse)
async def create_embeddings(request: EmbeddingsRequest):
    """
    Generate embeddings for input text(s).
    
    OpenAI-compatible embeddings endpoint.
    Supports both single strings and lists of strings.
    
    Args:
        request: Embeddings request
        
    Returns:
        Embeddings response with vectors
        
    Raises:
        HTTPException: If no model loaded or generation fails
    """
    manager = get_backend_manager()
    
    # Check if model is loaded
    if not manager.is_model_loaded():
        logger.error("Embeddings requested but no model loaded")
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail={
                "error": {
                    "message": EmbeddingsMessages.NO_MODEL_LOADED,
                    "type": ErrorCode.MODEL_NOT_LOADED.value,
                }
            }
        )
    
    logger.info(f"Embeddings request: model={request.model}")
    
    try:
        # Normalize input to list
        texts = [request.input] if isinstance(request.input, str) else request.input
        
        # Generate embeddings via backend manager
        embeddings_result = await manager.generate_embeddings(texts, request.model)
        
        # Build response
        embedding_objects = [
            EmbeddingObject(
                embedding=emb,
                index=idx
            )
            for idx, emb in enumerate(embeddings_result["embeddings"])
        ]
        
        # Calculate token usage (approximate)
        total_tokens = sum(len(text.split()) for text in texts)
        
        response = EmbeddingsResponse(
            data=embedding_objects,
            model=request.model,
            usage=EmbeddingsUsage(
                prompt_tokens=total_tokens,
                total_tokens=total_tokens
            )
        )
        
        logger.info(f"Generated {len(embedding_objects)} embeddings")
        return response
    
    except NotImplementedError:
        logger.error("Embeddings not supported by current backend")
        raise HTTPException(
            status_code=status.HTTP_501_NOT_IMPLEMENTED,
            detail={
                "error": {
                    "message": EmbeddingsMessages.UNSUPPORTED_MODEL,
                    "type": ErrorCode.NOT_IMPLEMENTED.value,
                }
            }
        )
    except Exception as e:
        logger.error(f"Embeddings failed: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.GENERATION_FAILED.value,
                }
            }
        )

