"""
Pipelines Module - Specialized ML task handlers

Each pipeline file handles ONE specific architecture/task.
Mirrors the extension's @Pipelines/ structure.

Factory pattern creates the correct pipeline based on:
1. Architecture (from Rust detection)
2. Model ID (pattern matching)
3. Task type (fallback)
"""

# Import all pipelines for external use
from .base import BasePipeline
from .florence2 import Florence2Pipeline
from .whisper import WhisperPipeline
from .text_generation import TextGenerationPipeline
from .clip import ClipPipeline
from .clap import ClapPipeline
from .janus import JanusPipeline
from .multimodal import MultimodalPipeline
from .embedding import EmbeddingPipeline
from .image_classification import ImageClassificationPipeline
from .code_completion import CodeCompletionPipeline
from .cross_encoder import CrossEncoderPipeline
from .text_to_speech import TextToSpeechPipeline
from .translation import TranslationPipeline
from .tokenizer import TokenizerPipeline
from .zero_shot_classification import ZeroShotClassificationPipeline

# Import smart factory and types
from .types import PipelineTask, PipelineTaskType
from .factory import PipelineFactory, create_pipeline


# Export public API
__all__ = [
    # Base
    "BasePipeline",
    # Pipelines
    "Florence2Pipeline",
    "WhisperPipeline",
    "TextGenerationPipeline",
    "ClipPipeline",
    "ClapPipeline",
    "JanusPipeline",
    "MultimodalPipeline",
    "EmbeddingPipeline",
    "ImageClassificationPipeline",
    "CodeCompletionPipeline",
    "CrossEncoderPipeline",
    "TextToSpeechPipeline",
    "TranslationPipeline",
    "TokenizerPipeline",
    "ZeroShotClassificationPipeline",
    # Factory and types
    "PipelineFactory",
    "PipelineTask",
    "PipelineTaskType",
    "create_pipeline",  # Legacy wrapper
]

