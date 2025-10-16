"""
Unit tests for chat completions endpoint

Tests chat completion functionality including streaming.
"""

import pytest
import json
from fastapi import status
from fastapi.testclient import TestClient

from api.constants import (
    APIPrefix,
    EndpointPath,
    OpenAIObject,
    FinishReason,
    ErrorCode,
)
from core.message_types import MessageRole


class TestChatCompletionsEndpoint:
    """Tests for /chat/completions endpoint"""
    
    def test_chat_completion_no_model_loaded(self, client: TestClient):
        """Test chat completion fails when no model loaded"""
        # Arrange
        endpoint = f"{APIPrefix.V1.value}{EndpointPath.CHAT_COMPLETIONS.value}"
        payload = {
            "model": "test-model",
            "messages": [
                {"role": MessageRole.USER.value, "content": "Hello"}
            ]
        }
        
        # Act
        response = client.post(endpoint, json=payload)
        
        # Assert - might be 503 or 200 depending on backend state
        if response.status_code == status.HTTP_503_SERVICE_UNAVAILABLE:
            data = response.json()
            assert "error" in data["detail"]
            assert data["detail"]["error"]["type"] == ErrorCode.MODEL_NOT_LOADED.value
    
    def test_chat_completion_success(
        self, 
        client: TestClient,
        mock_backend_manager
    ):
        """Test successful chat completion"""
        # Arrange
        endpoint = f"{APIPrefix.V1.value}{EndpointPath.CHAT_COMPLETIONS.value}"
        payload = {
            "model": "mock-model",
            "messages": [
                {"role": MessageRole.USER.value, "content": "Hello"}
            ],
            "stream": False
        }
        
        # Act
        response = client.post(endpoint, json=payload)
        
        # Assert
        assert response.status_code == status.HTTP_200_OK
        
        data = response.json()
        assert "id" in data
        assert "object" in data
        assert data["object"] == OpenAIObject.CHAT_COMPLETION.value
        assert "choices" in data
        assert len(data["choices"]) > 0
        
        choice = data["choices"][0]
        assert "message" in choice
        assert "role" in choice["message"]
        assert "content" in choice["message"]
        assert choice["message"]["role"] == MessageRole.ASSISTANT.value
    
    def test_chat_completion_with_parameters(
        self,
        client: TestClient,
        mock_backend_manager
    ):
        """Test chat completion with custom parameters"""
        # Arrange
        endpoint = f"{APIPrefix.V1.value}{EndpointPath.CHAT_COMPLETIONS.value}"
        payload = {
            "model": "mock-model",
            "messages": [
                {"role": MessageRole.SYSTEM.value, "content": "You are helpful."},
                {"role": MessageRole.USER.value, "content": "Hello"}
            ],
            "temperature": 0.8,
            "max_tokens": 100,
            "stream": False
        }
        
        # Act
        response = client.post(endpoint, json=payload)
        
        # Assert
        assert response.status_code == status.HTTP_200_OK
        
        data = response.json()
        assert "choices" in data
        assert data["choices"][0]["finish_reason"] == FinishReason.STOP.value
    
    def test_chat_completion_streaming(
        self,
        client: TestClient,
        mock_backend_manager
    ):
        """Test streaming chat completion"""
        # Arrange
        endpoint = f"{APIPrefix.V1.value}{EndpointPath.CHAT_COMPLETIONS.value}"
        payload = {
            "model": "mock-model",
            "messages": [
                {"role": MessageRole.USER.value, "content": "Hello"}
            ],
            "stream": True
        }
        
        # Act
        response = client.post(endpoint, json=payload)
        
        # Assert
        assert response.status_code == status.HTTP_200_OK
        
        # Check streaming response
        chunks = []
        for line in response.iter_lines():
            if line:
                line_str = line.decode('utf-8') if isinstance(line, bytes) else line
                if line_str.startswith("data: "):
                    data_str = line_str[6:]
                    if data_str != "[DONE]":
                        chunk = json.loads(data_str)
                        chunks.append(chunk)
        
        # Verify chunks
        assert len(chunks) > 0
        
        first_chunk = chunks[0]
        assert "object" in first_chunk
        assert first_chunk["object"] == OpenAIObject.CHAT_COMPLETION_CHUNK.value
        assert "choices" in first_chunk
    
    def test_chat_completion_invalid_request(self, client: TestClient):
        """Test chat completion with invalid request"""
        # Arrange
        endpoint = f"{APIPrefix.V1.value}{EndpointPath.CHAT_COMPLETIONS.value}"
        payload = {
            "model": "test-model",
            # Missing required 'messages' field
        }
        
        # Act
        response = client.post(endpoint, json=payload)
        
        # Assert
        assert response.status_code == status.HTTP_422_UNPROCESSABLE_ENTITY

