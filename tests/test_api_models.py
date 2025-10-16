"""
Unit tests for models endpoint

Tests model listing and retrieval functionality.
"""

import pytest
from fastapi import status
from fastapi.testclient import TestClient

from api.constants import APIPrefix, EndpointPath, OpenAIObject


class TestModelsEndpoint:
    """Tests for /models endpoint"""
    
    def test_list_models_success(self, client: TestClient):
        """Test successful model listing"""
        # Arrange
        endpoint = f"{APIPrefix.V1.value}{EndpointPath.MODELS.value}"
        
        # Act
        response = client.get(endpoint)
        
        # Assert
        assert response.status_code == status.HTTP_200_OK
        
        data = response.json()
        assert "object" in data
        assert data["object"] == OpenAIObject.LIST.value
        assert "data" in data
        assert isinstance(data["data"], list)
    
    def test_list_models_structure(self, client: TestClient):
        """Test model list response structure"""
        # Arrange
        endpoint = f"{APIPrefix.V1.value}{EndpointPath.MODELS.value}"
        
        # Act
        response = client.get(endpoint)
        
        # Assert
        data = response.json()
        
        if len(data["data"]) > 0:
            model = data["data"][0]
            assert "id" in model
            assert "object" in model
            assert model["object"] == OpenAIObject.MODEL.value
            assert "created" in model
            assert "owned_by" in model
    
    def test_get_model_by_id(self, client: TestClient):
        """Test retrieving specific model"""
        # Arrange
        model_id = "test-model"
        endpoint = f"{APIPrefix.V1.value}{EndpointPath.MODELS.value}/{model_id}"
        
        # Act
        response = client.get(endpoint)
        
        # Assert
        assert response.status_code == status.HTTP_200_OK
        
        data = response.json()
        assert "id" in data
        assert data["id"] == model_id
        assert "object" in data
        assert data["object"] == OpenAIObject.MODEL.value

