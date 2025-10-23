# LM Studio Backend

LM Studio integration for Tab Agent native host.

## Overview

This backend manages LM Studio server lifecycle. Model loading and inference can be handled either:
- **Via Extension** (using LM Studio.js SDK) - Direct HTTP to localhost:1234
- **Via Native App** (proxy mode) - Extension → Native → LM Studio API

## Architecture

```
Native App Startup:
├── Detect LM Studio installation (~/.lmstudio/)
├── Check if bootstrapped (lms CLI available)
├── Check if server running (localhost:1234)
└── Auto-start if not running (lms server start)

Extension Request:
├── Option A: Direct SDK (extension uses LM Studio.js)
└── Option B: Proxy (extension sends to native app)
```

## LM Studio Setup (Done by Installer)

### Installation
- Downloads from: `https://installers.lmstudio.ai/`
- Installs to: `%LOCALAPPDATA%\Programs\LM Studio\`
- Creates: `~/.lmstudio/` directory

### Bootstrap
- Command: `lms bootstrap`
- Creates: `~/.lmstudio/bin/lms.exe`
- Enables CLI commands

### Configuration
- File: `~/.lmstudio/preferences.json`
- Key: `developer.enableLocalLLMService = true`
- Enables headless server mode

## API Endpoints

LM Studio exposes OpenAI-compatible API:

### Base URL
```
http://localhost:1234
```

### Endpoints
- `GET /v1/models` - List available models
- `POST /v1/chat/completions` - Chat completion
- `POST /v1/completions` - Text completion
- `POST /v1/embeddings` - Generate embeddings

## Manager API

### Check Status
```python
from backends.lmstudio import LMStudioManager

manager = LMStudioManager()
status = manager.get_status()
# Returns:
# {
#   "installed": bool,
#   "bootstrapped": bool,
#   "server_running": bool,
#   "api_endpoint": "http://127.0.0.1:1234",
#   "current_model": Optional[str]
# }
```

### Ensure Server Running
```python
# Starts server if not already running
manager.ensure_server_running()
```

### Stop Server
```python
manager.stop_server()
```

### Proxy Chat Completion
```python
messages = [
    ChatMessage(role=MessageRole.USER, content="Hello")
]
response = manager.proxy_chat_completion(messages, settings)
```

## Native Messaging Protocol

### Check LM Studio Status
```json
{
  "action": "check_lmstudio"
}
```

**Response:**
```json
{
  "status": "success",
  "payload": {
    "installed": true,
    "bootstrapped": true,
    "server_running": true,
    "api_endpoint": "http://127.0.0.1:1234"
  }
}
```

### Start Server
```json
{
  "action": "start_lmstudio"
}
```

### Stop Server
```json
{
  "action": "stop_lmstudio"
}
```

## Error States

| State | Installed | Bootstrapped | Server Running | Action |
|-------|-----------|--------------|----------------|--------|
| ✅ Ready | ✅ | ✅ | ✅ | None needed |
| ⚠️ Server Down | ✅ | ✅ | ❌ | Call `ensure_server_running()` |
| ❌ Not Bootstrapped | ✅ | ❌ | ❌ | User must run: `lms bootstrap` |
| ❌ Not Installed | ❌ | ❌ | ❌ | User must install LM Studio |

## Extension Integration (TODO)

Extension will need to:

1. **Install SDK:**
   ```bash
   npm install lmstudio
   ```

2. **Check availability:**
   ```typescript
   // Check via native app
   const status = await chrome.runtime.sendNativeMessage('com.tabagent.host', {
     action: 'check_lmstudio'
   })
   
   // OR check directly
   const direct = await fetch('http://localhost:1234/v1/models')
     .then(() => true).catch(() => false)
   ```

3. **Choose mode:**
   ```typescript
   if (nativeAppAvailable) {
     // Use native app (it ensures server running)
     await sendToNativeApp(request)
   } else {
     // Use SDK directly
     const client = new LMStudioClient()
     await client.chat.completions.create({...})
   }
   ```

## Dependencies

- `requests` - HTTP communication with LM Studio API
- `subprocess` - Run lms CLI commands
- `pathlib` - Path handling

## Files

- `manager.py` - LM Studio lifecycle manager (278 lines)
- `__init__.py` - Module exports
- `README.md` - This file

