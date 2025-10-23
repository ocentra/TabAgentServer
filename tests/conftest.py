"""
Pytest configuration and shared fixtures

Provides test fixtures for API testing, backend mocking, etc.
"""

import pytest
from fastapi.testclient import TestClient
from typing import Generator

from api.main import app
from api.backend_manager import get_backend_manager, BackendManager
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

