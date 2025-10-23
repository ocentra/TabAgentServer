#!/usr/bin/env python3
"""
Native Messaging Host for Tab Agent Extension
This script handles communication between the Chrome extension and local system resources.
"""

import sys
import json
import struct
import logging
import os
from typing import Dict, Any, Optional

# Import typed message definitions from core
from core.message_types import (
    ActionType,
    EventType,
    BackendType,
    LoadModelRequest,
    GenerateRequest,
    UpdateSettingsRequest,
    UnloadModelRequest,
    GetModelStateRequest,
    ErrorResponse,
    SuccessResponse,
    LoadingStatus
)

# Import shared inference service (DRY - used by both HTTP and stdin)
from core.inference_service import get_inference_service

# Import backend implementations
from backends.bitnet import BitNetManager, BitNetConfig, GGUFValidator
from backends.lmstudio import LMStudioManager

# Configuration defaults
class Config:
    LOG_LEVEL = "DEBUG"
    LOG_FILE = "native_host.log"
    ALLOWED_COMMANDS = []
    COMMAND_TIMEOUT = 30
    MAX_MESSAGE_SIZE = 1024 * 1024

# Try to import custom configuration
try:
    import config
    # Override defaults with custom config
    for attr in dir(config):
        if not attr.startswith('_'):
            setattr(Config, attr, getattr(config, attr))
except ImportError:
    pass

# Set up logging
log_level = getattr(logging, Config.LOG_LEVEL.upper(), logging.DEBUG)
logging.basicConfig(
    filename=Config.LOG_FILE,
    level=log_level,
    format='%(asctime)s [%(levelname)s] %(message)s'
)

# Use shared inference service (DRY - same instance as HTTP API)
_inference_service = get_inference_service()

# Initialize backend managers (legacy - will migrate to service)
bitnet_manager: Optional[BitNetManager] = None
lmstudio_manager: Optional[LMStudioManager] = None

def get_message() -> Dict[str, Any]:
    """Read a message from stdin"""
    raw_length = sys.stdin.buffer.read(4)
    if not raw_length:
        sys.exit(0)
    message_length = struct.unpack('@I', raw_length)[0]
    
    # Check message size limit
    if message_length > Config.MAX_MESSAGE_SIZE:
        raise ValueError(f"Message too large: {message_length} bytes")
    
    message = sys.stdin.buffer.read(message_length).decode('utf-8')
    return json.loads(message)

def send_message(message_content: Dict[str, Any]) -> None:
    """Send a message to stdout"""
    encoded_content = json.dumps(message_content).encode('utf-8')
    encoded_length = struct.pack('@I', len(encoded_content))
    sys.stdout.buffer.write(encoded_length)
    sys.stdout.buffer.write(encoded_content)
    sys.stdout.buffer.flush()

# ==============================================================================
# MODEL TYPE DETECTION (Router Helpers)
# ==============================================================================

def is_gguf_or_bitnet(model_path: str) -> bool:
    """Check if model is GGUF or BitNet (Rust handles these)"""
    if not model_path:
        return False
    lower = model_path.lower()
    return (
        ".gguf" in lower or 
        "bitnet" in lower or 
        "llamacpp" in lower or
        "llama" in lower
    )

def is_onnx(model_path: str) -> bool:
    """Check if model is ONNX (Python handles these)"""
    if not model_path:
        return False
    return ".onnx" in model_path.lower()

def is_mediapipe(model_path: str) -> bool:
    """Check if model is MediaPipe (Python handles these)"""
    if not model_path:
        return False
    return "mediapipe" in model_path.lower()

# ==============================================================================
# RUST SERVICES (Required for ALL operations)
# ==============================================================================

# Try to import Rust handler (for GGUF/BitNet inference)
try:
    from tabagent_native_handler import handle_message as rust_handle_message
    RUST_HANDLER_AVAILABLE = True
    logging.info("Rust native handler loaded successfully")
except ImportError as e:
    RUST_HANDLER_AVAILABLE = False
    logging.warning(f"Rust native handler not available: {e}")
    logging.warning("GGUF/BitNet models will use Python fallback (deprecated)")

# Try to import Rust model cache (Required for ALL models)
try:
    from tabagent_model_cache_py import ModelCache
    RUST_MODEL_CACHE = ModelCache(os.path.join(os.path.dirname(__file__), "model_cache"))
    logging.info("Rust model cache loaded successfully")
