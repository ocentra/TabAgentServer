"""
BitNet inference backend manager.

Manages llama-server-bitnet subprocess for BitNet 1.58 models.
"""

import logging
import platform
import requests
from pathlib import Path
from typing import List, Optional, Dict, Any

from core.message_types import (
    ChatMessage,
    InferenceSettings,
    BackendType,
    AccelerationBackend,
)

from server_mgmt import (
    WrappedServer,
    ServerConfig,
    HealthCheckMethod,
)

from .config import (
    BitNetConfig,
    BitNetBackend,
    BitNetBinaryName,
)

from .validator import (
    detect_model_type,
    detect_bitnet_quant,
    is_bitnet_model,
    ModelType,
    BitNetQuant,
)

from core.performance_tracker import PerformanceTracker


logger = logging.getLogger(__name__)


class BitNetManager:
    """
    BitNet inference backend manager.
    
    Manages llama-server-bitnet subprocess for BitNet 1.58 models with
    optimized inference (2-6x faster than standard GGUF).
    """
    
    def __init__(self):
        """Initialize BitNet manager"""
        self.server: Optional[WrappedServer] = None
        self.config: Optional[BitNetConfig] = None
        self.model_path: Optional[Path] = None
        self.backend_type: Optional[BackendType] = None
        self._performance_tracker = PerformanceTracker()
        
        logger.info("BitNetManager initialized")
    
    def is_model_loaded(self) -> bool:
        """
        Check if a model is currently loaded.
        
        Returns:
            True if model is loaded and server is running
        """
        return (
            self.server is not None and
            self.server.is_running() and
            self.server.health_check()
        )
    
    def load_model(
        self,
        model_path: str,
        config: BitNetConfig
    ) -> bool:
        """
        Load BitNet model with llama-server-bitnet.
        
        Args:
            model_path: Path to BitNet 1.58 GGUF model
            config: BitNet configuration
            
        Returns:
            True if model loaded successfully
            
        Raises:
            FileNotFoundError: If model or binary not found
            ValueError: If model is not a BitNet 1.58 model
        """
        path = Path(model_path)
        if not path.exists():
            raise FileNotFoundError(f"Model file not found: {model_path}")
        
        # Validate that this is a BitNet model
        model_type = detect_model_type(model_path)
        if model_type != ModelType.BITNET_158:
            raise ValueError(
                f"Model is not a BitNet 1.58 model: {path.name}\n"
                f"Detected type: {model_type.value}\n"
                f"Use LlamaCppManager for regular GGUF models"
            )
        
        # Detect quant type
        quant_type = detect_bitnet_quant(model_path)
        logger.info(
            f"Loading BitNet 1.58 model: {path.name} "
            f"(quant: {quant_type.value if quant_type else 'unknown'})"
        )
        
        # Get binary path from BitNet/Release/
        binary_path = self._get_binary_path()
        
        if not binary_path.exists():
            raise FileNotFoundError(
                f"BitNet binary not found: {binary_path}\n"
                f"Expected: llama-server-bitnet\n"
                f"Build with: cd BitNet && ./build-all-{platform.system().lower()}.sh\n"
                f"Or check: BitNet/Release/cpu/{platform.system().lower()}/"
            )
        
        logger.info(
            f"Loading model with BitNet: {model_path} "
            f"(backend: {config.backend.value}, ngl: {config.ngl})"
        )
        
        # Build command arguments
        args = self._build_server_args(model_path, config)
        
        # Create server configuration
        server_config = ServerConfig(
            executable=str(binary_path),
            args=args,
            port=config.port,
            health_check_url=f"http://{config.host}:{config.port}/health",
            health_check_method=HealthCheckMethod.HTTP_GET,
            startup_timeout=60,
            health_check_interval=1.0,
            graceful_shutdown_timeout=5
        )
        
        # Start server
        try:
            self.server = WrappedServer(server_config)
            success = self.server.start()
            
            if success:
                self.config = config
                self.model_path = path
                self.backend_type = self._map_backend_type(config.backend)
                logger.info(f"llama-server-bitnet ready on port {config.port}")
                return True
            else:
                logger.error("llama-server-bitnet failed to start")
                self.server = None
                return False
                
        except Exception as e:
            logger.error(f"Failed to start llama-server-bitnet: {e}")
            self.server = None
            return False
    
    def unload_model(self) -> bool:
        """
        Unload model and stop server.
        
        Returns:
            True if unloaded successfully
        """
        if self.server is None:
            logger.info("No server running, nothing to unload")
            return True
        
        try:
            self.server.stop()
            self.server = None
            self.config = None
            self.model_path = None
            self.backend_type = None
            
            logger.info("llama-server-bitnet stopped and model unloaded")
            return True
            
        except Exception as e:
            logger.error(f"Error stopping llama-server-bitnet: {e}")
            return False
    
    async def generate(
        self,
        messages: List[ChatMessage],
        settings: InferenceSettings
    ) -> str:
        """
        Generate text using llama-server-bitnet.
        
        Args:
            messages: Chat messages
            settings: Inference settings
            
        Returns:
            Generated text
            
        Raises:
            RuntimeError: If no model loaded
        """
        if not self.is_model_loaded():
            raise RuntimeError("No model loaded or server not ready")
        
        # Build request payload
        payload = {
            "messages": [
                {"role": msg.role.value, "content": msg.content}
                for msg in messages
            ],
            "temperature": settings.temperature,
            "top_p": settings.top_p,
            "top_k": settings.top_k,
            "max_tokens": settings.max_new_tokens,
            "repeat_penalty": settings.repetition_penalty,
            "stream": False,
        }
        
        # Send request to llama-server-bitnet
        try:
            url = f"http://{self.config.host}:{self.config.port}/v1/chat/completions"
            response = requests.post(
                url,
                json=payload,
                timeout=self.config.timeout
            )
            
            response.raise_for_status()
            data = response.json()
            
            # Extract generated text
            if "choices" in data and len(data["choices"]) > 0:
                generated = data["choices"][0]["message"]["content"]
                logger.info(f"Generated {len(generated)} characters")
                return generated
            else:
                logger.error(f"Unexpected response format: {data}")
                return ""
                
        except requests.exceptions.RequestException as e:
            logger.error(f"Generation request failed: {e}")
            raise RuntimeError(f"Generation failed: {e}")
    
    async def generate_stream(
        self,
        messages: List[ChatMessage],
        settings: InferenceSettings
    ):
        """
        Generate streaming text using llama-server-bitnet.
        
        Args:
            messages: Chat messages
            settings: Inference settings
            
        Yields:
            Generated tokens as they are produced
            
        Raises:
            RuntimeError: If no model loaded
        """
        if not self.is_model_loaded():
            raise RuntimeError("No model loaded or server not ready")
        
        # Build request payload with streaming enabled
        payload = {
            "messages": [
                {"role": msg.role.value, "content": msg.content}
                for msg in messages
            ],
            "temperature": settings.temperature,
            "top_p": settings.top_p,
            "top_k": settings.top_k,
            "max_tokens": settings.max_new_tokens,
            "repeat_penalty": settings.repetition_penalty,
            "stream": True,
        }
        
        # Stream from llama-server-bitnet
        try:
            url = f"http://{self.config.host}:{self.config.port}/v1/chat/completions"
            
            response = requests.post(
                url,
                json=payload,
                stream=True,
                timeout=self.config.timeout
            )
            
            response.raise_for_status()
            
            # Process SSE stream
            for line in response.iter_lines():
                if not line:
                    continue
                
                line_str = line.decode('utf-8')
                
                # Skip empty lines and comments
                if not line_str.strip() or line_str.startswith(':'):
                    continue
                
                # Parse SSE data
                if line_str.startswith('data: '):
                    data_str = line_str[6:]  # Remove 'data: ' prefix
                    
                    # Check for [DONE] marker
                    if data_str.strip() == '[DONE]':
                        break
                    
                    try:
                        import json
                        data = json.loads(data_str)
                        
                        # Extract delta content
                        if "choices" in data and len(data["choices"]) > 0:
                            delta = data["choices"][0].get("delta", {})
                            content = delta.get("content")
                            
                            if content:
                                yield content
                    
                    except json.JSONDecodeError:
                        logger.warning(f"Could not parse SSE data: {data_str}")
                        continue
            
            logger.info("Streaming generation complete")
                
        except requests.exceptions.RequestException as e:
            logger.error(f"Streaming generation failed: {e}")
            raise RuntimeError(f"Streaming failed: {e}")
    
    async def generate_embeddings(self, texts: List[str]) -> List[List[float]]:
        """
        Generate embeddings using llama-server-bitnet /embeddings endpoint.
        
        Args:
            texts: List of text strings
            
        Returns:
            List of embedding vectors
            
        Raises:
            RuntimeError: If model not loaded or embeddings not supported
        """
        if not self.is_model_loaded():
            raise RuntimeError("No model loaded or server not ready")
        
        try:
            embeddings = []
            
            # llama-server supports /embeddings endpoint
            url = f"http://{self.config.host}:{self.config.port}/v1/embeddings"
            
            for text in texts:
                payload = {
                    "input": text,
                    "encoding_format": "float"
                }
                
                response = requests.post(
                    url,
                    json=payload,
                    timeout=self.config.timeout
                )
                
                response.raise_for_status()
                data = response.json()
                
                # Extract embedding from response
                if "data" in data and len(data["data"]) > 0:
                    embedding = data["data"][0]["embedding"]
                    embeddings.append(embedding)
                else:
                    logger.warning(f"No embedding in response for text: {text[:50]}...")
                    embeddings.append([])
            
            logger.info(f"Generated {len(embeddings)} embeddings via llama-server-bitnet")
            return embeddings
        
        except requests.exceptions.RequestException as e:
            logger.error(f"Embedding generation failed: {e}")
            raise RuntimeError(f"Failed to generate embeddings: {e}")
    
    def get_model_info(self) -> Dict[str, Any]:
        """
        Get information about loaded model.
        
        Returns:
            Dictionary with model information
        """
        if not self.is_model_loaded():
            return {
                "loaded": False,
                "error": "No model loaded"
            }
        
        quant_type = detect_bitnet_quant(str(self.model_path)) if self.model_path else None
        
        return {
            "loaded": True,
            "model_path": str(self.model_path),
            "model_type": "BitNet 1.58",
            "quant_type": quant_type.value if quant_type else "unknown",
            "backend": self.backend_type.value if self.backend_type else None,
            "acceleration": self.config.backend.value if self.config else None,
            "ngl": self.config.ngl if self.config else 0,
            "context_size": self.config.context_size if self.config else 0,
            "port": self.config.port if self.config else 0,
        }
    
    def get_state(self) -> Dict[str, Any]:
        """
        Get current manager state.
        
        Returns:
            Dictionary with state information including performance metrics
        """
        state = {
            "isReady": self.server is not None and self.server.is_running(),
            "backend": self.backend_type.value if self.backend_type else None,
            "modelPath": str(self.model_path) if self.model_path else None
        }
        
        # Add performance metrics
        stats = self._performance_tracker.get_current_stats()
        state.update(stats)
        
        return state
    
    @staticmethod
    def _get_binary_path() -> Path:
        """
        Get path to llama-server-bitnet binary from BitNet/Release/.
        
        Returns:
            Path to binary
        """
        # Determine platform
        system_name = platform.system().lower()
        
        # Binary name
        if system_name == "windows":
            binary_name = "llama-server-bitnet.exe"
        else:
            binary_name = "llama-server-bitnet"
        
        # Path to BitNet/Release/cpu/{platform}/
        # backends/bitnet/ -> backends/ -> Server/ -> BitNet/Release/
        backend_dir = Path(__file__).parent.parent.parent
        binary_path = backend_dir / "BitNet" / "Release" / "cpu" / system_name / binary_name
        
        return binary_path
    
    @staticmethod
    def _build_server_args(model_path: str, config: BitNetConfig) -> List[str]:
        """
        Build command-line arguments for llama-server-bitnet.
        
        Args:
            model_path: Path to model file
            config: BitNet configuration
            
        Returns:
            List of command arguments
        """
        args = [
            "--model", model_path,
            "--port", str(config.port),
            "--host", config.host,
            "-c", str(config.context_size),
            "-b", str(config.n_batch),
        ]
        
        # GPU layers (if CUDA backend)
        if config.ngl > 0:
            args.extend(["-ngl", str(config.ngl)])
        
        # CPU threads
        if config.n_threads:
            args.extend(["-t", str(config.n_threads)])
        
        return args
    
    @staticmethod
    def _map_backend_type(backend: BitNetBackend) -> BackendType:
        """
        Map BitNet backend to BackendType.
        
        Args:
            backend: BitNet backend
            
        Returns:
            BackendType enum value
        """
        mapping = {
            BitNetBackend.CPU_TL1: BackendType.BITNET_CPU,
            BitNetBackend.CPU_TL2: BackendType.BITNET_CPU,
            BitNetBackend.GPU_CUDA: BackendType.BITNET_GPU,
        }
        
        return mapping.get(backend, BackendType.BITNET_CPU)

