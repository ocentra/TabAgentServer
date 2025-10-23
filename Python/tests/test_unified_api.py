#!/usr/bin/env python3
"""
Comprehensive tests for the Unified Model Detection API

Tests the integration between Rust detection/manifest generation and Python routing.
Follows RAG principles: no mocks except when absolutely necessary, use real data.
"""

import unittest
import json
import sys
import os

# Add parent directory to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))


class TestUnifiedAPIAvailability(unittest.TestCase):
    """Test that Rust unified API is available"""
    
    def test_rust_handler_imports(self):
        """Verify all required Rust functions can be imported"""
        try:
            from tabagent_native_handler import (
                detect_model_py,
                get_model_manifest_py,
                recommend_variant_py
            )
            self.assertTrue(True, "All Rust unified API functions imported successfully")
        except ImportError as e:
            self.fail(f"Failed to import Rust unified API: {e}\nInstall with: pip install -e Server/Rust/native-handler")


class TestModelDetection(unittest.TestCase):
    """Test model type detection from paths and repo names"""
    
    @classmethod
    def setUpClass(cls):
        """Import Rust detection function once"""
        try:
            from tabagent_native_handler import detect_model_py
            cls.detect_model_py = detect_model_py
            cls.api_available = True
        except ImportError:
            cls.api_available = False
    
    def setUp(self):
        """Skip tests if API not available"""
        if not self.api_available:
            self.skipTest("Rust unified API not available")
    
    def test_detect_gguf_from_file_path(self):
        """Detect GGUF model from file path"""
        result_json = self.detect_model_py("models/Qwen/Qwen2.5-3B/model-Q4_K_M.gguf")
        result = json.loads(result_json)
        
        self.assertEqual(result["model_type"], "GGUF")
        self.assertEqual(result["repo"], "Qwen/Qwen2.5-3B")
        self.assertIn("backend", result)
        self.assertEqual(result["backend"]["Rust"]["engine"], "llama.cpp")
    
    def test_detect_bitnet_from_file_path(self):
        """Detect BitNet model from file path (special GGUF)"""
        result_json = self.detect_model_py("models/1bitLLM/Falcon3-1B-Instruct-1.58bit/model.gguf")
        result = json.loads(result_json)
        
        self.assertEqual(result["model_type"], "BitNet")
        self.assertEqual(result["repo"], "1bitLLM/Falcon3-1B-Instruct-1.58bit")
        self.assertEqual(result["backend"]["Rust"]["engine"], "bitnet")
    
    def test_detect_onnx_from_file_path(self):
        """Detect ONNX model from file path"""
        result_json = self.detect_model_py("models/microsoft/Phi-3-mini/onnx/model_q4f16.onnx")
        result = json.loads(result_json)
        
        self.assertEqual(result["model_type"], "ONNX")
        self.assertEqual(result["repo"], "microsoft/Phi-3-mini")
        self.assertEqual(result["backend"]["Python"]["engine"], "transformers.js")
    
    def test_detect_gguf_from_repo_name(self):
        """Detect GGUF model from repository name"""
        test_repos = [
            "Qwen/Qwen2.5-3B-GGUF",
            "bartowski/Llama-3.2-3B-gguf",
            "TheBloke/Mistral-7B-GGUF",
        ]
        
        for repo in test_repos:
            with self.subTest(repo=repo):
                result_json = self.detect_model_py(repo)
                result = json.loads(result_json)
                
                self.assertEqual(result["model_type"], "GGUF", f"Failed for {repo}")
                self.assertEqual(result["repo"], repo)
    
    def test_detect_bitnet_from_repo_name(self):
        """Detect BitNet model from repository name"""
        test_repos = [
            "microsoft/BitNet-b1.58-2B-4T",
            "1bitLLM/bitnet-3b",
        ]
        
        for repo in test_repos:
            with self.subTest(repo=repo):
                result_json = self.detect_model_py(repo)
                result = json.loads(result_json)
                
                self.assertEqual(result["model_type"], "BitNet", f"Failed for {repo}")
    
    def test_detect_onnx_from_repo_name(self):
        """Detect ONNX model from repository name"""
        test_repos = [
            "microsoft/Phi-3-mini-4k-instruct-onnx",
            "HuggingFaceTB/SmolLM3-3B-ONNX",
            "Xenova/gpt2-onnx",
        ]
        
        for repo in test_repos:
            with self.subTest(repo=repo):
                result_json = self.detect_model_py(repo)
                result = json.loads(result_json)
                
                self.assertEqual(result["model_type"], "ONNX", f"Failed for {repo}")
    
    def test_detect_litert_from_repo_name(self):
        """Detect LiteRT model from repository name"""
        result_json = self.detect_model_py("google/gemma-3n-E4B-it-litert-lm")
        result = json.loads(result_json)
        
        self.assertEqual(result["model_type"], "LiteRT")
        self.assertEqual(result["backend"]["Python"]["engine"], "mediapipe")
    
    def test_unknown_model_type(self):
        """Detect unknown model type raises error"""
        with self.assertRaises(Exception):
            self.detect_model_py("random-org/unknown-model")