except ImportError as e:
    RUST_MODEL_CACHE = None
    logging.error(f"Rust model cache not available: {e}")
    logging.error("Model storage will not work! Build model-cache-bindings first.")

# Try to import Rust database (Required for saving chat history)
try:
    from embedded_db import EmbeddedDB
    RUST_DATABASE = EmbeddedDB(os.path.join(os.path.dirname(__file__), "database"))
    logging.info("Rust database loaded successfully")
except ImportError as e:
    RUST_DATABASE = None
    logging.error(f"Rust database not available: {e}")
    logging.error("Chat history will not persist! Build db-bindings first.")

# ==============================================================================
# MODEL STORAGE HELPERS (ALL models go through Rust cache)
# ==============================================================================

def ensure_model_cached(repo_id: str, file_path: str, progress_callback: Optional[Callable] = None) -> bytes:
    """
    Ensure model file is in Rust cache, download if needed.
    
    Args:
        repo_id: HuggingFace repo ID (e.g., "microsoft/phi-2")
        file_path: File path within repo (e.g., "model.onnx")
        progress_callback: Optional progress callback
        
    Returns:
        Model bytes from cache
        
    Raises:
        RuntimeError: If model cache not available or download fails
    """
    if RUST_MODEL_CACHE is None:
        raise RuntimeError("Rust model cache not available - build model-cache-bindings first")
    
    # Check if file exists in cache
    if not RUST_MODEL_CACHE.has_file(repo_id, file_path):
        logging.info(f"Model not in cache, downloading: {repo_id}/{file_path}")
        
        # Download via Rust
        RUST_MODEL_CACHE.download_file(repo_id, file_path, progress_callback)
        logging.info(f"Download complete: {repo_id}/{file_path}")
    
    # Get file bytes from cache
    model_bytes = RUST_MODEL_CACHE.get_file(repo_id, file_path)
    if model_bytes is None:
        raise RuntimeError(f"Failed to get model from cache: {repo_id}/{file_path}")
    
    logging.info(f"Retrieved model from cache: {len(model_bytes)} bytes")
    return model_bytes

def save_message_to_db(chat_id: str, role: str, content: str, **kwargs) -> bool:
    """
    Save message to Rust database.
    
    Args:
        chat_id: Chat ID
        role: Message role (user/assistant/system)
        content: Message content
        **kwargs: Additional metadata
        
    Returns:
        True if successful
    """
    if RUST_DATABASE is None:
        logging.error("Rust database not available - message will not persist")
        return False
    
    try:
        import uuid
        import time
        
        message_node = {
            "id": f"msg_{uuid.uuid4().hex}",
            "type": "Message",
            "chat_id": chat_id,
            "sender": role,
            "text_content": content,
            "timestamp": int(time.time() * 1000),
            **kwargs
        }
        
        node_id = RUST_DATABASE.insert_node(message_node)
        logging.debug(f"Saved message to DB: {node_id}")
        return True
        
    except Exception as e:
        logging.error(f"Failed to save message to DB: {e}")
        return False

def handle_ping(message: Dict[str, Any]) -> Dict[str, Any]:
    """Handle ping message"""
    return {
        "status": "success",
        "response": "pong",
        "version": "1.0.0",
        "pid": os.getpid()
    }

def handle_get_system_info(message: Dict[str, Any]) -> Dict[str, Any]:
    """
    Handle system info request.
    
    Returns comprehensive system information.
    Uses shared system_info_builder (DRY - same code as HTTP API).
    """
    try:
        from hardware.hardware_detection import get_hardware_detector
        from hardware.system_info_builder import build_system_info_dict
        
        # Get verbose flag from message (optional)
        verbose = message.get("verbose", False)
        
        # Get hardware info
        hardware_detector = get_hardware_detector()
        hardware_info = hardware_detector.get_hardware_info()
        
        # Use shared builder (DRY principle - same as HTTP endpoint)
        system_info = build_system_info_dict(hardware_info, verbose)
        
        return {
            "status": "success",
            **system_info
        }
    
    except Exception as e:
        logger.error(f"Error getting system info: {e}", exc_info=True)
        return {
            "status": "error",
            "message": f"Failed to get system info: {str(e)}"
        }

