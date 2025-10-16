"""
Unit tests for backend adapters

Tests adapter pattern implementation for existing backends.
"""

import pytest
from unittest.mock import Mock, AsyncMock
from typing import Optional

from api.backend_adapter import BitNetBackendAdapter
from core.message_types import (
    ChatMessage,
    InferenceSettings,
    BackendType,
    MessageRole,
)


class TestBitNetBackendAdapter:
    """Tests for BitNetBackendAdapter"""
    
    def test_adapter_initialization(self):
        """Test adapter can be initialized with BitNetManager"""
        # Arrange
        mock_manager = Mock()
        mock_manager.is_model_loaded = False
        mock_manager.backend = None
        mock_manager.current_model_path = None
        
        # Act
        adapter = BitNetBackendAdapter(mock_manager)
        
        # Assert
        assert adapter is not None
        assert adapter._manager == mock_manager
    
    def test_is_loaded_delegates_to_manager(self):
        """Test is_loaded delegates to manager"""
        # Arrange
        mock_manager = Mock()
        mock_manager.is_model_loaded = True
        adapter = BitNetBackendAdapter(mock_manager)
        
        # Act
        result = adapter.is_loaded()
        
        # Assert
        assert result is True
    
    def test_get_backend_type_returns_manager_backend(self):
        """Test get_backend_type returns manager's backend"""
        # Arrange
        mock_manager = Mock()
        mock_manager.backend = BackendType.BITNET_CPU
        adapter = BitNetBackendAdapter(mock_manager)
        
        # Act
        result = adapter.get_backend_type()
        
        # Assert
        assert result == BackendType.BITNET_CPU
    
    def test_get_model_path_returns_manager_path(self):
        """Test get_model_path returns manager's path"""
        # Arrange
        expected_path = "/path/to/model.gguf"
        mock_manager = Mock()
        mock_manager.current_model_path = expected_path
        adapter = BitNetBackendAdapter(mock_manager)
        
        # Act
        result = adapter.get_model_path()
        
        # Assert
        assert result == expected_path
    
    @pytest.mark.asyncio
    async def test_generate_calls_manager(self):
        """Test generate calls manager.generate"""
        # Arrange
        mock_manager = Mock()
        mock_manager.generate.return_value = "Generated text"
        mock_manager.update_settings = Mock()
        mock_manager.get_state.return_value = {}
        
        adapter = BitNetBackendAdapter(mock_manager)
        
        messages = [
            ChatMessage(role=MessageRole.USER, content="Test")
        ]
        settings = InferenceSettings(temperature=0.7)
        
        # Act
        result = await adapter.generate(messages, settings)
        
        # Assert
        assert result == "Generated text"
        mock_manager.update_settings.assert_called_once_with(settings)
        mock_manager.generate.assert_called_once()
    
    def test_get_stats_returns_none_when_no_state(self):
        """Test get_stats returns None when no state available"""
        # Arrange
        mock_manager = Mock()
        adapter = BitNetBackendAdapter(mock_manager)
        
        # Act
        result = adapter.get_stats()
        
        # Assert
        assert result is None
    
    def test_get_stats_returns_performance_stats(self):
        """Test get_stats returns PerformanceStats when available"""
        # Arrange
        mock_manager = Mock()
        adapter = BitNetBackendAdapter(mock_manager)
        
        # Set last stats
        adapter._last_stats = {
            "time_to_first_token": 0.1,
            "tokens_per_second": 50.0,
            "input_tokens": 10,
            "output_tokens": 20,
        }
        
        # Act
        result = adapter.get_stats()
        
        # Assert
        assert result is not None
        assert result.time_to_first_token == 0.1
        assert result.tokens_per_second == 50.0
        assert result.input_tokens == 10
        assert result.output_tokens == 20

