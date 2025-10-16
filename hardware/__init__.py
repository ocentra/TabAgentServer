"""
Hardware detection and backend selection for TabAgent.

Provides hardware capability detection, acceleration backend detection,
and intelligent backend selection with VRAM-aware configuration.
"""

from .hardware_detection import (
    HardwareDetector,
    WindowsHardwareDetector,
    create_hardware_detector,
    NVIDIAGPUKeyword,
    AMDGPUKeyword,
    OSType,
)

from .engine_detection import (
    AccelerationDetector,
    InferenceEngineDetector,
    EngineInfo,
    EngineType,
    EngineAvailability,
)

from .backend_selector import (
    BackendSelector,
    GPULayerCalculator,
    BackendSelectionResult,
    SelectionStrategy,
)

__all__ = [
    # Hardware detection
    'HardwareDetector',
    'WindowsHardwareDetector',
    'create_hardware_detector',
    'NVIDIAGPUKeyword',
    'AMDGPUKeyword',
    'OSType',
    
    # Engine detection
    'AccelerationDetector',
    'InferenceEngineDetector',
    'EngineInfo',
    'EngineType',
    'EngineAvailability',
    
    # Backend selection
    'BackendSelector',
    'GPULayerCalculator',
    'BackendSelectionResult',
    'SelectionStrategy',
]