def handle_execute_command(message: Dict[str, Any]) -> Dict[str, Any]:
    """Handle command execution request"""
    import subprocess
    
    command = message.get("command", "")
    if not command:
        return {"status": "error", "message": "No command provided"}
    
    try:
        # Security check: whitelist allowed commands
        if Config.ALLOWED_COMMANDS and command not in Config.ALLOWED_COMMANDS:
            return {"status": "error", "message": "Command not allowed"}
        
        # Log the command execution
        logging.info(f"Executing command: {command}")
        
        # Execute the command and return result
        result = subprocess.run(
            command, 
            shell=True, 
            capture_output=True, 
            text=True,
            timeout=Config.COMMAND_TIMEOUT
        )
        
        return {
            "status": "success",
            "command": command,
            "stdout": result.stdout,
            "stderr": result.stderr,
            "returncode": result.returncode
        }
    except subprocess.TimeoutExpired:
        logging.error(f"Command execution timeout: {command}")
        return {
            "status": "error",
            "message": f"Command timed out after {Config.COMMAND_TIMEOUT} seconds"
        }
    except Exception as e:
        logging.error(f"Command execution error: {str(e)}")
        return {
            "status": "error",
            "message": str(e)
        }


def handle_load_model(message: Dict[str, Any]) -> Dict[str, Any]:
    """
    Handle model loading request from extension.
    
    ALL models go through Rust model-cache first!
    """
    try:
        # Validate request
        request = LoadModelRequest(**message)
        model_path = request.modelPath
        
        # Extract repo_id and file from path
        # Format: "repo_owner/repo_name/model_file.ext"
        # OR absolute path (legacy)
        if "/" in model_path and not os.path.isabs(model_path):
            parts = model_path.split("/")
            if len(parts) >= 3:
                repo_id = f"{parts[0]}/{parts[1]}"
                file_path = "/".join(parts[2:])
            else:
                return {
                    "status": "error",
                    "message": f"Invalid model path format: {model_path}"
                }
        else:
            # Legacy absolute path - not supported anymore
            return {
                "status": "error",
                "message": "Absolute paths not supported - use HuggingFace repo format: owner/repo/file.ext"
            }
        
        # Progress callback for downloads
        def download_progress(loaded: int, total: int):
            send_message({
                "type": EventType.MODEL_LOADING_PROGRESS.value,
                "payload": {
                    "status": LoadingStatus.DOWNLOADING.value,
                    "progress": int((loaded / total) * 100) if total > 0 else 0,
                    "file": file_path,
                    "message": f"Downloading: {loaded}/{total} bytes"
                }
            })
        
        # Get model from Rust cache (downloads if needed)
        logging.info(f"Ensuring model in cache: {repo_id}/{file_path}")
        model_bytes = ensure_model_cached(repo_id, file_path, download_progress)
        
        # Model is now in cache - use inference service to load it
        # But pass the bytes, not the path
        # TODO: Update inference service to accept bytes
        
        return {
            "status": "success",
            "message": f"Model ready: {file_path}",
            "payload": {
                "isReady": True,
                "modelPath": model_path,
                "size": len(model_bytes)
            }
        }
    
    except RuntimeError as e:
        logging.error(f"Model cache error: {e}")
        return {
            "status": "error",
            "message": str(e)
        }
    except Exception as e:
        logging.error(f"Error loading model: {e}")
        return {
            "status": "error",
            "message": f"Failed to load model: {str(e)}"
        }


def handle_generate(message: Dict[str, Any]) -> Dict[str, Any]:
    """
    Handle text generation request from extension.
    
    After generation, saves to Rust database!
    """
    try:
        # Validate request
        request = GenerateRequest(**message)
        
        # Stream callback for native messaging
        def stream_callback(token: str, tps: Optional[str], num_tokens: int):
            send_message({
                "type": EventType.GENERATION_UPDATE.value,
                "payload": {
                    "token": token,
                    "tps": tps,
                    "numTokens": num_tokens
                }
            })
        
        # Use shared service (same logic for HTTP and native)
        result = _inference_service.generate(
            messages=request.messages,
            settings=request.settings,
            stream_callback=stream_callback
        )
        
        # Save to Rust database
        if result.get("status") == "success" and "text" in result:
            chat_id = message.get("chat_id", "default")
            
            # Save user message
            if request.messages:
                last_user_msg = request.messages[-1]
                save_message_to_db(
                    chat_id=chat_id,
                    role=last_user_msg.role,
                    content=last_user_msg.content
                )
            
            # Save assistant response
            save_message_to_db(
                chat_id=chat_id,
                role="assistant",
                content=result["text"]
            )
        
        return result
    
    except Exception as e:
        logging.error(f"Generation error: {e}")
        return {
            "status": "error",
            "type": EventType.GENERATION_ERROR.value,
            "message": str(e)
        }


