"""
HuggingFace Transformers inference backend.

Handles SafeTensors and PyTorch models using the transformers library.
Supports various tasks: text-generation, feature-extraction, etc.
"""

import logging
from pathlib import Path
from typing import Dict, Any, Optional, List
import torch

from .base_backend import TextGenerationBackend, EmbeddingBackend

logger = logging.getLogger(__name__)


class TransformersTextGenBackend(TextGenerationBackend):
    """
    Transformers backend for text generation using PyTorch/SafeTensors models.
    
    Supports:
    - Causal LM (GPT-like models)
    - SafeTensors format
    - PyTorch checkpoints
    - Quantization (int8, int4 with bitsandbytes)
    """
    
    def __init__(self):
        """Initialize Transformers backend"""
        super().__init__()
        self._transformers = None
        self.device = "cuda" if torch.cuda.is_available() else "cpu"
        logger.info(f"TransformersTextGenBackend initialized (device: {self.device})")
    
    def _ensure_transformers(self):
        """Lazy load transformers library"""
        if self._transformers is None:
            try:
                import transformers
                self._transformers = transformers
                logger.info(f"Transformers loaded (version: {transformers.__version__})")
            except ImportError:
                raise RuntimeError(
                    "Transformers not installed. "
                    "Install with: pip install transformers torch"
                )
    
    def load_model(
        self,
        model_path: str,
        task: str,
        dtype: Optional[str] = None,
        device: Optional[str] = None,
        load_in_8bit: bool = False,
        load_in_4bit: bool = False,
        trust_remote_code: bool = False,
        **kwargs
    ) -> bool:
        """
        Load model from HuggingFace or local path.
        
        Args:
            model_path: HuggingFace model ID or local path
            task: Task type (text-generation, text2text-generation, etc.)
            dtype: Data type (float32, float16, bfloat16)
            device: Device to use (cuda, cpu)
            load_in_8bit: Use 8-bit quantization
            load_in_4bit: Use 4-bit quantization
            trust_remote_code: Allow custom code execution
            **kwargs: Additional arguments for from_pretrained()
            
        Returns:
            True if successful
        """
        self._ensure_transformers()
        
        try:
            # Use provided device or default
            device_to_use = device or self.device
            
            # Convert dtype string to torch dtype
            torch_dtype = None
            if dtype:
                dtype_map = {
                    "float32": torch.float32,
                    "float16": torch.float16,
                    "bfloat16": torch.bfloat16,
                    "fp32": torch.float32,
                    "fp16": torch.float16,
                    "bf16": torch.bfloat16,
                }
                torch_dtype = dtype_map.get(dtype.lower())
            
            # Load model based on task
            logger.info(f"Loading model: {model_path} (task: {task}, device: {device_to_use})")
            
            if task in ["text-generation", "text2text-generation"]:
                self.model = self._transformers.AutoModelForCausalLM.from_pretrained(
                    model_path,
                    torch_dtype=torch_dtype,
                    device_map="auto" if device_to_use == "cuda" else None,
                    load_in_8bit=load_in_8bit,
                    load_in_4bit=load_in_4bit,
                    trust_remote_code=trust_remote_code,
                    **kwargs
                )
            elif task == "feature-extraction":
                self.model = self._transformers.AutoModel.from_pretrained(
                    model_path,
                    torch_dtype=torch_dtype,
                    device_map="auto" if device_to_use == "cuda" else None,
                    trust_remote_code=trust_remote_code,
                    **kwargs
                )
            else:
                # Try AutoModel as fallback
                self.model = self._transformers.AutoModel.from_pretrained(
                    model_path,
                    torch_dtype=torch_dtype,
                    device_map="auto" if device_to_use == "cuda" else None,
                    trust_remote_code=trust_remote_code,
                    **kwargs
                )
            
            # Load tokenizer
            self.tokenizer = self._transformers.AutoTokenizer.from_pretrained(
                model_path,
                trust_remote_code=trust_remote_code
            )
            
            # Move to device if not using device_map
            if device_map != "auto" and device_to_use != "cpu":
                self.model = self.model.to(device_to_use)
            
            self.model.eval()
            
            # Store configuration
            self.model_path = Path(model_path)
            self.config = {
                "task": task,
                "device": device_to_use,
                "dtype": str(torch_dtype) if torch_dtype else None,
                "load_in_8bit": load_in_8bit,
                "load_in_4bit": load_in_4bit,
            }
            self._is_loaded = True
            
            logger.info(f"Model loaded successfully: {model_path}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to load model: {e}", exc_info=True)
            return False
    
    def generate(
        self,
        prompt: str,
        max_length: int = 512,
        temperature: float = 0.7,
        top_p: float = 0.9,
        top_k: int = 50,
        num_return_sequences: int = 1,
        do_sample: bool = True,
        **kwargs
    ) -> Dict[str, Any]:
        """
        Generate text from prompt.
        
        Args:
            prompt: Input text
            max_length: Maximum length of generated sequence
            temperature: Sampling temperature
            top_p: Nucleus sampling parameter
            top_k: Top-k sampling parameter
            num_return_sequences: Number of sequences to generate
            do_sample: Whether to use sampling
            **kwargs: Additional generation parameters
            
        Returns:
            Dictionary with generated text and metadata
        """
        if not self._is_loaded:
            raise RuntimeError("Model not loaded")
        
        try:
            # Tokenize input
            inputs = self.tokenizer(
                prompt,
                return_tensors="pt",
                padding=True,
                truncation=True
            )
            
            # Move to device
            device = next(self.model.parameters()).device
            inputs = {k: v.to(device) for k, v in inputs.items()}
            
            # Generate
            with torch.no_grad():
                outputs = self.model.generate(
                    **inputs,
                    max_length=max_length,
                    temperature=temperature,
                    top_p=top_p,
                    top_k=top_k,
                    num_return_sequences=num_return_sequences,
                    do_sample=do_sample,
                    pad_token_id=self.tokenizer.eos_token_id,
                    **kwargs
                )
            
            # Decode outputs
            generated_texts = [
                self.tokenizer.decode(output, skip_special_tokens=True)
                for output in outputs
            ]
            
            return {
                "success": True,
                "generated_text": generated_texts[0] if num_return_sequences == 1 else generated_texts,
                "all_generated_texts": generated_texts,
                "num_sequences": num_return_sequences,
            }
            
        except Exception as e:
            logger.error(f"Generation failed: {e}", exc_info=True)
            return {
                "success": False,
                "error": str(e)
            }


