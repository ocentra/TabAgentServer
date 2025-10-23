"""
Embedding Clustering Utilities

Clustering algorithms for embeddings:
- K-Means clustering
- Hierarchical clustering
- DBSCAN (density-based)
- Cluster evaluation metrics

Used for:
- Topic discovery
- Document organization
- User segmentation
- Anomaly detection
"""

import logging
from typing import List, Dict, Any, Optional, Tuple
from enum import Enum
from dataclasses import dataclass
import numpy as np

logger = logging.getLogger(__name__)


class ClusteringAlgorithm(str, Enum):
    """Supported clustering algorithms"""
    KMEANS = "kmeans"
    HIERARCHICAL = "hierarchical"
    DBSCAN = "dbscan"


class LinkageMethod(str, Enum):
    """Linkage methods for hierarchical clustering"""
    SINGLE = "single"
    COMPLETE = "complete"
    AVERAGE = "average"
    WARD = "ward"


@dataclass
class ClusterResult:
    """
    Result of clustering operation.
    
    Attributes:
        labels: Cluster label for each embedding
        n_clusters: Number of clusters found
        centroids: Cluster centroids (if applicable)
        silhouette_score: Quality metric
    """
    labels: List[int]
    n_clusters: int
    centroids: Optional[List[List[float]]] = None
    silhouette_score: Optional[float] = None


class EmbeddingClusterer:
    """
    Clustering algorithms for embeddings.
    
    Provides multiple clustering methods for organizing embeddings.
    """
    
    @staticmethod
    def kmeans(
        embeddings: List[List[float]],
        n_clusters: int,
        max_iterations: int = 300,
        random_seed: Optional[int] = None
    ) -> ClusterResult:
        """
        K-Means clustering on embeddings.
        
        Args:
            embeddings: List of embedding vectors
            n_clusters: Number of clusters
            max_iterations: Maximum iterations
            random_seed: Random seed for reproducibility
            
        Returns:
            Cluster result with labels and centroids
        """
        from sklearn.cluster import KMeans
        
        X = np.array(embeddings, dtype=np.float32)
        
        kmeans = KMeans(
            n_clusters=n_clusters,
            max_iter=max_iterations,
            random_state=random_seed,
            n_init=10
        )
        
        labels = kmeans.fit_predict(X)
        centroids = kmeans.cluster_centers_.tolist()
        
        # Compute silhouette score
        silhouette = EmbeddingClusterer._compute_silhouette_score(X, labels)
        
        logger.info(f"K-Means clustering: {n_clusters} clusters, silhouette={silhouette:.3f}")
        
        return ClusterResult(
            labels=labels.tolist(),
            n_clusters=n_clusters,
            centroids=centroids,
            silhouette_score=silhouette
        )
    
    @staticmethod
    def hierarchical(
        embeddings: List[List[float]],
        n_clusters: int,
        linkage: LinkageMethod = LinkageMethod.WARD
    ) -> ClusterResult:
        """
        Hierarchical clustering on embeddings.
        
        Args:
            embeddings: List of embedding vectors
            n_clusters: Number of clusters
            linkage: Linkage method
            
        Returns:
            Cluster result with labels
        """
        from sklearn.cluster import AgglomerativeClustering
        
        X = np.array(embeddings, dtype=np.float32)
        
        clustering = AgglomerativeClustering(
            n_clusters=n_clusters,
            linkage=linkage.value
        )
        
        labels = clustering.fit_predict(X)
        
        # Compute silhouette score
        silhouette = EmbeddingClusterer._compute_silhouette_score(X, labels)
        
        logger.info(f"Hierarchical clustering: {n_clusters} clusters, silhouette={silhouette:.3f}")
        
        return ClusterResult(
            labels=labels.tolist(),
            n_clusters=n_clusters,
            silhouette_score=silhouette
        )
    
    @staticmethod
    def dbscan(
        embeddings: List[List[float]],
        eps: float = 0.5,
        min_samples: int = 5
    ) -> ClusterResult:
        """
        DBSCAN (density-based) clustering.
        
        Automatically determines number of clusters.
        Good for finding outliers.
        
        Args:
            embeddings: List of embedding vectors
            eps: Maximum distance between samples
            min_samples: Minimum samples in neighborhood
            
        Returns:
            Cluster result (noise points labeled as -1)
        """
        from sklearn.cluster import DBSCAN
        
        X = np.array(embeddings, dtype=np.float32)
        
        clustering = DBSCAN(eps=eps, min_samples=min_samples, metric='cosine')
        labels = clustering.fit_predict(X)
        
        # Count clusters (excluding noise = -1)
        n_clusters = len(set(labels)) - (1 if -1 in labels else 0)
        n_noise = list(labels).count(-1)
        
        logger.info(f"DBSCAN clustering: {n_clusters} clusters, {n_noise} noise points")
        
        return ClusterResult(
            labels=labels.tolist(),
            n_clusters=n_clusters
        )
    
    @staticmethod
    def _compute_silhouette_score(X: np.ndarray, labels: np.ndarray) -> float:
        """Compute silhouette score for cluster quality"""
        try:
            from sklearn.metrics import silhouette_score
            
            # Need at least 2 clusters
            if len(set(labels)) < 2:
                return 0.0
            
            return float(silhouette_score(X, labels, metric='cosine'))
        
        except Exception as e:
            logger.warning(f"Could not compute silhouette score: {e}")
            return 0.0