def handle_update_settings(message: Dict[str, Any]) -> Dict[str, Any]:
    """
    Handle inference settings update (SET_PARAMS).
    
    Updates settings for the currently active backend manager.
    """
    global bitnet_manager
    
    try:
        request = UpdateSettingsRequest(**message)
        
        updated_backends = []
        
        # Try updating via InferenceService (all backends)
        if _inference_service is not None:
            manager = _inference_service.get_active_manager()
            if manager and hasattr(manager, 'update_settings'):
                manager.update_settings(request.settings)
                backend_type = _inference_service.get_backend_type()
                if backend_type:
                    updated_backends.append(backend_type.value)
        
        # Fallback: Update BitNet directly
        if not updated_backends and bitnet_manager is not None:
            if hasattr(bitnet_manager, 'update_settings'):
                bitnet_manager.update_settings(request.settings)
                updated_backends.append("BitNet")
        
        if not updated_backends:
            return {
                "status": "error",
                "message": "No active backend to update settings"
            }
        
        logging.info(f"Inference settings updated: {', '.join(updated_backends)}")
        
        return {
            "status": "success",
            "message": f"Settings updated ({', '.join(updated_backends)})",
            "updated_backends": updated_backends,
            "settings": {
                "temperature": request.settings.temperature,
                "top_p": request.settings.top_p,
                "top_k": request.settings.top_k,
                "max_tokens": request.settings.max_tokens
            }
        }
    
    except Exception as e:
        logging.error(f"Error updating settings: {e}")
        return {
            "status": "error",
            "message": str(e)
        }


def handle_get_model_state(message: Dict[str, Any]) -> Dict[str, Any]:
    """Get current model state"""
    global bitnet_manager
    
    try:
        if bitnet_manager is None:
            return {
                "status": "success",
                "payload": {
                    "isReady": False,
                    "backend": None,
                    "modelPath": None
                }
            }
        
        state = bitnet_manager.get_state()
        
        return {
            "status": "success",
            "payload": state
        }
    
    except Exception as e:
        logging.error(f"Error getting model state: {e}")
        return {
            "status": "error",
            "message": str(e)
        }


def handle_unload_model(message: Dict[str, Any]) -> Dict[str, Any]:
    """Unload current model"""
    global bitnet_manager
    
    try:
        if bitnet_manager is not None and bitnet_manager.is_model_loaded:
            bitnet_manager.unload_model()
            logging.info("Model unloaded")
        
        return {
            "status": "success",
            "message": "Model unloaded"
        }
    
    except Exception as e:
        logging.error(f"Error unloading model: {e}")
        return {
            "status": "error",
            "message": str(e)
        }


def handle_pull_model(message: Dict[str, Any]) -> Dict[str, Any]:
    """
    Download model from HuggingFace via Rust model-cache.
    
    Args:
        message: Contains 'model_name' (repo/file format) or 'model'
        
    Returns:
        Status response
    """
    try:
        if RUST_MODEL_CACHE is None:
            return {
                "status": "error",
                "message": "Rust model cache not available - build model-cache-bindings first"
            }
        
        model_name = message.get("model_name") or message.get("model")
        
        if not model_name:
            return {
                "status": "error",
                "message": "model_name is required (format: owner/repo/file.ext)"
            }
        
        # Parse model path
        parts = model_name.split("/")
        if len(parts) < 3:
            return {
                "status": "error",
                "message": "Invalid format - use: owner/repo/file.ext"
            }
        
        repo_id = f"{parts[0]}/{parts[1]}"
        file_path = "/".join(parts[2:])
        
        logging.info(f"Pull request for model: {repo_id}/{file_path}")
        
        # Progress callback
        def progress(loaded: int, total: int):
            send_message({
                "type": "MODEL_DOWNLOAD_PROGRESS",
                "payload": {
                    "loaded": loaded,
                    "total": total,
                    "progress": int((loaded / total) * 100) if total > 0 else 0
                }
            })
        
        # Download via Rust
        RUST_MODEL_CACHE.download_file(repo_id, file_path, progress)
        
        return {
            "status": "success",
            "message": f"Model {model_name} downloaded successfully",
            "model": model_name
        }
    
    except Exception as e:
        logging.error(f"Pull error: {e}")
        return {
            "status": "error",
            "message": str(e)
        }


