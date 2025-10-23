"""
Secure secrets and token management for TabAgent

⚠️ SECURITY:
- Tokens are loaded from environment variables or .env file
- NEVER hardcode tokens in code
- .env file is gitignored
- Tokens are NEVER logged or exposed in error messages
"""

import os
from pathlib import Path
from typing import Optional
import logging

logger = logging.getLogger(__name__)

class SecretsManager:
    """Manages API tokens and secrets securely"""
    
    _instance = None
    _initialized = False
    
    def __new__(cls):
        if cls._instance is None:
            cls._instance = super().__new__(cls)
        return cls._instance
    
    def __init__(self):
        if not self._initialized:
            self._load_secrets()
            SecretsManager._initialized = True
    
    def _load_secrets(self):
        """Load secrets from environment variables and .env file"""
        # Try to load from .env file if it exists
        env_file = Path(__file__).parent.parent / '.env'
        if env_file.exists():
            try:
                with open(env_file, 'r') as f:
                    for line in f:
                        line = line.strip()
                        if line and not line.startswith('#') and '=' in line:
                            key, value = line.split('=', 1)
                            # Only set if not already in environment
                            if key not in os.environ:
                                os.environ[key] = value.strip()
                logger.info("Loaded secrets from .env file")
            except Exception as e:
                logger.warning(f"Could not load .env file: {e}")
        
        # Cache tokens
        self._hf_token = os.getenv('HUGGINGFACE_TOKEN')
        self._openai_key = os.getenv('OPENAI_API_KEY')
        self._anthropic_key = os.getenv('ANTHROPIC_API_KEY')
        
        # Cache paths
        self._model_cache_dir = os.getenv('MODEL_CACHE_DIR')
        self._database_path = os.getenv('DATABASE_PATH')
        
        # Log availability (NOT the actual tokens!)
        logger.info(f"HuggingFace token available: {bool(self._hf_token)}")
        logger.info(f"OpenAI key available: {bool(self._openai_key)}")
        logger.info(f"Anthropic key available: {bool(self._anthropic_key)}")
    
    @property
    def huggingface_token(self) -> Optional[str]:
        """Get HuggingFace API token"""
        return self._hf_token
    
    @property
    def openai_key(self) -> Optional[str]:
        """Get OpenAI API key"""
        return self._openai_key
    
    @property
    def anthropic_key(self) -> Optional[str]:
        """Get Anthropic API key"""
        return self._anthropic_key
    
    @property
    def model_cache_dir(self) -> Optional[str]:
        """Get custom model cache directory"""
        return self._model_cache_dir
    
    @property
    def database_path(self) -> Optional[str]:
        """Get custom database path"""
        return self._database_path
    
    def get_download_headers(self) -> dict:
        """
        Get HTTP headers for downloading models from HuggingFace
        
        Returns:
            dict: Headers including authorization if token is available
        """
        headers = {
            'User-Agent': 'TabAgent/1.0'
        }
        
        if self._hf_token:
            headers['Authorization'] = f'Bearer {self._hf_token}'
        
        return headers
    
    def has_huggingface_token(self) -> bool:
        """Check if HuggingFace token is configured"""
        return bool(self._hf_token)
    
    def can_download_gated_model(self, model_id: str) -> bool:
        """
        Check if we can download a gated model
        
        Args:
            model_id: The model identifier (e.g., "test-gemma-3n-e4b-litert")
        
        Returns:
            bool: True if we can download (either public or we have token)
        """
        # For now, simple check: if model requires token, we need one
        # In future, we can check specific model requirements
        return self.has_huggingface_token()


# Global singleton instance
_secrets = SecretsManager()


def get_secrets() -> SecretsManager:
    """Get the global secrets manager instance"""
    return _secrets


def get_huggingface_token() -> Optional[str]:
    """Convenience function to get HuggingFace token"""
    return _secrets.huggingface_token


def get_download_headers() -> dict:
    """Convenience function to get download headers"""
    return _secrets.get_download_headers()

