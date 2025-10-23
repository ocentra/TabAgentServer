"""
Reranking Endpoints

JinaAI-compatible document reranking using specialized models.
"""

import logging
from typing import List, Dict, Any
from fastapi import APIRouter, HTTPException, status
from pydantic import BaseModel, Field

from ..backend_manager import get_backend_manager
from ..constants import ErrorCode

logger = logging.getLogger(__name__)

router = APIRouter()


class RerankingMessages:
    """Messages for reranking operations (no string literals)"""
    NO_MODEL_LOADED = "No model loaded"
    RERANKING_FAILED = "Failed to rerank documents"
    UNSUPPORTED_MODEL = "Model does not support reranking"
    INVALID_INPUT = "Invalid input: query and documents are required"


class RerankingRequest(BaseModel):
    """Request for document reranking"""
    query: str = Field(..., description="Search query")
    documents: List[str] = Field(..., description="List of documents to rerank")
    model: str = Field(..., description="Reranking model identifier")
    top_k: int = Field(None, description="Return top K documents (optional)")


class RerankingResult(BaseModel):
    """Single reranking result"""
    index: int
    document: str
    relevance_score: float


class RerankingResponse(BaseModel):
    """Reranking API response"""
    results: List[RerankingResult]
    model: str
    usage: Dict[str, int]


@router.post("/reranking", response_model=RerankingResponse)
async def rerank_documents(request: RerankingRequest):
    """
    Rerank documents based on relevance to query.
    
    JinaAI-compatible reranking endpoint.
    Uses specialized reranker models (e.g., bge-reranker-base).
    
    Args:
        request: Reranking request
        
    Returns:
        Reranked documents with relevance scores
        
    Raises:
        HTTPException: If no model loaded or reranking fails
    """
    manager = get_backend_manager()
    
    # Check if model is loaded
    if not manager.is_model_loaded():
        logger.error("Reranking requested but no model loaded")
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail={
                "error": {
                    "message": RerankingMessages.NO_MODEL_LOADED,
                    "type": ErrorCode.MODEL_NOT_LOADED.value,
                }
            }
        )
    
    logger.info(f"Reranking request: model={request.model}, docs={len(request.documents)}")
    
    try:
        # Validate input
        if not request.query or not request.documents:
            raise ValueError(RerankingMessages.INVALID_INPUT)
        
        # Generate reranking via backend manager
        reranking_result = await manager.rerank_documents(
            query=request.query,
            documents=request.documents,
            model=request.model,
            top_k=request.top_k
        )
        
        # Build response
        results = [
            RerankingResult(
                index=res["index"],
                document=res["document"],
                relevance_score=res["score"]
            )
            for res in reranking_result["results"]
        ]
        
        # Apply top_k if specified
        if request.top_k:
            results = results[:request.top_k]
        
        response = RerankingResponse(
            results=results,
            model=request.model,
            usage={
                "total_tokens": reranking_result.get("total_tokens", 0)
            }
        )
        
        logger.info(f"Reranked {len(results)} documents")
        return response
    
    except NotImplementedError:
        logger.error("Reranking not supported by current backend")
        raise HTTPException(
            status_code=status.HTTP_501_NOT_IMPLEMENTED,
            detail={
                "error": {
                    "message": RerankingMessages.UNSUPPORTED_MODEL,
                    "type": ErrorCode.NOT_IMPLEMENTED.value,
                }
            }
        )
    except ValueError as e:
        logger.error(f"Invalid reranking request: {e}")
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.INVALID_REQUEST.value,
                }
            }
        )
    except Exception as e:
        logger.error(f"Reranking failed: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.GENERATION_FAILED.value,
                }
            }
        )


@router.post("/rerank", response_model=RerankingResponse)
async def rerank_documents_alias(request: RerankingRequest):
    """
    Rerank documents (alternative route name).
    
    Alias for /reranking endpoint for compatibility.
    """
    return await rerank_documents(request)

