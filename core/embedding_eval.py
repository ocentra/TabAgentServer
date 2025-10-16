"""
Embedding Evaluation and Similarity Utilities

Comprehensive tools for RAG (Retrieval Augmented Generation):
- Similarity metrics (cosine, dot product, euclidean)
- Distance calculations
- Top-K retrieval
- Batch comparisons
- Evaluation metrics

Used across all backends (ONNX, llama.cpp, MediaPipe).
"""

import logging
from typing import List, Tuple, Dict, Any, Optional
from enum import Enum
from dataclasses import dataclass
import numpy as np

logger = logging.getLogger(__name__)


class SimilarityMetric(str, Enum):
    """Similarity metric types for embeddings"""
    COSINE = "cosine"
    DOT_PRODUCT = "dot_product"
    EUCLIDEAN = "euclidean"
    MANHATTAN = "manhattan"
    INNER_PRODUCT = "inner_product"


class NormalizationType(str, Enum):
    """Normalization types for embeddings"""
    L2 = "l2"
    L1 = "l1"
    MAX = "max"
    NONE = "none"


@dataclass
class SimilarityResult:
    """
    Result of similarity comparison.
    
    Attributes:
        score: Similarity score
        index: Index of compared item
        metadata: Optional metadata about the item
    """
    score: float
    index: int
    metadata: Optional[Dict[str, Any]] = None


@dataclass
class RetrievalResult:
    """
    Result of top-K retrieval.
    
    Attributes:
        query_index: Index of query embedding
        results: List of similarity results sorted by score
        metric: Metric used for comparison
    """
    query_index: int
    results: List[SimilarityResult]
    metric: SimilarityMetric


