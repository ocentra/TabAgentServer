"""
Core module for native host.
Contains shared types and configurations.
"""

# Hardware detection and backend selection
from Python.hardware import (
    HardwareDetector,
    WindowsHardwareDetector,
    create_hardware_detector,
    NVIDIAGPUKeyword,
    AMDGPUKeyword,
    OSType,
    AccelerationDetector,
    InferenceEngineDetector,
    EngineInfo,
    EngineType,
    EngineAvailability,
    BackendSelector,
    GPULayerCalculator,
    BackendSelectionResult,
    SelectionStrategy,
)

# Server management
from Python.server_mgmt import (
    PortManager,
    ServerType,
    PortAllocation,
    get_port_manager,
    reset_port_manager,
    WrappedServer,
    ServerConfig,
    ServerState,
    HealthCheckMethod,
    ShutdownMethod,
)

# Model management
from Python.models import (
    ModelLibrary,
    ModelManager,
    ModelInfo,
    ModelStatus,
    ModelUseCase,
    ModelLicense,
)

# Message types
from .message_types import (
    # Enums
    ActionType,
    EventType,
    BackendType,
    ModelType,
    LoadingStatus,
    BitNetQuantType,
    MessageRole,
    GPUVendor,
    GPUType,
    AccelerationBackend,
    
    # Hardware models
    CPUInfo,
    GPUInfo,
    NPUInfo,
    HardwareCapabilities,
    HardwareInfo,
    BackendConfig,
    
    # Request/Response models
    ChatMessage,
    InferenceSettings,
    LoadModelRequest,
    GenerateRequest,
    UpdateSettingsRequest,
    UnloadModelRequest,
    GetModelStateRequest,
    StopGenerationRequest,
    BaseResponse,
    ErrorResponse,
    SuccessResponse,
    SystemInfoResponse,
    LoadingProgressPayload,
    GenerationUpdatePayload,
    GenerationCompletePayload,
    ModelStatePayload,
    WorkerReadyPayload
)

# Configuration
from .config import (
    LOG_LEVEL,
    LOG_FILE,
    ALLOWED_COMMANDS,
    COMMAND_TIMEOUT,
    MAX_MESSAGE_SIZE,
    BitNetConfig,
    BITNET_CONFIG
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
    
    # Server management
    'PortManager',
    'ServerType',
    'PortAllocation',
    'get_port_manager',
    'reset_port_manager',
    'WrappedServer',
    'ServerConfig',
    'ServerState',
    'HealthCheckMethod',
    'ShutdownMethod',
    
    # Model management
    'ModelLibrary',
    'ModelManager',
    'ModelInfo',
    'ModelStatus',
    'ModelUseCase',
    'ModelLicense',
    
    # Message types - Enums
    'ActionType',
    'EventType',
    'BackendType',
    'ModelType',
    'LoadingStatus',
    'BitNetQuantType',
    'MessageRole',
    'GPUVendor',
    'GPUType',
    'AccelerationBackend',
    
    # Message types - Hardware models
    'CPUInfo',
    'GPUInfo',
    'NPUInfo',
    'HardwareCapabilities',
    'HardwareInfo',
    'BackendConfig',
    
    # Message types - Request models
    'ChatMessage',
    'InferenceSettings',
    'LoadModelRequest',
    'GenerateRequest',
    'UpdateSettingsRequest',
    'UnloadModelRequest',
    'GetModelStateRequest',
    'StopGenerationRequest',
    
    # Message types - Response models
    'BaseResponse',
    'ErrorResponse',
    'SuccessResponse',
    'SystemInfoResponse',
    
    # Message types - Payload models
    'LoadingProgressPayload',
    'GenerationUpdatePayload',
    'GenerationCompletePayload',
    'ModelStatePayload',
    'WorkerReadyPayload',
    
    # Configuration
    'LOG_LEVEL',
    'LOG_FILE',
    'ALLOWED_COMMANDS',
    'COMMAND_TIMEOUT',
    'MAX_MESSAGE_SIZE',
    'BitNetConfig',
    'BITNET_CONFIG',
]

