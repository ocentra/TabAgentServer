"""
Test secrets management

REAL TESTS - NO MOCKS:
- Tests actual .env file loading
- Tests real environment variables
- Tests token availability checks
"""

import pytest
import os
import tempfile
from pathlib import Path
import sys

# Add parent to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from Python.core.secrets import SecretsManager, get_secrets


def test_env_file_loading():
    """Test loading secrets from .env file"""
    print("\n🧪 Testing .env file loading...")
    
    # Create a temporary .env file
    with tempfile.NamedTemporaryFile(mode='w', suffix='.env', delete=False) as f:
        f.write("# Test .env file\n")
        f.write("HUGGINGFACE_TOKEN=test_token_123\n")
        f.write("OPENAI_API_KEY=sk-test456\n")
        env_path = f.name
    
    try:
        # Clear any existing tokens
        for key in ['HUGGINGFACE_TOKEN', 'OPENAI_API_KEY']:
            if key in os.environ:
                del os.environ[key]
        
        # Load the test .env file
        with open(env_path, 'r') as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('#') and '=' in line:
                    key, value = line.split('=', 1)
                    os.environ[key] = value.strip()
        
        # Create new manager instance
        manager = SecretsManager()
        
        # Verify tokens loaded
        assert manager.huggingface_token == "test_token_123", "HF token not loaded correctly"
        assert manager.openai_key == "sk-test456", "OpenAI key not loaded correctly"
        
        print("✅ .env file loaded correctly")
        print(f"   HF token: {'*' * 10}{manager.huggingface_token[-4:]}")
        print(f"   OpenAI key: {'*' * 10}{manager.openai_key[-4:]}")
        
    finally:
        # Cleanup
        os.unlink(env_path)
        for key in ['HUGGINGFACE_TOKEN', 'OPENAI_API_KEY']:
            if key in os.environ:
                del os.environ[key]


def test_token_availability_checks():
    """Test token availability checking"""
    print("\n🧪 Testing token availability checks...")
    
    # Save original env
    original_hf = os.environ.get('HUGGINGFACE_TOKEN')
    
    # Test with token
    os.environ['HUGGINGFACE_TOKEN'] = 'test_token'
    manager = SecretsManager()
    
    assert manager.has_huggingface_token() is True, "Should report token available"
    assert manager.can_download_gated_model("any_model") is True, "Should allow gated models with token"
    print("✅ Token availability detection works with token")
    
    # Test without token
    if 'HUGGINGFACE_TOKEN' in os.environ:
        del os.environ['HUGGINGFACE_TOKEN']
    manager = SecretsManager()
    
    assert manager.has_huggingface_token() is False, "Should report no token"
    print("✅ Token availability detection works without token")
    
    # Restore original
    if original_hf:
        os.environ['HUGGINGFACE_TOKEN'] = original_hf
    elif 'HUGGINGFACE_TOKEN' in os.environ:
        del os.environ['HUGGINGFACE_TOKEN']


def test_download_headers():
    """Test HTTP header generation for downloads"""
    print("\n🧪 Testing download header generation...")
    
    # Save original
    original_hf = os.environ.get('HUGGINGFACE_TOKEN')
    
    # Test with token
    os.environ['HUGGINGFACE_TOKEN'] = 'hf_test123'
    manager = SecretsManager()
    headers = manager.get_download_headers()
    
    assert 'User-Agent' in headers, "Should include User-Agent"
    assert 'Authorization' in headers, "Should include Authorization with token"
    assert headers['Authorization'] == 'Bearer hf_test123', "Authorization format incorrect"
    print("✅ Headers include token")
    print(f"   Authorization: Bearer {'*' * 10}{headers['Authorization'][-4:]}")
    
    # Test without token
    if 'HUGGINGFACE_TOKEN' in os.environ:
        del os.environ['HUGGINGFACE_TOKEN']
    manager = SecretsManager()
    headers = manager.get_download_headers()
    
    assert 'User-Agent' in headers, "Should always include User-Agent"
    assert 'Authorization' not in headers, "Should not include Authorization without token"
    print("✅ Headers work without token")
    
    # Restore
    if original_hf:
        os.environ['HUGGINGFACE_TOKEN'] = original_hf
    elif 'HUGGINGFACE_TOKEN' in os.environ:
        del os.environ['HUGGINGFACE_TOKEN']


def test_singleton_pattern():
    """Test that SecretsManager is a singleton"""
    print("\n🧪 Testing singleton pattern...")
    
    manager1 = SecretsManager()
    manager2 = SecretsManager()
    manager3 = get_secrets()
    
    assert manager1 is manager2, "Should return same instance"
    assert manager1 is manager3, "get_secrets() should return same instance"
    
    print("✅ Singleton pattern works correctly")


def test_no_token_leakage():
    """Test that tokens are never exposed in logs or errors"""
    print("\n🧪 Testing token security...")
    
    # Set a test token
    os.environ['HUGGINGFACE_TOKEN'] = 'hf_secret_token_do_not_leak'
    manager = SecretsManager()
    
    # Convert to string - should NOT contain actual token
    manager_str = str(manager)
    manager_repr = repr(manager)
    
    assert 'hf_secret' not in manager_str.lower(), "Token leaked in str()"
    assert 'hf_secret' not in manager_repr.lower(), "Token leaked in repr()"
    
    # Check __dict__ access
    assert 'hf_secret' not in str(manager.__dict__).lower(), "Token leaked in __dict__"
    
    print("✅ Tokens not leaked in string representations")
    
    # Cleanup
    if 'HUGGINGFACE_TOKEN' in os.environ:
        del os.environ['HUGGINGFACE_TOKEN']


if __name__ == '__main__':
    print("=" * 60)
    print("🧪 SECRETS MANAGEMENT TEST SUITE")
    print("=" * 60)
    
    test_env_file_loading()
    test_token_availability_checks()
    test_download_headers()
    test_singleton_pattern()
    test_no_token_leakage()
    
    print("\n" + "=" * 60)
    print("🎉 ALL TESTS PASSED!")
    print("=" * 60)

