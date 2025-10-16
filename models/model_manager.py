"""
Model library and download management for TabAgent.

Provides:
- Curated model library
- HuggingFace integration
- Model discovery and search
- Download management
- User custom models
"""

import json
import logging
from pathlib import Path
from typing import Dict, List, Optional
from enum import Enum
from dataclasses import dataclass

from core.message_types import ModelType


logger = logging.getLogger(__name__)


class ModelStatus(str, Enum):
    """Model download/availability status"""
    AVAILABLE = "available"
    DOWNLOADED = "downloaded"
    NOT_FOUND = "not_found"
    DOWNLOADING = "downloading"
    ERROR = "error"


class ModelUseCase(str, Enum):
    """Model use case categories"""
    CHAT = "chat"
    CODE = "code"
    REASONING = "reasoning"
    GENERAL = "general"
    EXPERIMENTAL = "experimental"
    LOW_RESOURCE = "low-resource"


class ModelLicense(str, Enum):
    """Common model licenses"""
    MIT = "mit"
    APACHE_2_0 = "apache-2.0"
    LLAMA3 = "llama3"
    GEMMA = "gemma"
    UNKNOWN = "unknown"


@dataclass(frozen=True)
class ModelInfo:
    """
    Information about a model in the library.
    
    Attributes:
        name: Model identifier/name
        repo: HuggingFace repository
        model_type: Type of model (gguf, bitnet, etc.)
        variants: Available quantization variants
        description: Human-readable description
        size_gb: Estimated size in GB
        context_length: Maximum context length
        recommended: Whether this is a recommended model
        use_cases: List of use cases
        license: Model license
        notes: Additional notes
    """
    name: str
    repo: str
    model_type: ModelType
    variants: List[str]
    description: str
    size_gb: float
    context_length: int
    recommended: bool
    use_cases: List[ModelUseCase]
    license: ModelLicense
    notes: Optional[str] = None


class ModelLibrary:
    """
    Curated model library manager.
    
    Manages built-in and user-added models.
    """
    
    def __init__(self, library_path: Optional[Path] = None):
        """
        Initialize model library.
        
        Args:
            library_path: Path to models_library.json (default: core/models_library.json)
        """
        if library_path is None:
            library_path = Path(__file__).parent / "models_library.json"
        
        self.library_path = library_path
        self._models: Dict[str, ModelInfo] = {}
        self._load_library()
        
        logger.info(f"ModelLibrary initialized with {len(self._models)} models")
    
    def _load_library(self) -> None:
        """Load model library from JSON file"""
        try:
            with open(self.library_path, 'r', encoding='utf-8') as f:
                data = json.load(f)
            
            models_data = data.get("models", {})
            
            for name, info in models_data.items():
                try:
                    # Parse model type
                    model_type_str = info.get("type", "gguf_regular")
                    try:
                        model_type = ModelType(model_type_str)
                    except ValueError:
                        logger.warning(f"Unknown model type '{model_type_str}' for {name}, defaulting to gguf_regular")
                        model_type = ModelType.GGUF_REGULAR
                    
                    # Parse use cases
                    use_cases_str = info.get("use_cases", ["general"])
                    use_cases = []
                    for uc in use_cases_str:
                        try:
                            use_cases.append(ModelUseCase(uc))
                        except ValueError:
                            logger.warning(f"Unknown use case '{uc}' for {name}")
                    
                    # Parse license
                    license_str = info.get("license", "unknown")
                    try:
                        license_type = ModelLicense(license_str)
                    except ValueError:
                        license_type = ModelLicense.UNKNOWN
                    
                    # Create ModelInfo
                    model_info = ModelInfo(
                        name=name,
                        repo=info["repo"],
                        model_type=model_type,
                        variants=info.get("variants", []),
                        description=info.get("description", ""),
                        size_gb=float(info.get("size_gb", 0.0)),
                        context_length=int(info.get("context_length", 4096)),
                        recommended=bool(info.get("recommended", False)),
                        use_cases=use_cases,
                        license=license_type,
                        notes=info.get("notes")
                    )
                    
                    self._models[name] = model_info
                    
                except Exception as e:
                    logger.error(f"Error parsing model '{name}': {e}")
                    continue
            
            logger.info(f"Loaded {len(self._models)} models from library")
            
        except FileNotFoundError:
            logger.warning(f"Model library not found: {self.library_path}")
        except json.JSONDecodeError as e:
            logger.error(f"Invalid JSON in model library: {e}")
        except Exception as e:
            logger.error(f"Error loading model library: {e}")
    
    def get_model(self, name: str) -> Optional[ModelInfo]:
        """
        Get model information by name.
        
        Args:
            name: Model name
            
        Returns:
            ModelInfo if found, None otherwise
        """
        return self._models.get(name)
    
    def list_models(
        self,
        model_type: Optional[ModelType] = None,
        use_case: Optional[ModelUseCase] = None,
        recommended_only: bool = False
    ) -> List[ModelInfo]:
        """
        List models with optional filtering.
        
        Args:
            model_type: Filter by model type
            use_case: Filter by use case
            recommended_only: Only show recommended models
            
        Returns:
            List of ModelInfo objects
        """
        models = list(self._models.values())
        
        # Apply filters
        if model_type is not None:
            models = [m for m in models if m.model_type == model_type]
        
        if use_case is not None:
            models = [m for m in models if use_case in m.use_cases]
        
        if recommended_only:
            models = [m for m in models if m.recommended]
        
        # Sort by size (smallest first)
        models.sort(key=lambda m: m.size_gb)
        
        return models
    
    def search_models(self, query: str) -> List[ModelInfo]:
        """
        Search models by name or description.
        
        Args:
            query: Search query string
            
        Returns:
            List of matching ModelInfo objects
        """
        query_lower = query.lower()
        matches = []
        
        for model in self._models.values():
            # Search in name, description, repo
            if (query_lower in model.name.lower() or
                query_lower in model.description.lower() or
                query_lower in model.repo.lower()):
                matches.append(model)
        
        return matches
    
    def get_recommended_models(self) -> List[ModelInfo]:
        """
        Get recommended models.
        
        Returns:
            List of recommended ModelInfo objects
        """
        return self.list_models(recommended_only=True)


