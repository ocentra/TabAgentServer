# BitNet Integration - Server Side

## Overview

This is the **server-side** implementation for BitNet 1.58 model inference. The extension remains unchanged and sends standard messages. This server layer handles model detection, routing, and inference.

## Architecture

```
Extension (unchanged)
    ↓ Chrome Native Messaging (stdin/stdout)
native_host.py (entry point)
    ↓
backends/bitnet/ (BitNet 1.58 models)
    ↓
llama-server binary (CPU) OR BitNet GPU modules
```

## Project Structure

```
Server/
├── core/                          # Shared types and config
│   ├── __init__.py
│   ├── message_types.py          # Protocol definitions (Enums, Pydantic)
│   └── config.py                 # Typed configuration
│
├── backends/                      # Inference backend implementations
│   ├── __init__.py
│   ├── bitnet/                   # BitNet 1.58 backend
│   │   ├── __init__.py
│   │   ├── manager.py            # BitNet lifecycle manager
│   │   ├── validator.py          # GGUF validator & detector
│   │   └── binaries/             # BitNet platform-specific executables
│   │       ├── README.md         # Build instructions
│   │       ├── windows/
│   │       │   └── llama-server.exe
│   │       ├── macos/
│   │       │   └── llama-server
│   │       └── linux/
│   │           └── llama-server
│   └── lmstudio/                 # LM Studio backend (future)
│       └── __init__.py
│
├── BitNet/                        # BitNet source (submodule/fork)
├── build-tool/                    # Build scripts (existing)
├── tests/                         # Tests (existing)
│
├── native_host.py                 # Main entry point
├── config.py                      # Config shim (imports from core/)
├── requirements.txt               # Python dependencies
├── com.tabagent.host.json        # Native messaging manifest
└── README.md                      # This file
```

## Message Protocol

### Load Model
```json
{
  "action": "load_model",
  "modelPath": "/path/to/model.gguf",
  "isBitNet": true
}
```

**Response:**
```json
{
  "status": "success",
  "type": "workerReady",
  "payload": {
    "backend": "bitnet_cpu",
    "modelPath": "/path/to/model.gguf",
    "executionProvider": "bitnet_cpu"
  }
}
```

### Generate Text
```json
{
  "action": "generate",
  "messages": [
    {"role": "user", "content": "Hello"}
  ],
  "settings": {
    "temperature": 0.7,
    "top_p": 0.9,
    "max_new_tokens": 512
  }
}
```

**Response (streaming):**
```json
{
  "type": "generationUpdate",
  "payload": {
    "token": "Hello",
    "tps": "15.3",
    "numTokens": 1
  }
}
```

**Response (complete):**
```json
{
  "status": "success",
  "type": "generationComplete",
  "payload": {
    "output": "Hello! How can I help you?",
    "generatedText": "Hello! How can I help you?"
  }
}
```

### Unload Model
```json
{
  "action": "unload_model"
}
```

### Get Model State
```json
{
  "action": "get_model_state"
}
```

**Response:**
```json
{
  "status": "success",
  "payload": {
    "isReady": true,
    "backend": "bitnet_cpu",
    "modelPath": "/path/to/model.gguf"
  }
}
```

## Model Detection

The system auto-detects BitNet models using:

1. **Filename patterns:** `bitnet`, `b1.58`, `b1_58`, `1.58bit`
2. **Quantization types:** `i2_s`, `tl1`, `tl2` in filename
3. **GGUF metadata:** Architecture and model name fields

## Lifecycle Management

- ✅ **Load once** - Model stays in memory
- ✅ **Stateless** - No chat history stored (extension manages)
- ✅ **Reusable** - Multiple inference calls without reload
- ✅ **Unload on demand** - When switching models or cleanup

## Backend Selection

| Model Type | Backend | Implementation |
|------------|---------|----------------|
| BitNet 1.58 (i2_s, tl1, tl2) | BitNet CPU | llama-server subprocess |
| BitNet 1.58 (GPU) | BitNet GPU | Python + CUDA (future) |
| Regular GGUF | LM Studio | (future implementation) |
| Other formats | LM Studio | (future implementation) |

## Configuration

Edit `core/config.py`:

```python
from core.config import BITNET_CONFIG

# Customize settings
BITNET_CONFIG.cpu_port = 8765
BITNET_CONFIG.cpu_context_size = 4096
BITNET_CONFIG.default_temperature = 0.7
BITNET_CONFIG.startup_timeout_seconds = 5.0
```

Or use the root-level shim `config.py` that re-exports everything from `core/`.

## Installation

```bash
cd Server/
pip install -r requirements.txt
```

## Dependencies

- **pydantic** - Data validation and strong typing
- **requests** - HTTP communication with llama-server
- **torch** - GPU detection (CUDA support)
- **typing-extensions** - Enhanced type hints

## Testing Locally

1. **Build BitNet binary** (or place pre-built in `binaries/`)
2. **Start native host:**
   ```bash
   python native_host.py
   ```
3. **Send test message (stdin):**
   ```json
   {"action": "ping"}
   ```

## Future Enhancements

- [ ] GPU backend implementation (BitNet CUDA)
- [ ] LM Studio integration
- [ ] Generation stopping/interruption
- [ ] Model caching strategies
- [ ] Performance monitoring

