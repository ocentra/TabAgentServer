"""
Message types and data structures for native host communication.
Strongly typed protocol definitions matching extension expectations.
"""

from enum import Enum
from typing import Dict, List, Optional, Any, Literal, TypedDict
from dataclasses import dataclass
from pydantic import BaseModel, Field


# Action Types (from extension)
class ActionType(str, Enum):
    """Actions that can be received from extension"""
    PING = "ping"
    GET_SYSTEM_INFO = "get_system_info"
    EXECUTE_COMMAND = "execute_command"
    
    # Model actions (generic - works for all backends)
    PULL_MODEL = "pull_model"
    LOAD_MODEL = "load_model"
    GENERATE = "generate"
    GET_MODEL_STATE = "get_model_state"
    UPDATE_SETTINGS = "update_settings"
    UNLOAD_MODEL = "unload_model"
    DELETE_MODEL = "delete_model"
    STOP_GENERATION = "stop_generation"
    
    # Embeddings and RAG actions
    GENERATE_EMBEDDINGS = "generate_embeddings"
    SEMANTIC_SEARCH = "semantic_search"
    RERANK_DOCUMENTS = "rerank_documents"
    CLUSTER_TEXTS = "cluster_texts"
    RECOMMEND_ITEMS = "recommend_items"
    COMPUTE_SIMILARITY = "compute_similarity"
    
    # Configuration actions
    GET_PARAMS = "get_params"
    SET_PARAMS = "set_params"
    GET_RECIPES = "get_recipes"
    GET_REGISTERED_MODELS = "get_registered_models"
    
    # Resource management (for agentic systems)
    QUERY_RESOURCES = "query_resources"
    LIST_LOADED_MODELS = "list_loaded_models"
    SELECT_ACTIVE_MODEL = "select_active_model"
    ESTIMATE_MODEL_SIZE = "estimate_model_size"
    
    # Hardware detection and recommendations
    GET_HARDWARE_INFO = "get_hardware_info"
    CHECK_MODEL_FEASIBILITY = "check_model_feasibility"
    GET_RECOMMENDED_MODELS = "get_recommended_models"
    
    # HuggingFace Authentication
    SET_HF_TOKEN = "set_hf_token"
    GET_HF_TOKEN_STATUS = "get_hf_token_status"
    CLEAR_HF_TOKEN = "clear_hf_token"
    
    # Chat history & sync (hybrid storage)
    CREATE_CONVERSATION = "create_conversation"
    GET_CONVERSATION = "get_conversation"
    LIST_CONVERSATIONS = "list_conversations"
    SEARCH_CONVERSATIONS = "search_conversations"
    ADD_MESSAGES = "add_messages"
    SYNC_PUSH = "sync_push"
    SYNC_PULL = "sync_pull"
    
    # LM Studio specific actions
    CHECK_LMSTUDIO = "check_lmstudio"
    START_LMSTUDIO = "start_lmstudio"
    STOP_LMSTUDIO = "stop_lmstudio"
    LMSTUDIO_STATUS = "lmstudio_status"


# Event Types (to extension)
class EventType(str, Enum):
    """Events sent to extension"""
    # Model loading events
    WORKER_ENV_READY = "workerEnvReady"
    WORKER_READY = "workerReady"
    MANIFEST_UPDATED = "manifestUpdated"
    
    # Progress events
    MODEL_LOADING_PROGRESS = "modelWorkerLoadingProgress"
    
    # Generation events
    GENERATION_UPDATE = "generationUpdate"
    GENERATION_COMPLETE = "generationComplete"
    GENERATION_STOPPED = "generationStopped"
    GENERATION_ERROR = "generationError"
    
    # Error events
    ERROR = "error"


# Loading Status Types
class LoadingStatus(str, Enum):
    """Model loading status values"""
    INITIATE = "initiate"
    PROGRESS = "progress"
    DONE = "done"
    ERROR = "error"
    CACHED = "cached"


