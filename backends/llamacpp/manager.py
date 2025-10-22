"""
llama.cpp inference backend manager.

Manages llama-server subprocess with multiple acceleration backends.
"""

import logging
import requests
from pathlib import Path
from typing import List, Optional, Dict, Any
from enum import Enum

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
    LlamaCppConfig,
    LlamaCppBackend,
    LlamaCppBinaryName,
)
from core.performance_tracker import PerformanceTracker


logger = logging.getLogger(__name__)


class MessageFormat(str, Enum):
    """Chat message format for llama-server"""
    CHATML = "chatml"
    LLAMA2 = "llama2"
    LLAMA3 = "llama3"
    MISTRAL = "mistral"


class LlamaCppManager:
    """
    llama.cpp inference backend manager.
    
    Manages llama-server subprocess with automatic backend selection
    and VRAM-aware configuration.
    """
    
    def __init__(self):
        """Initialize llama.cpp manager"""
        self.server: Optional[WrappedServer] = None
        self.config: Optional[LlamaCppConfig] = None
        self.model_path: Optional[Path] = None
        self.backend_type: Optional[BackendType] = None
        self._performance_tracker = PerformanceTracker()
        
        logger.info("LlamaCppManager initialized")
    
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
        config: LlamaCppConfig
    ) -> bool:
        """
        Load model with llama-server.
        
        Args:
            model_path: Path to GGUF model file
            config: llama.cpp configuration
            
        Returns:
            True if model loaded successfully
            
        Raises:
            FileNotFoundError: If model or binary not found
        """
        path = Path(model_path)
        if not path.exists():
            raise FileNotFoundError(f"Model file not found: {model_path}")
        
        # Get optimal llama-server from BitNet/BitnetRelease/
        # Uses CPU architecture detection for 2-5x speedup
        from hardware.cpu_architecture import get_optimal_binary_path
        
        system_name = platform.system().lower()
        binary_name = "llama-server.exe" if system_name == "windows" else "llama-server"
        
        # Path to BitnetRelease/ directory
        backend_dir = Path(__file__).parent.parent.parent
        bitnet_release_dir = backend_dir / "BitNet" / "BitnetRelease"
        
        if not bitnet_release_dir.exists():
            raise FileNotFoundError(
                f"BitnetRelease directory not found: {bitnet_release_dir}\n"
                f"Initialize submodule: git submodule update --init --recursive"
            )
        
        # Detect optimal CPU variant (same logic as BitNet manager)
        try:
            binary_path = get_optimal_binary_path(
                bitnet_release_dir,
                binary_name=binary_name,
                compute_type="cpu"
            )
            
            if binary_path and binary_path.exists():
                logger.info(f"Selected llama.cpp variant: {binary_path.parent.name}")
            else:
                raise FileNotFoundError(f"Binary not found at {binary_path}")
                
        except Exception as e:
            logger.warning(f"CPU detection failed: {e}, trying standard fallback")
            # Fallback to standard
            binary_path = bitnet_release_dir / "cpu" / system_name / "standard" / binary_name
            if not binary_path.exists():
                raise FileNotFoundError(
                    f"llama.cpp binary not found at {binary_path}"
                )
        
        logger.info(
            f"Loading model with llama.cpp: {model_path} "
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
                logger.info(f"llama-server ready on port {config.port}")
                return True
            else:
                logger.error("llama-server failed to start")
                self.server = None
                return False
                
        except Exception as e:
            logger.error(f"Failed to start llama-server: {e}")
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
            
            logger.info("llama-server stopped and model unloaded")
            return True
            
        except Exception as e:
            logger.error(f"Error stopping llama-server: {e}")
            return False
    
    async def generate(
        self,
        messages: List[ChatMessage],
        settings: InferenceSettings
    ) -> str:
        """
        Generate text using llama-server.
        
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
        
        # Send request to llama-server
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
        Generate streaming text using llama-server.
        
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
        
        # Stream from llama-server
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
        Generate embeddings using llama-server /embeddings endpoint.
        
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
            
            logger.info(f"Generated {len(embeddings)} embeddings via llama-server")
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
        
        return {
            "loaded": True,
            "model_path": str(self.model_path),
            "backend": self.backend_type.value if self.backend_type else None,
            "acceleration": self.config.backend.value if self.config else None,
            "ngl": self.config.ngl if self.config else 0,
            "context_size": self.config.context_size if self.config else 0,
            "port": self.config.port if self.config else 0,
        }
    
    @staticmethod
    def _build_server_args(model_path: str, config: LlamaCppConfig) -> List[str]:
        """
        Build command-line arguments for llama-server.
        
        Args:
            model_path: Path to model file
            config: llama.cpp configuration
            
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
        
        # GPU layers
        if config.ngl > 0:
            args.extend(["-ngl", str(config.ngl)])
        
        # CPU threads
        if config.n_threads:
            args.extend(["-t", str(config.n_threads)])
        
        # Backend-specific flags
        if config.backend == LlamaCppBackend.VULKAN:
            args.append("--vulkan")
        elif config.backend == LlamaCppBackend.ROCM:
            args.append("--rocm")
        elif config.backend == LlamaCppBackend.METAL:
            args.append("--metal")
        # CUDA is default if ngl > 0
        
        return args
    
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
    def _map_backend_type(backend: LlamaCppBackend) -> BackendType:
        """
        Map llama.cpp backend to BackendType.
        
        Args:
            backend: llama.cpp backend
            
        Returns:
            BackendType enum value
        """
        mapping = {
            LlamaCppBackend.CPU: BackendType.LLAMA_CPP_CPU,
            LlamaCppBackend.CUDA: BackendType.LLAMA_CPP_CUDA,
            LlamaCppBackend.VULKAN: BackendType.LLAMA_CPP_VULKAN,
            LlamaCppBackend.ROCM: BackendType.LLAMA_CPP_ROCM,
            LlamaCppBackend.METAL: BackendType.LLAMA_CPP_METAL,
        }
        
        return mapping.get(backend, BackendType.LLAMA_CPP_CPU)

