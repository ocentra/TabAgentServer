"""
CrossEncoderPipeline - Document reranking

For: Cross-encoder models for reranking search results
Examples: ms-marco-MiniLM, bge-reranker, cross-encoder/ms-marco-MiniLM-L-6-v2

Uses sentence-transformers CrossEncoder for efficient reranking.
"""

import logging
from typing import Any, Dict, List, Optional, Tuple
from .base import BasePipeline

logger = logging.getLogger(__name__)


class CrossEncoderPipeline(BasePipeline):
    """
    Cross-encoder reranking pipeline.
    
    Scores query-document pairs for relevance ranking.
    Uses sentence-transformers CrossEncoder for efficient inference.
    """
    
    def pipeline_type(self) -> str:
        return "text-classification"
    
    def load(self, model_id: str, options: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        """
        Load cross-encoder model using sentence-transformers.
        
        Args:
            model_id: HuggingFace model ID (e.g., "cross-encoder/ms-marco-MiniLM-L-6-v2")
            options: Loading options (device, max_length, etc.)
        
        Returns:
            Status dict with 'status' and 'message'
        """
        try:
            logger.info(f"[CrossEncoder] Loading model: {model_id}")
            
            from sentence_transformers import CrossEncoder
            import torch
            
            opts = options or {}
            
            # Determine device
            device = opts.get("device", "cuda" if torch.cuda.is_available() else "cpu")
            logger.info(f"[CrossEncoder] Using device: {device}")
            
            # Load cross-encoder
            logger.info(f"[CrossEncoder] Initializing CrossEncoder...")
            self.model = CrossEncoder(
                model_id,
                device=device,
                max_length=opts.get("max_length", 512),
                trust_remote_code=opts.get("trust_remote_code", False)
            )
            
            self._loaded = True
            logger.info(f"[CrossEncoder] ✅ Model loaded successfully on {device}")
            
            return {
                "status": "success",
                "message": f"Model {model_id} loaded on {device}",
                "device": device,
                "max_length": opts.get("max_length", 512)
            }
            
        except Exception as e:
            logger.error(f"[CrossEncoder] ❌ Load failed: {e}", exc_info=True)
            self._loaded = False
            return {
                "status": "error",
                "message": f"Failed to load model: {str(e)}"
            }
    
    def generate(self, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """
        Score query-document pairs for reranking.
        
        Args:
            input_data: Dict with:
                - query: Search query string
                - documents: List of document texts
                - batch_size: Batch size for scoring (default: 32)
                - top_k: Return only top K results (default: all)
                - show_progress_bar: Show progress for large batches (default: False)
        
        Returns:
            Dict with 'status', 'ranked_documents', and 'scores'
        """
        if not self.is_loaded():
            return {"status": "error", "message": "Model not loaded"}
        
        try:
            query = input_data.get("query")
            documents = input_data.get("documents")
            
            if not query:
                return {"status": "error", "message": "No query provided"}
            if not documents:
                return {"status": "error", "message": "No documents provided"}
            
            # Get parameters
            batch_size = input_data.get("batch_size", 32)
            top_k = input_data.get("top_k", len(documents))
            show_progress = input_data.get("show_progress_bar", False)
            
            logger.debug(f"[CrossEncoder] Scoring {len(documents)} documents for query")
            
            # Create query-document pairs
            pairs = [[query, doc] for doc in documents]
            
            # Score all pairs
            scores = self.model.predict(
                pairs,
                batch_size=batch_size,
                show_progress_bar=show_progress
            )
            
            # Convert scores to list
            scores_list = scores.tolist() if hasattr(scores, 'tolist') else list(scores)
            
            # Create ranked results
            results = list(zip(documents, scores_list))
            results.sort(key=lambda x: x[1], reverse=True)
            
            # Apply top_k
            results = results[:top_k]
            
            logger.debug(f"[CrossEncoder] ✅ Ranked {len(results)} documents")
            
            return {
                "status": "success",
                "ranked_documents": [
                    {
                        "text": doc,
                        "score": float(score),
                        "rank": i + 1
                    }
                    for i, (doc, score) in enumerate(results)
                ],
                "query": query
            }
            
        except Exception as e:
            logger.error(f"[CrossEncoder] ❌ Scoring failed: {e}", exc_info=True)
            return {
                "status": "error",
                "message": f"Scoring failed: {str(e)}"
            }
    
    def unload(self):
        """Unload model from memory"""
        try:
            if hasattr(self, 'model'):
                del self.model
            
            import torch
            if torch.cuda.is_available():
                torch.cuda.empty_cache()
            
            self._loaded = False
            logger.info("[CrossEncoder] Model unloaded")
            
        except Exception as e:
            logger.error(f"[CrossEncoder] Error during unload: {e}")
