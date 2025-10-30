"""
EmbeddingPipeline - Text embedding generation

For: Sentence transformers, embedding models
Supports: E5, BGE, Instructor, all-MiniLM, etc.

Uses sentence-transformers for efficient batch embedding generation.
"""

import logging
from typing import Any, Dict, List, Optional, Union
from .base import BasePipeline

logger = logging.getLogger(__name__)


class EmbeddingPipeline(BasePipeline):
    """
    Embedding generation pipeline.
    
    Uses sentence-transformers for efficient text embedding.
    Supports batching, pooling strategies, and normalization.
    """
    
    def pipeline_type(self) -> str:
        return "feature-extraction"
    
    def load(self, model_id: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Load embedding model using sentence-transformers.
        
        Args:
            model_id: HuggingFace model ID (e.g., "sentence-transformers/all-MiniLM-L6-v2")
            options: Loading options (device, etc.)
        
        Returns:
            Status dict with 'status' and 'message'
        """
        try:
            logger.info(f"[Embedding] Loading model: {model_id}")
            
            from sentence_transformers import SentenceTransformer
            import torch
            
            opts = options or {}
            
            # Determine device
            device = opts.get("device", "cuda" if torch.cuda.is_available() else "cpu")
            logger.info(f"[Embedding] Using device: {device}")
            
            # Load sentence-transformer model
            logger.info(f"[Embedding] Initializing SentenceTransformer...")
            self.model = SentenceTransformer(
                model_id,
                device=device,
                trust_remote_code=opts.get("trust_remote_code", False)
            )
            
            # Get model info
            embedding_dim = self.model.get_sentence_embedding_dimension()
            max_seq_length = self.model.max_seq_length
            
            self._loaded = True
            logger.info(f"[Embedding] ✅ Model loaded: dim={embedding_dim}, max_length={max_seq_length}")
            
            return {
                "status": "success",
                "message": f"Model {model_id} loaded on {device}",
                "device": device,
                "embedding_dimension": embedding_dim,
                "max_sequence_length": max_seq_length
            }
            
        except Exception as e:
            logger.error(f"[Embedding] ❌ Load failed: {e}", exc_info=True)
            self._loaded = False
            return {
                "status": "error",
                "message": f"Failed to load model: {str(e)}"
            }
    
    def generate(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Generate embeddings for text(s).
        
        Args:
            input_data: Dict with:
                - texts: Single text string or list of texts
                - batch_size: Batch size for encoding (default: 32)
                - normalize_embeddings: Whether to L2 normalize (default: True)
                - show_progress_bar: Show progress for large batches (default: False)
                - convert_to_numpy: Return numpy arrays (default: False)
        
        Returns:
            Dict with 'status', 'embeddings', and metadata
        """
        if not self.is_loaded():
            return {"status": "error", "message": "Model not loaded"}
        
        try:
            # Get input texts
            texts = input_data.get("texts")
            if not texts:
                return {"status": "error", "message": "No texts provided"}
            
            # Handle single string input
            single_input = isinstance(texts, str)
            if single_input:
                texts = [texts]
            
            # Get encoding parameters
            batch_size = input_data.get("batch_size", 32)
            normalize = input_data.get("normalize_embeddings", True)
            show_progress = input_data.get("show_progress_bar", False)
            to_numpy = input_data.get("convert_to_numpy", False)
            
            logger.debug(f"[Embedding] Encoding {len(texts)} texts (batch_size={batch_size})")
            
            # Generate embeddings
            embeddings = self.model.encode(
                texts,
                batch_size=batch_size,
                normalize_embeddings=normalize,
                show_progress_bar=show_progress,
                convert_to_numpy=to_numpy
            )
            
            # Convert to list format for gRPC serialization
            if not to_numpy:
                import torch
                if isinstance(embeddings, torch.Tensor):
                    embeddings = embeddings.cpu().numpy()
            
            embeddings_list = embeddings.tolist()
            
            # Return single embedding if single input
            if single_input:
                embeddings_list = embeddings_list[0]
            
            logger.debug(f"[Embedding] ✅ Generated {len(texts)} embeddings")
            
            return {
                "status": "success",
                "embeddings": embeddings_list,
                "count": len(texts) if not single_input else 1,
                "dimension": len(embeddings_list[0]) if not single_input else len(embeddings_list)
            }
            
        except Exception as e:
            logger.error(f"[Embedding] ❌ Generation failed: {e}", exc_info=True)
            return {
                "status": "error",
                "message": f"Embedding generation failed: {str(e)}"
            }
    
    def similarity(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Compute semantic similarity between texts.
        
        Args:
            input_data: Dict with:
                - texts1: First text(s)
                - texts2: Second text(s)
                - metric: 'cosine' or 'dot' (default: 'cosine')
        
        Returns:
            Dict with 'status' and 'similarities'
        """
        if not self.is_loaded():
            return {"status": "error", "message": "Model not loaded"}
        
        try:
            from sentence_transformers import util
            
            texts1 = input_data.get("texts1")
            texts2 = input_data.get("texts2")
            
            if not texts1 or not texts2:
                return {"status": "error", "message": "Both texts1 and texts2 required"}
            
            # Generate embeddings
            emb1 = self.model.encode(texts1, convert_to_tensor=True)
            emb2 = self.model.encode(texts2, convert_to_tensor=True)
            
            # Compute similarity
            metric = input_data.get("metric", "cosine")
            if metric == "cosine":
                similarities = util.cos_sim(emb1, emb2)
            elif metric == "dot":
                similarities = util.dot_score(emb1, emb2)
            else:
                return {"status": "error", "message": f"Unknown metric: {metric}"}
            
            return {
                "status": "success",
                "similarities": similarities.cpu().numpy().tolist(),
                "metric": metric
            }
            
        except Exception as e:
            logger.error(f"[Embedding] ❌ Similarity computation failed: {e}", exc_info=True)
            return {
                "status": "error",
                "message": f"Similarity computation failed: {str(e)}"
            }
    
    def unload(self):
        """Unload model from memory"""
        try:
            if hasattr(self, 'model'):
                del self.model
            
            # Clear CUDA cache if using GPU
            import torch
            if torch.cuda.is_available():
                torch.cuda.empty_cache()
            
            self._loaded = False
            logger.info("[Embedding] Model unloaded")
            
        except Exception as e:
            logger.error(f"[Embedding] Error during unload: {e}")
