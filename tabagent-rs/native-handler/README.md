# Native Handler (native-handler)

**PyO3 binding for GGUF/BitNet native messaging**

## Purpose

This crate provides the Rust-side message handler for native messaging (Chrome Extension) and other transports. It intercepts messages for GGUF/BitNet models and handles them using Rust, while letting Python handle ONNX/MediaPipe models.

## Architecture

```
Chrome Extension → native_host.py
                      ↓
            Detect model type
                      ↓
         ┌────────────┴────────────┐
         ↓                         ↓
    GGUF/BitNet              ONNX/MediaPipe
         ↓                         ↓
    handle_message()         Python handlers
    (this crate)             (backends/*.py)
         ↓                         ↓
    Return response          Return response
    (NO fallback)            (NO fallback)
```

**Key:** Python decides ONCE, no retry, no fallback. Dead end if wrong type.

## Integration with Existing Code

**DOES NOT DUPLICATE:**
- `model-loader` - Uses it for FFI inference
- `model-cache` - Uses it for downloads/storage
- `storage` - Uses it for database operations
- `hardware` - Uses it for CPU/GPU detection

**HOW IT FITS:**
- Called by `Server/native_host.py` BEFORE Python backend routing
- Returns `Some(response)` if handled, `None` if not GGUF/BitNet
- Matches Python's message format (action: LOAD_MODEL, GENERATE, etc.)

## Message Flow

```python
# In native_host.py
from tabagent_native_handler import handle_message

def main():
    while True:
        message = get_message()
        model = message.get("modelPath") or message.get("model", "")
        
        # ONE-TIME routing decision (NO fallback)
        if is_gguf_or_bitnet(model):
            # Rust MUST handle - no fallback
            response_json = handle_message(json.dumps(message))
            send_message(json.loads(response_json))
            
        elif is_onnx(model):
            # Python handles
            response = onnx_handler(message)
            send_message(response)
            
        else:
            # Unknown type - dead end
            send_message({"status": "error", "message": "Unknown model type"})

def is_gguf_or_bitnet(model: str) -> bool:
    lower = model.lower()
    return ".gguf" in lower or "bitnet" in lower or "llamacpp" in lower
```

## Supported Actions

| Action | Status | Uses Crates |
|--------|--------|-------------|
| `LOAD_MODEL` | 🟡 Skeleton | model-cache, model-loader |
| `GENERATE` | ❌ Not implemented | model-loader (Phase 3) |
| `UNLOAD_MODEL` | 🟡 Skeleton | model-loader |
| `GET_MODEL_STATE` | 🟡 Skeleton | model-loader |
| `PULL_MODEL` | 🟡 Skeleton | model-cache |
| `DELETE_MODEL` | 🟡 Skeleton | model-cache |

## Expected Message Format

Matches Python's existing format from `Server/core/message_types.py`:

```json
{
  "action": "LOAD_MODEL",
  "modelPath": "/path/to/model.gguf",
  "settings": {
    "n_gpu_layers": 0,
    "temperature": 0.7,
    ...
  }
}
```

## Response Format

Also matches Python's format:

```json
{
  "status": "success",
  "message": "Model loaded",
  "payload": {
    "isReady": true,
    "backend": "Rust-GGUF",
    "modelPath": "/path/to/model.gguf"
  }
}
```

## Building

```bash
cd Server/tabagent-rs/native-handler
maturin develop --release  # Dev install
# OR
maturin build --release    # Wheel in target/wheels/
```

## Integration Status

- [x] Crate structure created
- [x] PyO3 function signature defined
- [x] Message parsing (action, modelPath)
- [x] Model type detection (GGUF/BitNet check)
- [ ] Actual model loading (uses model-cache + model-loader)
- [ ] Actual generation (blocked on model-loader Phase 3)
- [ ] Model cache integration
- [ ] Database integration for chat history
- [ ] Python integration in native_host.py

## When This Runs

This handler is invoked for EVERY message in:
- Chrome Native Messaging (via stdin)
- FastAPI HTTP requests (if integrated)
- WebRTC data channel messages (if integrated)

It self-selects based on model type, not transport.

## Why This Exists

**Instead of:** Python backends for GGUF/BitNet (backends/bitnet/, backends/llamacpp/)
**We have:** Single Rust implementation via PyO3
**Benefit:** Native speed, no code duplication (DRY), eventually ALL models here

## Future Vision

```
Phase 1: GGUF/BitNet → Rust (NOW)
Phase 2: ONNX → Rust (when rust-bert/candle mature)
Phase 3: MediaPipe → Rust (when available)
Phase 4: Python only for transformers/safetensors (if needed)
```

**Goal:** Move ALL model handling to Rust, Python becomes thin transport layer.

