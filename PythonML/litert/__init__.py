"""
LiteRT module for quantized edge model inference.

Handles LiteRT models (e.g., Gemma LiteRT quantized models).
Reference: https://ai.google.dev/edge/litert
"""

from .manager import LiteRTManager

__all__ = ['LiteRTManager']