def handle_delete_model(message: Dict[str, Any]) -> Dict[str, Any]:
    """
    Delete a downloaded model.
    
    Args:
        message: Contains 'model_name' or 'model'
        
    Returns:
        Status response
    """
    try:
        from models import ModelManager
        
        model_name = message.get("model_name") or message.get("model")
        
        if not model_name:
            return {
                "status": "error",
                "message": "model_name is required"
            }
        
        logging.info(f"Delete request for model: {model_name}")
        
        manager = ModelManager()
        
        # Check if model exists
        if not manager.is_model_downloaded(model_name):
            return {
                "status": "error",
                "message": f"Model not found: {model_name}"
            }
        
        # Delete model
        success = manager.delete_model(model_name)
        
        if success:
            return {
                "status": "success",
                "message": f"Model {model_name} deleted successfully",
                "model": model_name
            }
        else:
            return {
                "status": "error",
                "message": f"Failed to delete model: {model_name}"
            }
    
    except Exception as e:
        logging.error(f"Delete error: {e}")
        return {
            "status": "error",
            "message": str(e)
        }


def handle_stop_generation(message: Dict[str, Any]) -> Dict[str, Any]:
    """
    Stop ongoing generation.
    
    Sets stop flag on all active managers.
    """
    global bitnet_manager
    
    logging.info("Stop generation requested")
    
    stopped_backends = []
    
    try:
        # Try stopping BitNet
        if bitnet_manager is not None and bitnet_manager.is_model_loaded:
            if hasattr(bitnet_manager, 'stop_generation'):
                bitnet_manager.stop_generation()
                stopped_backends.append("BitNet")
        
        # Try stopping via InferenceService
        if _inference_service is not None:
            manager = _inference_service.get_active_manager()
            if manager and hasattr(manager, 'halt_generation'):
                manager.halt_generation()
                backend_type = _inference_service.get_backend_type()
                if backend_type:
                    stopped_backends.append(backend_type.value)
        
        message_text = f"Generation stopped ({', '.join(stopped_backends)})" if stopped_backends else "No active generation to stop"
        
        return {
            "status": "success",
            "message": message_text,
            "stopped_backends": stopped_backends
        }
    
    except Exception as e:
        logging.error(f"Error stopping generation: {e}")
        return {
            "status": "error",
            "message": str(e)
        }


def handle_check_lmstudio(message: Dict[str, Any]) -> Dict[str, Any]:
    """
    Check LM Studio installation and runtime status
    """
    global lmstudio_manager
    
    try:
        # Initialize LM Studio manager if needed
        if lmstudio_manager is None:
            lmstudio_manager = LMStudioManager()
        
        # Get status
        status = lmstudio_manager.get_status()
        
        logging.info(f"LM Studio status: installed={status['installed']}, running={status['server_running']}")
        
        return {
            "status": "success",
            "payload": status
        }
    
    except Exception as e:
        logging.error(f"Error checking LM Studio: {e}")
        return {
            "status": "error",
            "message": str(e)
        }


def handle_start_lmstudio(message: Dict[str, Any]) -> Dict[str, Any]:
    """
    Start LM Studio server
    """
    global lmstudio_manager
    
    try:
        # Initialize LM Studio manager if needed
        if lmstudio_manager is None:
            lmstudio_manager = LMStudioManager()
        
        # Ensure server is running
        success = lmstudio_manager.ensure_server_running()
        
        if success:
            logging.info("LM Studio server started successfully")
            return {
                "status": "success",
                "message": "LM Studio server is running",
                "payload": {
                    "server_running": True,
                    "api_endpoint": f"http://127.0.0.1:1234"
                }
            }
        else:
            return {
                "status": "error",
                "message": "Failed to start LM Studio server"
            }
    
    except RuntimeError as e:
        logging.error(f"Cannot start LM Studio: {e}")
        return {
            "status": "error",
            "message": str(e)
        }
    except Exception as e:
        logging.error(f"Error starting LM Studio: {e}")
        return {
            "status": "error",
            "message": str(e)
        }


