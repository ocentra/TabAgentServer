"""
Embedding Model Registry and Utilities

Comprehensive support for embedding models across modalities:
- Text embeddings (semantic search, classification)
- Image embeddings (visual search, similarity)
- Audio embeddings (speech, music)
- Multi-modal embeddings (CLIP, ImageBind)

Based on state-of-the-art models for production use.
"""

from enum import Enum
from dataclasses import dataclass
from typing import List, Dict, Any, Optional


class EmbeddingModality(str, Enum):
    """Embedding modality types"""
    TEXT = "text"
    IMAGE = "image"
    AUDIO = "audio"
    VIDEO = "video"
    MULTIMODAL = "multimodal"


class EmbeddingUseCase(str, Enum):
    """Common embedding use cases"""
    SEMANTIC_SEARCH = "semantic_search"
    CLASSIFICATION = "classification"
    CLUSTERING = "clustering"
    RECOMMENDATION = "recommendation"
    SIMILARITY = "similarity"
    RETRIEVAL = "retrieval"
    RERANKING = "reranking"


class EmbeddingModelSize(str, Enum):
    """Model size categories"""
    TINY = "tiny"         # < 50M params
    SMALL = "small"       # 50M - 150M params
    BASE = "base"         # 150M - 400M params
    LARGE = "large"       # 400M - 1B params
    XLARGE = "xlarge"     # > 1B params


@dataclass
class EmbeddingModelInfo:
    """
    Information about an embedding model.
    
    Attributes:
        model_id: Unique identifier
        name: Display name
        modality: Primary modality
        dimension: Embedding vector dimension
        use_cases: Supported use cases
        size: Model size category
        backend: Recommended backend (onnx, llamacpp, mediapipe)
        normalize: Whether embeddings are pre-normalized
        repo_id: HuggingFace repository ID
    """
    model_id: str
    name: str
    modality: EmbeddingModality
    dimension: int
    use_cases: List[EmbeddingUseCase]
    size: EmbeddingModelSize
    backend: str
    normalize: bool = True
    repo_id: Optional[str] = None
    description: Optional[str] = None


