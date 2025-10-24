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
from Python.core.message_types import (
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
from Python.core.inference_service import get_inference_service

# Import backend implementations
from Python.backends.lmstudio import LMStudioManager

# Import specialized pipelines (NEW - compose model-cache + model-loader)
from Python.backends.pipelines import create_pipeline, BasePipeline

# Import HuggingFace authentication
from Python.core.hf_auth import (
    get_hf_token,
    set_hf_token,
    clear_hf_token,
    has_hf_token,
    is_auth_error,
    create_auth_required_response
)

# Configuration defaults
class Config:
    LOG_LEVEL = "DEBUG"
    LOG_FILE = "native_host.log"
    ALLOWED_COMMANDS = []
    
    # Backend Migration Flags (Python → Rust transition control)
    # Set to True when migrating inference to Rust
    ONNX_USE_RUST = False      # ONNX: Currently Python (onnxruntime), will migrate to Rust
    LITERT_USE_RUST = False    # LiteRT: Currently Python (mediapipe), will migrate to Rust
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
# BitNet manager removed - now handled via Rust native-handler
lmstudio_manager: Optional[LMStudioManager] = None

# Pipeline registry (NEW - replaces direct model loading)
_loaded_pipelines: Dict[str, BasePipeline] = {}

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
    from tabagent_native_handler import (
        handle_message as rust_handle_message,
        detect_model_py,
        get_model_manifest_py,
        recommend_variant_py
    )
    RUST_HANDLER_AVAILABLE = True
    RUST_UNIFIED_API_AVAILABLE = True
    logging.info("Rust native handler loaded successfully (including unified API)")
except ImportError as e:
    RUST_HANDLER_AVAILABLE = False
    RUST_UNIFIED_API_AVAILABLE = False
    logging.error(f"Rust native handler not available: {e}")
    logging.error("GGUF/BitNet models will FAIL without Rust handler!")
    logging.error("Install: pip install -e Server/Rust/native-handler")
    logging.error("The old Python subprocess approach (llama-server.exe) is deprecated and removed.")

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


# ==============================================================================
# UNIFIED MODEL LOADING API
# ==============================================================================

def load_transformers_python(
    model_path: str,
    task: str,
    auth_token: Optional[str] = None
) -> Dict[str, Any]:
    """
    Load SafeTensors/PyTorch model using HuggingFace transformers library.
    
    Args:
        model_path: HuggingFace model ID or local path
        task: Task type (text-generation, feature-extraction, etc.)
        auth_token: Optional HuggingFace API token
        
    Returns:
        Dict with status and model info
    """
    try:
        from backends.transformers_backend import (
            TransformersTextGenBackend,
            TransformersEmbeddingBackend
        )
        
        logging.info(f"[Transformers] Loading model: {model_path} (task: {task})")
        
        # Select appropriate backend based on task
        if task in ["text-generation", "text2text-generation"]:
            backend = TransformersTextGenBackend()
        elif task == "feature-extraction":
            backend = TransformersEmbeddingBackend()
        else:
            # Default to text generation
            logging.warning(f"[Transformers] Unknown task '{task}', defaulting to text-generation")
            backend = TransformersTextGenBackend()
        
        # Load model
        success = backend.load_model(
            model_path=model_path,
            task=task,
            trust_remote_code=True  # Allow custom model code
        )
        
        if success:
            logging.info(f"[Transformers] Model loaded successfully: {model_path}")
            return {
                "status": "success",
                "backend": "python-transformers",
                "message": f"Transformers model loaded: {model_path}",
                "modelType": "SafeTensors",
                "task": task,
                "source": model_path,
                "device": backend.device if hasattr(backend, 'device') else "unknown"
            }
        else:
            return {
                "status": "error",
                "message": "Failed to load transformers model"
            }
    
    except Exception as e:
        logging.error(f"[Transformers] Error loading model: {e}", exc_info=True)
        return {
            "status": "error",
            "message": f"Transformers backend error: {str(e)}"
        }


def load_via_pipeline(
    source: str,
    detected_task: Optional[str],
    auth_token: Optional[str],
    model_info: Dict[str, Any]
) -> Dict[str, Any]:
    """
    Load model using specialized pipeline architecture.
    
    This replaces direct transformers loading with pipeline-based approach:
    1. Create specialized pipeline based on task
    2. Pipeline delegates to model-cache (download) + transformers (load)
    3. Store pipeline instance for later generation calls
    
    Args:
        source: Model ID or path
        detected_task: Task type from Rust detection (e.g., "image-to-text")
        auth_token: HuggingFace auth token
        model_info: Full model info from Rust detection
        
    Returns:
        Load result dict
    """
    global _loaded_pipelines
    
    try:
        # Get pipeline type (normalize to pipeline format)
        pipeline_type = detected_task or "text-generation"
        architecture = model_info.get("architecture")
        
        logging.info(f"[Pipeline] Creating pipeline for task: {pipeline_type}, architecture: {architecture}")
        
        # Create specialized pipeline (architecture takes priority over task)
        pipeline = create_pipeline(pipeline_type, architecture=architecture)
        if not pipeline:
            return {
                "status": "error",
                "message": f"Unsupported pipeline type: {pipeline_type}, architecture: {architecture}"
            }
        
        # Load model via pipeline - include model_info for format routing
        load_options = {
            "model_info": model_info,  # ← Pipeline uses this to route to correct backend!
            "device": "cuda" if model_info.get("has_cuda") else "cpu",
            "auth_token": auth_token,
            "trust_remote_code": True  # For specialized models like Florence2
        }
        
        result = pipeline.load(source, load_options)
        
        if result.get("status") == "success":
            # Store pipeline for generation
            _loaded_pipelines[source] = pipeline
            logging.info(f"[Pipeline] Model loaded successfully: {source}")
            
            # Add model info to result
            result.update({
                "backend": "python-pipeline",
                "modelType": model_info.get("model_type"),
                "task": pipeline_type,
                "source": source
            })
        
        return result
        
    except Exception as e:
        logging.error(f"[Pipeline] Load failed: {e}")
        return {
            "status": "error",
            "message": f"Pipeline load error: {str(e)}"
        }


def generate_via_pipeline(
    source: str,
    input_data: Dict[str, Any]
) -> Dict[str, Any]:
    """
    Generate using loaded pipeline.
    
    Args:
        source: Model ID (used as pipeline key)
        input_data: Input parameters (text, image, audio, etc.)
        
    Returns:
        Generation result
    """
    global _loaded_pipelines
    
    pipeline = _loaded_pipelines.get(source)
    if not pipeline:
        return {
            "status": "error",
            "message": f"Model not loaded: {source}. Call load_model first."
        }
    
    try:
        result = pipeline.generate(input_data)
        return result
    except Exception as e:
        logging.error(f"[Pipeline] Generate failed: {e}")
        return {
            "status": "error",
            "message": f"Pipeline generate error: {str(e)}"
        }


def load_model_unified(
    source: str,
    variant: Optional[str] = None,
    auth_token: Optional[str] = None,
    save_token: bool = True
) -> Dict[str, Any]:
    """
    Unified model loading entry point that automatically detects model type
    and routes to the correct backend (Rust or Python).
    
    This function leverages the Rust unified detection API to:
    1. Detect model type from file path or repo name
    2. Fetch available quantizations (for ONNX models)
    3. Recommend optimal variant based on hardware
    4. Route to appropriate backend
    5. Handle HuggingFace authentication flow
    
    Args:
        source: File path or HuggingFace repo ID (e.g., "microsoft/Phi-3-mini-4k-instruct-onnx")
        variant: Optional specific variant/quantization to load (e.g., "onnx/model_q4f16.onnx")
        auth_token: Optional HuggingFace API token for private repos
        save_token: Whether to save the token for future use (default: True)
    
    Returns:
        Dict with status, backend used, and model info
        If authentication required: {"status": "auth_required", "provider": "huggingface", ...}
    
    Examples:
        # Auto-detect and load best variant
        result = load_model_unified("microsoft/Phi-3-mini-4k-instruct-onnx")
        
        # Load with authentication
        result = load_model_unified(
            "google/gemma-2b-it",
            auth_token="hf_xxxxxxxxxxxxx"
        )
        
        # Handle auth_required response
        result = load_model_unified("google/gemma-2b-it")
        if result["status"] == "auth_required":
            # Extension will show HuggingFaceLoginDialog
            # Then retry with token
            token = get_token_from_user()
            result = load_model_unified("google/gemma-2b-it", auth_token=token)
        
        # Load GGUF model (auto-routes to Rust)
        result = load_model_unified("Qwen/Qwen2.5-3B-GGUF")
    """
    if not RUST_UNIFIED_API_AVAILABLE:
        return {
            "status": "error",
            "message": "Unified API not available - Rust handler not installed"
        }
    
    try:
        # Step 0: Handle authentication
        # If token provided, save it
        if auth_token and save_token:
            if set_hf_token(auth_token):
                logging.info("[Unified API] HuggingFace token saved successfully")
            else:
                logging.warning("[Unified API] Failed to save HuggingFace token")
        
        # If no token provided, try to get stored token
        if not auth_token:
            auth_token = get_hf_token()
            if auth_token:
                logging.debug("[Unified API] Using stored HuggingFace token")
        
        # Step 1: Detect model type (with comprehensive task detection)
        logging.info(f"[Unified API] Detecting model type and task for: {source}")
        model_info_json = detect_model_py(source, auth_token)
        model_info = json.loads(model_info_json)
        
        model_type = model_info.get("model_type")
        backend = model_info.get("backend", {})
        detected_task = model_info.get("task")
        logging.info(f"[Unified API] Detected - type: {model_type}, backend: {backend}, task: {detected_task}")
        
        # Step 2: For ONNX models, get manifest and select variant
        if model_type == "ONNX" and "/" in source and not source.endswith(".onnx"):
            # Source is a repo ID, not a file path
            logging.info(f"[Unified API] Fetching ONNX manifest for: {source}")
            
            try:
                manifest_json = get_model_manifest_py(source, auth_token, None)
                manifest = json.loads(manifest_json)
            except Exception as manifest_error:
                # Check if it's an authentication error
                error_str = str(manifest_error)
                if is_auth_error(error_str):
                    logging.warning(f"[Unified API] Authentication required for: {source}")
                    return create_auth_required_response(source, error_str)
                else:
                    # Re-raise if not auth error
                    raise
            
            if not variant:
                # Auto-select best variant
                logging.info(f"[Unified API] Recommending variant for: {source}")
                variant = recommend_variant_py(source, 16.0, 0.0)  # Placeholder RAM/VRAM
                logging.info(f"[Unified API] Recommended variant: {variant}")
            
            # Update source to include variant
            source = f"{source}/{variant}" if not source.endswith(variant) else source
        
        # Step 3: Route based on backend.engine (DRY - Rust decides routing!)
        if backend.get("Rust"):
            # Rust-based backends: GGUF, BitNet (llama.cpp, bitnet.dll)
            engine = backend["Rust"]["engine"]
            
            if not RUST_HANDLER_AVAILABLE:
                return {
                    "status": "error",
                    "message": f"{model_type} model requires Rust handler (not available)"
                }
            
            logging.info(f"[Unified API] Routing to Rust ({engine}): {source}")
            rust_message = {
                "action": "load_model",
                "modelPath": source
            }
            result = rust_handle_message(json.dumps(rust_message))
            return json.loads(result)
        
        elif backend.get("Python"):
            # Python-based backends - Route through pipeline architecture
            engine = backend["Python"]["engine"]
            logging.info(f"[Unified API] Routing to Python pipeline ({engine}): {source}")
            
            if engine == "transformers":
                # SafeTensors/PyTorch models - Use specialized pipeline
                return load_via_pipeline(source, detected_task, auth_token, model_info)
            
            elif engine == "onnxruntime":
                # ONNX models - check migration flag
                if Config.ONNX_USE_RUST:
                    # Future: Rust ONNX Runtime
                    logging.info("[Unified API] Using Rust ONNX Runtime (migration enabled)")
                    return {"status": "error", "message": "Rust ONNX Runtime not yet implemented"}
                else:
                    # Current: Extension handles ONNX via transformers.js
                    # For server-side ONNX, we would also use pipeline
                    logging.info("[Unified API] ONNX delegated to extension (transformers.js)")
                    return {
                        "status": "success",
                        "backend": "extension-transformersjs",
                        "message": f"ONNX model ready: {source}",
                        "modelType": model_type,
                        "task": detected_task,
                        "source": source,
                        "variant": variant
                    }
            
            elif engine == "mediapipe":
                # LiteRT models - check migration flag
                if Config.LITERT_USE_RUST:
                    # Future: Rust MediaPipe
                    logging.info("[Unified API] Using Rust MediaPipe (migration enabled)")
                    return {"status": "error", "message": "Rust MediaPipe not yet implemented"}
                else:
                    # Current: Python MediaPipe
                    logging.info("[Unified API] Using Python MediaPipe")
                    return {
                        "status": "success",
                        "backend": "python-mediapipe",
                        "message": f"LiteRT model ready: {source}",
                        "modelType": model_type,
                        "task": detected_task,
                        "source": source
                    }
            
            else:
                return {
                    "status": "error",
                    "message": f"Unknown Python engine: {engine}"
                }
        
        else:
            return {
                "status": "error",
                "message": f"Invalid backend configuration: {backend}"
            }
    
    except Exception as e:
        logging.error(f"[Unified API] Error: {e}")
        return {
            "status": "error",
            "message": f"Unified API error: {str(e)}"
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
        from Python.models import ModelManager
        
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


# ==============================================================================
# HUGGINGFACE AUTHENTICATION HANDLERS
# ==============================================================================

def handle_set_hf_token(message: Dict[str, Any]) -> Dict[str, Any]:
    """
    Store HuggingFace API token securely.
    
    Args:
        message: {"token": "hf_xxxxx"}
    
    Returns:
        Success/error response
    """
    try:
        token = message.get("token")
        if not token:
            return {"status": "error", "message": "Token is required"}
        
        if set_hf_token(token):
            logging.info("[HF Auth] Token stored successfully")
            return {
                "status": "success",
                "message": "HuggingFace token stored securely"
            }
        else:
            return {
                "status": "error",
                "message": "Failed to store token"
            }
    
    except Exception as e:
        logging.error(f"[HF Auth] Error storing token: {e}")
        return {"status": "error", "message": str(e)}


def handle_get_hf_token_status(message: Dict[str, Any]) -> Dict[str, Any]:
    """
    Check if HuggingFace token is stored.
    
    Returns:
        {"status": "success", "hasToken": true/false}
    """
    try:
        has_token = has_hf_token()
        return {
            "status": "success",
            "hasToken": has_token,
            "message": "Token is stored" if has_token else "No token stored"
        }
    
    except Exception as e:
        logging.error(f"[HF Auth] Error checking token: {e}")
        return {"status": "error", "message": str(e)}


def handle_clear_hf_token(message: Dict[str, Any]) -> Dict[str, Any]:
    """
    Remove stored HuggingFace token.
    
    Returns:
        Success/error response
    """
    try:
        if clear_hf_token():
            logging.info("[HF Auth] Token cleared successfully")
            return {
                "status": "success",
                "message": "HuggingFace token removed"
            }
        else:
            return {
                "status": "error",
                "message": "Failed to clear token"
            }
    
    except Exception as e:
        logging.error(f"[HF Auth] Error clearing token: {e}")
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
        
        # HuggingFace authentication handlers
        ActionType.SET_HF_TOKEN.value: handle_set_hf_token,
        ActionType.GET_HF_TOKEN_STATUS.value: handle_get_hf_token_status,
        ActionType.CLEAR_HF_TOKEN.value: handle_clear_hf_token,
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
                # ===== RUST HANDLER (GGUF/BitNet) - MANDATORY =====
                if RUST_HANDLER_AVAILABLE:
                    logging.info(f"Routing {action} for GGUF/BitNet to Rust handler")
                    try:
                        # Call Rust handler - uses direct FFI to llama.dll (10-50x faster than subprocess)
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
                    # RUST IS REQUIRED - No Python fallback for GGUF/BitNet!
                    # The old Python subprocess approach (llama-server.exe) is deprecated and 10-50x slower.
                    # Rust uses direct FFI to llama.dll for optimal performance.
                    logging.error(f"Rust handler REQUIRED for GGUF/BitNet model: {model_path}")
                    logging.error("Install with: pip install -e Server/Rust/native-handler")
                    response = {
                        "status": "error",
                        "message": (
                            f"Rust native handler is REQUIRED for GGUF/BitNet models.\n"
                            f"The old Python subprocess approach is deprecated.\n\n"
                            f"Install the Rust handler:\n"
                            f"  1. cd Server/Rust/native-handler\n"
                            f"  2. pip install -e .\n\n"
                            f"Model: {model_path}"
                        )
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