def handle_stop_lmstudio(message: Dict[str, Any]) -> Dict[str, Any]:
    """
    Stop LM Studio server
    """
    global lmstudio_manager
    
    try:
        if lmstudio_manager is None:
            return {
                "status": "error",
                "message": "LM Studio manager not initialized"
            }
        
        success = lmstudio_manager.stop_server()
        
        if success:
            logging.info("LM Studio server stopped")
            return {
                "status": "success",
                "message": "LM Studio server stopped"
            }
        else:
            return {
                "status": "error",
                "message": "Failed to stop LM Studio server"
            }
    
    except Exception as e:
        logging.error(f"Error stopping LM Studio: {e}")
        return {
            "status": "error",
            "message": str(e)
        }

# ==============================================================================
# EMBEDDINGS & RAG HANDLERS (Feature Parity with HTTP API)
# ==============================================================================

def handle_generate_embeddings(message: Dict[str, Any]) -> Dict[str, Any]:
    """Generate embeddings for texts"""
    try:
        texts = message.get("texts", [])
        model = message.get("model", "default")
        
        if not texts:
            return {"status": "error", "message": "No texts provided"}
        
        # Use UnifiedRequestHandler (shared logic)
        from core.unified_handler import get_unified_handler
        handler = get_unified_handler()
        
        # Generate embeddings
        import asyncio
        result = asyncio.run(handler.generate_embeddings(texts, model))
        
        return {
            "status": "success",
            "embeddings": result["embeddings"],
            "model": result["model"],
            "total_tokens": len(texts) * 50  # Estimate
        }
    
    except Exception as e:
        logging.error(f"Embeddings error: {e}")
        return {"status": "error", "message": str(e)}


def handle_rerank_documents(message: Dict[str, Any]) -> Dict[str, Any]:
    """Rerank documents by relevance"""
    try:
        query = message.get("query", "")
        documents = message.get("documents", [])
        model = message.get("model", "default")
        top_k = message.get("top_k")
        
        if not query or not documents:
            return {"status": "error", "message": "Query and documents required"}
        
        # Use UnifiedRequestHandler (shared logic)
        from core.unified_handler import get_unified_handler
        handler = get_unified_handler()
        
        # Rerank
        import asyncio
        result = asyncio.run(handler.rerank_documents(query, documents, model, top_k))
        
        return {
            "status": "success",
            "results": result["results"],
            "total_tokens": result.get("total_tokens", 0)
        }
    
    except Exception as e:
        logging.error(f"Reranking error: {e}")
        return {"status": "error", "message": str(e)}


def handle_get_params(message: Dict[str, Any]) -> Dict[str, Any]:
    """Get current generation parameters"""
    try:
        from core.unified_handler import get_unified_handler
        handler = get_unified_handler()
        
        params = handler.get_params()
        
        return {
            "status": "success",
            "params": params
        }
    
    except Exception as e:
        logging.error(f"Get params error: {e}")
        return {"status": "error", "message": str(e)}


def handle_set_params(message: Dict[str, Any]) -> Dict[str, Any]:
    """Set generation parameters"""
    try:
        params = message.get("params", {})
        
        from core.unified_handler import get_unified_handler
        handler = get_unified_handler()
        
        updated_params = handler.set_params(params)
        
        return {
            "status": "success",
            "message": "Parameters updated",
            "params": updated_params
        }
    
    except Exception as e:
        logging.error(f"Set params error: {e}")
        return {"status": "error", "message": str(e)}


def handle_get_recipes(message: Dict[str, Any]) -> Dict[str, Any]:
    """List available recipes"""
    try:
        from core.unified_handler import get_unified_handler
        handler = get_unified_handler()
        
        result = handler.get_recipes()
        result["status"] = "success"
        
        return result
    
    except Exception as e:
        logging.error(f"Get recipes error: {e}")
        return {"status": "error", "message": str(e)}


