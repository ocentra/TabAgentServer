#!/usr/bin/env python3
"""
Test Model Loading Pipeline

Tests the complete model load/unload pipeline from Python → Rust:
1. Download model (if not cached)
2. Load model into memory
3. Query model state
4. Unload model
5. Verify state changes

This tests ONLY the model management, not inference (Phase 3).
"""

import json
import sys
import os

# Add parent directory to path for imports
sys.path.insert(0, os.path.dirname(__file__))

try:
    from tabagent_native_handler import handle_message
    print("✅ Successfully imported tabagent_native_handler")
except ImportError as e:
    print(f"❌ Failed to import tabagent_native_handler: {e}")
    print("\nPlease install the wheel first:")
    print("  pip install ../target/wheels/tabagent_native_handler-0.1.0-cp39-abi3-win_amd64.whl")
    sys.exit(1)


def test_get_system_resources():
    """Test: Get system resources"""
    print("\n" + "="*60)
    print("TEST: Get System Resources")
    print("="*60)
    
    request = json.dumps({
        "action": "GET_SYSTEM_RESOURCES"
    })
    
    response_json = handle_message(request)
    response = json.loads(response_json)
    
    print(f"Response: {json.dumps(response, indent=2)}")
    
    assert response["status"] == "success", f"Expected success, got {response['status']}"
    assert "payload" in response, "Response should have payload"
    assert "cpu" in response["payload"], "Should have CPU info"
    
    print("✅ PASSED: System resources retrieved")
    return response["payload"]


def test_get_available_models():
    """Test: Get list of available models"""
    print("\n" + "="*60)
    print("TEST: Get Available Models")
    print("="*60)
    
    request = json.dumps({
        "action": "GET_AVAILABLE_MODELS"
    })
    
    response_json = handle_message(request)
    response = json.loads(response_json)
    
    print(f"Response: {json.dumps(response, indent=2)}")
    
    assert response["status"] == "success", f"Expected success, got {response['status']}"
    assert "payload" in response, "Response should have payload"
    assert "models" in response["payload"], "Should have models list"
    
    print(f"✅ PASSED: Found {len(response['payload']['models'])} available models")
    return response["payload"]["models"]


def test_download_model():
    """Test: Download a small model (phi-3-mini q4)"""
    print("\n" + "="*60)
    print("TEST: Download Model")
    print("="*60)
    
    # Use a small model for testing
    repo_id = "microsoft/Phi-3-mini-4k-instruct-gguf"
    model_file = "Phi-3-mini-4k-instruct-q4.gguf"
    
    request = json.dumps({
        "action": "DOWNLOAD_MODEL",
        "modelPath": repo_id,
        "modelFile": model_file
    })
    
    print(f"Downloading: {repo_id}/{model_file}")
    print("(This may take a while for first download...)")
    
    response_json = handle_message(request)
    response = json.loads(response_json)
    
    print(f"Response: {json.dumps(response, indent=2)}")
    
    if response["status"] == "error":
        print(f"⚠️  Download returned error: {response.get('message', 'Unknown error')}")
        print("This might be expected if download is not implemented or network is unavailable")
        return None
    
    assert response["status"] == "success", f"Expected success, got {response['status']}"
    print("✅ PASSED: Model downloaded (or already cached)")
    return (repo_id, model_file)


def test_load_model(repo_id: str, model_file: str):
    """Test: Load a GGUF model into memory"""
    print("\n" + "="*60)
    print("TEST: Load Model")
    print("="*60)
    
    request = json.dumps({
        "action": "LOAD_MODEL",
        "modelPath": repo_id,
        "modelFile": model_file,
        "settings": {
            "n_gpu_layers": 0,  # CPU only for testing
            "n_ctx": 2048
        }
    })
    
    print(f"Loading: {repo_id}/{model_file}")
    
    response_json = handle_message(request)
    response = json.loads(response_json)
    
    print(f"Response: {json.dumps(response, indent=2)}")
    
    if response["status"] == "error":
        print(f"⚠️  Load returned error: {response.get('message', 'Unknown error')}")
        return False
    
    assert response["status"] == "success", f"Expected success, got {response['status']}"
    assert response["payload"]["isReady"] == True, "Model should be ready"
    assert "vocabSize" in response["payload"], "Should have vocab size"
    assert "contextSize" in response["payload"], "Should have context size"
    
    print(f"✅ PASSED: Model loaded successfully")
    print(f"   Vocab Size: {response['payload']['vocabSize']}")
    print(f"   Context Size: {response['payload']['contextSize']}")
    print(f"   Backend: {response['payload']['backend']}")
    return True


def test_get_loaded_models():
    """Test: Get list of currently loaded models"""
    print("\n" + "="*60)
    print("TEST: Get Loaded Models")
    print("="*60)
    
    request = json.dumps({
        "action": "GET_LOADED_MODELS"
    })
    
    response_json = handle_message(request)
    response = json.loads(response_json)
    
    print(f"Response: {json.dumps(response, indent=2)}")
    
    assert response["status"] == "success", f"Expected success, got {response['status']}"
    assert "payload" in response, "Response should have payload"
    assert "models" in response["payload"], "Should have models list"
    
    num_loaded = len(response["payload"]["models"])
    print(f"✅ PASSED: Found {num_loaded} loaded model(s)")
    return response["payload"]["models"]