class EmbeddingModelRegistry:
    """
    Registry of popular embedding models.
    
    Curated list of state-of-the-art models for various use cases.
    """
    
    # Text Embedding Models
    TEXT_MODELS = {
        "all-minilm-l6-v2": EmbeddingModelInfo(
            model_id="all-minilm-l6-v2",
            name="All-MiniLM-L6-v2",
            modality=EmbeddingModality.TEXT,
            dimension=384,
            use_cases=[
                EmbeddingUseCase.SEMANTIC_SEARCH,
                EmbeddingUseCase.SIMILARITY,
                EmbeddingUseCase.CLUSTERING
            ],
            size=EmbeddingModelSize.SMALL,
            backend="onnx",
            repo_id="sentence-transformers/all-MiniLM-L6-v2",
            description="Fast, lightweight text embeddings. Great for general purpose."
        ),
        "all-mpnet-base-v2": EmbeddingModelInfo(
            model_id="all-mpnet-base-v2",
            name="All-MPNet-Base-v2",
            modality=EmbeddingModality.TEXT,
            dimension=768,
            use_cases=[
                EmbeddingUseCase.SEMANTIC_SEARCH,
                EmbeddingUseCase.RETRIEVAL,
                EmbeddingUseCase.SIMILARITY
            ],
            size=EmbeddingModelSize.BASE,
            backend="onnx",
            repo_id="sentence-transformers/all-mpnet-base-v2",
            description="High-quality text embeddings. Best overall performance."
        ),
        "bge-small-en-v1.5": EmbeddingModelInfo(
            model_id="bge-small-en-v1.5",
            name="BGE Small EN v1.5",
            modality=EmbeddingModality.TEXT,
            dimension=384,
            use_cases=[
                EmbeddingUseCase.RETRIEVAL,
                EmbeddingUseCase.SEMANTIC_SEARCH
            ],
            size=EmbeddingModelSize.SMALL,
            backend="onnx",
            repo_id="BAAI/bge-small-en-v1.5",
            description="BAAI's BGE model. Excellent for retrieval tasks."
        ),
        "bge-base-en-v1.5": EmbeddingModelInfo(
            model_id="bge-base-en-v1.5",
            name="BGE Base EN v1.5",
            modality=EmbeddingModality.TEXT,
            dimension=768,
            use_cases=[
                EmbeddingUseCase.RETRIEVAL,
                EmbeddingUseCase.SEMANTIC_SEARCH,
                EmbeddingUseCase.RERANKING
            ],
            size=EmbeddingModelSize.BASE,
            backend="onnx",
            repo_id="BAAI/bge-base-en-v1.5",
            description="State-of-the-art retrieval performance."
        ),
        "e5-small-v2": EmbeddingModelInfo(
            model_id="e5-small-v2",
            name="E5 Small v2",
            modality=EmbeddingModality.TEXT,
            dimension=384,
            use_cases=[
                EmbeddingUseCase.SEMANTIC_SEARCH,
                EmbeddingUseCase.RETRIEVAL
            ],
            size=EmbeddingModelSize.SMALL,
            backend="onnx",
            repo_id="intfloat/e5-small-v2",
            description="Microsoft E5 embeddings. Competitive performance."
        ),
        "gte-small": EmbeddingModelInfo(
            model_id="gte-small",
            name="GTE Small",
            modality=EmbeddingModality.TEXT,
            dimension=384,
            use_cases=[
                EmbeddingUseCase.SEMANTIC_SEARCH,
                EmbeddingUseCase.RETRIEVAL
            ],
            size=EmbeddingModelSize.SMALL,
            backend="onnx",
            repo_id="thenlper/gte-small",
            description="Alibaba GTE model. Fast and efficient."
        ),
    }
    
    # Image Embedding Models
    IMAGE_MODELS = {
        "clip-vit-base-patch32": EmbeddingModelInfo(
            model_id="clip-vit-base-patch32",
            name="CLIP ViT-B/32",
            modality=EmbeddingModality.MULTIMODAL,
            dimension=512,
            use_cases=[
                EmbeddingUseCase.SIMILARITY,
                EmbeddingUseCase.RETRIEVAL,
                EmbeddingUseCase.CLASSIFICATION
            ],
            size=EmbeddingModelSize.BASE,
            backend="onnx",
            repo_id="openai/clip-vit-base-patch32",
            description="OpenAI CLIP. Text-image multimodal embeddings."
        ),
        "clip-vit-large-patch14": EmbeddingModelInfo(
            model_id="clip-vit-large-patch14",
            name="CLIP ViT-L/14",
            modality=EmbeddingModality.MULTIMODAL,
            dimension=768,
            use_cases=[
                EmbeddingUseCase.SIMILARITY,
                EmbeddingUseCase.RETRIEVAL,
                EmbeddingUseCase.RECOMMENDATION
            ],
            size=EmbeddingModelSize.LARGE,
            backend="onnx",
            repo_id="openai/clip-vit-large-patch14",
            description="Larger CLIP model. Better quality, slower inference."
        ),
    }
    
    # Specialized Models
    SPECIALIZED_MODELS = {
        "bge-reranker-base": EmbeddingModelInfo(
            model_id="bge-reranker-base",
            name="BGE Reranker Base",
            modality=EmbeddingModality.TEXT,
            dimension=768,
            use_cases=[EmbeddingUseCase.RERANKING],
            size=EmbeddingModelSize.BASE,
            backend="onnx",
            repo_id="BAAI/bge-reranker-base",
            description="Cross-encoder for document reranking. High accuracy."
        ),
        "bge-reranker-large": EmbeddingModelInfo(
            model_id="bge-reranker-large",
            name="BGE Reranker Large",
            modality=EmbeddingModality.TEXT,
            dimension=1024,
            use_cases=[EmbeddingUseCase.RERANKING],
            size=EmbeddingModelSize.LARGE,
            backend="onnx",
            repo_id="BAAI/bge-reranker-large",
            description="Larger reranker. Best reranking performance."
        ),
    }
    
    @classmethod
    def get_all_models(cls) -> Dict[str, EmbeddingModelInfo]:
        """Get all registered embedding models"""
        all_models = {}
        all_models.update(cls.TEXT_MODELS)
        all_models.update(cls.IMAGE_MODELS)
        all_models.update(cls.SPECIALIZED_MODELS)
        return all_models
    
    @classmethod
    def get_model(cls, model_id: str) -> Optional[EmbeddingModelInfo]:
        """Get specific model by ID"""
        all_models = cls.get_all_models()
        return all_models.get(model_id)
    
    @classmethod
    def get_models_by_modality(cls, modality: EmbeddingModality) -> Dict[str, EmbeddingModelInfo]:
        """Get all models for specific modality"""
        all_models = cls.get_all_models()
        return {
            model_id: info
            for model_id, info in all_models.items()
            if info.modality == modality
        }
    
    @classmethod
    def get_models_by_use_case(cls, use_case: EmbeddingUseCase) -> Dict[str, EmbeddingModelInfo]:
        """Get all models supporting specific use case"""
        all_models = cls.get_all_models()
        return {
            model_id: info
            for model_id, info in all_models.items()
            if use_case in info.use_cases
        }
    
    @classmethod
    def get_recommended_model(
        cls,
        modality: EmbeddingModality,
        use_case: EmbeddingUseCase,
        prefer_small: bool = True
    ) -> Optional[EmbeddingModelInfo]:
        """
        Get recommended model for modality and use case.
        
        Args:
            modality: Embedding modality
            use_case: Primary use case
            prefer_small: Prefer smaller models (faster)
            
        Returns:
            Recommended model or None
        """
        candidates = cls.get_models_by_use_case(use_case)
        candidates = {
            model_id: info
            for model_id, info in candidates.items()
            if info.modality == modality or info.modality == EmbeddingModality.MULTIMODAL
        }
        
        if not candidates:
            return None
        
        # Sort by size
        sorted_models = sorted(
            candidates.items(),
            key=lambda x: ["tiny", "small", "base", "large", "xlarge"].index(x[1].size.value)
        )
        
        # Return smallest if prefer_small, otherwise largest
        if prefer_small:
            return sorted_models[0][1]
        else:
            return sorted_models[-1][1]


