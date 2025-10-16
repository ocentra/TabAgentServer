"""
API Request/Response Types

OpenAI-compatible API types that extend core message types.
"""

from typing import List, Optional, Dict, Any, Union
from pydantic import BaseModel, Field

from core.message_types import (
    ChatMessage,
    InferenceSettings,
    MessageRole,
    BackendType,
)
from core.recipe_types import RecipeType, ModelCapabilities
from .constants import OpenAIObject, FinishReason


# Request Models

class ChatCompletionRequest(BaseModel):
    """OpenAI chat completion request"""
    model: str = Field(..., description="Model identifier", examples=["Phi-3.5-mini", "llama-3.2-1b", "current"])
    messages: List[ChatMessage] = Field(..., description="List of chat messages")
    temperature: Optional[float] = Field(None, ge=0.0, le=2.0, description="Sampling temperature (0=deterministic, 2=very random)", examples=[0.7])
    max_tokens: Optional[int] = Field(None, ge=1, description="Maximum tokens to generate", examples=[512])
    stream: Optional[bool] = Field(False, description="Stream responses via SSE", examples=[False, True])
    stop: Optional[Union[str, List[str]]] = Field(None, description="Stop sequences")
    top_p: Optional[float] = Field(None, ge=0.0, le=1.0, description="Nucleus sampling", examples=[0.9])
    frequency_penalty: Optional[float] = Field(None, ge=-2.0, le=2.0, description="Frequency penalty")
    presence_penalty: Optional[float] = Field(None, ge=-2.0, le=2.0, description="Presence penalty")
    n: Optional[int] = Field(1, ge=1, description="Number of completions")
    user: Optional[str] = Field(None, description="User identifier")
    
    model_config = {
        "json_schema_extra": {
            "examples": [
                {
                    "model": "Phi-3.5-mini",
                    "messages": [
                        {"role": "user", "content": "Explain quantum computing in simple terms"}
                    ],
                    "temperature": 0.7,
                    "max_tokens": 500,
                    "stream": False
                }
            ]
        }
    }
    
    def to_inference_settings(self) -> InferenceSettings:
        """Convert to InferenceSettings for backend"""
        return InferenceSettings(
            temperature=self.temperature or 0.7,
            top_p=self.top_p or 0.9,
            max_new_tokens=self.max_tokens or 512,
        )


class CompletionRequest(BaseModel):
    """OpenAI text completion request"""
    model: str = Field(..., description="Model identifier", examples=["Phi-3.5-mini"])
    prompt: Union[str, List[str]] = Field(..., description="Text prompt(s)", examples=["Write a poem about AI"])
    temperature: Optional[float] = Field(None, ge=0.0, le=2.0, description="Sampling temperature", examples=[0.7])
    max_tokens: Optional[int] = Field(None, ge=1, description="Maximum tokens to generate", examples=[512])
    stream: Optional[bool] = Field(False, description="Stream responses via SSE", examples=[False])
    stop: Optional[Union[str, List[str]]] = Field(None, description="Stop sequences")
    
    model_config = {
        "json_schema_extra": {
            "examples": [
                {
                    "model": "current",
                    "prompt": "Write a short poem about artificial intelligence",
                    "temperature": 0.8,
                    "max_tokens": 200,
                    "stream": False
                }
            ]
        }
    }


# Response Models

class ChatCompletionMessage(BaseModel):
    """Chat completion message"""
    role: MessageRole
    content: str


class ChatCompletionChoice(BaseModel):
    """Single chat completion choice"""
    index: int
    message: ChatCompletionMessage
    finish_reason: FinishReason


class ChatCompletionUsage(BaseModel):
    """Token usage statistics"""
    prompt_tokens: int
    completion_tokens: int
    total_tokens: int


class ChatCompletionResponse(BaseModel):
    """Chat completion response"""
    id: str
    object: OpenAIObject = OpenAIObject.CHAT_COMPLETION
    created: int
    model: str
    choices: List[ChatCompletionChoice]
    usage: Optional[ChatCompletionUsage] = None


# Streaming Models

