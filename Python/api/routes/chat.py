"""
Chat Completions Endpoints

OpenAI-compatible chat and text completion endpoints.
Zero string literals - all constants from api.constants.
"""

import json
import logging
from fastapi import APIRouter, HTTPException, status
from fastapi.responses import StreamingResponse

from ..types import (
    ChatCompletionRequest,
    ChatCompletionChunk,
    CompletionRequest,
)
from ..constants import (
    MediaType,
    HTTPHeader,
    CacheControl,
    ConnectionType,
    SSEPrefix,
    SSEMessage,
    ErrorCode,
)
from ..backend_manager import get_backend_manager

logger = logging.getLogger(__name__)

router = APIRouter()


# [ENDPOINT] POST /api/v1/chat/completions - Chat completion (OpenAI-compatible)
@router.post(
    "/chat/completions",
    summary="Chat Completion",
    description="""
    ## Generate AI responses from chat messages
    
    **IMPORTANT: Load a model first!**
    
    ### Quick Start Workflow:
    
    1️⃣ **Load a model** (only needed once):
    ```bash
    POST /api/v1/load
    {
      "model": "path/to/model.gguf"
    }
    ```
    
    2️⃣ **Then use this endpoint**:
    ```bash
    POST /api/v1/chat/completions
    {
      "model": "current",
      "messages": [{"role": "user", "content": "Hello!"}]
    }
    ```
    
    ### Streaming Support:
    Set `"stream": true` for real-time token generation via Server-Sent Events (SSE).
    
    ### Common Errors:
    - **503 No model loaded**: Run `POST /api/v1/load` first
    - **500 Generation failed**: Check model is compatible with backend
    """,
    responses={
        200: {
            "description": "Successful completion",
            "content": {
                "application/json": {
                    "example": {
                        "id": "chatcmpl-123",
                        "object": "chat.completion",
                        "created": 1677652288,
                        "model": "current",
                        "choices": [{
                            "index": 0,
                            "message": {
                                "role": "assistant",
                                "content": "Hello! How can I help you today?"
                            },
                            "finish_reason": "stop"
                        }],
                        "usage": {
                            "prompt_tokens": 10,
                            "completion_tokens": 8,
                            "total_tokens": 18
                        }
                    }
                }
            }
        },
        503: {
            "description": "No model loaded - Load a model first using POST /api/v1/load",
            "content": {
                "application/json": {
                    "example": {
                        "error": {
                            "message": "No model loaded",
                            "type": "model_not_loaded",
                            "hint": "Load a model first: POST /api/v1/load {\"model\": \"path/to/model.gguf\"}"
                        }
                    }
                }
            }
        },
        500: {
            "description": "Generation failed",
            "content": {
                "application/json": {
                    "example": {
                        "error": {
                            "message": "Generation failed: Backend error",
                            "type": "generation_failed"
                        }
                    }
                }
            }
        }
    }
)
async def chat_completions(request: ChatCompletionRequest):
    """
    Create chat completion
    
    OpenAI-compatible endpoint for chat completions with streaming support.
    
    Args:
        request: Chat completion request
        
    Returns:
        Chat completion response (streaming or non-streaming)
        
    Raises:
        HTTPException: If no model loaded or generation fails
    """
    manager = get_backend_manager()
    
    # Check if model is loaded
    if not manager.is_model_loaded():
        logger.error("Chat completion requested but no model loaded")
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail={
                "error": {
                    "message": "No model loaded",
                    "type": ErrorCode.MODEL_NOT_LOADED.value,
                    "code": ErrorCode.MODEL_NOT_LOADED.value,
                    "hint": "Load a model first: POST /api/v1/load with {\"model\": \"path/to/model.gguf\"}"
                }
            }
        )
    
    logger.info(f"Chat completion: model={request.model}, stream={request.stream}")
    
    try:
        # Non-streaming response
        if not request.stream:
            response = await manager.chat_completion(request)
            return response
        
        # Streaming response
        else:
            return StreamingResponse(
                _stream_chat_completion(manager, request),
                media_type=MediaType.EVENT_STREAM.value,
                headers={
                    HTTPHeader.CACHE_CONTROL.value: CacheControl.NO_CACHE.value,
                    HTTPHeader.CONNECTION.value: ConnectionType.KEEP_ALIVE.value,
                }
            )
    
    except Exception as e:
        logger.error(f"Generation failed: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.GENERATION_FAILED.value,
                    "code": ErrorCode.GENERATION_FAILED.value,
                }
            }
        )