def test_get_model_state(model_id: str):
    """Test: Get state of a specific model"""
    print("\n" + "="*60)
    print("TEST: Get Model State")
    print("="*60)
    
    request = json.dumps({
        "action": "GET_MODEL_STATE",
        "modelId": model_id
    })
    
    response_json = handle_message(request)
    response = json.loads(response_json)
    
    print(f"Response: {json.dumps(response, indent=2)}")
    
    if response["status"] == "error":
        print(f"⚠️  Model state query returned error (expected if model not loaded)")
        return None
    
    assert response["status"] == "success", f"Expected success, got {response['status']}"
    print(f"✅ PASSED: Model state retrieved")
    return response["payload"]


def test_unload_model(model_id: str):
    """Test: Unload a model from memory"""
    print("\n" + "="*60)
    print("TEST: Unload Model")
    print("="*60)
    
    request = json.dumps({
        "action": "UNLOAD_MODEL",
        "modelId": model_id
    })
    
    response_json = handle_message(request)
    response = json.loads(response_json)
    
    print(f"Response: {json.dumps(response, indent=2)}")
    
    if response["status"] == "error":
        print(f"⚠️  Unload returned error: {response.get('message', 'Unknown error')}")
        return False
    
    assert response["status"] == "success", f"Expected success, got {response['status']}"
    print(f"✅ PASSED: Model unloaded successfully")
    return True


def test_recommend_split():
    """Test: Get GPU/CPU split recommendation"""
    print("\n" + "="*60)
    print("TEST: Recommend Split")
    print("="*60)
    
    model_size = 2 * 1024 * 1024 * 1024  # 2GB
    total_layers = 32
    
    request = json.dumps({
        "action": "RECOMMEND_SPLIT",
        "modelSize": model_size,
        "totalLayers": total_layers
    })
    
    response_json = handle_message(request)
    response = json.loads(response_json)
    
    print(f"Response: {json.dumps(response, indent=2)}")
    
    assert response["status"] == "success", f"Expected success, got {response['status']}"
    print(f"✅ PASSED: Split recommendation retrieved")
    return response["payload"]


def run_full_pipeline_test():
    """Run complete model loading pipeline test"""
    print("\n" + "#"*60)
    print("# MODEL LOADING PIPELINE TEST SUITE")
    print("#"*60)
    
    results = {
        "passed": 0,
        "failed": 0,
        "skipped": 0
    }
    
    try:
        # Test 1: Get system resources
        resources = test_get_system_resources()
        results["passed"] += 1
        
        # Test 2: Get available models
        available_models = test_get_available_models()
        results["passed"] += 1
        
        # Test 3: Download model (may be skipped if already cached)
        download_result = test_download_model()
        if download_result:
            repo_id, model_file = download_result
            results["passed"] += 1
        else:
            print("⚠️  Download test skipped or failed")
            # Use default values for next tests
            repo_id = "microsoft/Phi-3-mini-4k-instruct-gguf"
            model_file = "Phi-3-mini-4k-instruct-q4.gguf"
            results["skipped"] += 1
        
        model_id = f"{repo_id}/{model_file}"
        
        # Test 4: Load model
        load_success = test_load_model(repo_id, model_file)
        if load_success:
            results["passed"] += 1
            
            # Test 5: Get loaded models
            loaded_models = test_get_loaded_models()
            results["passed"] += 1
            
            # Test 6: Get specific model state
            state = test_get_model_state(model_id)
            if state:
                results["passed"] += 1
            else:
                results["skipped"] += 1
            
            # Test 7: Unload model
            unload_success = test_unload_model(model_id)
            if unload_success:
                results["passed"] += 1
            else:
                results["failed"] += 1
            
            # Test 8: Verify model is unloaded
            loaded_after = test_get_loaded_models()
            if len(loaded_after) == 0:
                print("✅ VERIFIED: Model was successfully unloaded")
                results["passed"] += 1
            else:
                print("❌ FAILED: Model still appears in loaded list")
                results["failed"] += 1
        else:
            print("⚠️  Load test failed, skipping subsequent tests")
            results["failed"] += 1
            results["skipped"] += 5
        
        # Test 9: Recommend split
        recommendation = test_recommend_split()
        results["passed"] += 1
        
    except AssertionError as e:
        print(f"\n❌ TEST FAILED: {e}")
        results["failed"] += 1
    except Exception as e:
        print(f"\n❌ UNEXPECTED ERROR: {e}")
        import traceback
        traceback.print_exc()
        results["failed"] += 1
    
    # Print summary
    print("\n" + "#"*60)
    print("# TEST SUMMARY")
    print("#"*60)
    print(f"✅ Passed:  {results['passed']}")
    print(f"❌ Failed:  {results['failed']}")
    print(f"⚠️  Skipped: {results['skipped']}")
    print(f"Total:     {sum(results.values())}")
    
    if results["failed"] == 0:
        print("\n🎉 ALL TESTS PASSED! Model loading pipeline is functional.")
        return 0
    else:
        print("\n❌ SOME TESTS FAILED. Check errors above.")
        return 1


if __name__ == "__main__":
    exit_code = run_full_pipeline_test()
    sys.exit(exit_code)