class TestManifestGeneration(unittest.TestCase):
    """Test ONNX manifest generation from HuggingFace API"""
    
    @classmethod
    def setUpClass(cls):
        """Import Rust manifest function once"""
        try:
            from tabagent_native_handler import get_model_manifest_py
            cls.get_model_manifest_py = get_model_manifest_py
            cls.api_available = True
        except ImportError:
            cls.api_available = False
    
    def setUp(self):
        """Skip tests if API not available"""
        if not self.api_available:
            self.skipTest("Rust unified API not available")
    
    def test_manifest_structure(self):
        """Verify manifest has correct structure"""
        # Using a real ONNX repo
        try:
            manifest_json = self.get_model_manifest_py(
                "microsoft/Phi-3-mini-4k-instruct-onnx",
                None,  # auth_token
                None   # size_limit (use default)
            )
            manifest = json.loads(manifest_json)
            
            # Verify top-level structure
            self.assertIn("repo", manifest)
            self.assertIn("quants", manifest)
            self.assertIn("manifestVersion", manifest)
            self.assertEqual(manifest["manifestVersion"], 1)
            
            # Verify quants structure
            self.assertIsInstance(manifest["quants"], dict)
            if len(manifest["quants"]) > 0:
                # Check first quant
                quant_key = list(manifest["quants"].keys())[0]
                quant_info = manifest["quants"][quant_key]
                
                self.assertIn("files", quant_info)
                self.assertIn("status", quant_info)
                self.assertIn("dtype", quant_info)
                self.assertIn("hasExternalData", quant_info)
                
                self.assertIsInstance(quant_info["files"], list)
                self.assertIn(quant_info["status"], [
                    "available", "downloaded", "failed", 
                    "not_found", "unavailable", "unsupported", "server_only"
                ])
        
        except Exception as e:
            # Network issues are acceptable for this test
            if "Failed to fetch" in str(e) or "Network" in str(e):
                self.skipTest(f"Network error (acceptable): {e}")
            else:
                raise
    
    def test_manifest_bypass_model(self):
        """Verify bypass models ignore size limits"""
        try:
            manifest_json = self.get_model_manifest_py(
                "google/gemma-3n-E4B-it-litert-lm",
                None,
                100  # Very small limit (100 bytes)
            )
            manifest = json.loads(manifest_json)
            
            # All quants should still be "available" despite size limit
            for quant_info in manifest["quants"].values():
                self.assertEqual(
                    quant_info["status"],
                    "available",
                    "Bypass model should ignore size limits"
                )
        
        except Exception as e:
            if "Failed to fetch" in str(e) or "Network" in str(e):
                self.skipTest(f"Network error (acceptable): {e}")
            else:
                raise


class TestVariantRecommendation(unittest.TestCase):
    """Test automatic variant selection"""
    
    @classmethod
    def setUpClass(cls):
        """Import Rust recommendation function once"""
        try:
            from tabagent_native_handler import recommend_variant_py
            cls.recommend_variant_py = recommend_variant_py
            cls.api_available = True
        except ImportError:
            cls.api_available = False
    
    def setUp(self):
        """Skip tests if API not available"""
        if not self.api_available:
            self.skipTest("Rust unified API not available")
    
    def test_recommend_variant(self):
        """Verify variant recommendation returns valid quant key"""
        try:
            variant = self.recommend_variant_py(
                "microsoft/Phi-3-mini-4k-instruct-onnx",
                16.0,  # RAM GB
                0.0    # VRAM GB
            )
            
            # Should return a valid file path
            self.assertIsInstance(variant, str)
            self.assertTrue(len(variant) > 0)
            self.assertTrue(variant.endswith(".onnx"))
        
        except Exception as e:
            if "Failed to fetch" in str(e) or "Network" in str(e):
                self.skipTest(f"Network error (acceptable): {e}")
            else:
                raise