def handle_get_registered_models(message: Dict[str, Any]) -> Dict[str, Any]:
    """List registered models"""
    try:
        from core.unified_handler import get_unified_handler
        handler = get_unified_handler()
        
        result = handler.get_registered_models()
        result["status"] = "success"
        
        return result
    
    except Exception as e:
        logging.error(f"Get registered models error: {e}")
        return {"status": "error", "message": str(e)}


# ==============================================================================
# RESOURCE MANAGEMENT HANDLERS (For Agentic Systems)
# ==============================================================================

def handle_query_resources(message: Dict[str, Any]) -> Dict[str, Any]:
    """Query available VRAM/RAM resources"""
    try:
        from core.unified_handler import get_unified_handler
        handler = get_unified_handler()
        
        result = handler.query_resources()
        result["status"] = "success"
        
        return result
    
    except Exception as e:
        logging.error(f"Query resources error: {e}")
        return {"status": "error", "message": str(e)}


def handle_estimate_model_size(message: Dict[str, Any]) -> Dict[str, Any]:
    """Estimate model memory requirements and suggest offload strategies"""
    try:
        model_path = message.get("model_path", "")
        
        if not model_path:
            return {"status": "error", "message": "model_path required"}
        
        from core.unified_handler import get_unified_handler
        handler = get_unified_handler()
        
        result = handler.estimate_model_size(model_path)
        
        if "error" in result:
            return {"status": "error", "message": result["error"]}
        
        result["status"] = "success"
        return result
    
    except Exception as e:
        logging.error(f"Estimate model size error: {e}")
        return {"status": "error", "message": str(e)}


def handle_list_loaded_models(message: Dict[str, Any]) -> Dict[str, Any]:
    """List all currently loaded models"""
    try:
        from core.unified_handler import get_unified_handler
        handler = get_unified_handler()
        
        result = handler.list_loaded_models()
        result["status"] = "success"
        
        return result
    
    except Exception as e:
        logging.error(f"List loaded models error: {e}")
        return {"status": "error", "message": str(e)}


def handle_select_active_model(message: Dict[str, Any]) -> Dict[str, Any]:
    """Select which loaded model to use for inference"""
    try:
        model_id = message.get("model_id", "")
        
        if not model_id:
            return {"status": "error", "message": "model_id required"}
        
        from core.unified_handler import get_unified_handler
        handler = get_unified_handler()
        
        result = handler.select_active_model(model_id)
        result["status"] = "success"
        result["message"] = f"Active model set to: {model_id}"
        
        return result
    
    except ValueError as e:
        return {"status": "error", "message": str(e)}
    except Exception as e:
        logging.error(f"Select active model error: {e}")
        return {"status": "error", "message": str(e)}


