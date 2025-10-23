#!/usr/bin/env python3
"""
Developer Setup Script - Store HuggingFace Token
DO NOT COMMIT THIS FILE TO GIT
"""

import sys
import os

# Add current directory to path
sys.path.insert(0, os.path.dirname(__file__))

from core.hf_auth import set_hf_token, get_hf_token, has_hf_token

def setup_dev_token():
    """Set up development HuggingFace token"""
    
    # Your development token (DO NOT COMMIT)
    DEV_TOKEN = "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"
    
    print("[*] Setting up HuggingFace authentication for development...")
    
    # Check if token already exists
    if has_hf_token():
        print("[OK] Token already stored!")
        print("     To update, run with --force flag")
        return
    
    # Store the token
    if set_hf_token(DEV_TOKEN):
        print("[OK] Development token stored securely!")
        print("     Location: OS keyring (Windows Credential Manager / macOS Keychain / Linux Secret Service)")
        print("     or encrypted file in ~/.tabagent/.hf_token.enc")
        print()
        print("[SUCCESS] You're all set! The token will be used automatically for HuggingFace API calls.")
    else:
        print("[ERROR] Failed to store token")
        sys.exit(1)

def verify_token():
    """Verify token is stored and accessible"""
    print("\n[*] Verifying token storage...")
    
    if has_hf_token():
        token = get_hf_token()
        if token:
            # Show partial token for verification
            masked = token[:7] + "..." + token[-4:]
            print(f"[OK] Token found: {masked}")
        else:
            print("[WARN] Token flag shows stored, but retrieval failed")
    else:
        print("[ERROR] No token stored")

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
            print("[*] Cleared existing token")
        
        setup_dev_token()
        verify_token()

