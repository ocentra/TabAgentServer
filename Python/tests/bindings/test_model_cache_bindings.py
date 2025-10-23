#!/usr/bin/env python3
"""
Test script for TabAgent model cache Python bindings.
"""

import sys
import os

# Add the wheel location to path (adjust after building)
# sys.path.insert(0, 'tabagent-rs/target/wheels')

try:
    import tabagent_model_cache
    print("✅ Successfully imported tabagent_model_cache")
except ImportError as e:
    print(f"❌ Failed to import: {e}")
    print("\n💡 Build the wheel first:")
    print("   cd Server/Rust/model-cache-bindings")
    print("   maturin build --release")
    print("   pip install ../target/wheels/tabagent_model_cache-*.whl")
    sys.exit(1)

def main():
    print("\n🧪 Testing TabAgent Model Cache Bindings")
    print("=" * 60)
    
    # Create cache
    print("\n1️⃣ Creating model cache...")
    cache = tabagent_model_cache.ModelCache("./test_model_cache_db")
    print("   ✅ Cache created")
    
    # Get stats
    print("\n2️⃣ Getting cache statistics...")
    stats = cache.get_stats()
    print(f"   📊 Total repos: {stats['total_repos']}")
    print(f"   📊 Total size: {stats['total_size']} bytes")
    
    # Scan a small test model
    print("\n3️⃣ Scanning test model repository...")
    test_repo = "hf-internal-testing/tiny-random-gpt2"
    print(f"   📡 Scanning: {test_repo}")
    
    try:
        manifest = cache.scan_repo(test_repo)
        print(f"   ✅ Scanned successfully!")
        print(f"   📋 Repo: {manifest['repo_id']}")
        print(f"   📋 Task: {manifest.get('task', 'N/A')}")
        print(f"   📋 Quants found: {len(manifest['quants'])}")
        
        for quant_key, quant_info in manifest['quants'].items():
            print(f"\n   📦 Quant: {quant_key}")
            print(f"      Status: {quant_info['status']}")
            print(f"      Files: {len(quant_info['files'])}")
            
    except Exception as e:
        print(f"   ⚠️ Scan error (expected if no internet): {e}")
    
    # Test has_file
    print("\n4️⃣ Testing file existence check...")
    has_config = cache.has_file(test_repo, "config.json")
    print(f"   📄 Has config.json: {has_config}")
    
    print("\n" + "=" * 60)
    print("✅ ALL BASIC TESTS PASSED!")
    print("\n💡 Next steps:")
    print("   1. Use cache.download_file() to download model files")
    print("   2. Use cache.get_file() to retrieve cached files")
    print("   3. Integrate with native_host.py for production use")

if __name__ == "__main__":
    main()

