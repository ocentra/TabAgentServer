"""
Pytest configuration and shared fixtures

Provides test fixtures for API testing, backend mocking, etc.
"""

import pytest
from fastapi.testclient import TestClient
from typing import Generator

from api.main import app
from api.backend_manager import get_backend_manager, BackendManager
from api.backend_adapter import BitNetBackendAdapter
from core.message_types import (
    ChatMessage,
    InferenceSettings,
    BackendType,
    MessageRole,
)


@pytest.fixture
def client() -> TestClient:
    """
    FastAPI test client
    
    Returns:
        TestClient for API testing
    """
    return TestClient(app)


@pytest.fixture
def mock_backend_manager() -> Generator[BackendManager, None, None]:
    """
    Mock backend manager for testing
    
    Yields:
        BackendManager with mock backend
    """
    from unittest.mock import Mock
    from api.types import PerformanceStats
    
    # Create mock backend
    mock_backend = Mock()
    mock_backend.is_loaded.return_value = True
    mock_backend.get_backend_type.return_value = BackendType.BITNET_CPU
    mock_backend.get_model_path.return_value = "/mock/model.gguf"
    
    # Mock generate methods
    async def mock_generate(messages, settings):
        return "Mock generated response"
    
    async def mock_generate_stream(messages, settings):
        for token in ["Mock ", "streamed ", "response"]:
            yield token
    
    mock_backend.generate = mock_generate
    mock_backend.generate_stream = mock_generate_stream
    
    # Mock stats
    mock_backend.get_stats.return_value = PerformanceStats(
        time_to_first_token=0.1,
        tokens_per_second=50.0,
        input_tokens=10,
        output_tokens=20,
    )
    
    # Set mock backend in manager
    manager = get_backend_manager()
    original_backend = manager._current_backend
    manager.set_backend(mock_backend, "mock-model")
    
    yield manager
    
    # Restore original backend
    manager._current_backend = original_backend


@pytest.fixture
def sample_chat_messages() -> list[ChatMessage]:
    """
    Sample chat messages for testing
    
    Returns:
        List of ChatMessage objects
    """
    return [
        ChatMessage(role=MessageRole.SYSTEM, content="You are a helpful assistant."),
        ChatMessage(role=MessageRole.USER, content="Hello!"),
    ]


@pytest.fixture
def sample_inference_settings() -> InferenceSettings:
    """
    Sample inference settings for testing
    
    Returns:
        InferenceSettings object
    """
    return InferenceSettings(
        temperature=0.7,
        top_k=40,
        top_p=0.9,
        max_new_tokens=100,
    )