class TransformersEmbeddingBackend(EmbeddingBackend):
    """
    Transformers backend for feature extraction/embeddings.
    
    Supports:
    - BERT, RoBERTa, DistilBERT
    - Sentence transformers
    - Custom embedding models
    """
    
    def __init__(self):
        """Initialize embeddings backend"""
        super().__init__()
        self._transformers = None
        self.device = "cuda" if torch.cuda.is_available() else "cpu"
        logger.info(f"TransformersEmbeddingBackend initialized (device: {self.device})")
    
    def _ensure_transformers(self):
        """Lazy load transformers"""
        if self._transformers is None:
            try:
                import transformers
                self._transformers = transformers
            except ImportError:
                raise RuntimeError("Transformers not installed")
    
    def load_model(
        self,
        model_path: str,
        task: str = "feature-extraction",
        **kwargs
    ) -> bool:
        """Load embedding model"""
        self._ensure_transformers()
        
        try:
            self.model = self._transformers.AutoModel.from_pretrained(model_path)
            self.tokenizer = self._transformers.AutoTokenizer.from_pretrained(model_path)
            self.model.to(self.device)
            self.model.eval()
            
            self.model_path = Path(model_path)
            self._is_loaded = True
            
            logger.info(f"Embedding model loaded: {model_path}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to load embedding model: {e}")
            return False
    
    def embed(self, text: str, **kwargs) -> List[float]:
        """Generate embeddings for text"""
        if not self._is_loaded:
            raise RuntimeError("Model not loaded")
        
        try:
            # Tokenize
            inputs = self.tokenizer(
                text,
                return_tensors="pt",
                padding=True,
                truncation=True
            )
            inputs = {k: v.to(self.device) for k, v in inputs.items()}
            
            # Get embeddings
            with torch.no_grad():
                outputs = self.model(**inputs)
                # Use CLS token embedding or mean pooling
                embeddings = outputs.last_hidden_state[:, 0, :].cpu().numpy()
            
            return embeddings[0].tolist()
            
        except Exception as e:
            logger.error(f"Embedding generation failed: {e}")
            raise
    
    def generate(self, prompt: str, **generation_params) -> Dict[str, Any]:
        """Not applicable for embedding backend"""
        raise NotImplementedError("Embedding backend does not support text generation")

