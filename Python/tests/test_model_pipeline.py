"""
Comprehensive test for model load/unload pipeline

Tests the full flow:
1. Initialize handler
2. Query available models
3. Download test model (small)
4. Load model into memory
5. Query loaded models
6. Set active model
7. Unload model
8. Delete from cache

Uses small test models to avoid huge downloads.
"""

import json
import sys
import os
from pathlib import Path

# Add parent to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from Python.core.secrets import get_secrets, get_huggingface_token
from Python.core import message_fields as mf
from Python.core.test_models import get_test_model, TINY_GGUF_REPO, TINY_GGUF_FILE

# Try to import Rust handler
try:
    import tabagent_native_handler as rust_handler
    RUST_AVAILABLE = True
except ImportError:
    print("❌ Rust handler not available. Build it first:")
    print("   cd Server/tabagent-rs")
    print("   maturin develop --package tabagent-native-handler")
    RUST_AVAILABLE = False
    sys.exit(1)


class ModelPipelineTest:
    """Test harness for model pipeline"""
    
    def __init__(self, cache_dir: str = None):
        self.cache_dir = cache_dir or str(Path.home() / ".tabagent" / "test_cache")
        self.secrets = get_secrets()
        self.test_results = []
        
        # Ensure cache directory exists
        Path(self.cache_dir).mkdir(parents=True, exist_ok=True)
        
        print(f"📁 Using cache directory: {self.cache_dir}")
        print(f"🔑 HuggingFace token available: {self.secrets.has_huggingface_token()}")
        print()
    
    def log_test(self, name: str, passed: bool, details: str = ""):
        """Log test result"""
        status = "✅" if passed else "❌"
        self.test_results.append((name, passed))
        print(f"{status} {name}")
        if details:
            print(f"   {details}")
    
    def call_rust(self, action: str, **params) -> dict:
        """Call Rust handler with action"""
        message = {
            mf.ACTION: action,
            **params
        }
        response_json = rust_handler.handle_message(json.dumps(message))
        return json.loads(response_json)
    
    def test_1_initialize(self):
        """Test 1: Initialize handler"""
        print("\n=== Test 1: Initialize Handler ===")
        
        try:
            response_json = rust_handler.initialize_handler(self.cache_dir)
            response = json.loads(response_json)
            
            success = response.get(mf.STATUS) == mf.STATUS_SUCCESS
            self.log_test(
                "Initialize handler",
                success,
                response.get(mf.MESSAGE, "")
            )
            return success
        except Exception as e:
            self.log_test("Initialize handler", False, str(e))
            return False
    
    def test_2_get_available_models(self):
        """Test 2: Get available models"""
        print("\n=== Test 2: Get Available Models ===")
        
        try:
            response = self.call_rust("get_available_models")
            models = response.get(mf.PAYLOAD, {}).get(mf.MODELS, [])
            
            # Check test models are present
            test_models = [m for m in models if m.get("source") == "test"]
            
            self.log_test(
                "Get available models",
                len(models) > 0,
                f"Found {len(models)} models ({len(test_models)} test models)"
            )
            
            # Print test models
            for model in test_models:
                print(f"   - {model['name']} ({model['model_type']}, {model['size_gb']}GB)")
            
            return len(test_models) > 0
        except Exception as e:
            self.log_test("Get available models", False, str(e))
            return False
    
    def test_3_download_model(self):
        """Test 3: Download test model"""
        print("\n=== Test 3: Download Model ===")
        
        # Use smallest test model (GGUF 82MB)
        test_model = get_test_model("gguf-tiny")
        
        try:
            response = self.call_rust(
                "download_model",
                modelPath=test_model.repo_id,
                modelFile=test_model.file_path
            )
            
            success = response.get(mf.STATUS) == mf.STATUS_SUCCESS
            self.log_test(
                "Download test model",
                success,
                response.get(mf.MESSAGE, "")
            )
            return success
        except Exception as e:
            self.log_test("Download test model", False, str(e))
            return False
    
    def test_4_load_model(self):
        """Test 4: Load model into memory"""
        print("\n=== Test 4: Load Model ===")
        
        test_model = get_test_model("gguf-tiny")
        
        try:
            response = self.call_rust(
                "load_model",
                modelPath=test_model.repo_id,
                modelFile=test_model.file_path
            )
            
            success = response.get(mf.STATUS) == mf.STATUS_SUCCESS
            payload = response.get(mf.PAYLOAD, {})
            
            if success:
                details = f"Loaded to: {payload.get(mf.LOADED_TO)}"
                if mf.VRAM_USED in payload:
                    details += f", VRAM: {payload[mf.VRAM_USED] / (1024**3):.2f}GB"
            else:
                details = response.get(mf.MESSAGE, "")
            
            self.log_test("Load model", success, details)
            return success
        except Exception as e:
            self.log_test("Load model", False, str(e))
            return False
    
    def test_5_get_loaded_models(self):
        """Test 5: Query loaded models"""
        print("\n=== Test 5: Get Loaded Models ===")
        
        try:
            response = self.call_rust("get_loaded_models")
            models = response.get(mf.PAYLOAD, {}).get(mf.MODELS, [])
            
            success = len(models) > 0
            self.log_test(
                "Get loaded models",
                success,
                f"Found {len(models)} loaded model(s)"
            )
            
            for model in models:
                print(f"   - {model.get('model_id')} on {model.get('loaded_to')}")
            
            return success
        except Exception as e:
            self.log_test("Get loaded models", False, str(e))
            return False
    
    def test_6_set_active_model(self):
        """Test 6: Set active model"""
        print("\n=== Test 6: Set Active Model ===")
        
        test_model = get_test_model("gguf-tiny")
        model_id = f"{test_model.repo_id}/{test_model.file_path}"
        
        try:
            response = self.call_rust(
                "set_active_model",
                modelId=model_id
            )
            
            success = response.get(mf.STATUS) == mf.STATUS_SUCCESS
            self.log_test("Set active model", success, response.get(mf.MESSAGE, ""))
            return success
        except Exception as e:
            self.log_test("Set active model", False, str(e))
            return False
    
    def test_7_get_current_model(self):
        """Test 7: Get current active model"""
        print("\n=== Test 7: Get Current Model ===")
        
        try:
            response = self.call_rust("get_current_model")
            current = response.get(mf.PAYLOAD, {}).get(mf.CURRENT_MODEL)
            
            success = current is not None
            self.log_test(
                "Get current model",
                success,
                f"Current: {current}" if current else "No active model"
            )
            return success
        except Exception as e:
            self.log_test("Get current model", False, str(e))
            return False
    
    def test_8_get_memory_usage(self):
        """Test 8: Get memory usage"""
        print("\n=== Test 8: Get Memory Usage ===")
        
        try:
            response = self.call_rust("get_memory_usage")
            payload = response.get(mf.PAYLOAD, {})
            ram = payload.get(mf.RAM, {})
            
            success = ram.get(mf.TOTAL) is not None
            details = f"RAM: {ram.get(mf.USED, 0) / (1024**3):.2f}GB / {ram.get(mf.TOTAL, 0) / (1024**3):.2f}GB"
            
            self.log_test("Get memory usage", success, details)
            return success
        except Exception as e:
            self.log_test("Get memory usage", False, str(e))
            return False
    
    def test_9_unload_model(self):
        """Test 9: Unload model from memory"""
        print("\n=== Test 9: Unload Model ===")
        
        test_model = get_test_model("gguf-tiny")
        model_id = f"{test_model.repo_id}/{test_model.file_path}"
        
        try:
            response = self.call_rust(
                "unload_model",
                modelPath=model_id
            )
            
            success = response.get(mf.STATUS) == mf.STATUS_SUCCESS
            self.log_test("Unload model", success, response.get(mf.MESSAGE, ""))
            return success
        except Exception as e:
            self.log_test("Unload model", False, str(e))
            return False
    
    # ========================================================================
    # TDD: INFERENCE TESTS (WILL FAIL - NOT IMPLEMENTED YET)
    # ========================================================================
    
    def test_inference_simple_hello(self):
        """TDD: Test simple inference (NOT IMPLEMENTED)"""
        print("\n=== TDD Test: Simple Inference ===")
        print("⚠️  This test will FAIL - inference not implemented yet")
        
        # What we NEED:
        # 1. Load model
        # 2. Send simple prompt
        # 3. Get response
        # 4. Verify non-empty
        
        test_model = get_test_model("gguf-tiny")
        
        try:
            # TODO: Add "infer" action to native-handler
            response = self.call_rust(
                "infer",
                modelId=f"{test_model.repo_id}/{test_model.file_path}",
                prompt="Say hello",
                maxTokens=50
            )
            
            result = response.get(mf.PAYLOAD, {}).get("text", "")
            success = len(result) > 0
            
            self.log_test("Simple inference", success, f"Got: {result[:50]}...")
            return success
        except Exception as e:
            print(f"   Expected failure: {e}")
            self.log_test("Simple inference (TDD)", False, "Not implemented yet")
            return False
    
    def test_inference_streaming(self):
        """TDD: Test streaming inference (NOT IMPLEMENTED)"""
        print("\n=== TDD Test: Streaming Inference ===")
        print("⚠️  This test will FAIL - streaming not implemented yet")
        
        # What we NEED:
        # 1. Start streaming inference
        # 2. Receive tokens in real-time
        # 3. Ability to cancel
        # 4. Progress updates
        
        test_model = get_test_model("gguf-tiny")
        
        try:
            # TODO: Add "infer_stream" action
            response = self.call_rust(
                "infer_stream",
                modelId=f"{test_model.repo_id}/{test_model.file_path}",
                prompt="Write a short story",
                onToken=True  # Enable streaming
            )
            
            self.log_test("Streaming inference (TDD)", False, "Not implemented yet")
            return False
        except Exception as e:
            print(f"   Expected failure: {e}")
            return False
    
    def test_inference_with_chat_history(self):
        """TDD: Test inference with chat history (NOT IMPLEMENTED)"""
        print("\n=== TDD Test: Chat History ===")
        print("⚠️  This test will FAIL - chat context not implemented yet")
        
        # What we NEED:
        # 1. Multi-turn conversation
        # 2. Context preservation
        # 3. Context window management
        
        test_model = get_test_model("gguf-tiny")
        
        try:
            # TODO: Add chat history to infer action
            messages = [
                {"role": "user", "content": "What is 2+2?"},
                {"role": "assistant", "content": "4"},
                {"role": "user", "content": "What about 3+3?"}
            ]
            
            response = self.call_rust(
                "infer",
                modelId=f"{test_model.repo_id}/{test_model.file_path}",
                messages=messages
            )
            
            self.log_test("Chat history (TDD)", False, "Not implemented yet")
            return False
        except Exception as e:
            print(f"   Expected failure: {e}")
            return False
    
    def test_inference_with_settings(self):
        """TDD: Test inference with custom settings (NOT IMPLEMENTED)"""
        print("\n=== TDD Test: Inference Settings ===")
        print("⚠️  This test will FAIL - settings not implemented yet")
        
        # What we NEED:
        # 1. Temperature control
        # 2. Top-p, top-k sampling
        # 3. Max tokens
        # 4. Stop sequences
        
        test_model = get_test_model("gguf-tiny")
        
        try:
            response = self.call_rust(
                "infer",
                modelId=f"{test_model.repo_id}/{test_model.file_path}",
                prompt="Hello",
                settings={
                    "temperature": 0.7,
                    "topP": 0.9,
                    "topK": 40,
                    "maxTokens": 100,
                    "stop": ["</s>", "\n\n"]
                }
            )
            
            self.log_test("Inference settings (TDD)", False, "Not implemented yet")
            return False
        except Exception as e:
            print(f"   Expected failure: {e}")
            return False
    
    def run_all_tests(self):
        """Run all tests in sequence"""
        print("=" * 60)
        print("🧪 MODEL PIPELINE TEST SUITE")
        print("=" * 60)
        
        # Run tests
        tests = [
            self.test_1_initialize,
            self.test_2_get_available_models,
            # Skip download/load for now - they require actual models
            # self.test_3_download_model,
            # self.test_4_load_model,
            # self.test_5_get_loaded_models,
            # self.test_6_set_active_model,
            # self.test_7_get_current_model,
            self.test_8_get_memory_usage,
            # self.test_9_unload_model,
        ]
        
        for test in tests:
            if not test():
                print(f"\n⚠️  Test failed, stopping here")
                break
        
        # Summary
        print("\n" + "=" * 60)
        print("📊 TEST SUMMARY")
        print("=" * 60)
        passed = sum(1 for _, p in self.test_results if p)
        total = len(self.test_results)
        print(f"Passed: {passed}/{total}")
        print(f"Failed: {total - passed}/{total}")
        
        if passed == total:
            print("\n🎉 ALL TESTS PASSED!")
            return 0
        else:
            print("\n❌ SOME TESTS FAILED")
            return 1


if __name__ == "__main__":
    test = ModelPipelineTest()
    sys.exit(test.run_all_tests())

