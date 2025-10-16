"""
RAG (Retrieval-Augmented Generation) Endpoints

Semantic search, retrieval, and evaluation utilities for RAG applications.
"""

import logging
from typing import List, Dict, Any, Optional
from fastapi import APIRouter, HTTPException, status
from pydantic import BaseModel, Field

from ..backend_manager import get_backend_manager
from ..constants import ErrorCode
from core.embedding_eval import (
    RAGRetriever,
    EmbeddingMetrics,
    SimilarityMetric,
    cosine_similarity,
)
from core.embedding_clustering import (
    EmbeddingClusterer,
    RecommendationEngine,
    ClusteringAlgorithm,
    ClusterResult,
)
from core.embedding_models import (
    EmbeddingModelRegistry,
    EmbeddingModality,
    EmbeddingUseCase,
)

logger = logging.getLogger(__name__)

router = APIRouter()


class RAGMessages:
    """Messages for RAG operations (no string literals)"""
    NO_MODEL_LOADED = "No model loaded"
    RETRIEVAL_FAILED = "Failed to retrieve documents"
    CLUSTERING_FAILED = "Failed to cluster embeddings"
    RECOMMENDATION_FAILED = "Failed to generate recommendations"
    INVALID_INPUT = "Invalid input"
    UNSUPPORTED_BACKEND = "Backend does not support embeddings"


class SemanticSearchRequest(BaseModel):
    """Request for semantic search"""
    query: str = Field(..., description="Search query")
    documents: List[str] = Field(..., description="Document corpus to search")
    model: str = Field(..., description="Embedding model")
    k: int = Field(5, ge=1, le=100, description="Number of results to return")
    score_threshold: Optional[float] = Field(None, ge=0.0, le=1.0, description="Minimum similarity score")
    metric: str = Field("cosine", description="Similarity metric (cosine, dot_product, euclidean)")


class SemanticSearchResult(BaseModel):
    """Single search result"""
    index: int
    document: str
    score: float


class SemanticSearchResponse(BaseModel):
    """Semantic search response"""
    query: str
    results: List[SemanticSearchResult]
    total_documents: int
    metric: str


class SimilarityRequest(BaseModel):
    """Request to compute similarity between two texts"""
    text1: str = Field(..., description="First text")
    text2: str = Field(..., description="Second text")
    model: str = Field(..., description="Embedding model")
    metric: str = Field("cosine", description="Similarity metric")


class SimilarityResponse(BaseModel):
    """Similarity computation response"""
    similarity: float
    metric: str
    text1_length: int
    text2_length: int


class EvaluationRequest(BaseModel):
    """Request for embedding evaluation metrics"""
    embeddings1: List[List[float]] = Field(..., description="First set of embeddings")
    embeddings2: List[List[float]] = Field(..., description="Second set of embeddings")
    metric_type: str = Field("average_similarity", description="Metric type")


class EvaluationResponse(BaseModel):
    """Evaluation metrics response"""
    metric: str
    score: float
    embedding_count: int


@router.post("/semantic-search", response_model=SemanticSearchResponse)
async def semantic_search(request: SemanticSearchRequest):
    """
    Semantic search over document corpus.
    
    Core RAG retrieval function. Finds most relevant documents for query.
    
    Args:
        request: Semantic search request
        
    Returns:
        Top-K most relevant documents with similarity scores
        
    Raises:
        HTTPException: If no model loaded or search fails
    """
    manager = get_backend_manager()
    
    if not manager.is_model_loaded():
        logger.error("Semantic search requested but no model loaded")
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail={
                "error": {
                    "message": RAGMessages.NO_MODEL_LOADED,
                    "type": ErrorCode.MODEL_NOT_LOADED.value,
                }
            }
        )
    
    logger.info(f"Semantic search: query='{request.query[:50]}...', docs={len(request.documents)}, k={request.k}")
    
    try:
        # Generate embeddings for query and documents
        all_texts = [request.query] + request.documents
        embeddings_result = await manager.generate_embeddings(all_texts, request.model)
        
        all_embeddings = embeddings_result["embeddings"]
        query_embedding = all_embeddings[0]
        doc_embeddings = all_embeddings[1:]
        
        # Parse metric
        try:
            metric = SimilarityMetric(request.metric.lower())
        except ValueError:
            metric = SimilarityMetric.COSINE
        
        # Perform retrieval
        retriever = RAGRetriever(metric=metric, normalize=True)
        results = retriever.retrieve(
            query_embedding,
            doc_embeddings,
            documents=request.documents,
            k=request.k,
            score_threshold=request.score_threshold
        )
        
        # Build response
        search_results = [
            SemanticSearchResult(
                index=r["index"],
                document=r["document"],
                score=r["score"]
            )
            for r in results
        ]
        
        response = SemanticSearchResponse(
            query=request.query,
            results=search_results,
            total_documents=len(request.documents),
            metric=request.metric
        )
        
        logger.info(f"Semantic search complete: {len(search_results)} results")
        return response
    
    except NotImplementedError as e:
        logger.error(f"Backend doesn't support embeddings: {e}")
        raise HTTPException(
            status_code=status.HTTP_501_NOT_IMPLEMENTED,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.NOT_IMPLEMENTED.value,
                }
            }
        )
    except Exception as e:
        logger.error(f"Semantic search failed: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.GENERATION_FAILED.value,
                }
            }
        )