def main():
    """Main message loop"""
    logging.info("Native host started")
    
    # Initialize LM Studio manager at startup
    global lmstudio_manager
    try:
        lmstudio_manager = LMStudioManager()
        # Auto-start LM Studio server if installed and bootstrapped
        if lmstudio_manager.is_installed and lmstudio_manager.is_bootstrapped:
            logging.info("Attempting to start LM Studio server at startup")
            lmstudio_manager.ensure_server_running()
    except Exception as e:
        logging.warning(f"Failed to initialize LM Studio at startup: {e}")
    
    # Message handlers (strongly typed)
    handlers: Dict[str, Any] = {
        ActionType.PING.value: handle_ping,
        ActionType.GET_SYSTEM_INFO.value: handle_get_system_info,
        ActionType.EXECUTE_COMMAND.value: handle_execute_command,
        
        # Model handlers (generic - routes to appropriate backend)
        ActionType.PULL_MODEL.value: handle_pull_model,
        ActionType.LOAD_MODEL.value: handle_load_model,
        ActionType.GENERATE.value: handle_generate,
        ActionType.GET_MODEL_STATE.value: handle_get_model_state,
        ActionType.UPDATE_SETTINGS.value: handle_update_settings,
        ActionType.UNLOAD_MODEL.value: handle_unload_model,
        ActionType.DELETE_MODEL.value: handle_delete_model,
        ActionType.STOP_GENERATION.value: handle_stop_generation,
        
        # Embeddings & RAG handlers (Feature Parity with HTTP API)
        ActionType.GENERATE_EMBEDDINGS.value: handle_generate_embeddings,
        ActionType.RERANK_DOCUMENTS.value: handle_rerank_documents,
        ActionType.GET_PARAMS.value: handle_get_params,
        ActionType.SET_PARAMS.value: handle_set_params,
        ActionType.GET_RECIPES.value: handle_get_recipes,
        ActionType.GET_REGISTERED_MODELS.value: handle_get_registered_models,
        
        # Resource management (For agentic systems)
        ActionType.QUERY_RESOURCES.value: handle_query_resources,
        ActionType.ESTIMATE_MODEL_SIZE.value: handle_estimate_model_size,
        ActionType.LIST_LOADED_MODELS.value: handle_list_loaded_models,
        ActionType.SELECT_ACTIVE_MODEL.value: handle_select_active_model,
        
        # LM Studio lifecycle handlers
        ActionType.CHECK_LMSTUDIO.value: handle_check_lmstudio,
        ActionType.START_LMSTUDIO.value: handle_start_lmstudio,
        ActionType.STOP_LMSTUDIO.value: handle_stop_lmstudio,
    }
    
    while True:
        try:
            message = get_message()
            logging.debug(f"Received message: {message}")
            
            # Get the action and model info
            action = message.get("action", "")
            model_path = message.get("modelPath") or message.get("model") or ""
            
            # ============================================================
            # ROUTING: Determine if Rust or Python should handle this
            # ============================================================
            
            # Check if this is a model-related action that needs routing
            model_actions = {
                ActionType.LOAD_MODEL.value,
                ActionType.GENERATE.value,
                ActionType.UNLOAD_MODEL.value,
                ActionType.GET_MODEL_STATE.value,
                ActionType.PULL_MODEL.value,
                ActionType.DELETE_MODEL.value,
                ActionType.UPDATE_SETTINGS.value,
                ActionType.STOP_GENERATION.value,
            }
            
            if action in model_actions and is_gguf_or_bitnet(model_path):
                # ===== RUST HANDLER (GGUF/BitNet) =====
                if RUST_HANDLER_AVAILABLE:
                    logging.info(f"Routing {action} for GGUF/BitNet to Rust handler")
                    try:
                        # Call Rust handler - it MUST return a response
                        response_json = rust_handle_message(json.dumps(message))
                        response = json.loads(response_json)
                        logging.debug(f"Rust handler response: {response}")
                    except Exception as e:
                        logging.error(f"Rust handler error: {e}", exc_info=True)
                        response = {
                            "status": "error",
                            "message": f"Rust handler error: {str(e)}"
                        }
                else:
                    # Rust not available - log warning and use Python fallback
                    logging.warning(f"Rust handler not available for {model_path}, using Python fallback")
                    if action in handlers:
                        response = handlers[action](message)
                    else:
                        response = {
                            "status": "error",
                            "message": f"Rust handler unavailable and no Python fallback for: {action}"
                        }
            
            elif action in handlers:
                # ===== PYTHON HANDLER (ONNX/MediaPipe/Other) =====
                logging.debug(f"Handling {action} with Python handler")
                response = handlers[action](message)
            
            else:
                # ===== UNKNOWN ACTION =====
                response = {
                    "status": "error",
                    "message": f"Unknown action: {action}"
                }
            
            send_message(response)
            
        except json.JSONDecodeError as e:
            error_response = {
                "status": "error",
                "message": f"Invalid JSON: {str(e)}"
            }
            send_message(error_response)
            sys.exit(1)
        except struct.error as e:
            error_response = {
                "status": "error",
                "message": f"Message format error: {str(e)}"
            }
            send_message(error_response)
            sys.exit(1)
        except ValueError as e:
            error_response = {
                "status": "error",
                "message": f"Message size error: {str(e)}"
            }
            send_message(error_response)
            sys.exit(1)
        except KeyboardInterrupt:
            logging.info("Native host interrupted")
            sys.exit(0)
        except Exception as e:
            logging.error(f"Unexpected error: {str(e)}")
            error_response = {
                "status": "error",
                "message": f"Unexpected error: {str(e)}"
            }
            send_message(error_response)
            sys.exit(1)

if __name__ == '__main__':
    main()