class ChatCompletionChunkDelta(BaseModel):
    """Delta in streaming chunk"""
    role: Optional[MessageRole] = None
    content: Optional[str] = None


class ChatCompletionChunkChoice(BaseModel):
    """Single streaming chunk choice"""
    index: int
    delta: ChatCompletionChunkDelta
    finish_reason: Optional[FinishReason] = None


class ChatCompletionChunk(BaseModel):
    """Streaming chat completion chunk"""
    id: str
    object: OpenAIObject = OpenAIObject.CHAT_COMPLETION_CHUNK
    created: int
    model: str
    choices: List[ChatCompletionChunkChoice]


# Model List Models

class ModelInfo(BaseModel):
    """Individual model information"""
    id: str
    object: OpenAIObject = OpenAIObject.MODEL
    created: int
    owned_by: str


class ModelListResponse(BaseModel):
    """List of models response"""
    object: OpenAIObject = OpenAIObject.LIST
    data: List[ModelInfo]


# Stats Models

class PerformanceStats(BaseModel):
    """Generation performance statistics"""
    time_to_first_token: Optional[float] = Field(None, description="TTFT in seconds")
    tokens_per_second: Optional[float] = Field(None, description="Average TPS")
    input_tokens: Optional[int] = Field(None, description="Input token count")
    output_tokens: Optional[int] = Field(None, description="Output token count")
    total_time: Optional[float] = Field(None, description="Total generation time in seconds")


# Health Models

class HealthStatus(BaseModel):
    """Health check response"""
    status: str
    model_loaded: bool
    backend: Optional[BackendType] = None
    uptime: Optional[float] = None


# Model Management Models

class ModelPullRequest(BaseModel):
    """Request to download a model"""
    model: str = Field(..., description="HuggingFace checkpoint or model identifier", examples=["microsoft/Phi-3.5-mini-instruct"])
    repo: Optional[str] = Field(None, description="HuggingFace repo")
    variant: Optional[str] = Field(None, description="Model variant/quantization", examples=["Q4_K_M", "fp16"])
    recipe: Optional[RecipeType] = Field(None, description="Recipe defining how to run the model", examples=["onnx-npu", "llama-cuda", "bitnet-gpu"])
    model_name: Optional[str] = Field(None, description="Custom model name for registration", examples=["user.MyModel"])
    capabilities: Optional[Dict[str, Any]] = Field(None, description="Model capabilities (reasoning, vision, audio)")
    
    model_config = {
        "json_schema_extra": {
            "examples": [
                {
                    "model": "microsoft/Phi-3.5-mini-instruct",
                    "recipe": "onnx-npu",
                    "model_name": "Phi-3.5-Mini-NPU"
                },
                {
                    "model": "unsloth/Phi-4-mini-instruct-GGUF:Q4_K_M",
                    "recipe": "llama-cuda",
                    "model_name": "user.Phi-4-Mini-GGUF"
                }
            ]
        }
    }


class ModelLoadRequest(BaseModel):
    """Request to load a model"""
    model: str = Field(..., description="Model identifier, path, or registered name", examples=["Phi-3.5-Mini-NPU", "path/to/model.gguf"])
    backend: Optional[BackendType] = Field(None, description="Specific backend to use")
    recipe: Optional[RecipeType] = Field(None, description="Recipe to use for loading", examples=["onnx-hybrid", "llama-vulkan"])
    
    model_config = {
        "json_schema_extra": {
            "examples": [
                {
                    "model": "Phi-3.5-Mini-NPU",
                    "recipe": "onnx-npu"
                },
                {
                    "model": "path/to/model.gguf",
                    "recipe": "llama-cuda"
                }
            ]
        }
    }


class ModelUnloadRequest(BaseModel):
    """Request to unload current model"""
    pass


class ModelDeleteRequest(BaseModel):
    """Request to delete a model"""
    model: str = Field(..., description="Model identifier to delete")


class ModelOperationResponse(BaseModel):
    """Response for model operations"""
    success: bool
    message: str
    model: Optional[str] = None
    backend: Optional[BackendType] = None

