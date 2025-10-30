"""
Rust File Provider
==================

Intercepts HuggingFace model file requests and fetches from Rust's ModelCache via gRPC.
Replaces transformers' default download behavior.

This mirrors the extension's fetch intercept pattern:
- Extension: IndexedDB cache → Network download
- Here: Rust gRPC cache → (Rust handles download)
"""

import logging
import os
from pathlib import Path
from typing import Optional
import sys

# Add generated protos to path
sys.path.insert(0, str(Path(__file__).parent.parent / 'generated'))

try:
    from generated import ml_inference_pb2, ml_inference_pb2_grpc
except ImportError:
    # Graceful fallback if protos not generated yet
    ml_inference_pb2 = None
    ml_inference_pb2_grpc = None

logger = logging.getLogger(__name__)

class RustFileProvider:
    """
    Provides model files from Rust's ModelCache via gRPC.
    
    Used by pipelines to fetch config.json, model weights, etc.
    without downloading from HuggingFace directly.
    """
    
    def __init__(self, grpc_stub):
        """
        Args:
            grpc_stub: ModelManagementServiceStub from gRPC client
        """
        self.stub = grpc_stub
        logger.info("RustFileProvider initialized")
    
    def get_file(self, model_id: str, file_path: str) -> bytes:
        """
        Get a model file from Rust's cache.
        
        Args:
            model_id: HuggingFace model ID (e.g., "microsoft/Florence-2-base")
            file_path: Relative file path (e.g., "config.json", "model.safetensors")
        
        Returns:
            File contents as bytes
        
        Raises:
            RuntimeError: If file cannot be retrieved
        """
        if not self.stub:
            raise RuntimeError("RustFileProvider not connected to gRPC service")
        
        logger.info(f"Fetching {file_path} for {model_id} from Rust")
        
        try:
            # Stream file chunks from Rust
            request = ml_inference_pb2.ModelFileRequest(
                model_id=model_id,
                file_path=file_path
            )
            
            chunks = []
            total_received = 0
            
            for chunk in self.stub.GetModelFile(request):
                chunks.append(chunk.data)
                total_received += len(chunk.data)
                
                if chunk.is_last:
                    logger.debug(f"Received final chunk for {file_path} (total: {total_received} bytes)")
                    break
            
            data = b''.join(chunks)
            logger.info(f"✅ Retrieved {file_path} from Rust ({len(data)} bytes)")
            return data
            
        except Exception as e:
            logger.error(f"❌ Failed to get {file_path} from Rust: {e}")
            raise RuntimeError(f"Failed to get {file_path} from Rust: {e}")
    
    def get_file_path(self, model_id: str, file_path: str) -> Optional[Path]:
        """
        Get a file and save to temp location, returning path.
        
        Some libraries need file paths instead of bytes.
        
        Args:
            model_id: HuggingFace model ID
            file_path: Relative file path
        
        Returns:
            Path to temporary file with contents
        """
        import tempfile
        
        data = self.get_file(model_id, file_path)
        
        # Create temp file with same extension
        suffix = Path(file_path).suffix
        with tempfile.NamedTemporaryFile(mode='wb', delete=False, suffix=suffix) as f:
            f.write(data)
            temp_path = Path(f.name)
        
        logger.debug(f"Wrote {file_path} to temp file: {temp_path}")
        return temp_path


def install_offline_mode():
    """
    Configure transformers to NEVER download from HuggingFace.
    
    Forces all file access to go through our RustFileProvider.
    """
    # Block internet access for transformers
    os.environ['HF_HUB_OFFLINE'] = '1'
    os.environ['TRANSFORMERS_OFFLINE'] = '1'
    
    logger.info("✅ Transformers offline mode enabled - will use RustFileProvider")


# Install offline mode on module import
install_offline_mode()

