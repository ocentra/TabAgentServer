"""
Unit tests for backend manager

Tests BackendManager functionality and backend coordination.
"""

import pytest
from unittest.mock import Mock, AsyncMock

from api.backend_manager import BackendManager
from core.message_types import (
    ChatMessage,
    InferenceSettings,
    BackendType,
    MessageRole,
)
from api.types import ChatCompletionRequest


class TestBackendManager:
    """Tests for BackendManager"""
    
    def test_initialization(self):
        """Test BackendManager initializes correctly"""
        # Act
        manager = BackendManager()
        
        # Assert
        assert manager is not None
        assert manager._current_backend is None
        assert manager._model_id == "unknown"
    
    def test_set_backend(self):
        """Test setting active backend"""
        # Arrange
        manager = BackendManager()
        mock_backend = Mock()
        model_id = "test-model"
        
        # Act
        manager.set_backend(mock_backend, model_id)
        
        # Assert
        assert manager._current_backend == mock_backend
        assert manager._model_id == model_id
    
    def test_is_model_loaded_false_when_no_backend(self):
        """Test is_model_loaded returns False when no backend"""
        # Arrange
        manager = BackendManager()
        
        # Act
        result = manager.is_model_loaded()
        
        # Assert
        assert result is False
    
    def test_is_model_loaded_delegates_to_backend(self):
        """Test is_model_loaded delegates to backend"""
        # Arrange
        manager = BackendManager()
        mock_backend = Mock()
        mock_backend.is_loaded.return_value = True
        manager.set_backend(mock_backend, "test-model")
        
        # Act
        result = manager.is_model_loaded()
        
        # Assert
        assert result is True
        mock_backend.is_loaded.assert_called_once()
    
    def test_get_backend_type_returns_none_when_no_backend(self):
        """Test get_backend_type returns None when no backend"""
        # Arrange
        manager = BackendManager()
        
        # Act
        result = manager.get_backend_type()
        
        # Assert
        assert result is None
    
    def test_get_backend_type_returns_backend_type(self):
        """Test get_backend_type returns backend's type"""
        # Arrange
        manager = BackendManager()
        mock_backend = Mock()
        mock_backend.get_backend_type.return_value = BackendType.BITNET_CPU
        manager.set_backend(mock_backend, "test-model")
        
        # Act
        result = manager.get_backend_type()
        
        # Assert
        assert result == BackendType.BITNET_CPU
    
    def test_get_model_id(self):
        """Test get_model_id returns current model ID"""
        # Arrange
        manager = BackendManager()
        model_id = "test-model-id"
        mock_backend = Mock()
        manager.set_backend(mock_backend, model_id)
        
        # Act
        result = manager.get_model_id()
        
        # Assert
        assert result == model_id
    
    @pytest.mark.asyncio
    async def test_chat_completion_raises_when_no_model(self):
        """Test chat_completion raises when no model loaded"""
        # Arrange
        manager = BackendManager()
        request = ChatCompletionRequest(
            model="test",
            messages=[
                ChatMessage(role=MessageRole.USER, content="Test")
            ]
        )
        
        # Act & Assert
        with pytest.raises(RuntimeError, match="No model loaded"):
            await manager.chat_completion(request)
    
    @pytest.mark.asyncio
    async def test_chat_completion_success(self):
        """Test successful chat completion"""
        # Arrange
        manager = BackendManager()
        mock_backend = Mock()
        mock_backend.is_loaded.return_value = True
        
        # Mock async generate
        async def mock_generate(messages, settings):
            return "Generated response"
        
        mock_backend.generate = mock_generate
        mock_backend.get_stats.return_value = None
        
        manager.set_backend(mock_backend, "test-model")
        
        request = ChatCompletionRequest(
            model="test-model",
            messages=[
                ChatMessage(role=MessageRole.USER, content="Test")
            ]
        )
        
        # Act
        result = await manager.chat_completion(request)
        
        # Assert
        assert result is not None
        assert len(result.choices) > 0
        assert result.choices[0].message.content == "Generated response"
        assert result.choices[0].message.role == MessageRole.ASSISTANT