@router.post("/completions")
async def completions(request: CompletionRequest):
    """
    Create text completion
    
    OpenAI-compatible endpoint for text completions.
    Converts prompt to chat messages internally.
    
    Args:
        request: Completion request
        
    Returns:
        Completion response (streaming or non-streaming)
        
    Raises:
        HTTPException: If no model loaded or generation fails
    """
    manager = get_backend_manager()
    
    # Check if model is loaded
    if not manager.is_model_loaded():
        logger.error("Completion requested but no model loaded")
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail={
                "error": {
                    "message": "No model loaded",
                    "type": ErrorCode.MODEL_NOT_LOADED.value,
                    "code": ErrorCode.MODEL_NOT_LOADED.value,
                }
            }
        )
    
    logger.info(f"Text completion: model={request.model}, stream={request.stream}")
    
    try:
        # Convert prompt to chat messages
        from core.message_types import ChatMessage, MessageRole
        
        # Handle both string and list of strings
        prompt_text = request.prompt if isinstance(request.prompt, str) else "\n".join(request.prompt)
        
        # Create chat message from prompt
        messages = [ChatMessage(role=MessageRole.USER, content=prompt_text)]
        
        # Convert to ChatCompletionRequest
        from ..types import ChatCompletionRequest
        chat_request = ChatCompletionRequest(
            model=request.model,
            messages=messages,
            temperature=request.temperature,
            max_tokens=request.max_tokens,
            stream=request.stream,
            stop=request.stop
        )
        
        # Route to chat completions logic
        if not request.stream:
            response = await manager.chat_completion(chat_request)
            return response
        else:
            return StreamingResponse(
                _stream_chat_completion(manager, chat_request),
                media_type=MediaType.EVENT_STREAM.value,
                headers={
                    HTTPHeader.CACHE_CONTROL.value: CacheControl.NO_CACHE.value,
                    HTTPHeader.CONNECTION.value: ConnectionType.KEEP_ALIVE.value,
                }
            )
    
    except Exception as e:
        logger.error(f"Completion failed: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.GENERATION_FAILED.value,
                    "code": ErrorCode.GENERATION_FAILED.value,
                }
            }
        )


@router.post("/responses")
async def responses(request: dict):
    """
    Create responses using alternative API format.
    
    Alternative to /chat/completions with different request/response structure.
    Accepts either string input or list of message dictionaries.
    
    Args:
        request: Response request (flexible input format)
        
    Returns:
        Response (streaming or non-streaming)
        
    Raises:
        HTTPException: If no model loaded or generation fails
    """
    manager = get_backend_manager()
    
    # Check if model is loaded
    if not manager.is_model_loaded():
        logger.error("Response requested but no model loaded")
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail={
                "error": {
                    "message": "No model loaded",
                    "type": ErrorCode.MODEL_NOT_LOADED.value,
                }
            }
        )
    
    try:
        from core.message_types import ChatMessage, MessageRole
        from ..types import ChatCompletionRequest
        
        # Extract input (can be string or list of messages)
        input_data = request.get("input", "")
        model_name = request.get("model", "default")
        stream = request.get("stream", False)
        
        # Convert input to chat messages
        if isinstance(input_data, str):
            messages = [ChatMessage(role=MessageRole.USER, content=input_data)]
        elif isinstance(input_data, list):
            # Assume list of message dicts
            messages = [
                ChatMessage(role=msg.get("role", "user"), content=msg.get("content", ""))
                for msg in input_data
            ]
        else:
            raise ValueError("Invalid input format. Expected string or list of messages")
        
        # Create chat request
        chat_request = ChatCompletionRequest(
            model=model_name,
            messages=messages,
            temperature=request.get("temperature"),
            max_tokens=request.get("max_output_tokens") or request.get("max_tokens"),
            stream=stream,
            top_p=request.get("top_p")
        )
        
        # Route to chat completions logic
        if not stream:
            response = await manager.chat_completion(chat_request)
            return response
        else:
            return StreamingResponse(
                _stream_chat_completion(manager, chat_request),
                media_type=MediaType.EVENT_STREAM.value,
                headers={
                    HTTPHeader.CACHE_CONTROL.value: CacheControl.NO_CACHE.value,
                    HTTPHeader.CONNECTION.value: ConnectionType.KEEP_ALIVE.value,
                }
            )
    
    except Exception as e:
        logger.error(f"Response generation failed: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={
                "error": {
                    "message": str(e),
                    "type": ErrorCode.GENERATION_FAILED.value,
                }
            }
        )


async def _stream_chat_completion(manager, request: ChatCompletionRequest):
    """
    Generate streaming chat completion
    
    Args:
        manager: Backend manager
        request: Chat completion request
        
    Yields:
        SSE-formatted chunks
    """
    try:
        async for chunk in manager.chat_completion_stream(request):
            # Serialize chunk to JSON
            chunk_json = chunk.model_dump_json(exclude_none=True)
            
            # Format as SSE
            sse_line = f"{SSEPrefix.DATA.value}{chunk_json}\n\n"
            yield sse_line
        
        # Send [DONE] message
        done_line = f"{SSEPrefix.DATA.value}{SSEMessage.DONE.value}\n\n"
        yield done_line
    
    except Exception as e:
        logger.error(f"Streaming error: {e}", exc_info=True)
        # Send error as SSE
        error_data = {
            "error": {
                "message": str(e),
                "type": ErrorCode.GENERATION_FAILED.value,
            }
        }
        error_line = f"{SSEPrefix.DATA.value}{json.dumps(error_data)}\n\n"
        yield error_line