# Backend Types
class BackendType(str, Enum):
    """Available inference backends"""
    BITNET_CPU = "bitnet_cpu"
    BITNET_GPU = "bitnet_gpu"
    ONNX_CPU = "onnx_cpu"
    ONNX_CUDA = "onnx_cuda"
    ONNX_DIRECTML = "onnx_directml"
    ONNX_NPU = "onnx_npu"
    LLAMA_CPP_CPU = "llama_cpp_cpu"
    LLAMA_CPP_CUDA = "llama_cpp_cuda"
    LLAMA_CPP_VULKAN = "llama_cpp_vulkan"
    LLAMA_CPP_ROCM = "llama_cpp_rocm"
    LLAMA_CPP_METAL = "llama_cpp_metal"
    MEDIAPIPE_CPU = "mediapipe_cpu"
    MEDIAPIPE_GPU = "mediapipe_gpu"
    MEDIAPIPE_NPU = "mediapipe_npu"
    LMSTUDIO = "lmstudio"
    UNKNOWN = "unknown"


# Model Types
class ModelType(str, Enum):
    """Detected model types"""
    BITNET_158 = "bitnet_1.58"
    GGUF_REGULAR = "gguf_regular"
    SAFETENSORS = "safetensors"
    PYTORCH = "pytorch"
    ONNX = "onnx"
    MEDIAPIPE_TASK = "mediapipe_task"  # .task bundle format
    UNKNOWN = "unknown"


# BitNet Quantization Types
class BitNetQuantType(str, Enum):
    """BitNet specific quantization types"""
    I2_S = "i2_s"
    TL1 = "tl1"
    TL2 = "tl2"


# Message Role Types
class MessageRole(str, Enum):
    """Chat message roles"""
    SYSTEM = "system"
    USER = "user"
    ASSISTANT = "assistant"


# Pydantic Models for Request/Response validation

class ChatMessage(BaseModel):
    """Single chat message"""
    role: MessageRole
    content: str


class InferenceSettings(BaseModel):
    """Inference generation settings"""
    temperature: float = Field(default=0.7, ge=0.0, le=2.0)
    top_k: int = Field(default=40, ge=0)
    top_p: float = Field(default=0.9, ge=0.0, le=1.0)
    max_new_tokens: int = Field(default=512, ge=1)
    repetition_penalty: float = Field(default=1.0, ge=1.0)
    do_sample: bool = Field(default=True)


class LoadModelRequest(BaseModel):
    """Request to load a model"""
    action: Literal[ActionType.LOAD_MODEL]
    modelPath: str
    isBitNet: Optional[bool] = None
    dtype: Optional[str] = None
    task: Optional[str] = None


class GenerateRequest(BaseModel):
    """Request to generate text"""
    action: Literal[ActionType.GENERATE]
    messages: List[ChatMessage]
    settings: Optional[InferenceSettings] = None


class UpdateSettingsRequest(BaseModel):
    """Request to update inference settings"""
    action: Literal[ActionType.UPDATE_SETTINGS]
    settings: InferenceSettings


class UnloadModelRequest(BaseModel):
    """Request to unload model"""
    action: Literal[ActionType.UNLOAD_MODEL]


class StopGenerationRequest(BaseModel):
    """Request to stop generation"""
    action: Literal[ActionType.STOP_GENERATION]


class CheckLMStudioRequest(BaseModel):
    """Request to check LM Studio status"""
    action: Literal[ActionType.CHECK_LMSTUDIO]


class StartLMStudioRequest(BaseModel):
    """Request to start LM Studio server"""
    action: Literal[ActionType.START_LMSTUDIO]


class StopLMStudioRequest(BaseModel):
    """Request to stop LM Studio server"""
    action: Literal[ActionType.STOP_LMSTUDIO]


class LMStudioStatusPayload(BaseModel):
    """LM Studio status payload"""
    installed: bool
    bootstrapped: bool
    server_running: bool
    api_endpoint: str
    current_model: Optional[str] = None


