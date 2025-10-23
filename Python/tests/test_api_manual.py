"""
Test script for TabAgent API Server

Tests the HTTP API using existing backends.
"""

import requests
import json


def test_health():
    """Test health endpoint"""
    print("Testing /health...")
    resp = requests.get("http://localhost:8000/api/v1/health")
    print(f"Status: {resp.status_code}")
    print(json.dumps(resp.json(), indent=2))
    print()


def test_models():
    """Test models list endpoint"""
    print("Testing /models...")
    resp = requests.get("http://localhost:8000/api/v1/models")
    print(f"Status: {resp.status_code}")
    print(json.dumps(resp.json(), indent=2))
    print()


def test_chat_completion():
    """Test chat completion (non-streaming)"""
    print("Testing /chat/completions (non-streaming)...")
    
    payload = {
        "model": "bitnet-3b",
        "messages": [
            {"role": "user", "content": "Say hello!"}
        ],
        "stream": False
    }
    
    resp = requests.post(
        "http://localhost:8000/api/v1/chat/completions",
        json=payload
    )
    
    print(f"Status: {resp.status_code}")
    if resp.status_code == 200:
        print(json.dumps(resp.json(), indent=2))
    else:
        print(resp.text)
    print()


def test_chat_completion_stream():
    """Test streaming chat completion"""
    print("Testing /chat/completions (streaming)...")
    
    payload = {
        "model": "bitnet-3b",
        "messages": [
            {"role": "user", "content": "Count to 5"}
        ],
        "stream": True
    }
    
    with requests.post(
        "http://localhost:8000/api/v1/chat/completions",
        json=payload,
        stream=True
    ) as resp:
        print(f"Status: {resp.status_code}")
        
        if resp.status_code == 200:
            for line in resp.iter_lines():
                if line:
                    line_str = line.decode('utf-8')
                    if line_str.startswith("data: "):
                        data = line_str[6:]  # Remove "data: " prefix
                        if data != "[DONE]":
                            chunk = json.loads(data)
                            content = chunk.get("choices", [{}])[0].get("delta", {}).get("content", "")
                            if content:
                                print(content, end="", flush=True)
            print()  # Newline after stream
        else:
            print(resp.text)
    print()


if __name__ == "__main__":
    print("=== TabAgent API Server Tests ===\n")
    
    try:
        test_health()
        test_models()
        # test_chat_completion()  # Uncomment when model is loaded
        # test_chat_completion_stream()  # Uncomment when model is loaded
        
        print("✅ All tests completed!")
    except requests.exceptions.ConnectionError:
        print("❌ Server not running. Start with: python -m api.main")
    except Exception as e:
        print(f"❌ Error: {e}")

