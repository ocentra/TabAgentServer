"""
API Constants

All string literals for API endpoints, headers, and configuration.
Single source of truth for API-related constants.
"""

from enum import Enum


class APIVersion(str, Enum):
    """API version identifiers"""
    V0 = "v0"
    V1 = "v1"


class APIPrefix(str, Enum):
    """API path prefixes"""
    V0 = "/api/v0"
    V1 = "/api/v1"


class EndpointPath(str, Enum):
    """API endpoint paths (relative to prefix)"""
    # Health
    HEALTH = "/health"
    
    # Models
    MODELS = "/models"
    MODELS_BY_ID = "/models/{model_id}"
    MODELS_PULL = "/models/pull"
    MODELS_LOAD = "/models/load"
    MODELS_UNLOAD = "/models/unload"
    
    # Chat
    CHAT_COMPLETIONS = "/chat/completions"
    COMPLETIONS = "/completions"
    
    # Stats
    STATS = "/stats"


class HTTPHeader(str, Enum):
    """HTTP header names"""
    CONTENT_TYPE = "Content-Type"
    CACHE_CONTROL = "Cache-Control"
    CONNECTION = "Connection"
    ACCEPT = "Accept"


class MediaType(str, Enum):
    """Media type constants"""
    JSON = "application/json"
    EVENT_STREAM = "text/event-stream"
    PLAIN_TEXT = "text/plain"


class CacheControl(str, Enum):
    """Cache control directives"""
    NO_CACHE = "no-cache"
    NO_STORE = "no-store"
    MUST_REVALIDATE = "must-revalidate"


class ConnectionType(str, Enum):
    """Connection type values"""
    KEEP_ALIVE = "keep-alive"
    CLOSE = "close"


class OpenAIObject(str, Enum):
    """OpenAI API object type identifiers"""
    CHAT_COMPLETION = "chat.completion"
    CHAT_COMPLETION_CHUNK = "chat.completion.chunk"
    COMPLETION = "text_completion"
    MODEL = "model"
    LIST = "list"


class FinishReason(str, Enum):
    """Completion finish reasons"""
    STOP = "stop"
    LENGTH = "length"
    ERROR = "error"
    CANCELLED = "cancelled"


class ServerConfig(str, Enum):
    """Server configuration constants"""
    DEFAULT_HOST = "0.0.0.0"
    DEFAULT_PORT = "8000"
    LOG_FORMAT = "%(asctime)s [%(levelname)s] %(message)s"


class SSEPrefix(str, Enum):
    """Server-Sent Events prefixes"""
    DATA = "data: "
    EVENT = "event: "
    ID = "id: "
    RETRY = "retry: "


class SSEMessage(str, Enum):
    """Special SSE messages"""
    DONE = "[DONE]"


class APITag(str, Enum):
    """OpenAPI documentation tags"""
    HEALTH = "health"
    MODELS = "models"
    CHAT = "chat"
    COMPLETIONS = "completions"
    STATS = "stats"


class ErrorCode(str, Enum):
    """Application error codes"""
    # Client errors
    INVALID_REQUEST = "invalid_request"
    INVALID_MODEL = "invalid_model"
    CONTEXT_LENGTH_EXCEEDED = "context_length_exceeded"
    
    # Server errors
    MODEL_NOT_LOADED = "model_not_loaded"
    BACKEND_ERROR = "backend_error"
    GENERATION_FAILED = "generation_failed"
    NOT_IMPLEMENTED = "not_implemented"
    
    # System errors
    HARDWARE_ERROR = "hardware_error"
    RESOURCE_EXHAUSTED = "resource_exhausted"


class LogLevel(str, Enum):
    """Logging level names"""
    DEBUG = "DEBUG"
    INFO = "INFO"
    WARNING = "WARNING"
    ERROR = "ERROR"
    CRITICAL = "CRITICAL"

