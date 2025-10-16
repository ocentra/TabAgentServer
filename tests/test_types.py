"""
Unit tests for API types

Tests Pydantic models and type conversions.
"""

import pytest
from pydantic import ValidationError

from api.types import (
    ChatCompletionRequest,
    ChatCompletionResponse,
    ChatCompletionChunk,
    ModelInfo,
    PerformanceStats,
)
from core.message_types import ChatMessage, MessageRole, InferenceSettings
from api.constants import OpenAIObject, FinishReason


class TestChatCompletionRequest:
    """Tests for ChatCompletionRequest"""
    
    def test_valid_request(self):
        """Test valid chat completion request"""
        # Arrange & Act
        request = ChatCompletionRequest(
            model="test-model",
            messages=[
                ChatMessage(role=MessageRole.USER, content="Hello")
            ]
        )
        
        # Assert
        assert request.model == "test-model"
        assert len(request.messages) == 1
        assert request.temperature is None  # Optional field
        assert request.stream is False  # Default value
    
    def test_request_with_all_parameters(self):
        """Test request with all parameters"""
        # Arrange & Act
        request = ChatCompletionRequest(
            model="test-model",
            messages=[
                ChatMessage(role=MessageRole.USER, content="Hello")
            ],
            temperature=0.8,
            max_tokens=100,
            stream=True,
            stop=["STOP"],
            top_p=0.95
        )
        
        # Assert
        assert request.temperature == 0.8
        assert request.max_tokens == 100
        assert request.stream is True
        assert request.stop == ["STOP"]
        assert request.top_p == 0.95
    
    def test_to_inference_settings(self):
        """Test conversion to InferenceSettings"""
        # Arrange
        request = ChatCompletionRequest(
            model="test-model",
            messages=[
                ChatMessage(role=MessageRole.USER, content="Hello")
            ],
            temperature=0.8,
            max_tokens=200,
            top_p=0.95
        )
        
        # Act
        settings = request.to_inference_settings()
        
        # Assert
        assert isinstance(settings, InferenceSettings)
        assert settings.temperature == 0.8
        assert settings.max_new_tokens == 200
        assert settings.top_p == 0.95
    
    def test_invalid_temperature(self):
        """Test validation rejects invalid temperature"""
        # Act & Assert
        with pytest.raises(ValidationError):
            ChatCompletionRequest(
                model="test",
                messages=[
                    ChatMessage(role=MessageRole.USER, content="Test")
                ],
                temperature=3.0  # Invalid: > 2.0
            )
    
    def test_invalid_max_tokens(self):
        """Test validation rejects invalid max_tokens"""
        # Act & Assert
        with pytest.raises(ValidationError):
            ChatCompletionRequest(
                model="test",
                messages=[
                    ChatMessage(role=MessageRole.USER, content="Test")
                ],
                max_tokens=0  # Invalid: < 1
            )


class TestPerformanceStats:
    """Tests for PerformanceStats"""
    
    def test_valid_stats(self):
        """Test valid performance stats"""
        # Arrange & Act
        stats = PerformanceStats(
            time_to_first_token=0.1,
            tokens_per_second=50.0,
            input_tokens=10,
            output_tokens=20
        )
        
        # Assert
        assert stats.time_to_first_token == 0.1
        assert stats.tokens_per_second == 50.0
        assert stats.input_tokens == 10
        assert stats.output_tokens == 20
    
    def test_optional_fields(self):
        """Test stats with optional fields"""
        # Arrange & Act
        stats = PerformanceStats()
        
        # Assert
        assert stats.time_to_first_token is None
        assert stats.tokens_per_second is None
        assert stats.input_tokens is None
        assert stats.output_tokens is None


class TestModelInfo:
    """Tests for ModelInfo"""
    
    def test_valid_model_info(self):
        """Test valid model info"""
        # Arrange & Act
        info = ModelInfo(
            id="test-model",
            created=1234567890,
            owned_by="tabagent"
        )
        
        # Assert
        assert info.id == "test-model"
        assert info.object == OpenAIObject.MODEL
        assert info.created == 1234567890
        assert info.owned_by == "tabagent"

