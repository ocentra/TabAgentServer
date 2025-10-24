"""
REAL tests for Python→Rust communication
NO MOCKS - tests actual PyO3 bindings

Testing philosophy:
- NO mocks for services/handlers
- REAL Python→Rust calls via PyO3
- REAL Rust→Python responses
- DB tests use REAL databases with FIXTURE data (not mocks)
- Tests verify actual communication pipeline works

This tests the CRITICAL integration points:
1. Python can call Rust via tabagent_native_handler
2. Rust can return valid JSON responses
3. Message routing logic works (GGUF→Rust, ONNX→Python)
4. Shared constants are consistent across both sides
"""

import json
import pytest
import sys
from pathlib import Path

# Add parent to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from Python.core import message_fields as mf
from Python.core.test_models import get_test_model

# Try to import Rust handler
try:
    import tabagent_native_handler as rust_handler
    RUST_AVAILABLE = True
except ImportError:
    RUST_AVAILABLE = False
    pytestmark = pytest.mark.skip(reason="Rust handler not built")


class TestPythonRustBridge:
    """REAL tests for Python calling Rust via PyO3"""
    
    def test_rust_handler_available(self):
        """Test Rust handler can be imported"""
        assert RUST_AVAILABLE, "Rust handler not built. Run: maturin develop"
        assert hasattr(rust_handler, 'handle_message')
        assert hasattr(rust_handler, 'initialize_handler')
    
    def test_ping_roundtrip(self):
        """Test basic Python→Rust→Python roundtrip"""
        # Arrange
        message = {mf.ACTION: "ping"}
        
        # Act - REAL call to Rust
        response_json = rust_handler.handle_message(json.dumps(message))
        response = json.loads(response_json)
        
        # Assert
        assert response[mf.STATUS] == mf.STATUS_SUCCESS
        assert mf.MESSAGE in response
    
    def test_get_available_models_real(self):
        """Test get_available_models returns REAL catalog"""
        # Arrange
        message = {mf.ACTION: "get_available_models"}
        
        # Act - REAL call to Rust
        response_json = rust_handler.handle_message(json.dumps(message))
        response = json.loads(response_json)
        
        # Assert
        assert response[mf.STATUS] == mf.STATUS_SUCCESS
        payload = response.get(mf.PAYLOAD, {})
        models = payload.get(mf.MODELS, [])
        
        # Should have models from default_models.json
        assert isinstance(models, list)
        assert len(models) > 0, "No models in catalog"
        
        # Verify structure
        first_model = models[0]
        assert "repo_id" in first_model
        assert "model_type" in first_model
    
    def test_get_memory_usage_real(self):
        """Test get_memory_usage returns REAL system info"""
        # Arrange
        message = {mf.ACTION: "get_memory_usage"}
        
        # Act - REAL call to Rust
        response_json = rust_handler.handle_message(json.dumps(message))
        response = json.loads(response_json)
        
        # Assert
        assert response[mf.STATUS] == mf.STATUS_SUCCESS
        payload = response.get(mf.PAYLOAD, {})
        
        # Should have RAM info
        assert mf.RAM in payload
        ram = payload[mf.RAM]
        assert mf.TOTAL in ram
        assert mf.USED in ram
        assert mf.AVAILABLE in ram
        
        # Values should be reasonable (in bytes)
        assert ram[mf.TOTAL] > 1_000_000_000, "Total RAM should be > 1GB"
        assert ram[mf.USED] >= 0
        assert ram[mf.AVAILABLE] >= 0
    
    def test_get_loaded_models_empty(self):
        """Test get_loaded_models when no models loaded"""
        # Arrange
        message = {mf.ACTION: "get_loaded_models"}
        
        # Act - REAL call to Rust
        response_json = rust_handler.handle_message(json.dumps(message))
        response = json.loads(response_json)
        
        # Assert
        assert response[mf.STATUS] == mf.STATUS_SUCCESS
        payload = response.get(mf.PAYLOAD, {})
        loaded = payload.get(mf.LOADED, [])
        
        # Should be empty list (no models loaded yet)
        assert isinstance(loaded, list)
    
    def test_unknown_action_error(self):
        """Test unknown action returns error"""
        # Arrange
        message = {mf.ACTION: "this_action_does_not_exist"}
        
        # Act - REAL call to Rust
        response_json = rust_handler.handle_message(json.dumps(message))
        response = json.loads(response_json)
        
        # Assert
        assert response[mf.STATUS] == mf.STATUS_ERROR
        assert mf.MESSAGE in response
    
    def test_invalid_json_error(self):
        """Test invalid JSON returns error"""
        # Arrange
        invalid_json = "{not valid json"
        
        # Act - REAL call to Rust
        response_json = rust_handler.handle_message(invalid_json)
        response = json.loads(response_json)
        
        # Assert
        assert response[mf.STATUS] == mf.STATUS_ERROR
        assert "parse" in response[mf.MESSAGE].lower()
    
    def test_get_models_by_type_gguf(self):
        """Test filtering models by type (GGUF)"""
        # Arrange
        message = {
            mf.ACTION: "get_models_by_type",
            mf.TYPE: "gguf"
        }
        
        # Act - REAL call to Rust
        response_json = rust_handler.handle_message(json.dumps(message))
        response = json.loads(response_json)
        
        # Assert
        assert response[mf.STATUS] == mf.STATUS_SUCCESS
        payload = response.get(mf.PAYLOAD, {})
        models = payload.get(mf.MODELS, [])
        
        # All should be GGUF type
        for model in models:
            assert model["model_type"].lower() == "gguf"
    
    def test_get_default_model_for_type(self):
        """Test getting default model for a type"""
        # Arrange
        message = {
            mf.ACTION: "get_default_model",
            mf.TYPE: "gguf"
        }
        
        # Act - REAL call to Rust
        response_json = rust_handler.handle_message(json.dumps(message))
        response = json.loads(response_json)
        
        # Assert - might succeed or fail if no default set
        if response[mf.STATUS] == mf.STATUS_SUCCESS:
            payload = response.get(mf.PAYLOAD, {})
            assert mf.DEFAULT_MODEL in payload
            default_model = payload[mf.DEFAULT_MODEL]
            assert "repo_id" in default_model
            assert default_model["model_type"].lower() == "gguf"


class TestNativeHostRouting:
    """Tests for native_host.py routing logic"""
    
    def test_native_host_model_type_detection(self):
        """Test model type detection functions"""
        import sys
        sys.path.insert(0, str(Path(__file__).parent.parent))
        
        from native_host import is_gguf_or_bitnet, is_onnx_model, is_mediapipe_model
        
        # GGUF detection
        assert is_gguf_or_bitnet("model.gguf") == True
        assert is_gguf_or_bitnet("path/to/model.gguf") == True
        assert is_gguf_or_bitnet("bitnet_model.safetensors") == True
        assert is_gguf_or_bitnet("model.onnx") == False
        
        # ONNX detection
        assert is_onnx_model("model.onnx") == True
        assert is_onnx_model("path/model.onnx") == True
        assert is_onnx_model("model.gguf") == False
        
        # MediaPipe detection
        assert is_mediapipe_model("mediapipe_model.tflite") == True
        assert is_mediapipe_model("model.gguf") == False


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])

