"""
ONNX Runtime inference backend manager.

Handles model loading and inference with multiple execution providers:
- CPU, CUDA, DirectML, VitisAI (NPU), ROCm
"""

import logging
from pathlib import Path
from typing import List, Optional, Dict, Any
from enum import Enum

from Python.core.message_types import (
    ChatMessage,
    InferenceSettings,
    BackendType,
    AccelerationBackend,
)

from .config import (
    ONNXRTConfig,
    ONNXProvider,
    DEFAULT_CPU_CONFIG,
    DEFAULT_CUDA_CONFIG,
    DEFAULT_DIRECTML_CONFIG,
    DEFAULT_NPU_CONFIG,
    DEFAULT_ROCM_CONFIG,
)
from Python.core.performance_tracker import PerformanceTracker


logger = logging.getLogger(__name__)


class ModelFormat(str, Enum):
    """Supported ONNX model formats"""
    ONNX = "onnx"
    ONNX_OPTIMIZED = "onnx_optimized"
    ONNX_QUANTIZED = "onnx_quantized"


class ONNXRuntimeManager:
    """
    ONNX Runtime inference backend manager.
    
    Manages model loading and inference with automatic provider selection
    based on available hardware.
    """
    
    def __init__(self):
        """Initialize ONNX Runtime manager"""
        self.session: Optional[Any] = None
        self.model_path: Optional[Path] = None
        self.config: Optional[ONNXRTConfig] = None
        self.backend_type: Optional[BackendType] = None
        self.tokenizer: Optional[Any] = None
        self.tokenizer_path: Optional[Path] = None
        self._performance_tracker = PerformanceTracker()
        
        # Lazy import ONNX Runtime
        self._onnxruntime = None
        
        logger.info("ONNXRuntimeManager initialized")
    
    def _ensure_onnxruntime(self):
        """Lazy load ONNX Runtime"""
        if self._onnxruntime is None:
            try:
                import onnxruntime as ort
                self._onnxruntime = ort
                logger.info(f"ONNX Runtime loaded (version: {ort.__version__})")
            except ImportError:
                raise RuntimeError(
                    "ONNX Runtime not installed. "
                    "Install with: pip install onnxruntime"
                )
    
    def is_model_loaded(self) -> bool:
        """
        Check if a model is currently loaded.
        
        Returns:
            True if model is loaded
        """
        return self.session is not None
    
    def load_model(
        self,
        model_path: str,
        acceleration: AccelerationBackend = AccelerationBackend.CPU,
        config: Optional[ONNXRTConfig] = None,
        tokenizer_path: Optional[str] = None
    ) -> bool:
        """
        Load ONNX model with specified acceleration.
        
        Args:
            model_path: Path to ONNX model file
            acceleration: Hardware acceleration to use
            config: ONNX RT configuration (optional)
            
        Returns:
            True if model loaded successfully
            
        Raises:
            FileNotFoundError: If model file not found
            RuntimeError: If ONNX Runtime not available
        """
        self._ensure_onnxruntime()
        
        path = Path(model_path)
        if not path.exists():
            raise FileNotFoundError(f"Model file not found: {model_path}")
        
        # Select configuration based on acceleration
        if config is None:
            config = self._get_config_for_acceleration(acceleration)
        
        logger.info(
            f"Loading ONNX model: {model_path} with providers: {config.providers}"
        )
        
        try:
            # Create session options
            sess_options = self._onnxruntime.SessionOptions()
            sess_options.graph_optimization_level = config.optimization_level.value
            sess_options.enable_profiling = config.enable_profiling
            sess_options.log_severity_level = config.log_severity_level
            
            if config.intra_op_num_threads > 0:
                sess_options.intra_op_num_threads = config.intra_op_num_threads
            if config.inter_op_num_threads > 0:
                sess_options.inter_op_num_threads = config.inter_op_num_threads
            
            # Create inference session
            self.session = self._onnxruntime.InferenceSession(
                str(path),
                sess_options=sess_options,
                providers=config.providers
            )
            
            # Log actual providers used
            actual_providers = self.session.get_providers()
            logger.info(f"Model loaded with providers: {actual_providers}")
            
            self.model_path = path
            self.config = config
            self.backend_type = self._get_backend_type(acceleration)
            
            # Load tokenizer if provided
            if tokenizer_path:
                self._load_tokenizer(tokenizer_path)
            
            return True
            
        except Exception as e:
            logger.error(f"Failed to load ONNX model: {e}")
            self.session = None
            return False
    
    def unload_model(self) -> bool:
        """
        Unload current model.
        
        Returns:
            True if model unloaded successfully
        """
        if self.session is None:
            logger.info("No model loaded, nothing to unload")
            return True
        
        try:
            # ONNX Runtime sessions are managed by Python GC
            self.session = None
            self.model_path = None
            self.config = None
            self.backend_type = None
            self.tokenizer = None
            
            logger.info("ONNX model unloaded")
            return True
            
        except Exception as e:
            logger.error(f"Error unloading model: {e}")
            return False
    
    async def generate(
        self,
        messages: List[ChatMessage],
        settings: InferenceSettings
    ) -> str:
        """
        Generate text using ONNX model.
        
        Uses onnxruntime-genai for models that support it,
        falls back to manual tokenization for others.
        
        Args:
            messages: Chat messages
            settings: Inference settings
            
        Returns:
            Generated text
            
        Raises:
            RuntimeError: If no model loaded
        """
        if self.session is None:
            raise RuntimeError("No model loaded")
        
        try:
            # Try using onnxruntime-genai if available (recommended for GenAI models)
            return await self._generate_with_genai(messages, settings)
        except ImportError:
            # Fall back to manual inference
            logger.info("onnxruntime-genai not available, using manual inference")
            return await self._generate_manual(messages, settings)
    
    async def generate_stream(
        self,
        messages: List[ChatMessage],
        settings: InferenceSettings
    ):
        """
        Generate streaming text using ONNX model.
        
        Args:
            messages: Chat messages
            settings: Inference settings
            
        Yields:
            Generated tokens
            
        Raises:
            RuntimeError: If no model loaded
        """
        if self.session is None:
            raise RuntimeError("No model loaded")
        
        try:
            # Try using onnxruntime-genai streaming
            async for token in self._generate_stream_genai(messages, settings):
                yield token
        except ImportError:
            # Fall back to manual streaming
            logger.info("onnxruntime-genai not available, using manual streaming")
            async for token in self._generate_stream_manual(messages, settings):
                yield token
    
    async def _generate_with_genai(
        self,
        messages: List[ChatMessage],
        settings: InferenceSettings
    ) -> str:
        """
        Generate using onnxruntime-genai.
        
        This is the recommended path for GenAI models (Phi, Llama, etc.)
        """
        try:
            import onnxruntime_genai as og
        except ImportError:
            raise ImportError("onnxruntime-genai not installed")
        
        # Load model and tokenizer with genai
        model = og.Model(str(self.model_path.parent))
        tokenizer = og.Tokenizer(model)
        
        # Build prompt from messages
        prompt = self._build_prompt(messages)
        
        # Tokenize
        input_tokens = tokenizer.encode(prompt)
        
        # Set generation parameters
        params = og.GeneratorParams(model)
        params.input_ids = input_tokens
        params.set_search_options(
            max_length=settings.max_new_tokens,
            temperature=settings.temperature,
            top_p=settings.top_p,
            top_k=settings.top_k
        )
        
        # Generate
        generator = og.Generator(model, params)
        
        generated_text = ""
        while not generator.is_done():
            generator.compute_logits()
            generator.generate_next_token()
            
            new_token = generator.get_next_tokens()[0]
            generated_text = tokenizer.decode(new_token)
        
        logger.info(f"Generated {len(generated_text)} characters")
        return generated_text
    
    async def _generate_stream_genai(
        self,
        messages: List[ChatMessage],
        settings: InferenceSettings
    ):
        """Generate streaming using onnxruntime-genai"""
        try:
            import onnxruntime_genai as og
        except ImportError:
            raise ImportError("onnxruntime-genai not installed")
        
        # Load model and tokenizer
        model = og.Model(str(self.model_path.parent))
        tokenizer = og.Tokenizer(model)
        
        # Build prompt
        prompt = self._build_prompt(messages)
        input_tokens = tokenizer.encode(prompt)
        
        # Set params
        params = og.GeneratorParams(model)
        params.input_ids = input_tokens
        params.set_search_options(
            max_length=settings.max_new_tokens,
            temperature=settings.temperature,
            top_p=settings.top_p,
            top_k=settings.top_k
        )
        
        # Stream tokens
        generator = og.Generator(model, params)
        
        while not generator.is_done():
            generator.compute_logits()
            generator.generate_next_token()
            
            new_token = generator.get_next_tokens()[0]
            token_text = tokenizer.decode([new_token])
            
            yield token_text
    
    async def _generate_manual(
        self,
        messages: List[ChatMessage],
        settings: InferenceSettings
    ) -> str:
        """
        Generate using manual ONNX session inference.
        
        Fallback for models without genai support.
        """
        if self.tokenizer is None:
            raise RuntimeError("Tokenizer not loaded. Manual inference requires tokenizer.")
        
        # Build prompt
        prompt = self._build_prompt(messages)
        
        # Tokenize
        inputs = self.tokenizer(prompt, return_tensors="np")
        
        # Run inference
        import numpy as np
        
        ort_inputs = {
            inp.name: inputs[inp.name].astype(np.int64)
            for inp in self.session.get_inputs()
            if inp.name in inputs
        }
        
        outputs = self.session.run(None, ort_inputs)
        
        # Decode output
        output_ids = outputs[0][0]
        generated_text = self.tokenizer.decode(output_ids, skip_special_tokens=True)
        
        # Remove prompt from output
        if generated_text.startswith(prompt):
            generated_text = generated_text[len(prompt):].strip()
        
        return generated_text
    
    async def _generate_stream_manual(
        self,
        messages: List[ChatMessage],
        settings: InferenceSettings
    ):
        """
        Manual streaming fallback.
        
        For now, generates full text and yields it as single chunk.
        True token-by-token streaming requires model-specific logic.
        """
        full_text = await self._generate_manual(messages, settings)
        yield full_text
    
    async def generate_embeddings(self, texts: List[str]) -> List[List[float]]:
        """
        Generate embeddings for texts using ONNX model.
        
        Args:
            texts: List of text strings
            
        Returns:
            List of embedding vectors
            
        Raises:
            RuntimeError: If model not loaded or not an embedding model
        """
        if self.session is None:
            raise RuntimeError("No model loaded")
        
        if self.tokenizer is None:
            raise RuntimeError("Tokenizer required for embeddings")
        
        try:
            import numpy as np
            
            embeddings = []
            
            for text in texts:
                # Tokenize
                inputs = self.tokenizer(text, return_tensors="np", padding=True, truncation=True)
                
                # Prepare ONNX inputs
                ort_inputs = {
                    inp.name: inputs[inp.name].astype(np.int64)
                    for inp in self.session.get_inputs()
                    if inp.name in inputs
                }
                
                # Run inference
                outputs = self.session.run(None, ort_inputs)
                
                # Extract embedding (usually first output, pooled)
                embedding = outputs[0][0]  # Shape: [hidden_size]
                
                # Normalize if needed (L2 normalization)
                norm = np.linalg.norm(embedding)
                if norm > 0:
                    embedding = embedding / norm
                
                embeddings.append(embedding.tolist())
            
            logger.info(f"Generated {len(embeddings)} embeddings")
            return embeddings
        
        except Exception as e:
            logger.error(f"Embedding generation failed: {e}")
            raise RuntimeError(f"Failed to generate embeddings: {e}")
    
    def get_model_info(self) -> Dict[str, Any]:
        """
        Get information about loaded model.
        
        Returns:
            Dictionary with model information
        """
        if self.session is None:
            return {
                "loaded": False,
                "error": "No model loaded"
            }
        
        return {
            "loaded": True,
            "model_path": str(self.model_path),
            "backend": self.backend_type.value if self.backend_type else None,
            "providers": self.session.get_providers(),
            "inputs": [inp.name for inp in self.session.get_inputs()],
            "outputs": [out.name for out in self.session.get_outputs()],
        }
    
    @staticmethod
    def _get_config_for_acceleration(
        acceleration: AccelerationBackend
    ) -> ONNXRTConfig:
        """
        Get ONNX RT config for acceleration type.
        
        Args:
            acceleration: Hardware acceleration type
            
        Returns:
            ONNXRTConfig for the acceleration
        """
        config_map = {
            AccelerationBackend.CPU: DEFAULT_CPU_CONFIG,
            AccelerationBackend.CUDA: DEFAULT_CUDA_CONFIG,
            AccelerationBackend.DIRECTML: DEFAULT_DIRECTML_CONFIG,
            AccelerationBackend.NPU: DEFAULT_NPU_CONFIG,
            AccelerationBackend.ROCM: DEFAULT_ROCM_CONFIG,
        }
        
        config = config_map.get(acceleration, DEFAULT_CPU_CONFIG)
        logger.info(f"Selected config for {acceleration.value}: {config.providers}")
        return config
    
    @staticmethod
    def _get_backend_type(acceleration: AccelerationBackend) -> BackendType:
        """
        Map acceleration to backend type.
        
        Args:
            acceleration: Acceleration backend
            
        Returns:
            BackendType enum value
        """
        mapping = {
            AccelerationBackend.CPU: BackendType.ONNX_CPU,
            AccelerationBackend.CUDA: BackendType.ONNX_CUDA,
            AccelerationBackend.DIRECTML: BackendType.ONNX_DIRECTML,
            AccelerationBackend.NPU: BackendType.ONNX_NPU,
        }
        
        return mapping.get(acceleration, BackendType.ONNX_CPU)
    
    def _load_tokenizer(self, tokenizer_path: str) -> bool:
        """
        Load tokenizer for ONNX model.
        
        Args:
            tokenizer_path: Path to tokenizer directory or file
            
        Returns:
            True if loaded successfully
        """
        try:
            from transformers import AutoTokenizer
            
            self.tokenizer = AutoTokenizer.from_pretrained(tokenizer_path)
            self.tokenizer_path = Path(tokenizer_path)
            logger.info(f"Tokenizer loaded from {tokenizer_path}")
            return True
            
        except ImportError:
            logger.warning(
                "Transformers not installed. "
                "Install with: pip install transformers"
            )
            return False
        except Exception as e:
            logger.error(f"Failed to load tokenizer: {e}")
            return False
    
    def _build_prompt(self, messages: List[ChatMessage]) -> str:
        """
        Build prompt string from chat messages.
        
        Args:
            messages: List of chat messages
            
        Returns:
            Formatted prompt string
        """
        # Simple format for now
        prompt_parts = []
        for msg in messages:
            prompt_parts.append(f"{msg.role.value}: {msg.content}")
        return "\n".join(prompt_parts) + "\nassistant:"
    
    def get_state(self) -> Dict[str, Any]:
        """
        Get current manager state.
        
        Returns:
            Dictionary with state information including performance metrics
        """
        state = {
            "isReady": self.session is not None,
            "backend": self.backend_type.value if self.backend_type else None,
            "modelPath": str(self.model_path) if self.model_path else None
        }
        
        # Add performance metrics
        stats = self._performance_tracker.get_current_stats()
        state.update(stats)
        
        return state
    
    @staticmethod
    def get_available_providers() -> List[str]:
        """
        Get list of available ONNX Runtime providers.
        
        Returns:
            List of provider names
            
        Raises:
            RuntimeError: If ONNX Runtime not available
        """
        try:
            import onnxruntime as ort
            return ort.get_available_providers()
        except ImportError:
            raise RuntimeError("ONNX Runtime not installed")

