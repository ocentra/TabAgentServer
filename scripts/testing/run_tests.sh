#!/bin/bash
# Test runner for TabAgent
# Runs all Rust and Python tests
#
# TDD inference tests (will FAIL - not implemented yet):
#   cd tabagent-rs
#   cargo test --package model-loader -- --ignored --nocapture
#   cargo test --package tabagent-native-handler -- --ignored --nocapture

set -e

echo "=============================="
echo "🧪 TABAGENT TEST SUITE"
echo "=============================="
echo

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Navigate to Server root, then Rust workspace
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SERVER_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$SERVER_ROOT/tabagent-rs"

echo "📦 Running Rust tests..."
echo

# Run each crate's tests
echo "1️⃣  Testing model-cache..."
cargo test --package tabagent-model-cache --lib --tests -- --nocapture || {
    echo -e "${RED}❌ model-cache tests failed${NC}"
    exit 1
}

echo
echo "2️⃣  Testing hardware..."
cargo test --package tabagent-hardware --lib --tests -- --nocapture || {
    echo -e "${RED}❌ hardware tests failed${NC}"
    exit 1
}

echo
echo "3️⃣  Testing storage..."
cargo test --package storage --lib --tests -- --nocapture || {
    echo -e "${RED}❌ storage tests failed${NC}"
    exit 1
}

echo
echo "4️⃣  Testing query..."
cargo test --package query --lib --tests -- --nocapture || {
    echo -e "${RED}❌ query tests failed${NC}"
    exit 1
}

echo
echo "5️⃣  Testing model-loader..."
cargo test --package model-loader --lib --tests -- --nocapture || {
    echo -e "${RED}❌ model-loader tests failed${NC}"
    exit 1
}

echo
echo "6️⃣  Testing native-handler..."
cargo test --package tabagent-native-handler --lib --tests -- --nocapture || {
    echo -e "${RED}❌ native-handler tests failed${NC}"
    exit 1
}

echo
echo -e "${GREEN}✅ All Rust tests passed!${NC}"
echo

# Python tests
cd "$SERVER_ROOT"
echo "🐍 Running Python tests..."
echo

echo "1️⃣  Testing secrets..."
python -m pytest tests/test_secrets.py -v --tb=short || {
    echo -e "${RED}❌ Secrets tests failed${NC}"
    exit 1
}

echo
echo "2️⃣  Testing Python↔Rust bridge..."
python -m pytest tests/test_python_rust_bridge.py -v --tb=short || {
    echo -e "${RED}❌ Python↔Rust bridge tests failed${NC}"
    exit 1
}

echo
echo "3️⃣  Testing API endpoints..."
python -m pytest tests/test_api_endpoints.py -v --tb=short || {
    echo -e "${RED}❌ API endpoint tests failed${NC}"
    exit 1
}

echo
echo "4️⃣  Testing backend services..."
python -m pytest tests/test_backend_real.py -v --tb=short || {
    echo -e "${RED}❌ Backend tests failed${NC}"
    exit 1
}

echo
echo -e "${GREEN}✅ All Python tests passed!${NC}"
echo

echo "=============================="
echo -e "${GREEN}🎉 ALL TESTS PASSED!${NC}"
echo "=============================="

