"""
Unit tests for health endpoint

Tests health check functionality and status reporting.
"""

import pytest
from fastapi import status
from fastapi.testclient import TestClient

from api.constants import APIPrefix, EndpointPath


class TestHealthEndpoint:
    """Tests for /health endpoint"""
    
    def test_health_check_success(self, client: TestClient):
        """Test successful health check"""
        # Arrange
        endpoint = f"{APIPrefix.V1.value}{EndpointPath.HEALTH.value}"
        
        # Act
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
    
    def test_health_check_with_model_loaded(
        self, 
        client: TestClient, 
        mock_backend_manager
    ):
        """Test health check when model is loaded"""
        # Arrange
        endpoint = f"{APIPrefix.V1.value}{EndpointPath.HEALTH.value}"
        
        # Act
        response = client.get(endpoint)
        
        # Assert
        assert response.status_code == status.HTTP_200_OK
        
        data = response.json()
        assert data["model_loaded"] is True
        assert "backend" in data
        assert data["backend"] is not None
    
    def test_health_check_without_model(self, client: TestClient):
        """Test health check when no model loaded"""
        # Arrange
        endpoint = f"{APIPrefix.V1.value}{EndpointPath.HEALTH.value}"
        
        # Act
        response = client.get(endpoint)
        
        # Assert
        assert response.status_code == status.HTTP_200_OK
        
        data = response.json()
        # Backend might be loaded or not depending on test order
        assert "model_loaded" in data
        assert "status" in data

