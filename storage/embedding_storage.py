"""
Embedding Storage Manager
==========================

Manages vector embeddings in ArangoDB.
Mirrors IndexedDB Embedding class for consistency.

Each embedding is stored as a document with:
- id: unique embedding ID
- vector: numpy array stored as bytes
- model: model name used for embedding
- source_type: entity type (conversation, message, entity)
- source_id: reference to source node
- dimension: vector dimension
- metadata: additional metadata

Future: When ArangoDB 3.12+ is used, can leverage ArangoSearch for vector similarity.
For now: Store embeddings, provide API for client-side or Python-side similarity search.
"""

import logging
import time
import numpy as np
from typing import List, Dict, Any, Optional, Tuple
from .database import get_database

logger = logging.getLogger(__name__)


class EmbeddingStorage:
    """
    Embedding storage manager.
    
    Mirrors IndexedDB Embedding operations.
    """
    
    def __init__(self):
        self.db = get_database()
    
    def create_embedding(
        self,
        embedding_id: str,
        vector: np.ndarray,
        model: str,
        source_type: str,
        source_id: str,
        metadata: Optional[Dict[str, Any]] = None
    ) -> str:
        """
        Create a new embedding.
        
        Mirrors: IndexedDB Embedding.create()
        
        Args:
            embedding_id: Unique embedding ID
            vector: Embedding vector (numpy array)
            model: Model name used for embedding
            source_type: Type of source (conversation, message, entity)
            source_id: Source node ID
            metadata: Optional additional metadata
            
        Returns:
            Embedding ID
        """
        try:
            embeddings = self.db.get_collection("embeddings")
            
            # Convert numpy array to bytes for storage
            vector_bytes = vector.tobytes()
            dimension = len(vector)
            
            now = int(time.time() * 1000)
            
            doc = {
                "_key": embedding_id,
                "vector_bytes": vector_bytes.hex(),  # Store as hex string
                "dimension": dimension,
                "model": model,
                "source_type": source_type,
                "source_id": source_id,
                "created_at": now,
                "metadata": metadata or {}
            }
            
            embeddings.insert(doc)
            
            # Update source node's embedding_id reference
            self._link_embedding_to_node(source_id, embedding_id)
            
            logger.info(f"Created embedding: {embedding_id} for {source_type}:{source_id}")
            return embedding_id
            
        except Exception as e:
            logger.error(f"Failed to create embedding: {e}")
            raise
    
    def get_embedding(self, embedding_id: str) -> Optional[Dict[str, Any]]:
        """
        Get embedding by ID.
        
        Mirrors: IndexedDB Embedding.read()
        
        Args:
            embedding_id: Embedding ID
            
        Returns:
            Embedding data with vector as numpy array
        """
        try:
            embeddings = self.db.get_collection("embeddings")
            doc = embeddings.get(embedding_id)
            
            if doc:
                return self._format_embedding(doc)
            
            return None
            
        except Exception as e:
            logger.error(f"Failed to get embedding: {e}")
            return None
    
    def get_embedding_by_source(
        self,
        source_type: str,
        source_id: str,
        model: Optional[str] = None
    ) -> Optional[Dict[str, Any]]:
        """
        Get embedding by source.
        
        Args:
            source_type: Source type (conversation, message, entity)
            source_id: Source node ID
            model: Optional model filter
            
        Returns:
            Embedding data or None
        """
        try:
            aql = """
                FOR emb IN embeddings
                    FILTER emb.source_type == @source_type
                    FILTER emb.source_id == @source_id
            """
            
            bind_vars = {
                "source_type": source_type,
                "source_id": source_id
            }
            
            if model:
                aql += " FILTER emb.model == @model"
                bind_vars["model"] = model
            
            aql += " SORT emb.created_at DESC LIMIT 1 RETURN emb"
            
            results = self.db.execute(aql, bind_vars)
            
            if results:
                return self._format_embedding(results[0])
            
            return None
            
        except Exception as e:
            logger.error(f"Failed to get embedding by source: {e}")
            return None
    
    def search_similar(
        self,
        query_vector: np.ndarray,
        source_type: Optional[str] = None,
        model: Optional[str] = None,
        limit: int = 10,
        threshold: float = 0.0
    ) -> List[Tuple[Dict[str, Any], float]]:
        """
        Search for similar embeddings using cosine similarity.
        
        Note: This is a Python-side implementation. For large-scale production,
        consider using ArangoDB 3.12+ with ArangoSearch for native vector search.
        
        Args:
            query_vector: Query embedding vector
            source_type: Optional filter by source type
            model: Optional filter by model
            limit: Maximum results
            threshold: Minimum similarity threshold (0-1)
            
        Returns:
            List of (embedding, similarity_score) tuples, sorted by similarity desc
        """
        try:
            # Build query
            aql = "FOR emb IN embeddings"
            bind_vars: Dict[str, Any] = {}
            
            filters = []
            if source_type:
                filters.append("emb.source_type == @source_type")
                bind_vars["source_type"] = source_type
            if model:
                filters.append("emb.model == @model")
                bind_vars["model"] = model
            
            if filters:
                aql += " FILTER " + " AND ".join(filters)
            
            aql += " RETURN emb"
            
            results = self.db.execute(aql, bind_vars)
            
            # Calculate similarities (Python-side)
            similarities: List[Tuple[Dict[str, Any], float]] = []
            
            for doc in results:
                emb_data = self._format_embedding(doc)
                vector = emb_data["vector"]
                
                # Cosine similarity
                similarity = self._cosine_similarity(query_vector, vector)
                
                if similarity >= threshold:
                    similarities.append((emb_data, float(similarity)))
            
            # Sort by similarity descending
            similarities.sort(key=lambda x: x[1], reverse=True)
            
            return similarities[:limit]
            
        except Exception as e:
            logger.error(f"Failed to search similar embeddings: {e}")
            return []
    
    def delete_embedding(self, embedding_id: str) -> bool:
        """
        Delete embedding.
        
        Mirrors: IndexedDB Embedding.delete()
        
        Args:
            embedding_id: Embedding ID
            
        Returns:
            True if successful
        """
        try:
            embeddings = self.db.get_collection("embeddings")
            embeddings.delete(embedding_id)
            
            logger.info(f"Deleted embedding: {embedding_id}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to delete embedding: {e}")
            return False
    
    def delete_embeddings_by_source(
        self,
        source_type: str,
        source_id: str
    ) -> int:
        """
        Delete all embeddings for a source.
        
        Args:
            source_type: Source type
            source_id: Source ID
            
        Returns:
            Number of deleted embeddings
        """
        try:
            aql = """
                FOR emb IN embeddings
                    FILTER emb.source_type == @source_type
                    FILTER emb.source_id == @source_id
                    REMOVE emb IN embeddings
                    RETURN 1
            """
            
            results = self.db.execute(aql, {
                "source_type": source_type,
                "source_id": source_id
            })
            
            count = len(results)
            logger.info(f"Deleted {count} embeddings for {source_type}:{source_id}")
            return count
            
        except Exception as e:
            logger.error(f"Failed to delete embeddings by source: {e}")
            return 0
    
    def _link_embedding_to_node(self, node_id: str, embedding_id: str) -> None:
        """
        Update node's embedding_id reference.
        
        Args:
            node_id: Node ID
            embedding_id: Embedding ID
        """
        try:
            nodes = self.db.get_collection("nodes")
            node = nodes.get(node_id)
            
            if node:
                node["embedding_id"] = embedding_id
                nodes.update(node)
                
        except Exception as e:
            logger.warning(f"Failed to link embedding to node: {e}")
    
    def _format_embedding(self, doc: Dict[str, Any]) -> Dict[str, Any]:
        """
        Format embedding document for API response.
        
        Converts vector bytes back to numpy array.
        
        Args:
            doc: ArangoDB document
            
        Returns:
            Formatted embedding with numpy vector
        """
        # Convert hex string back to bytes, then to numpy array
        vector_bytes = bytes.fromhex(doc["vector_bytes"])
        vector = np.frombuffer(vector_bytes, dtype=np.float32)
        
        return {
            "id": doc["_key"],
            "vector": vector,
            "dimension": doc.get("dimension"),
            "model": doc.get("model"),
            "source_type": doc.get("source_type"),
            "source_id": doc.get("source_id"),
            "created_at": doc.get("created_at"),
            "metadata": doc.get("metadata", {})
        }
    
    @staticmethod
    def _cosine_similarity(vec1: np.ndarray, vec2: np.ndarray) -> float:
        """
        Calculate cosine similarity between two vectors.
        
        Args:
            vec1: First vector
            vec2: Second vector
            
        Returns:
            Cosine similarity (0-1)
        """
        dot_product = np.dot(vec1, vec2)
        norm1 = np.linalg.norm(vec1)
        norm2 = np.linalg.norm(vec2)
        
        if norm1 == 0 or norm2 == 0:
            return 0.0
        
        return dot_product / (norm1 * norm2)


# Global singleton
_embedding_storage: Optional[EmbeddingStorage] = None


def get_embedding_storage() -> EmbeddingStorage:
    """
    Get global embedding storage instance.
    
    Returns:
        EmbeddingStorage singleton
    """
    global _embedding_storage
    if _embedding_storage is None:
        _embedding_storage = EmbeddingStorage()
    return _embedding_storage