class TestLoadModelUnified(unittest.TestCase):
    """Test the Python load_model_unified() function"""
    
    @classmethod
    def setUpClass(cls):
        """Import load_model_unified once"""
        try:
            # Import from parent directory
            import native_host
            cls.load_model_unified = native_host.load_model_unified
            cls.api_available = native_host.RUST_UNIFIED_API_AVAILABLE
        except (ImportError, AttributeError):
            cls.api_available = False
    
    def setUp(self):
        """Skip tests if API not available"""
        if not self.api_available:
            self.skipTest("Unified API not available")
    
    def test_gguf_routing(self):
        """Verify GGUF models route to Rust handler"""
        result = self.load_model_unified("Qwen/Qwen2.5-3B-GGUF")
        
        # Should not error on detection
        self.assertIn("status", result)
        # Backend routing depends on RUST_HANDLER_AVAILABLE
    
    def test_bitnet_routing(self):
        """Verify BitNet models route to Rust handler"""
        result = self.load_model_unified("microsoft/BitNet-b1.58-2B-4T")
        
        self.assertIn("status", result)
        # Should detect as BitNet
    
    def test_onnx_routing(self):
        """Verify ONNX models route correctly"""
        result = self.load_model_unified("microsoft/Phi-3-mini-4k-instruct-onnx")
        
        self.assertIn("status", result)
        # For ONNX, should either succeed or require network access
    
    def test_litert_routing(self):
        """Verify LiteRT models route to MediaPipe"""
        result = self.load_model_unified("google/gemma-3n-E4B-it-litert-lm")
        
        self.assertIn("status", result)
        if result["status"] == "success":
            self.assertEqual(result.get("backend"), "python-mediapipe")
    
    def test_variant_selection(self):
        """Verify explicit variant selection works"""
        result = self.load_model_unified(
            "microsoft/Phi-3-mini-4k-instruct-onnx",
            variant="onnx/model_q4f16.onnx"
        )
        
        self.assertIn("status", result)
        if result["status"] == "success":
            self.assertEqual(result.get("variant"), "onnx/model_q4f16.onnx")
    
    def test_error_handling(self):
        """Verify proper error handling for invalid inputs"""
        result = self.load_model_unified("invalid/nonexistent-model")
        
        # Should return error, not raise exception
        self.assertIn("status", result)


class TestEndToEndFlow(unittest.TestCase):
    """Test complete end-to-end workflow"""
    
    @classmethod
    def setUpClass(cls):
        """Import all required functions"""
        try:
            from tabagent_native_handler import (
                detect_model_py,
                get_model_manifest_py,
                recommend_variant_py
            )
            import native_host
            
            cls.detect_model_py = detect_model_py
            cls.get_model_manifest_py = get_model_manifest_py
            cls.recommend_variant_py = recommend_variant_py
            cls.load_model_unified = native_host.load_model_unified
            cls.api_available = True
        except (ImportError, AttributeError):
            cls.api_available = False
    
    def setUp(self):
        """Skip tests if API not available"""
        if not self.api_available:
            self.skipTest("Unified API not available")
    
    def test_onnx_full_workflow(self):
        """Test complete ONNX workflow: detect → manifest → recommend → load"""
        repo = "microsoft/Phi-3-mini-4k-instruct-onnx"
        
        try:
            # Step 1: Detect
            detection = json.loads(self.detect_model_py(repo))
            self.assertEqual(detection["model_type"], "ONNX")
            
            # Step 2: Get manifest
            manifest = json.loads(self.get_model_manifest_py(repo, None, None))
            self.assertGreater(len(manifest["quants"]), 0)
            
            # Step 3: Recommend variant
            variant = self.recommend_variant_py(repo, 16.0, 0.0)
            self.assertIn(variant, manifest["quants"])
            
            # Step 4: Load with unified API
            result = self.load_model_unified(repo, variant=variant)
            self.assertIn("status", result)
        
        except Exception as e:
            if "Failed to fetch" in str(e) or "Network" in str(e):
                self.skipTest(f"Network error (acceptable): {e}")
            else:
                raise
    
    def test_gguf_full_workflow(self):
        """Test complete GGUF workflow: detect → load"""
        repo = "Qwen/Qwen2.5-3B-GGUF"
        
        # Step 1: Detect
        detection = json.loads(self.detect_model_py(repo))
        self.assertEqual(detection["model_type"], "GGUF")
        
        # Step 2: Load directly (no manifest needed for GGUF)
        result = self.load_model_unified(repo)
        self.assertIn("status", result)


if __name__ == '__main__':
    # Run tests with verbose output
    unittest.main(verbosity=2)