class RecommendationEngine:
    """
    Recommendation system using embeddings.
    
    Content-based recommendations using embedding similarity.
    """
    
    def __init__(
        self,
        item_embeddings: List[List[float]],
        item_ids: Optional[List[str]] = None,
        item_metadata: Optional[List[Dict[str, Any]]] = None
    ):
        """
        Initialize recommendation engine.
        
        Args:
            item_embeddings: Embeddings for all items
            item_ids: Optional item IDs
            item_metadata: Optional metadata for items
        """
        self.item_embeddings = item_embeddings
        self.item_ids = item_ids or [str(i) for i in range(len(item_embeddings))]
        self.item_metadata = item_metadata
        
        logger.info(f"RecommendationEngine initialized with {len(item_embeddings)} items")
    
    def recommend_similar_items(
        self,
        item_index: int,
        k: int = 10,
        exclude_self: bool = True
    ) -> List[Dict[str, Any]]:
        """
        Recommend items similar to given item.
        
        Content-based filtering using embedding similarity.
        
        Args:
            item_index: Index of item to find similar items for
            k: Number of recommendations
            exclude_self: Exclude the item itself from results
            
        Returns:
            List of recommended items with similarity scores
        """
        from core.embedding_eval import EmbeddingEvaluator
        
        if item_index < 0 or item_index >= len(self.item_embeddings):
            raise ValueError(f"Invalid item index: {item_index}")
        
        query_embedding = self.item_embeddings[item_index]
        
        # Find similar items
        results = EmbeddingEvaluator.find_top_k_similar(
            query_embedding,
            self.item_embeddings,
            k=k + (1 if exclude_self else 0),  # +1 to account for self
            metadata=[{"item_id": iid, "metadata": meta} 
                     for iid, meta in zip(self.item_ids, self.item_metadata or [{}] * len(self.item_ids))]
        )
        
        # Remove self if requested
        if exclude_self:
            results = [r for r in results if r.index != item_index][:k]
        
        recommendations = [
            {
                "item_id": self.item_ids[r.index],
                "item_index": r.index,
                "similarity_score": r.score,
                "metadata": r.metadata
            }
            for r in results
        ]
        
        logger.info(f"Generated {len(recommendations)} recommendations for item {item_index}")
        return recommendations
    
    def recommend_for_user_profile(
        self,
        user_embedding: List[float],
        k: int = 10,
        score_threshold: Optional[float] = None
    ) -> List[Dict[str, Any]]:
        """
        Recommend items for user based on user profile embedding.
        
        User profile embedding can be average of liked items, for example.
        
        Args:
            user_embedding: User profile embedding
            k: Number of recommendations
            score_threshold: Minimum similarity threshold
            
        Returns:
            List of recommended items
        """
        from core.embedding_eval import EmbeddingEvaluator
        
        results = EmbeddingEvaluator.find_top_k_similar(
            user_embedding,
            self.item_embeddings,
            k=k,
            metadata=[{"item_id": iid, "metadata": meta} 
                     for iid, meta in zip(self.item_ids, self.item_metadata or [{}] * len(self.item_ids))]
        )
        
        # Filter by score threshold
        if score_threshold:
            results = [r for r in results if r.score >= score_threshold]
        
        recommendations = [
            {
                "item_id": self.item_ids[r.index],
                "item_index": r.index,
                "similarity_score": r.score,
                "metadata": r.metadata
            }
            for r in results
        ]
        
        logger.info(f"Generated {len(recommendations)} recommendations for user profile")
        return recommendations
    
    def find_diverse_recommendations(
        self,
        user_embedding: List[float],
        k: int = 10,
        diversity_weight: float = 0.3
    ) -> List[Dict[str, Any]]:
        """
        Recommend diverse items (avoid redundancy).
        
        Balances relevance with diversity using MMR-like approach.
        
        Args:
            user_embedding: User profile embedding
            k: Number of recommendations
            diversity_weight: Weight for diversity (0=pure relevance, 1=pure diversity)
            
        Returns:
            List of diverse recommendations
        """
        from core.embedding_eval import EmbeddingEvaluator
        
        # Get more candidates than needed
        candidates = EmbeddingEvaluator.find_top_k_similar(
            user_embedding,
            self.item_embeddings,
            k=k * 3
        )
        
        if not candidates:
            return []
        
        # MMR-like selection
        selected = [candidates[0]]  # Start with most relevant
        candidates = candidates[1:]
        
        while len(selected) < k and candidates:
            best_score = -float('inf')
            best_idx = 0
            
            for idx, candidate in enumerate(candidates):
                # Relevance score
                relevance = candidate.score
                
                # Diversity score (minimum similarity to selected items)
                min_sim = 1.0
                for selected_item in selected:
                    sel_emb = self.item_embeddings[selected_item.index]
                    cand_emb = self.item_embeddings[candidate.index]
                    sim = EmbeddingEvaluator.cosine_similarity(sel_emb, cand_emb)
                    min_sim = min(min_sim, sim)
                
                diversity = 1.0 - min_sim
                
                # Combined score
                score = (1 - diversity_weight) * relevance + diversity_weight * diversity
                
                if score > best_score:
                    best_score = score
                    best_idx = idx
            
            selected.append(candidates.pop(best_idx))
        
        recommendations = [
            {
                "item_id": self.item_ids[r.index],
                "item_index": r.index,
                "similarity_score": r.score,
                "metadata": r.metadata
            }
            for r in selected
        ]
        
        logger.info(f"Generated {len(recommendations)} diverse recommendations")
        return recommendations