@router.post("/similarity", response_model=SimilarityResponse)
async def compute_similarity(request: SimilarityRequest):
    """
    Compute similarity between two texts.
    
    Useful for checking semantic similarity, duplicate detection, etc.
    
    Args:
        request: Similarity request
        
    Returns:
        Similarity score
        
    Raises:
        HTTPException: If computation fails
    """
    manager = get_backend_manager()
    
    if not manager.is_model_loaded():
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail={
                "error": {
                    "message": RAGMessages.NO_MODEL_LOADED,
                    "type": ErrorCode.MODEL_NOT_LOADED.value,
                }
            }
        )
    
    logger.info(f"Computing similarity between two texts (metric: {request.metric})")
    
    try:
        # Generate embeddings
        texts = [request.text1, request.text2]
        embeddings_result = await manager.generate_embeddings(texts, request.model)
        
        emb1 = embeddings_result["embeddings"][0]
        emb2 = embeddings_result["embeddings"][1]
        
        # Compute similarity
        if request.metric.lower() == "cosine":
            score = cosine_similarity(emb1, emb2)
        elif request.metric.lower() == "dot_product":
            from core.embedding_eval import EmbeddingEvaluator
            score = EmbeddingEvaluator.dot_product(emb1, emb2)
        elif request.metric.lower() == "euclidean":
            from core.embedding_eval import EmbeddingEvaluator
            dist = EmbeddingEvaluator.euclidean_distance(emb1, emb2)
            score = 1.0 / (1.0 + dist)  # Convert distance to similarity
        else:
            score = cosine_similarity(emb1, emb2)
        
        response = SimilarityResponse(
            similarity=score,
            metric=request.metric,
            text1_length=len(request.text1),
            text2_length=len(request.text2)
        )
        
        logger.info(f"Similarity computed: {score:.4f}")
        return response
    
    except Exception as e:
        logger.error(f"Similarity computation failed: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.GENERATION_FAILED.value,
                }
            }
        )


@router.post("/evaluate-embeddings", response_model=EvaluationResponse)
async def evaluate_embeddings(request: EvaluationRequest):
    """
    Evaluate embedding quality.
    
    Computes metrics for comparing two sets of embeddings.
    Useful for testing embedding models or comparing embedding spaces.
    
    Args:
        request: Evaluation request
        
    Returns:
        Evaluation metrics
        
    Raises:
        HTTPException: If evaluation fails
    """
    logger.info(f"Evaluating embeddings: {len(request.embeddings1)} vs {len(request.embeddings2)}")
    
    try:
        # Compute requested metric
        if request.metric_type == "average_similarity":
            score = EmbeddingMetrics.average_cosine_similarity(
                request.embeddings1,
                request.embeddings2
            )
        else:
            # Default to average similarity
            score = EmbeddingMetrics.average_cosine_similarity(
                request.embeddings1,
                request.embeddings2
            )
        
        response = EvaluationResponse(
            metric=request.metric_type,
            score=score,
            embedding_count=len(request.embeddings1)
        )
        
        logger.info(f"Evaluation complete: {request.metric_type} = {score:.4f}")
        return response
    
    except ValueError as e:
        logger.error(f"Invalid evaluation input: {e}")
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
        logger.error(f"Evaluation failed: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.GENERATION_FAILED.value,
                }
            }
        )


class ClusterRequest(BaseModel):
    """Request for clustering embeddings"""
    texts: List[str] = Field(..., description="Texts to cluster")
    model: str = Field(..., description="Embedding model")
    n_clusters: int = Field(..., ge=2, le=100, description="Number of clusters")
    algorithm: str = Field("kmeans", description="Clustering algorithm")


class ClusterResponse(BaseModel):
    """Clustering response"""
    labels: List[int]
    n_clusters: int
    silhouette_score: Optional[float] = None
    algorithm: str


class RecommendationRequest(BaseModel):
    """Request for item recommendations"""
    items: List[str] = Field(..., description="Item texts/descriptions")
    query_item_index: int = Field(..., description="Index of item to find similar items for")
    model: str = Field(..., description="Embedding model")
    k: int = Field(10, ge=1, le=100, description="Number of recommendations")


class RecommendationResponse(BaseModel):
    """Recommendation response"""
    query_item: str
    recommendations: List[Dict[str, Any]]


