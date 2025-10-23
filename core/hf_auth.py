#!/usr/bin/env python3
"""
HuggingFace Authentication Manager

Handles secure token storage and authentication flow for HuggingFace API access.
Integrates with the extension's HuggingFaceLoginDialog for user authentication.
"""

import os
import json
import logging
from typing import Optional, Dict, Any
from pathlib import Path

# Try to use keyring for secure storage (OS credential store)
try:
    import keyring
    KEYRING_AVAILABLE = True
    SERVICE_NAME = "TabAgent.HuggingFace"
    logging.info("[HF Auth] Using OS keyring for secure token storage")
except ImportError:
    KEYRING_AVAILABLE = False
    logging.warning("[HF Auth] keyring not available, using encrypted file storage")
    logging.warning("[HF Auth] Install with: pip install keyring")

from cryptography.fernet import Fernet
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
import base64


class HuggingFaceAuthManager:
    """
    Manages HuggingFace API token storage and authentication flow.
    
    Uses OS credential store (keyring) if available, otherwise falls back
    to encrypted file storage.
    """
    
    def __init__(self, config_dir: Optional[str] = None):
        """
        Initialize auth manager.
        
        Args:
            config_dir: Directory for config files (default: ~/.tabagent)
        """
        if config_dir is None:
            config_dir = os.path.join(str(Path.home()), '.tabagent')
        
        self.config_dir = config_dir
        os.makedirs(self.config_dir, exist_ok=True)
        
        self.token_file = os.path.join(self.config_dir, '.hf_token.enc')
        self.key_file = os.path.join(self.config_dir, '.hf_key')
        
        # Initialize encryption key
        self._init_encryption()
    
    def _init_encryption(self):
        """Initialize or load encryption key for file-based storage"""
        if KEYRING_AVAILABLE:
            return  # Don't need encryption if using keyring
        
        if os.path.exists(self.key_file):
            with open(self.key_file, 'rb') as f:
                self.key = f.read()
        else:
            # Generate new key from machine-specific data
            machine_id = self._get_machine_id()
            kdf = PBKDF2HMAC(
                algorithm=hashes.SHA256(),
                length=32,
                salt=b'tabagent_hf_salt',
                iterations=100000,
            )
            self.key = base64.urlsafe_b64encode(kdf.derive(machine_id.encode()))
            
            with open(self.key_file, 'wb') as f:
                f.write(self.key)
            
            # Make key file readable only by user
            os.chmod(self.key_file, 0o600)
    
    def _get_machine_id(self) -> str:
        """Get machine-specific identifier"""
        # Try multiple sources for machine ID
        sources = [
            lambda: os.environ.get('COMPUTERNAME'),  # Windows
            lambda: os.environ.get('HOSTNAME'),       # Unix
            lambda: os.environ.get('USER'),           # Fallback
            lambda: 'default_machine',                 # Last resort
        ]
        
        for source in sources:
            machine_id = source()
            if machine_id:
                return machine_id
        
        return 'unknown'
    
    def set_token(self, token: str) -> bool:
        """
        Store HuggingFace API token securely.
        
        Args:
            token: HuggingFace API token (starts with 'hf_')
        
        Returns:
            True if stored successfully
        """
        if not token or not token.startswith('hf_'):
            logging.error("[HF Auth] Invalid token format (must start with 'hf_')")
            return False
        
        try:
            if KEYRING_AVAILABLE:
                # Use OS credential store
                keyring.set_password(SERVICE_NAME, 'api_token', token)
                logging.info("[HF Auth] Token stored in OS keyring")
            else:
                # Encrypt and save to file
                fernet = Fernet(self.key)
                encrypted = fernet.encrypt(token.encode())
                
                with open(self.token_file, 'wb') as f:
                    f.write(encrypted)
                
                os.chmod(self.token_file, 0o600)
                logging.info("[HF Auth] Token encrypted and stored in file")
            
            return True
        
        except Exception as e:
            logging.error(f"[HF Auth] Failed to store token: {e}")
            return False
    
    def get_token(self) -> Optional[str]:
        """
        Retrieve stored HuggingFace API token.
        
        Returns:
            Token string or None if not found
        """
        try:
            if KEYRING_AVAILABLE:
                token = keyring.get_password(SERVICE_NAME, 'api_token')
                if token:
                    logging.debug("[HF Auth] Token retrieved from OS keyring")
                return token
            else:
                if not os.path.exists(self.token_file):
                    return None
                
                with open(self.token_file, 'rb') as f:
                    encrypted = f.read()
                
                fernet = Fernet(self.key)
                token = fernet.decrypt(encrypted).decode()
                logging.debug("[HF Auth] Token decrypted from file")
                return token
        
        except Exception as e:
            logging.error(f"[HF Auth] Failed to retrieve token: {e}")
            return None
    
    def clear_token(self) -> bool:
        """
        Remove stored token.
        
        Returns:
            True if cleared successfully
        """
        try:
            if KEYRING_AVAILABLE:
                keyring.delete_password(SERVICE_NAME, 'api_token')
                logging.info("[HF Auth] Token removed from OS keyring")
            else:
                if os.path.exists(self.token_file):
                    os.remove(self.token_file)
                    logging.info("[HF Auth] Token file deleted")
            
            return True
        
        except Exception as e:
            logging.error(f"[HF Auth] Failed to clear token: {e}")
            return False
    
    def has_token(self) -> bool:
        """Check if token is stored"""
        return self.get_token() is not None


def is_auth_error(error_message: str) -> bool:
    """
    Check if error message indicates authentication is required.
    
    Args:
        error_message: Error message from HuggingFace API
    
    Returns:
        True if authentication error (401/403)
    """
    auth_indicators = [
        '401',
        '403',
        'unauthorized',
        'forbidden',
        'authentication',
        'access denied',
        'token required',
    ]
    
    error_lower = error_message.lower()
    return any(indicator in error_lower for indicator in auth_indicators)


def create_auth_required_response(repo: str, error_message: str) -> Dict[str, Any]:
    """
    Create standardized response for authentication required.
    
    This response format triggers the HuggingFaceLoginDialog in the extension.
    
    Args:
        repo: Repository ID that requires authentication
        error_message: Original error message
    
    Returns:
        Dict with auth_required status
    """
    return {
        "status": "auth_required",
        "provider": "huggingface",
        "repo": repo,
        "message": f"Authentication required to access {repo}",
        "instructions": {
            "step1": f"Visit https://huggingface.co/{repo} and accept the terms",
            "step2": "Go to https://huggingface.co/settings/tokens",
            "step3": "Create a new token with 'Read' permissions",
            "step4": "Provide the token to TabAgent"
        },
        "original_error": error_message
    }


# Global auth manager instance
_auth_manager: Optional[HuggingFaceAuthManager] = None


def get_auth_manager() -> HuggingFaceAuthManager:
    """Get or create global auth manager instance"""
    global _auth_manager
    if _auth_manager is None:
        _auth_manager = HuggingFaceAuthManager()
    return _auth_manager


# Convenience functions for direct use
def set_hf_token(token: str) -> bool:
    """Store HuggingFace token"""
    return get_auth_manager().set_token(token)


def get_hf_token() -> Optional[str]:
    """Retrieve HuggingFace token"""
    return get_auth_manager().get_token()


def clear_hf_token() -> bool:
    """Remove HuggingFace token"""
    return get_auth_manager().clear_token()


def has_hf_token() -> bool:
    """Check if token is stored"""
    return get_auth_manager().has_token()

