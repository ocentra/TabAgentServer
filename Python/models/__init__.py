"""
Model library and management for TabAgent.

Provides curated model library, model search, and HuggingFace
integration for model downloads.
"""

from .model_manager import (
    ModelLibrary,
    ModelManager,
    ModelInfo,
    ModelStatus,
    ModelUseCase,
    ModelLicense,
)

__all__ = [
    'ModelLibrary',
    'ModelManager',
    'ModelInfo',
    'ModelStatus',
    'ModelUseCase',
    'ModelLicense',
]