class EmbeddingEvaluator:
    """
    Comprehensive embedding evaluation utilities.
    
    Provides similarity metrics, distance calculations, and retrieval
    functions for RAG and semantic search applications.
    """
    
    @staticmethod
    def cosine_similarity(
        embedding1: List[float],
        embedding2: List[float]
    ) -> float:
        """
        Compute cosine similarity between two embeddings.
        
        Cosine similarity ranges from -1 (opposite) to 1 (identical).
        Most similar to MediaPipe's implementation.
        
        Args:
            embedding1: First embedding vector
            embedding2: Second embedding vector
            
        Returns:
            Cosine similarity score [-1, 1]
            
        Raises:
            ValueError: If embeddings have different dimensions or are empty
        """
        if len(embedding1) == 0 or len(embedding2) == 0:
            raise ValueError("Cannot compute similarity on empty embeddings")
        
        if len(embedding1) != len(embedding2):
            raise ValueError(
                f"Embeddings must have same dimension "
                f"({len(embedding1)} vs {len(embedding2)})"
            )
        
        vec1 = np.array(embedding1, dtype=np.float32)
        vec2 = np.array(embedding2, dtype=np.float32)
        
        norm1 = np.linalg.norm(vec1)
        norm2 = np.linalg.norm(vec2)
        
        if norm1 == 0 or norm2 == 0:
            raise ValueError("Cannot compute similarity on zero-norm embedding")
        
        similarity = np.dot(vec1, vec2) / (norm1 * norm2)
        return float(similarity)
    
    @staticmethod
    def dot_product(
        embedding1: List[float],
        embedding2: List[float]
    ) -> float:
        """
        Compute dot product between two embeddings.
        
        For normalized embeddings, equivalent to cosine similarity.
        
        Args:
            embedding1: First embedding vector
            embedding2: Second embedding vector
            
        Returns:
            Dot product score
            
        Raises:
            ValueError: If embeddings have different dimensions
        """
        if len(embedding1) != len(embedding2):
            raise ValueError(
                f"Embeddings must have same dimension "
                f"({len(embedding1)} vs {len(embedding2)})"
            )
        
        vec1 = np.array(embedding1, dtype=np.float32)
        vec2 = np.array(embedding2, dtype=np.float32)
        
        return float(np.dot(vec1, vec2))
    
    @staticmethod
    def euclidean_distance(
        embedding1: List[float],
        embedding2: List[float]
    ) -> float:
        """
        Compute Euclidean (L2) distance between two embeddings.
        
        Lower distance = more similar.
        
        Args:
            embedding1: First embedding vector
            embedding2: Second embedding vector
            
        Returns:
            Euclidean distance (0 = identical, higher = more different)
            
        Raises:
            ValueError: If embeddings have different dimensions
        """
        if len(embedding1) != len(embedding2):
            raise ValueError(
                f"Embeddings must have same dimension "
                f"({len(embedding1)} vs {len(embedding2)})"
            )
        
        vec1 = np.array(embedding1, dtype=np.float32)
        vec2 = np.array(embedding2, dtype=np.float32)
        
        return float(np.linalg.norm(vec1 - vec2))
    
    @staticmethod
    def manhattan_distance(
        embedding1: List[float],
        embedding2: List[float]
    ) -> float:
        """
        Compute Manhattan (L1) distance between two embeddings.
        
        Args:
            embedding1: First embedding vector
            embedding2: Second embedding vector
            
        Returns:
            Manhattan distance
            
        Raises:
            ValueError: If embeddings have different dimensions
        """
        if len(embedding1) != len(embedding2):
            raise ValueError(
                f"Embeddings must have same dimension "
                f"({len(embedding1)} vs {len(embedding2)})"
            )
        
        vec1 = np.array(embedding1, dtype=np.float32)
        vec2 = np.array(embedding2, dtype=np.float32)
        
        return float(np.sum(np.abs(vec1 - vec2)))
    
    @staticmethod
    def normalize_embedding(
        embedding: List[float],
        norm_type: NormalizationType = NormalizationType.L2
    ) -> List[float]:
        """
        Normalize embedding vector.
        
        Args:
            embedding: Embedding vector to normalize
            norm_type: Type of normalization
            
        Returns:
            Normalized embedding
        """
        vec = np.array(embedding, dtype=np.float32)
        
        if norm_type == NormalizationType.L2:
            norm = np.linalg.norm(vec)
            if norm > 0:
                vec = vec / norm
        
        elif norm_type == NormalizationType.L1:
            norm = np.sum(np.abs(vec))
            if norm > 0:
                vec = vec / norm
        
        elif norm_type == NormalizationType.MAX:
            max_val = np.max(np.abs(vec))
            if max_val > 0:
                vec = vec / max_val
        
        return vec.tolist()
    
    @staticmethod
    def compute_similarity_matrix(
        embeddings: List[List[float]],
        metric: SimilarityMetric = SimilarityMetric.COSINE
    ) -> np.ndarray:
        """
        Compute pairwise similarity matrix for all embeddings.
        
        Useful for finding duplicates or clustering.
        
        Args:
            embeddings: List of embedding vectors
            metric: Similarity metric to use
            
        Returns:
            NxN similarity matrix where N = len(embeddings)
        """
        n = len(embeddings)
        matrix = np.zeros((n, n), dtype=np.float32)
        
        evaluator = EmbeddingEvaluator()
        
        for i in range(n):
            for j in range(i, n):
                if metric == SimilarityMetric.COSINE:
                    score = evaluator.cosine_similarity(embeddings[i], embeddings[j])
                elif metric == SimilarityMetric.DOT_PRODUCT:
                    score = evaluator.dot_product(embeddings[i], embeddings[j])
                elif metric == SimilarityMetric.EUCLIDEAN:
                    # Convert distance to similarity (inverse)
                    dist = evaluator.euclidean_distance(embeddings[i], embeddings[j])
                    score = 1.0 / (1.0 + dist)
                else:
                    score = evaluator.cosine_similarity(embeddings[i], embeddings[j])
                
                matrix[i, j] = score
                matrix[j, i] = score  # Symmetric
        
        return matrix
    
    @staticmethod
    def find_top_k_similar(
        query_embedding: List[float],
        candidate_embeddings: List[List[float]],
        k: int = 5,
        metric: SimilarityMetric = SimilarityMetric.COSINE,
        metadata: Optional[List[Dict[str, Any]]] = None
    ) -> List[SimilarityResult]:
        """
        Find top-K most similar embeddings to query.
        
        Core function for RAG retrieval.
        
        Args:
            query_embedding: Query embedding vector
            candidate_embeddings: List of candidate embedding vectors
            k: Number of top results to return
            metric: Similarity metric to use
            metadata: Optional metadata for each candidate
            
        Returns:
            List of top-K similarity results sorted by score (descending)
        """
        evaluator = EmbeddingEvaluator()
        scores = []
        
        for idx, candidate in enumerate(candidate_embeddings):
            try:
                if metric == SimilarityMetric.COSINE:
                    score = evaluator.cosine_similarity(query_embedding, candidate)
                elif metric == SimilarityMetric.DOT_PRODUCT:
                    score = evaluator.dot_product(query_embedding, candidate)
                elif metric == SimilarityMetric.EUCLIDEAN:
                    # Convert distance to similarity (lower distance = higher similarity)
                    dist = evaluator.euclidean_distance(query_embedding, candidate)
                    score = 1.0 / (1.0 + dist)
                elif metric == SimilarityMetric.MANHATTAN:
                    dist = evaluator.manhattan_distance(query_embedding, candidate)
                    score = 1.0 / (1.0 + dist)
                else:
                    score = evaluator.cosine_similarity(query_embedding, candidate)
                
                result = SimilarityResult(
                    score=score,
                    index=idx,
                    metadata=metadata[idx] if metadata and idx < len(metadata) else None
                )
                scores.append(result)
            
            except Exception as e:
                logger.warning(f"Failed to compute similarity for index {idx}: {e}")
                continue
        
        # Sort by score descending
        scores.sort(key=lambda x: x.score, reverse=True)
        
        # Return top-K
        return scores[:k]
    
    @staticmethod
    def batch_find_similar(
        query_embeddings: List[List[float]],
        candidate_embeddings: List[List[float]],
        k: int = 5,
        metric: SimilarityMetric = SimilarityMetric.COSINE
    ) -> List[List[SimilarityResult]]:
        """
        Find top-K similar candidates for multiple queries (batch).
        
        Optimized for processing multiple queries at once.
        
        Args:
            query_embeddings: List of query embedding vectors
            candidate_embeddings: List of candidate embedding vectors
            k: Number of top results per query
            metric: Similarity metric to use
            
        Returns:
            List of top-K results for each query
        """
        results = []
        
        for query_emb in query_embeddings:
            top_k = EmbeddingEvaluator.find_top_k_similar(
                query_emb,
                candidate_embeddings,
                k=k,
                metric=metric
            )
            results.append(top_k)
        
        return results
    
    @staticmethod
    def compute_diversity(embeddings: List[List[float]]) -> float:
        """
        Compute diversity score for a set of embeddings.
        
        Higher diversity = embeddings are more spread out.
        Useful for evaluating embedding quality.
        
        Args:
            embeddings: List of embedding vectors
            
        Returns:
            Diversity score (average pairwise distance)
        """
        if len(embeddings) < 2:
            return 0.0
        
        evaluator = EmbeddingEvaluator()
        total_distance = 0.0
        count = 0
        
        for i in range(len(embeddings)):
            for j in range(i + 1, len(embeddings)):
                dist = evaluator.euclidean_distance(embeddings[i], embeddings[j])
                total_distance += dist
                count += 1
        
        return total_distance / count if count > 0 else 0.0