class EmbeddingDimensionInfo:
    """Common embedding dimensions and their trade-offs"""
    
    DIMENSIONS = {
        384: {
            "size": "small",
            "speed": "fast",
            "quality": "good",
            "use_case": "general purpose, mobile, real-time",
            "memory_mb": 1.5,  # Approximate for 1M vectors
        },
        512: {
            "size": "medium",
            "speed": "fast",
            "quality": "very good",
            "use_case": "multimodal (CLIP), balanced performance",
            "memory_mb": 2.0,
        },
        768: {
            "size": "base",
            "speed": "medium",
            "quality": "excellent",
            "use_case": "high-quality retrieval, production systems",
            "memory_mb": 3.0,
        },
        1024: {
            "size": "large",
            "speed": "slower",
            "quality": "state-of-the-art",
            "use_case": "maximum quality, research",
            "memory_mb": 4.0,
        },
        1536: {
            "size": "xlarge",
            "speed": "slow",
            "quality": "top-tier",
            "use_case": "specialized tasks, high-end systems",
            "memory_mb": 6.0,
        },
    }
    
    @classmethod
    def get_info(cls, dimension: int) -> Optional[Dict[str, Any]]:
        """Get information about embedding dimension"""
        return cls.DIMENSIONS.get(dimension)
    
    @classmethod
    def estimate_memory(cls, dimension: int, num_vectors: int) -> float:
        """
        Estimate memory usage for vector storage.
        
        Args:
            dimension: Embedding dimension
            num_vectors: Number of vectors to store
            
        Returns:
            Estimated memory in MB
        """
        # float32 = 4 bytes per dimension
        bytes_per_vector = dimension * 4
        total_bytes = bytes_per_vector * num_vectors
        return total_bytes / (1024 * 1024)  # Convert to MB