class ModelManager:
    """
    Model management including downloads and caching.
    
    Integrates with HuggingFace for model downloads.
    """
    
    def __init__(
        self,
        library_path: Optional[Path] = None,
        cache_dir: Optional[Path] = None
    ):
        """
        Initialize model manager.
        
        Args:
            library_path: Path to models_library.json
            cache_dir: HuggingFace cache directory
        """
        self.library = ModelLibrary(library_path)
        self.cache_dir = cache_dir
        
        # Try to import HuggingFace hub
        self._has_hf_hub = False
        try:
            import huggingface_hub
            self._hf_hub = huggingface_hub
            self._has_hf_hub = True
            logger.info("HuggingFace Hub available for downloads")
        except ImportError:
            logger.warning("HuggingFace Hub not installed, downloads disabled")
    
    def is_model_downloaded(self, model_name: str) -> bool:
        """
        Check if a model is downloaded.
        
        Args:
            model_name: Model name from library
            
        Returns:
            True if model is downloaded
        """
        if not self._has_hf_hub:
            return False
        
        model_info = self.library.get_model(model_name)
        if not model_info:
            return False
        
        try:
            # Check if model repo is in cache
            cache_info = self._hf_hub.scan_cache_dir()
            repos = {entry.repo_id for entry in cache_info.repos}
            return model_info.repo in repos
        except Exception as e:
            logger.debug(f"Error checking download status: {e}")
            return False
    
    def get_model_status(self, model_name: str) -> ModelStatus:
        """
        Get model download/availability status.
        
        Args:
            model_name: Model name from library
            
        Returns:
            ModelStatus enum value
        """
        model_info = self.library.get_model(model_name)
        if not model_info:
            return ModelStatus.NOT_FOUND
        
        if self.is_model_downloaded(model_name):
            return ModelStatus.DOWNLOADED
        
        return ModelStatus.AVAILABLE
    
    def download_model(
        self,
        model_name: str,
        variant: Optional[str] = None
    ) -> bool:
        """
        Download a model from HuggingFace.
        
        Args:
            model_name: Model name from library
            variant: Specific variant to download (optional)
            
        Returns:
            True if download successful
            
        Raises:
            RuntimeError: If HuggingFace Hub not available
            ValueError: If model not found
        """
        if not self._has_hf_hub:
            raise RuntimeError(
                "HuggingFace Hub not available. "
                "Install with: pip install huggingface-hub"
            )
        
        model_info = self.library.get_model(model_name)
        if not model_info:
            raise ValueError(f"Model not found: {model_name}")
        
        logger.info(f"Downloading model: {model_name} from {model_info.repo}")
        
        try:
            # Download model repository
            local_path = self._hf_hub.snapshot_download(
                repo_id=model_info.repo,
                cache_dir=self.cache_dir,
                local_files_only=False
            )
            
            logger.info(f"Model downloaded successfully to: {local_path}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to download model: {e}")
            return False
    
    def get_model_path(self, model_name: str) -> Optional[Path]:
        """
        Get local path to downloaded model.
        
        Args:
            model_name: Model name from library
            
        Returns:
            Path to model if downloaded, None otherwise
        """
        if not self._has_hf_hub:
            return None
        
        model_info = self.library.get_model(model_name)
        if not model_info or not self.is_model_downloaded(model_name):
            return None
        
        try:
            # Get path from HuggingFace cache
            local_path = self._hf_hub.snapshot_download(
                repo_id=model_info.repo,
                cache_dir=self.cache_dir,
                local_files_only=True  # Don't download, just get path
            )
            return Path(local_path)
        except Exception as e:
            logger.error(f"Error getting model path: {e}")
            return None

