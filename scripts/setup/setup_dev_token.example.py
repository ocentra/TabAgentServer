#!/usr/bin/env python3
"""
Developer Setup Script Template - Store HuggingFace Token

INSTRUCTIONS FOR OTHER DEVELOPERS:
1. Copy this file to setup_dev_token.py (not tracked by git)
2. Replace YOUR_TOKEN_HERE with your actual HuggingFace token
3. Run: python Server/setup_dev_token.py

Your token will be stored securely in:
- Windows: Credential Manager
- macOS: Keychain
- Linux: Secret Service
- Fallback: Encrypted file in ~/.tabagent/.hf_token.enc
"""

import sys
import os

# Add current directory to path
sys.path.insert(0, os.path.dirname(__file__))

from core.hf_auth import set_hf_token, get_hf_token, has_hf_token

def setup_dev_token():
    """Set up development HuggingFace token"""
    
    # TODO: Replace with your HuggingFace token
    # Get it from: https://huggingface.co/settings/tokens
    DEV_TOKEN = "YOUR_TOKEN_HERE"  # hf_xxxxxxxxxxxxxxxxxxxxxxxxxxxxx
    
    if DEV_TOKEN == "YOUR_TOKEN_HERE":
        print("❌ ERROR: Please edit this file and add your HuggingFace token!")
        print()
        print("Steps:")
        print("1. Copy setup_dev_token.example.py to setup_dev_token.py")
        print("2. Get your token from: https://huggingface.co/settings/tokens")
        print("3. Replace YOUR_TOKEN_HERE with your actual token")
        print("4. Run: python Server/setup_dev_token.py")
        sys.exit(1)
    
    print("🔐 Setting up HuggingFace authentication for development...")
    
    # Check if token already exists
    if has_hf_token():
        print("✅ Token already stored!")
        print("   To update, run with --force flag")
        return
    
    # Store the token
    if set_hf_token(DEV_TOKEN):
        print("✅ Development token stored securely!")
        print("   Location: OS keyring (Windows Credential Manager / macOS Keychain / Linux Secret Service)")
        print("   or encrypted file in ~/.tabagent/.hf_token.enc")
        print()
        print("🎉 You're all set! The token will be used automatically for HuggingFace API calls.")
    else:
        print("❌ Failed to store token")
        sys.exit(1)

def verify_token():
    """Verify token is stored and accessible"""
    print("\n🔍 Verifying token storage...")
    
    if has_hf_token():
        token = get_hf_token()
        if token:
            # Show partial token for verification
            masked = token[:7] + "..." + token[-4:]
            print(f"✅ Token found: {masked}")
        else:
            print("⚠️  Token flag shows stored, but retrieval failed")
    else:
        print("❌ No token stored")

if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description="Set up development HuggingFace token")
    parser.add_argument("--force", action="store_true", help="Force update existing token")
    parser.add_argument("--verify", action="store_true", help="Only verify token, don't set")
    args = parser.parse_args()
    
    if args.verify:
        verify_token()
    else:
        if args.force:
            from core.hf_auth import clear_hf_token
            clear_hf_token()
            print("🗑️  Cleared existing token")
        
        setup_dev_token()
        verify_token()