class RAGRetriever:
    """
    Retrieval-Augmented Generation utilities.
    
    Provides high-level RAG functionality using embeddings.
    """
    
    def __init__(
        self,
        metric: SimilarityMetric = SimilarityMetric.COSINE,
        normalize: bool = True
    ):
        """
        Initialize RAG retriever.
        
        Args:
            metric: Similarity metric to use
            normalize: Whether to L2-normalize embeddings
        """
        self.metric = metric
        self.normalize = normalize
        self.evaluator = EmbeddingEvaluator()
        
        logger.info(f"RAGRetriever initialized (metric: {metric.value}, normalize: {normalize})")
    
    def retrieve(
        self,
        query_embedding: List[float],
        document_embeddings: List[List[float]],
        documents: Optional[List[str]] = None,
        k: int = 5,
        score_threshold: Optional[float] = None
    ) -> List[Dict[str, Any]]:
        """
        Retrieve top-K most relevant documents for query.
        
        Core RAG retrieval function.
        
        Args:
            query_embedding: Query embedding vector
            document_embeddings: Document embedding vectors
            documents: Optional document texts
            k: Number of documents to retrieve
            score_threshold: Minimum similarity score (optional)
            
        Returns:
            List of retrieved documents with scores
        """
        # Normalize if requested
        if self.normalize:
            query_embedding = self.evaluator.normalize_embedding(
                query_embedding,
                NormalizationType.L2
            )
            document_embeddings = [
                self.evaluator.normalize_embedding(emb, NormalizationType.L2)
                for emb in document_embeddings
            ]
        
        # Find top-K similar
        results = self.evaluator.find_top_k_similar(
            query_embedding,
            document_embeddings,
            k=k,
            metric=self.metric
        )
        
        # Apply score threshold if specified
        if score_threshold is not None:
            results = [r for r in results if r.score >= score_threshold]
        
        # Build response with document texts
        retrieved = []
        for result in results:
            item = {
                "index": result.index,
                "score": result.score,
            }
            
            if documents and result.index < len(documents):
                item["document"] = documents[result.index]
            
            if result.metadata:
                item["metadata"] = result.metadata
            
            retrieved.append(item)
        
        logger.info(f"Retrieved {len(retrieved)} documents (threshold: {score_threshold})")
        return retrieved
    
    def batch_retrieve(
        self,
        query_embeddings: List[List[float]],
        document_embeddings: List[List[float]],
        documents: Optional[List[str]] = None,
        k: int = 5,
        score_threshold: Optional[float] = None
    ) -> List[List[Dict[str, Any]]]:
        """
        Retrieve top-K documents for multiple queries.
        
        Args:
            query_embeddings: List of query embedding vectors
            document_embeddings: Document embedding vectors
            documents: Optional document texts
            k: Number of documents per query
            score_threshold: Minimum similarity score
            
        Returns:
            List of retrieval results (one per query)
        """
        results = []
        
        for query_emb in query_embeddings:
            retrieved = self.retrieve(
                query_emb,
                document_embeddings,
                documents,
                k,
                score_threshold
            )
            results.append(retrieved)
        
        return results
    
    def rerank_by_similarity(
        self,
        query_embedding: List[float],
        documents: List[str],
        document_embeddings: List[List[float]]
    ) -> List[Tuple[str, float]]:
        """
        Rerank documents by similarity to query.
        
        Returns all documents sorted by relevance.
        
        Args:
            query_embedding: Query embedding vector
            documents: Document texts
            document_embeddings: Document embedding vectors
            
        Returns:
            List of (document, score) tuples sorted by score descending
        """
        results = self.retrieve(
            query_embedding,
            document_embeddings,
            documents,
            k=len(documents)  # Get all
        )
        
        return [(r["document"], r["score"]) for r in results if "document" in r]