@router.post("/cluster", response_model=ClusterResponse)
async def cluster_embeddings(request: ClusterRequest):
    """
    Cluster texts using embeddings.
    
    Useful for topic discovery, document organization, user segmentation.
    
    Args:
        request: Clustering request
        
    Returns:
        Cluster labels and quality metrics
        
    Raises:
        HTTPException: If clustering fails
    """
    manager = get_backend_manager()
    
    if not manager.is_model_loaded():
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail={
                "error": {
                    "message": RAGMessages.NO_MODEL_LOADED,
                    "type": ErrorCode.MODEL_NOT_LOADED.value,
                }
            }
        )
    
    logger.info(f"Clustering {len(request.texts)} texts into {request.n_clusters} clusters")
    
    try:
        # Generate embeddings
        embeddings_result = await manager.generate_embeddings(request.texts, request.model)
        embeddings = embeddings_result["embeddings"]
        
        # Perform clustering
        if request.algorithm.lower() == "kmeans":
            result = EmbeddingClusterer.kmeans(embeddings, request.n_clusters)
        elif request.algorithm.lower() == "hierarchical":
            result = EmbeddingClusterer.hierarchical(embeddings, request.n_clusters)
        elif request.algorithm.lower() == "dbscan":
            result = EmbeddingClusterer.dbscan(embeddings)
        else:
            result = EmbeddingClusterer.kmeans(embeddings, request.n_clusters)
        
        response = ClusterResponse(
            labels=result.labels,
            n_clusters=result.n_clusters,
            silhouette_score=result.silhouette_score,
            algorithm=request.algorithm
        )
        
        logger.info(f"Clustering complete: {result.n_clusters} clusters")
        return response
    
    except Exception as e:
        logger.error(f"Clustering failed: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.GENERATION_FAILED.value,
                }
            }
        )


@router.post("/recommend", response_model=RecommendationResponse)
async def recommend_items(request: RecommendationRequest):
    """
    Generate content-based recommendations.
    
    Finds items similar to query item using embeddings.
    Useful for recommendation systems, "similar items", etc.
    
    Args:
        request: Recommendation request
        
    Returns:
        Recommended items with similarity scores
        
    Raises:
        HTTPException: If recommendations fail
    """
    manager = get_backend_manager()
    
    if not manager.is_model_loaded():
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail={
                "error": {
                    "message": RAGMessages.NO_MODEL_LOADED,
                    "type": ErrorCode.MODEL_NOT_LOADED.value,
                }
            }
        )
    
    if request.query_item_index < 0 or request.query_item_index >= len(request.items):
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail={
                "error": {
                    "message": "Invalid query_item_index",
                    "type": ErrorCode.INVALID_REQUEST.value,
                }
            }
        )
    
    logger.info(f"Generating recommendations for item {request.query_item_index}")
    
    try:
        # Generate embeddings for all items
        embeddings_result = await manager.generate_embeddings(request.items, request.model)
        embeddings = embeddings_result["embeddings"]
        
        # Create recommendation engine
        engine = RecommendationEngine(
            item_embeddings=embeddings,
            item_ids=[str(i) for i in range(len(request.items))]
        )
        
        # Get recommendations
        recommendations = engine.recommend_similar_items(
            item_index=request.query_item_index,
            k=request.k,
            exclude_self=True
        )
        
        response = RecommendationResponse(
            query_item=request.items[request.query_item_index],
            recommendations=recommendations
        )
        
        logger.info(f"Generated {len(recommendations)} recommendations")
        return response
    
    except Exception as e:
        logger.error(f"Recommendation failed: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.GENERATION_FAILED.value,
                }
            }
        )


@router.get("/embedding-models")
async def list_embedding_models(
    modality: Optional[str] = None,
    use_case: Optional[str] = None
) -> Dict[str, Any]:
    """
    List available embedding models.
    
    Returns curated list of state-of-the-art embedding models.
    
    Args:
        modality: Filter by modality (text, image, multimodal)
        use_case: Filter by use case (semantic_search, classification, etc.)
        
    Returns:
        Dictionary of embedding models with info
    """
    try:
        if modality:
            mod = EmbeddingModality(modality.lower())
            models = EmbeddingModelRegistry.get_models_by_modality(mod)
        elif use_case:
            uc = EmbeddingUseCase(use_case.lower())
            models = EmbeddingModelRegistry.get_models_by_use_case(uc)
        else:
            models = EmbeddingModelRegistry.get_all_models()
        
        # Convert to dict format
        models_dict = {
            model_id: {
                "name": info.name,
                "modality": info.modality.value,
                "dimension": info.dimension,
                "use_cases": [uc.value for uc in info.use_cases],
                "size": info.size.value,
                "backend": info.backend,
                "repo_id": info.repo_id,
                "description": info.description
            }
            for model_id, info in models.items()
        }
        
        return {
            "models": models_dict,
            "total": len(models_dict),
            "filtered_by": modality or use_case or "none"
        }
    
    except ValueError as e:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.INVALID_REQUEST.value,
                }
            }
        )

