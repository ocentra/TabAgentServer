"""
PipelineFactory - Smart factory pattern

Mirrors extension's PipelineFactory.ts:
1. Model-specific routing FIRST (checks modelId patterns)
2. Task-based fallback (clean routing)
3. Proper logging with prefixes

NO DUMB IF/ELIF CHAINS!
"""

import logging
from typing import Optional

from .types import PipelineTask
from .base import BasePipeline

# Import all pipelines
from .text_generation import TextGenerationPipeline
from .embedding import EmbeddingPipeline
from .translation import TranslationPipeline
from .zero_shot_classification import ZeroShotClassificationPipeline
from .whisper import WhisperPipeline
from .florence2 import Florence2Pipeline
from .janus import JanusPipeline
from .multimodal import MultimodalPipeline
from .image_classification import ImageClassificationPipeline
from .cross_encoder import CrossEncoderPipeline
from .clap import ClapPipeline
from .clip import ClipPipeline
from .text_to_speech import TextToSpeechPipeline
from .code_completion import CodeCompletionPipeline
from .tokenizer import TokenizerPipeline

logger = logging.getLogger(__name__)
PREFIX = "[PipelineFactory]"


class PipelineFactory:
    """
    Smart factory for creating pipelines.
    
    Routing priority:
    1. Model-specific detection (modelId patterns)
    2. Task-based routing (enum-based)
    3. Default fallback (TextGeneration)
    """
    
    @staticmethod
    def create_pipeline(
        task: Optional[str] = None,
        model_id: Optional[str] = None,
        architecture: Optional[str] = None
    ) -> BasePipeline:
        """
        Create appropriate pipeline based on task type and optional modelId.
        
        Mirrors extension's createPipeline() logic:
        - Model-specific routing for specialized models (FIRST PRIORITY)
        - Task-based routing for generic models (FALLBACK)
        - Defaults to TextGenerationPipeline if unknown
        
        Args:
            task: Pipeline task type (e.g., 'text-generation', 'image-to-text')
            model_id: Optional model ID for specialized routing (e.g., 'Florence-2')
            architecture: Optional architecture hint from Rust detection
            
        Returns:
            Concrete pipeline instance
        """
        # Default to text generation if no task specified
        pipeline_task = task or PipelineTask.TEXT_GENERATION.value
        
        logger.info(f"{PREFIX} Creating pipeline for task: {pipeline_task}, "
                   f"modelId: {model_id or 'none'}, architecture: {architecture or 'none'}")
        
        # ====================================================================
        # PRIORITY 1: Architecture-specific routing (from Rust detection)
        # ====================================================================
        if architecture:
            arch_lower = architecture.lower()
            
            if arch_lower in ("florence2", "florence"):
                logger.info(f"{PREFIX} Detected Florence2 architecture, using Florence2Pipeline")
                return Florence2Pipeline()
            
            if arch_lower == "janus":
                logger.info(f"{PREFIX} Detected Janus architecture, using JanusPipeline")
                return JanusPipeline()
            
            if arch_lower in ("whisper", "moonshine"):
                logger.info(f"{PREFIX} Detected Whisper architecture, using WhisperPipeline")
                return WhisperPipeline()
            
            if arch_lower == "clip":
                logger.info(f"{PREFIX} Detected CLIP architecture, using ClipPipeline")
                return ClipPipeline()
            
            if arch_lower == "clap":
                logger.info(f"{PREFIX} Detected CLAP architecture, using ClapPipeline")
                return ClapPipeline()
        
        # ====================================================================
        # PRIORITY 2: Model-specific routing (modelId patterns)
        # ====================================================================
        if model_id:
            lower_model_id = model_id.lower()
            
            # Florence2 detection
            if "florence" in lower_model_id or "florence-2" in lower_model_id:
                logger.info(f"{PREFIX} Detected Florence2 model from ID, using Florence2Pipeline")
                return Florence2Pipeline()
            
            # Janus detection
            if "janus" in lower_model_id:
                logger.info(f"{PREFIX} Detected Janus model from ID, using JanusPipeline")
                return JanusPipeline()
            
            # Whisper detection
            if "whisper" in lower_model_id or "moonshine" in lower_model_id:
                logger.info(f"{PREFIX} Detected Whisper-like model, using WhisperPipeline")
                return WhisperPipeline()
            
            # CLIP detection
            if "clip" in lower_model_id and "clap" not in lower_model_id:
                logger.info(f"{PREFIX} Detected CLIP model, using ClipPipeline")
                return ClipPipeline()
            
            # CLAP detection
            if "clap" in lower_model_id:
                logger.info(f"{PREFIX} Detected CLAP model, using ClapPipeline")
                return ClapPipeline()
            
            # Cross-encoder detection (reranking)
            if "rerank" in lower_model_id or "cross-encoder" in lower_model_id:
                logger.info(f"{PREFIX} Detected cross-encoder model, using CrossEncoderPipeline")
                return CrossEncoderPipeline()
            
            # DINOv2 / Attention visualization detection
            if "dino" in lower_model_id or "with-attentions" in lower_model_id:
                logger.info(f"{PREFIX} Detected image classification with attentions, "
                          "using ImageClassificationPipeline")
                return ImageClassificationPipeline()
            
            # SpeechT5 detection (text-to-speech)
            if "speecht5" in lower_model_id or "tts" in lower_model_id:
                logger.info(f"{PREFIX} Detected text-to-speech model, using TextToSpeechPipeline")
                return TextToSpeechPipeline()
            
            # Code completion models detection
            if any(kw in lower_model_id for kw in ["code", "codellama", "starcoder"]):
                logger.info(f"{PREFIX} Detected code completion model, using CodeCompletionPipeline")
                return CodeCompletionPipeline()
        
        # ====================================================================
        # PRIORITY 3: Task-based routing (clean enum-based routing)
        # ====================================================================
        task_map = {
            PipelineTask.TEXT_GENERATION.value: TextGenerationPipeline,
            PipelineTask.FEATURE_EXTRACTION.value: EmbeddingPipeline,
            PipelineTask.TRANSLATION.value: TranslationPipeline,
            PipelineTask.ZERO_SHOT_CLASSIFICATION.value: ZeroShotClassificationPipeline,
            PipelineTask.IMAGE_TO_TEXT.value: MultimodalPipeline,  # Default to generic
            PipelineTask.VISUAL_LANGUAGE.value: MultimodalPipeline,  # Default to generic
            PipelineTask.AUTOMATIC_SPEECH_RECOGNITION.value: WhisperPipeline,
            PipelineTask.IMAGE_CLASSIFICATION.value: ImageClassificationPipeline,
            PipelineTask.TEXT_CLASSIFICATION.value: CrossEncoderPipeline,
            PipelineTask.TEXT_TO_SPEECH.value: TextToSpeechPipeline,
            PipelineTask.TOKEN_CLASSIFICATION.value: TokenizerPipeline,
            PipelineTask.AUDIO_CLASSIFICATION.value: ClapPipeline,
            PipelineTask.TOKENIZER.value: TokenizerPipeline,
        }
        
        pipeline_class = task_map.get(pipeline_task)
        if pipeline_class:
            logger.info(f"{PREFIX} Using task-based routing: {pipeline_class.__name__}")
            return pipeline_class()
        
        # ====================================================================
        # FALLBACK: Default to text generation for unknown tasks
        # ====================================================================
        logger.warning(f"{PREFIX} Unknown task '{pipeline_task}', defaulting to TextGenerationPipeline")
        return TextGenerationPipeline()


def create_pipeline(
    pipeline_type: str,
    architecture: Optional[str] = None,
    model_id: Optional[str] = None
) -> Optional[BasePipeline]:
    """
    Legacy wrapper for backwards compatibility.
    
    Use PipelineFactory.create_pipeline() for new code.
    """
    return PipelineFactory.create_pipeline(
        task=pipeline_type,
        model_id=model_id,
        architecture=architecture
    )