class EmbeddingMetrics:
    """
    Evaluation metrics for embedding quality.
    
    Useful for testing and comparing different embedding models.
    """
    
    @staticmethod
    def average_cosine_similarity(
        embeddings1: List[List[float]],
        embeddings2: List[List[float]]
    ) -> float:
        """
        Compute average cosine similarity between two sets of embeddings.
        
        Useful for comparing embedding spaces.
        
        Args:
            embeddings1: First set of embeddings
            embeddings2: Second set of embeddings (must be same length)
            
        Returns:
            Average cosine similarity
        """
        if len(embeddings1) != len(embeddings2):
            raise ValueError("Embedding sets must have same length")
        
        evaluator = EmbeddingEvaluator()
        similarities = []
        
        for emb1, emb2 in zip(embeddings1, embeddings2):
            sim = evaluator.cosine_similarity(emb1, emb2)
            similarities.append(sim)
        
        return float(np.mean(similarities))
    
    @staticmethod
    def compute_retrieval_metrics(
        relevant_indices: List[int],
        retrieved_indices: List[int],
        k: Optional[int] = None
    ) -> Dict[str, float]:
        """
        Compute retrieval evaluation metrics.
        
        Calculates Precision@K, Recall@K, F1@K, and MRR.
        
        Args:
            relevant_indices: Indices of relevant documents
            retrieved_indices: Indices of retrieved documents (ordered by relevance)
            k: Cutoff for metrics (if None, use all retrieved)
            
        Returns:
            Dictionary with precision, recall, f1, and mrr scores
        """
        if k is None:
            k = len(retrieved_indices)
        
        retrieved_at_k = set(retrieved_indices[:k])
        relevant_set = set(relevant_indices)
        
        # Precision@K = relevant items in top-K / K
        relevant_retrieved = retrieved_at_k.intersection(relevant_set)
        precision = len(relevant_retrieved) / k if k > 0 else 0.0
        
        # Recall@K = relevant items in top-K / total relevant
        recall = len(relevant_retrieved) / len(relevant_set) if len(relevant_set) > 0 else 0.0
        
        # F1@K = harmonic mean of precision and recall
        f1 = 2 * precision * recall / (precision + recall) if (precision + recall) > 0 else 0.0
        
        # MRR (Mean Reciprocal Rank) = 1 / rank of first relevant item
        mrr = 0.0
        for rank, idx in enumerate(retrieved_indices[:k], start=1):
            if idx in relevant_set:
                mrr = 1.0 / rank
                break
        
        return {
            "precision_at_k": precision,
            "recall_at_k": recall,
            "f1_at_k": f1,
            "mrr": mrr,
            "k": k
        }
    
    @staticmethod
    def compute_ndcg(
        relevance_scores: List[float],
        retrieved_indices: List[int],
        k: Optional[int] = None
    ) -> float:
        """
        Compute Normalized Discounted Cumulative Gain (NDCG@K).
        
        Standard metric for ranking quality.
        
        Args:
            relevance_scores: Relevance score for each document
            retrieved_indices: Indices of retrieved documents (ordered)
            k: Cutoff (if None, use all retrieved)
            
        Returns:
            NDCG score [0, 1] where 1 is perfect ranking
        """
        if k is None:
            k = len(retrieved_indices)
        
        # DCG = sum of (relevance / log2(rank + 1))
        dcg = 0.0
        for rank, idx in enumerate(retrieved_indices[:k], start=1):
            if idx < len(relevance_scores):
                relevance = relevance_scores[idx]
                dcg += relevance / np.log2(rank + 1)
        
        # IDCG = DCG of perfect ranking
        sorted_relevances = sorted(relevance_scores, reverse=True)[:k]
        idcg = 0.0
        for rank, relevance in enumerate(sorted_relevances, start=1):
            idcg += relevance / np.log2(rank + 1)
        
        # NDCG = DCG / IDCG
        return dcg / idcg if idcg > 0 else 0.0


# Convenience functions for common use cases

def cosine_similarity(embedding1: List[float], embedding2: List[float]) -> float:
    """Convenience: Compute cosine similarity"""
    return EmbeddingEvaluator.cosine_similarity(embedding1, embedding2)


def find_similar(
    query: List[float],
    candidates: List[List[float]],
    k: int = 5
) -> List[SimilarityResult]:
    """Convenience: Find top-K similar embeddings using cosine similarity"""
    return EmbeddingEvaluator.find_top_k_similar(query, candidates, k)


def normalize(embedding: List[float]) -> List[float]:
    """Convenience: L2-normalize embedding"""
    return EmbeddingEvaluator.normalize_embedding(embedding, NormalizationType.L2)

