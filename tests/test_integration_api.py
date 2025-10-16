"""
Integration tests for API

Tests full API flow with real backends.
"""

import pytest
from fastapi.testclient import TestClient

from api.constants import APIPrefix, EndpointPath
from core.message_types import MessageRole


class TestAPIIntegration:
    """Integration tests for full API flow"""
    
    def test_health_to_models_to_chat_flow(
        self,
        client: TestClient,
        mock_backend_manager
    ):
        """Test complete workflow: health check -> list models -> chat"""
        # 1. Health check
        health_response = client.get(
            f"{APIPrefix.V1.value}{EndpointPath.HEALTH.value}"
        )
        assert health_response.status_code == 200
        health_data = health_response.json()
        assert health_data["status"] == "ok"
        
        # 2. List models
        models_response = client.get(
            f"{APIPrefix.V1.value}{EndpointPath.MODELS.value}"
        )
        assert models_response.status_code == 200
        models_data = models_response.json()
        assert "data" in models_data
        
        # 3. Chat completion
        chat_response = client.post(
            f"{APIPrefix.V1.value}{EndpointPath.CHAT_COMPLETIONS.value}",
            json={
                "model": "mock-model",
                "messages": [
                    {"role": MessageRole.USER.value, "content": "Hello"}
                ]
            }
        )
        assert chat_response.status_code == 200
        chat_data = chat_response.json()
        assert "choices" in chat_data
        assert len(chat_data["choices"]) > 0
    
    def test_openai_compatibility(
        self,
        client: TestClient,
        mock_backend_manager
    ):
        """Test OpenAI client compatibility"""
        # This test verifies the response format matches OpenAI API
        
        response = client.post(
            f"{APIPrefix.V1.value}{EndpointPath.CHAT_COMPLETIONS.value}",
            json={
                "model": "mock-model",
                "messages": [
                    {"role": "user", "content": "Test"}
                ],
                "temperature": 0.7,
                "max_tokens": 100
            }
        )
        
        assert response.status_code == 200
        data = response.json()
        
        # Verify OpenAI-compatible structure
        required_fields = ["id", "object", "created", "model", "choices"]
        for field in required_fields:
            assert field in data
        
        # Verify choice structure
        choice = data["choices"][0]
        assert "index" in choice
        assert "message" in choice
        assert "finish_reason" in choice
        
        # Verify message structure
        message = choice["message"]
        assert "role" in message
        assert "content" in message

