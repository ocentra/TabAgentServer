"""
Message field name constants - SINGLE SOURCE OF TRUTH

⚠️  CRITICAL: MUST stay 100% synced with Rust tabagent-rs/common/src/actions.rs
⚠️  Both Python and Rust MUST have IDENTICAL constant names and values
⚠️  Any change here MUST be reflected in Rust, and vice versa

Eliminates string literals in Python code for message field names.
All clients (Chrome Native Messaging, FastAPI, WebRTC) speak the same language.

NO STRING LITERALS - Always import and use these constants.
"""

# ============================================================
# REQUEST FIELDS - Incoming message field names
# ============================================================

# Core fields
ACTION = "action"
MODEL_PATH = "modelPath"
MODEL_ID = "modelId"
MODEL_NAME = "model_name"
MODEL = "model"
REPO_ID = "repoId"
FILE_PATH = "filePath"
FILE_NAME = "fileName"
MODEL_FILE = "modelFile"
MODEL_SIZE = "modelSize"
TOTAL_LAYERS = "totalLayers"
CHECKPOINT = "checkpoint"
MODEL_TYPE = "modelType"
TYPE = "type"
SIZE_GB = "sizeGb"
LABELS = "labels"
SETTINGS = "settings"

# Generation fields
MESSAGES = "messages"
TEMPERATURE = "temperature"
TOP_K = "top_k"
TOP_P = "top_p"
MAX_NEW_TOKENS = "max_new_tokens"
REPETITION_PENALTY = "repetition_penalty"
DO_SAMPLE = "do_sample"

# Model loading fields
IS_BITNET = "isBitNet"
DTYPE = "dtype"
TASK = "task"
GPU_LAYERS = "gpuLayers"
CPU_LAYERS = "cpuLayers"

# ============================================================
# RESPONSE FIELDS - Outgoing message field names
# ============================================================

# Core response fields
STATUS = "status"
MESSAGE = "message"
PAYLOAD = "payload"
ERROR = "error"

# Model state fields
IS_READY = "isReady"
BACKEND = "backend"
LOADED_TO = "loadedTo"
VRAM_USED = "vramUsed"
RAM_USED = "ramUsed"
VRAM_FREED = "vramFreed"
RAM_FREED = "ramFreed"
CONFIG = "config"

# Collection fields
MODELS = "models"
COUNT = "count"
LOADED_COUNT = "loadedCount"
UNLOADED_MODELS = "unloadedModels"
DOWNLOADS = "downloads"
RECEIVED = "received"

# Progress fields
PROGRESS = "progress"
FILE = "file"
LOAD_ID = "loadId"
LOADED = "loaded"
TOTAL = "total"
NUM_TOKENS = "numTokens"
TPS = "tps"

# Generation output fields
TOKEN = "token"
OUTPUT = "output"
GENERATED_TEXT = "generatedText"

# Hardware info fields
HARDWARE = "hardware"
AVAILABLE_BACKENDS = "available_backends"
RECOMMENDED_BACKEND = "recommended_backend"
EXECUTION_PROVIDER = "executionProvider"

# LM Studio fields
INSTALLED = "installed"
BOOTSTRAPPED = "bootstrapped"
SERVER_RUNNING = "server_running"
API_ENDPOINT = "api_endpoint"

# Memory & resource fields (synced with Rust)
CURRENT_MODEL = "currentModel"
RAM = "ram"
VRAM = "vram"
LOADED_MODELS_COUNT = "loadedModelsCount"
MEMORY_USED_BY_MODELS = "memoryUsedByModels"
CACHED = "cached"
DEFAULT_MODEL = "defaultModel"
TIMESTAMP = "timestamp"
USED = "used"
AVAILABLE = "available"
# Note: TOTAL already defined above in Progress fields

# ============================================================
# STATUS VALUES - Status code constants
# ============================================================

STATUS_SUCCESS = "success"
STATUS_ERROR = "error"
STATUS_PENDING = "pending"

# ============================================================
# ROLE VALUES - Message role constants
# ============================================================

ROLE_SYSTEM = "system"
ROLE_USER = "user"
ROLE_ASSISTANT = "assistant"

# ============================================================
# LOADING STATUS VALUES - Model loading status
# ============================================================

LOADING_INITIATE = "initiate"
LOADING_PROGRESS = "progress"
LOADING_DONE = "done"
LOADING_ERROR = "error"
LOADING_CACHED = "cached"

# ============================================================
# DOWNLOAD STATUS VALUES - Download state tracking
# ============================================================

DOWNLOAD_DOWNLOADING = "downloading"
DOWNLOAD_COMPLETED = "completed"
DOWNLOAD_FAILED = "failed"
DOWNLOAD_CANCELLED = "cancelled"