class GetModelStateRequest(BaseModel):
    """Request model state"""
    action: Literal[ActionType.GET_MODEL_STATE]


# Response Types

class BaseResponse(BaseModel):
    """Base response structure"""
    status: Literal["success", "error"]


class ErrorResponse(BaseResponse):
    """Error response"""
    status: Literal["error"]
    message: str
    error: Optional[str] = None


class SuccessResponse(BaseResponse):
    """Success response"""
    status: Literal["success"]


class LoadingProgressPayload(BaseModel):
    """Loading progress event payload"""
    status: LoadingStatus
    file: str
    progress: int = Field(ge=0, le=100)
    loadId: Optional[str] = None
    loaded: Optional[int] = None
    total: Optional[int] = None
    message: Optional[str] = None


class GenerationUpdatePayload(BaseModel):
    """Generation update event payload"""
    token: str
    tps: Optional[str] = None
    numTokens: Optional[int] = None


class GenerationCompletePayload(BaseModel):
    """Generation complete event payload"""
    output: str
    generatedText: str
    tps: Optional[str] = None
    numTokens: Optional[int] = None


class ModelStatePayload(BaseModel):
    """Model state payload"""
    isReady: bool
    backend: Optional[BackendType] = None
    modelPath: Optional[str] = None


class WorkerReadyPayload(BaseModel):
    """Worker ready event payload"""
    backend: BackendType
    modelPath: str
    executionProvider: Optional[str] = None


# Hardware Detection Types

class GPUVendor(str, Enum):
    """GPU vendor identifiers"""
    NVIDIA = "nvidia"
    AMD = "amd"
    INTEL = "intel"
    UNKNOWN = "unknown"


class GPUType(str, Enum):
    """GPU type classification"""
    DISCRETE = "discrete"
    INTEGRATED = "integrated"
    UNKNOWN = "unknown"


class AccelerationBackend(str, Enum):
    """Hardware acceleration backend types"""
    CPU = "cpu"
    CUDA = "cuda"
    VULKAN = "vulkan"
    ROCM = "rocm"
    METAL = "metal"
    DIRECTML = "directml"
    NPU = "npu"
    UNKNOWN = "unknown"


# Pydantic Models for Hardware Info

class CPUInfo(BaseModel):
    """CPU device information"""
    name: str
    cores: int
    threads: int
    max_clock_speed_mhz: Optional[int] = None
    available: bool = True
    error: Optional[str] = None


class GPUInfo(BaseModel):
    """GPU device information"""
    name: str
    vendor: GPUVendor
    gpu_type: GPUType
    vram_mb: Optional[int] = None
    driver_version: Optional[str] = None
    available: bool = True
    error: Optional[str] = None


class NPUInfo(BaseModel):
    """NPU device information"""
    name: str
    driver_version: Optional[str] = None
    power_mode: Optional[str] = None
    available: bool = True
    error: Optional[str] = None


class HardwareCapabilities(BaseModel):
    """System hardware capabilities"""
    has_cuda: bool = False
    has_vulkan: bool = False
    has_rocm: bool = False
    has_metal: bool = False
    has_directml: bool = False
    has_npu: bool = False


class HardwareInfo(BaseModel):
    """Complete hardware information"""
    cpu: CPUInfo
    nvidia_gpus: List[GPUInfo]
    amd_gpus: List[GPUInfo]
    intel_gpus: List[GPUInfo]
    npu: Optional[NPUInfo] = None
    capabilities: HardwareCapabilities
    os_version: str


class BackendConfig(BaseModel):
    """Backend configuration for model loading"""
    backend: BackendType
    acceleration: AccelerationBackend
    device_index: int = 0
    settings: Dict[str, Any] = Field(default_factory=dict)


class SystemInfoResponse(BaseResponse):
    """System information response"""
    status: Literal["success"]
    hardware: HardwareInfo
    available_backends: List[BackendType]
    recommended_backend: BackendType


# Type aliases for clarity
MessagePayload = Dict[str, Any]
ResponsePayload = Dict[str, Any]

