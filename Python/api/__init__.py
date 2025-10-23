"""
TabAgent Server API

OpenAI-compatible HTTP server for local AI inference.
Uses core.inference_service (shared with native_host.py) - DRY principle.
"""

from .main import app
from .backend_manager import get_backend_manager
from .backend_adapter import get_inference_adapter

__all__ = ["app", "get_backend_manager", "get_inference_adapter"]
