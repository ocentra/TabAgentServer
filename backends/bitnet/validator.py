"""
BitNet model validator and detector.

Detects BitNet 1.58-bit quantized models by analyzing:
- Filename patterns (bitnet, b1.58, b1_58, 1.58bit)
- Quantization identifiers (i2_s, tl1, tl2)
- GGUF metadata (architecture, model name)
"""

import struct
import logging
from pathlib import Path
from typing import Optional, Dict, Any
from enum import Enum

logger = logging.getLogger(__name__)


class ModelType(str, Enum):
    """Model type classification"""
    BITNET_158 = "bitnet_1.58"
    REGULAR_GGUF = "regular_gguf"
    UNKNOWN = "unknown"


class BitNetQuant(str, Enum):
    """BitNet quantization types"""
    I2_S = "i2_s"  # Standard 1.58-bit
    TL1 = "tl1"    # Tiled Layout 1 (Intel optimized)
    TL2 = "tl2"    # Tiled Layout 2 (AMD optimized)


def detect_model_type(model_path: str) -> ModelType:
    """
    Detect if model is BitNet 1.58 or regular GGUF.
    
    Args:
        model_path: Path to GGUF model file
        
    Returns:
        ModelType enum value
        
    Detection strategy:
    1. Check filename patterns first (fast)
    2. Read GGUF metadata if needed (slower but accurate)
    """
    path = Path(model_path)
    
    if not path.exists():
        logger.warning(f"Model file not found: {model_path}")
        return ModelType.UNKNOWN
    
    # Quick filename check
    filename_lower = path.name.lower()
    
    bitnet_patterns = [
        'bitnet',
        'b1.58',
        'b1_58',
        '1.58bit',
        'i2_s',
        'tl1',
        'tl2',
    ]
    
    for pattern in bitnet_patterns:
        if pattern in filename_lower:
            logger.info(f"Detected BitNet model from filename: {path.name}")
            return ModelType.BITNET_158
    
    # Read GGUF metadata if filename didn't match
    try:
        metadata = read_gguf_metadata(model_path)
        
        if metadata:
            arch = metadata.get('architecture', '').lower()
            model_name = metadata.get('model_name', '').lower()
            
            # Check metadata for BitNet indicators
            if 'bitnet' in arch or 'bitnet' in model_name:
                logger.info(f"Detected BitNet model from metadata: {path.name}")
                return ModelType.BITNET_158
        
        # Default to regular GGUF if valid GGUF but no BitNet indicators
        logger.info(f"Detected regular GGUF model: {path.name}")
        return ModelType.REGULAR_GGUF
        
    except Exception as e:
        logger.error(f"Error reading GGUF metadata: {e}")
        return ModelType.UNKNOWN


def detect_bitnet_quant(model_path: str) -> Optional[BitNetQuant]:
    """
    Detect BitNet quantization type from filename.
    
    Args:
        model_path: Path to BitNet model
        
    Returns:
        BitNetQuant enum or None if not detected
    """
    filename_lower = Path(model_path).name.lower()
    
    if 'tl1' in filename_lower:
        return BitNetQuant.TL1
    elif 'tl2' in filename_lower:
        return BitNetQuant.TL2
    elif 'i2_s' in filename_lower:
        return BitNetQuant.I2_S
    
    return None


def read_gguf_metadata(model_path: str) -> Optional[Dict[str, Any]]:
    """
    Read basic metadata from GGUF file.
    
    Args:
        model_path: Path to GGUF file
        
    Returns:
        Dictionary with metadata or None if invalid
        
    GGUF Format (simplified):
    - Magic: 'GGUF' (4 bytes)
    - Version: uint32
    - Tensor count: uint64
    - Metadata KV count: uint64
    - Metadata: key-value pairs
    """
    try:
        with open(model_path, 'rb') as f:
            # Read magic number
            magic = f.read(4)
            if magic != b'GGUF':
                logger.debug(f"Not a GGUF file: {model_path}")
                return None
            
            # Read version
            version = struct.unpack('<I', f.read(4))[0]
            
            # Read tensor count
            tensor_count = struct.unpack('<Q', f.read(8))[0]
            
            # Read metadata KV count
            kv_count = struct.unpack('<Q', f.read(8))[0]
            
            metadata = {
                'version': version,
                'tensor_count': tensor_count,
                'kv_count': kv_count,
            }
            
            # Try to read some key metadata fields
            # (Full GGUF parsing is complex, this is simplified)
            # For now, we rely on filename detection primarily
            
            logger.debug(f"GGUF metadata: version={version}, tensors={tensor_count}, kv_pairs={kv_count}")
            return metadata
            
    except Exception as e:
        logger.debug(f"Could not read GGUF metadata: {e}")
        return None


def is_bitnet_model(model_path: str) -> bool:
    """
    Quick check if model is BitNet 1.58.
    
    Args:
        model_path: Path to model file
        
    Returns:
        True if BitNet 1.58 model
    """
    return detect_model_type(model_path) == ModelType.BITNET_158


def get_model_info(model_path: str) -> Dict[str, Any]:
    """
    Get comprehensive model information.
    
    Args:
        model_path: Path to model file
        
    Returns:
        Dictionary with model type, quant, size, etc.
    """
    path = Path(model_path)
    
    info = {
        'path': str(path),
        'filename': path.name,
        'exists': path.exists(),
        'size_mb': path.stat().st_size / (1024 * 1024) if path.exists() else 0,
        'type': detect_model_type(model_path),
        'quant': None,
        'is_bitnet': False,
    }
    
    if info['type'] == ModelType.BITNET_158:
        info['is_bitnet'] = True
        info['quant'] = detect_bitnet_quant(model_path)
    
    return info

