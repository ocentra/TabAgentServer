"""
ML Bridge Python Module.

This module provides ML inference functions for the TabAgent embedded database.
"""

from .ml_funcs import generate_embedding, extract_entities, summarize, health_check

__all__ = ['generate_embedding', 'extract_entities', 'summarize', 'health_check']
__version__ = '0.1.0'

