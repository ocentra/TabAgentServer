"""
Pipeline Types - Type-safe constants and enums

NO string literals! All task types defined as constants.
Mirrors extension's PipelineTypes.ts
"""

from enum import Enum
from typing import Literal


class PipelineTask(str, Enum):
    """
    Pipeline task types (type-safe enum).
    
    Mirrors extension's PipelineTypeEnum.
    Use these constants instead of string literals!
    """
    TEXT_GENERATION = "text-generation"
    TEXT_CLASSIFICATION = "text-classification"
    TOKEN_CLASSIFICATION = "token-classification"
    QUESTION_ANSWERING = "question-answering"
    FILL_MASK = "fill-mask"
    SUMMARIZATION = "summarization"
    TRANSLATION = "translation"
    TEXT2TEXT_GENERATION = "text2text-generation"
    FEATURE_EXTRACTION = "feature-extraction"
    IMAGE_CLASSIFICATION = "image-classification"
    ZERO_SHOT_CLASSIFICATION = "zero-shot-classification"
    AUTOMATIC_SPEECH_RECOGNITION = "automatic-speech-recognition"
    IMAGE_TO_TEXT = "image-to-text"
    OBJECT_DETECTION = "object-detection"
    ZERO_SHOT_OBJECT_DETECTION = "zero-shot-object-detection"
    DOCUMENT_QUESTION_ANSWERING = "document-question-answering"
    IMAGE_SEGMENTATION = "image-segmentation"
    DEPTH_ESTIMATION = "depth-estimation"
    VISUAL_LANGUAGE = "visual-language"
    TEXT_TO_SPEECH = "text-to-speech"
    AUDIO_CLASSIFICATION = "audio-classification"
    TOKENIZER = "tokenizer"


# Type alias for pipeline task strings
PipelineTaskType = Literal[
    "text-generation",
    "text-classification",
    "token-classification",
    "question-answering",
    "fill-mask",
    "summarization",
    "translation",
    "text2text-generation",
    "feature-extraction",
    "image-classification",
    "zero-shot-classification",
    "automatic-speech-recognition",
    "image-to-text",
    "object-detection",
    "zero-shot-object-detection",
    "document-question-answering",
    "image-segmentation",
    "depth-estimation",
    "visual-language",
    "text-to-speech",
    "audio-classification",
    "tokenizer",
]

