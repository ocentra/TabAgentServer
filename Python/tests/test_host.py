#!/usr/bin/env python3
"""
Test script for the Tab Agent Native Host
"""

import json
import struct
import subprocess
import sys

def send_message(proc, message_content):
    """Send a message to the native host"""
    encoded_content = json.dumps(message_content).encode('utf-8')
    encoded_length = struct.pack('@I', len(encoded_content))
    
    # Send length and message to native host
    if proc.stdin:
        proc.stdin.write(encoded_length)
        proc.stdin.write(encoded_content)
        proc.stdin.flush()

def get_message(proc):
    """Read a message from the native host"""
    if not proc.stdout:
        return None
    raw_length = proc.stdout.read(4)
    if not raw_length:
        return None
    message_length = struct.unpack('@I', raw_length)[0]
    message = proc.stdout.read(message_length).decode('utf-8')
    return json.loads(message)

# Start the native host process
try:
    process = subprocess.Popen(
        [sys.executable, 'native_host.py'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        bufsize=0
    )
except Exception as e:
    print(f"Failed to start native host: {e}")
    sys.exit(1)

print("Testing Tab Agent Native Host...")
print("=" * 40)

# Test 1: Ping
print("\nTest 1: Ping")
send_message(process, {"action": "ping"})
response = get_message(process)
print(f"Response: {response}")

# Test 2: Get system info
print("\nTest 2: Get system info")
send_message(process, {"action": "get_system_info"})
response = get_message(process)
print(f"Response: {response}")

# Test 3: Execute command (simple test)
print("\nTest 3: Execute command (echo test)")
send_message(process, {"action": "execute_command", "command": "echo Hello from native host"})
response = get_message(process)
print(f"Response: {response}")

# Clean up
process.terminate()
process.wait()

print("\n" + "=" * 40)
print("Testing completed!")
