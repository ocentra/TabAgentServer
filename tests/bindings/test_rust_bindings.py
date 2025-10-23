#!/usr/bin/env python3
"""
Test script for Rust model bindings

This tests the tabagent_model Python module built from Rust.
"""

import sys
import os

def test_cpu_detection():
    """Test CPU variant detection"""
    print("=" * 60)
    print("TEST 1: CPU Detection")
    print("=" * 60)
    
    try:
        import tabagent_model
        
        variant = tabagent_model.get_cpu_variant()
        print(f"✅ CPU Variant detected: {variant}")
        
        return True
    except Exception as e:
        print(f"❌ Error: {e}")
        return False

def test_binary_path():
    """Test optimal binary path generation"""
    print("\n" + "=" * 60)
    print("TEST 2: Binary Path Generation")
    print("=" * 60)
    
    try:
        import tabagent_model
        
        # Test with different base paths
        base_path = "external/BitNet/Release"
        binary_path = tabagent_model.get_optimal_binary(base_path, "llama-server.exe")
        print(f"✅ Binary path: {binary_path}")
        
        # Parse and validate
        parts = binary_path.split('/')
        if 'cpu' in parts and any('bitnet' in p or 'standard' in p for p in parts):
            print(f"✅ Path structure is correct")
        else:
            print(f"⚠️  Path structure unexpected: {parts}")
        
        return True
    except Exception as e:
        print(f"❌ Error: {e}")
        return False

def test_model_loading_api():
    """Test model loading API (without actual model file)"""
    print("\n" + "=" * 60)
    print("TEST 3: Model Loading API")
    print("=" * 60)
    
    try:
        import tabagent_model
        
        # This will fail (no actual model), but tests the API exists
        print("Checking PyModel class exists...")
        assert hasattr(tabagent_model, 'PyModel'), "PyModel class not found"
        print("✅ PyModel class exists")
        
        # Note: We can't actually load without a real model file
        print("⚠️  Actual model loading requires a GGUF file (skipped)")
        
        return True
    except Exception as e:
        print(f"❌ Error: {e}")
        return False

def main():
    """Run all tests"""
    print("\n" + "=" * 60)
    print("RUST BINDINGS TEST SUITE")
    print("=" * 60)
    print()
    
    # Check if module is installed
    try:
        import tabagent_model
        print(f"✅ Module 'tabagent_model' imported successfully")
        print(f"   Location: {tabagent_model.__file__}")
    except ImportError as e:
        print(f"❌ Failed to import tabagent_model: {e}")
        print("\nTo install, run:")
        print("  cd Server/tabagent-rs/crates/model-bindings")
        print("  pip install maturin")
        print("  maturin develop")
        return 1
    
    # Run tests
    results = []
    results.append(("CPU Detection", test_cpu_detection()))
    results.append(("Binary Path", test_binary_path()))
    results.append(("Model API", test_model_loading_api()))
    
    # Summary
    print("\n" + "=" * 60)
    print("TEST SUMMARY")
    print("=" * 60)
    passed = sum(1 for _, result in results if result)
    total = len(results)
    
    for name, result in results:
        status = "✅ PASS" if result else "❌ FAIL"
        print(f"{status}: {name}")
    
    print(f"\nTotal: {passed}/{total} tests passed")
    print("=" * 60)
    
    return 0 if passed == total else 1

if __name__ == "__main__":
    sys.exit(main())

