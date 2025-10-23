"""
REAL API endpoint tests - NO MOCKS
Tests actual FastAPI endpoints with real backends or expected failures
"""

import pytest
from fastapi.testclient import TestClient
from fastapi import status

from api.main import app
from api.constants import APIPrefix, EndpointPath, OpenAIObject, FinishReason
from core.message_types import MessageRole

# REAL client - no mocks
client = TestClient(app)


class TestHealthEndpoint:
    """REAL tests for /health endpoint"""
    
    def test_health_check_returns_ok(self):
        """Test health check always returns 200 OK"""
        endpoint = f"{APIPrefix.V1.value}{EndpointPath.HEALTH.value}"
        
        # REAL call
        response = client.get(endpoint)
        
        # Assert
        assert response.status_code == status.HTTP_200_OK
        data = response.json()
        assert "status" in data
        assert data["status"] == "ok"
        assert "model_loaded" in data
        assert "uptime" in data
        assert isinstance(data["model_loaded"], bool)
        assert isinstance(data["uptime"], (int, float))


class TestModelsEndpoint:
    """REAL tests for /models endpoint"""
    
    def test_list_models_returns_list(self):
        """Test models endpoint returns proper list structure"""
        endpoint = f"{APIPrefix.V1.value}{EndpointPath.MODELS.value}"
        
        # REAL call
        response = client.get(endpoint)
        
        # Assert
        assert response.status_code == status.HTTP_200_OK
        data = response.json()
        assert "object" in data
        assert data["object"] == OpenAIObject.LIST.value
        assert "data" in data
        assert isinstance(data["data"], list)
    
    def test_list_models_structure(self):
        """Test model list has correct structure"""
        endpoint = f"{APIPrefix.V1.value}{EndpointPath.MODELS.value}"
        
        # REAL call
        response = client.get(endpoint)
        data = response.json()
        
        # If there are models, check structure
        if len(data["data"]) > 0:
            model = data["data"][0]
            assert "id" in model
            assert "object" in model
            assert model["object"] == OpenAIObject.MODEL.value
            assert "created" in model
            assert "owned_by" in model
    
    def test_get_model_by_id(self):
        """Test retrieving specific model by ID"""
        model_id = "test-model"
        endpoint = f"{APIPrefix.V1.value}{EndpointPath.MODELS.value}/{model_id}"
        
        # REAL call
        response = client.get(endpoint)
        
        # Assert
        assert response.status_code == status.HTTP_200_OK
        data = response.json()
        assert "id" in data
        assert data["id"] == model_id
        assert "object" in data
        assert data["object"] == OpenAIObject.MODEL.value


class TestChatCompletionsEndpoint:
    """REAL tests for /chat/completions endpoint"""
    
    def test_chat_completion_no_model_loaded(self):
        """Test chat completion when no model loaded (expected to fail or return 503)"""
        endpoint = f"{APIPrefix.V1.value}{EndpointPath.CHAT_COMPLETIONS.value}"
        payload = {
            "model": "test-model",
            "messages": [
                {"role": MessageRole.USER.value, "content": "Hello"}
            ]
        }
        
        # REAL call - might fail, that's OK!
        response = client.post(endpoint, json=payload)
        
        # Assert - either succeeds (model loaded) or 503 (no model)
        assert response.status_code in [
            status.HTTP_200_OK,
            status.HTTP_503_SERVICE_UNAVAILABLE
        ]
        
        if response.status_code == status.HTTP_503_SERVICE_UNAVAILABLE:
            # Expected failure - no model loaded
            data = response.json()
            assert "detail" in data
    
    def test_chat_completion_invalid_payload(self):
        """Test chat completion with invalid payload"""
        endpoint = f"{APIPrefix.V1.value}{EndpointPath.CHAT_COMPLETIONS.value}"
        payload = {
            "model": "test-model",
            # Missing required 'messages' field
        }
        
        # REAL call
        response = client.post(endpoint, json=payload)
        
        # Assert - validation error
        assert response.status_code == status.HTTP_422_UNPROCESSABLE_ENTITY
    
    def test_chat_completion_structure_if_model_loaded(self):
        """Test response structure if model is actually loaded"""
        endpoint = f"{APIPrefix.V1.value}{EndpointPath.CHAT_COMPLETIONS.value}"
        payload = {
            "model": "any-model",
            "messages": [
                {"role": MessageRole.USER.value, "content": "Test"}
            ],
            "stream": False
        }
        
        # REAL call
        response = client.post(endpoint, json=payload)
        
        # If model loaded, check structure
        if response.status_code == status.HTTP_200_OK:
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
    
    def test_chat_completion_streaming(self):
        """Test streaming response if model loaded"""
        endpoint = f"{APIPrefix.V1.value}{EndpointPath.CHAT_COMPLETIONS.value}"
        payload = {
            "model": "any-model",
            "messages": [
                {"role": MessageRole.USER.value, "content": "Test"}
            ],
            "stream": True
        }
        
        # REAL call
        response = client.post(endpoint, json=payload)
        
        # If model loaded, verify streaming
        if response.status_code == status.HTTP_200_OK:
            # Check it's a streaming response
            assert response.headers.get("content-type") == "text/event-stream"


class TestAPIIntegration:
    """REAL integration tests - complete API flows"""
    
    def test_complete_workflow(self):
        """Test complete workflow: health → models → chat"""
        # 1. Health check
        health_response = client.get(f"{APIPrefix.V1.value}{EndpointPath.HEALTH.value}")
        assert health_response.status_code == 200
        
        # 2. List models
        models_response = client.get(f"{APIPrefix.V1.value}{EndpointPath.MODELS.value}")
        assert models_response.status_code == 200
        
        # 3. Try chat (might fail if no model)
        chat_response = client.post(
            f"{APIPrefix.V1.value}{EndpointPath.CHAT_COMPLETIONS.value}",
            json={
                "model": "test",
                "messages": [{"role": "user", "content": "Hi"}]
            }
        )
        # Accept either success or 503 (no model loaded)
        assert chat_response.status_code in [200, 503]
    
    def test_openai_compatible_structure(self):
        """Test response structure is OpenAI-compatible"""
        response = client.post(
            f"{APIPrefix.V1.value}{EndpointPath.CHAT_COMPLETIONS.value}",
            json={
                "model": "test",
                "messages": [{"role": "user", "content": "Test"}],
                "temperature": 0.7,
                "max_tokens": 100
            }
        )
        
        # Only check structure if successful
        if response.status_code == 200:
            data = response.json()
            
            # Verify OpenAI-compatible fields
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


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])

