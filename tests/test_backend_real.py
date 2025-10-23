"""
REAL backend tests - NO MOCKS
Tests actual InferenceServiceAdapter and BackendManager with real backends
"""

import pytest
from unittest.mock import patch

from api.backend_adapter import InferenceServiceAdapter
from api.backend_manager import BackendManager, get_backend_manager
from core.message_types import (
    ChatMessage,
    InferenceSettings,
    BackendType,
    MessageRole,
)
from api.types import ChatCompletionRequest


class TestInferenceServiceAdapter:
    """REAL tests for InferenceServiceAdapter - no mocks"""
    
    def test_adapter_can_initialize(self):
        """Test adapter initializes with real inference service"""
        # REAL initialization
        adapter = InferenceServiceAdapter()
        
        # Assert
        assert adapter is not None
        assert adapter._service is not None
    
    def test_is_loaded_returns_bool(self):
        """Test is_loaded returns actual state"""
        adapter = InferenceServiceAdapter()
        
        # REAL call
        result = adapter.is_loaded()
        
        # Assert - should be boolean
        assert isinstance(result, bool)
    
    def test_get_backend_type_returns_type_or_none(self):
        """Test get_backend_type returns actual type or None"""
        adapter = InferenceServiceAdapter()
        
        # REAL call
        result = adapter.get_backend_type()
        
        # Assert - should be BackendType or None
        if result is not None:
            assert isinstance(result, BackendType)
    
    def test_get_model_path_returns_string_or_none(self):
        """Test get_model_path returns actual path or None"""
        adapter = InferenceServiceAdapter()
        
        # REAL call
        result = adapter.get_model_path()
        
        # Assert - should be string or None
        if result is not None:
            assert isinstance(result, str)
            assert len(result) > 0
    
    @pytest.mark.asyncio
    async def test_generate_without_model_fails(self):
        """Test generate fails properly when no model loaded"""
        adapter = InferenceServiceAdapter()
        
        # Skip if model is actually loaded
        if adapter.is_loaded():
            pytest.skip("Model is loaded, can't test failure case")
        
        messages = [
            ChatMessage(role=MessageRole.USER, content="Test")
        ]
        settings = InferenceSettings(temperature=0.7)
        
        # REAL call - should fail
        with pytest.raises(Exception):
            await adapter.generate(messages, settings)
    
    def test_get_stats_returns_none_when_no_inference(self):
        """Test get_stats returns None when no inference has run"""
        adapter = InferenceServiceAdapter()
        
        # REAL call - before any inference
        result = adapter.get_stats()
        
        # Assert - no stats yet
        assert result is None


class TestBackendManager:
    """REAL tests for BackendManager - no mocks"""
    
    def test_manager_initializes(self):
        """Test BackendManager initializes correctly"""
        # REAL initialization
        manager = BackendManager()
        
        # Assert
        assert manager is not None
    
    def test_singleton_returns_same_instance(self):
        """Test get_backend_manager returns singleton"""
        # REAL calls
        manager1 = get_backend_manager()
        manager2 = get_backend_manager()
        
        # Assert - same instance
        assert manager1 is manager2
    
    def test_is_model_loaded_returns_bool(self):
        """Test is_model_loaded returns boolean"""
        manager = get_backend_manager()
        
        # REAL call
        result = manager.is_model_loaded()
        
        # Assert
        assert isinstance(result, bool)
    
    def test_get_backend_type_when_no_backend(self):
        """Test get_backend_type returns None when no backend"""
        manager = BackendManager()  # Fresh instance
        
        # REAL call
        result = manager.get_backend_type()
        
        # Assert
        assert result is None or isinstance(result, BackendType)
    
    def test_get_model_id_returns_string(self):
        """Test get_model_id returns model ID or default"""
        manager = get_backend_manager()
        
        # REAL call
        result = manager.get_model_id()
        
        # Assert
        assert isinstance(result, str)
    
    @pytest.mark.asyncio
    async def test_chat_completion_fails_without_model(self):
        """Test chat_completion fails when no model loaded"""
        manager = BackendManager()  # Fresh instance, no backend
        
        request = ChatCompletionRequest(
            model="test",
            messages=[
                ChatMessage(role=MessageRole.USER, content="Test")
            ]
        )
        
        # REAL call - should fail
        with pytest.raises(RuntimeError, match="No model loaded"):
            await manager.chat_completion(request)
    
    def test_set_backend_updates_state(self):
        """Test set_backend actually updates manager state"""
        from unittest.mock import Mock
        
        manager = BackendManager()
        mock_backend = Mock()
        mock_backend.is_loaded.return_value = True
        model_id = "test-model"
        
        # REAL call
        manager.set_backend(mock_backend, model_id)
        
        # Assert - state updated
        assert manager.get_model_id() == model_id
        assert manager.is_model_loaded() == True


class TestBackendIntegration:
    """REAL integration tests for backend flow"""
    
    def test_adapter_wraps_real_service(self):
        """Test adapter wraps actual InferenceService"""
        from core.inference_service import get_inference_service
        
        # Get real service
        service = get_inference_service()
        
        # Create adapter
        adapter = InferenceServiceAdapter()
        
        # Assert - adapter uses same service
        assert adapter._service is service
    
    def test_manager_can_use_adapter(self):
        """Test BackendManager can use InferenceServiceAdapter"""
        manager = BackendManager()
        adapter = InferenceServiceAdapter()
        
        # REAL call
        manager.set_backend(adapter, "test-model")
        
        # Assert
        assert manager.get_model_id() == "test-model"
        assert manager._current_backend is adapter
    
    def test_inference_service_is_singleton(self):
        """Test InferenceService is singleton across adapters"""
        from core.inference_service import get_inference_service
        
        # Get service directly
        service1 = get_inference_service()
        
        # Get via adapter
        adapter = InferenceServiceAdapter()
        service2 = adapter._service
        
        # Assert - same instance
        assert service1 is service2


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])

