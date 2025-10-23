"""
Chat History Endpoints
======================

Server-side chat history storage with embedding support.

Provides:
- POST /api/v1/conversations - Create conversation
- GET /api/v1/conversations - List conversations
- GET /api/v1/conversations/{id} - Get conversation
- DELETE /api/v1/conversations/{id} - Delete conversation
- POST /api/v1/conversations/{id}/messages - Add messages
- GET /api/v1/conversations/{id}/messages - Get messages
- POST /api/v1/conversations/search - Semantic search
- POST /api/v1/sync/push - Sync from IndexedDB
- POST /api/v1/sync/pull - Sync to IndexedDB

Storage Strategy:
- No native app: Extension uses IndexedDB
- With native app: Server DB + IndexedDB cache
- Automatic sync when native app connects

Related Files:
- storage/chat_storage.py - Conversation management
- storage/message_storage.py - Message storage
- storage/embedding_storage.py - Vector storage
- storage/sync_manager.py - Sync logic
"""

import logging
from typing import List, Dict, Any, Optional
from fastapi import APIRouter, HTTPException, status, Path as PathParam
from pydantic import BaseModel, Field

from storage import get_chat_storage, get_message_storage, get_sync_manager
from storage.chat_storage import Conversation
from storage.message_storage import Message
from ..constants import ErrorCode

logger = logging.getLogger(__name__)

router = APIRouter()


# =============================================================================
# REQUEST/RESPONSE MODELS
# =============================================================================

class CreateConversationRequest(BaseModel):
    """Create conversation request"""
    conversation_id: str = Field(..., description="Unique conversation ID")
    user_id: str = Field(default="default", description="User identifier")
    title: str = Field(..., description="Conversation title")
    topic: Optional[str] = Field(None, description="Conversation topic")
    domain: Optional[str] = Field(None, description="Domain/category")


class CreateMessageRequest(BaseModel):
    """Create message request"""
    message_id: str = Field(..., description="Unique message ID")
    role: str = Field(..., description="Message role (user/assistant/system)")
    content: str = Field(..., description="Message content")


class SyncPushRequest(BaseModel):
    """Sync push request"""
    conversations: List[Dict[str, Any]] = Field(..., description="Conversations from IndexedDB")
    messages: List[Dict[str, Any]] = Field(..., description="Messages from IndexedDB")


class SearchConversationsRequest(BaseModel):
    """Search conversations request"""
    query: str = Field(..., description="Search query")
    user_id: Optional[str] = Field(None, description="Filter by user")
    limit: int = Field(20, ge=1, le=100, description="Max results")


# =============================================================================
# CONVERSATION ENDPOINTS
# =============================================================================

# [ENDPOINT] POST /api/v1/conversations - Create conversation
@router.post(
    "/conversations",
    summary="Create Conversation",
    description="""
    ## Create new conversation
    
    Stores conversation in server database.
    Extension syncs IndexedDB to server when native app available.
    """,
    tags=["chat-history"]
)
async def create_conversation(request: CreateConversationRequest):
    """Create new conversation"""
    try:
        chat_storage = get_chat_storage()
        
        conversation = chat_storage.create_conversation(
            conversation_id=request.conversation_id,
            user_id=request.user_id,
            title=request.title,
            topic=request.topic,
            domain=request.domain
        )
        
        return {
            "status": "success",
            "conversation": conversation.to_dict()
        }
    
    except Exception as e:
        logger.error(f"Create conversation failed: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={"error": {"message": str(e), "type": ErrorCode.BACKEND_ERROR.value}}
        )


# [ENDPOINT] GET /api/v1/conversations - List conversations
@router.get(
    "/conversations",
    summary="List Conversations",
    description="""
    ## List all conversations
    
    Optional filtering by user_id.
    Ordered by most recent first.
    """,
    tags=["chat-history"]
)
async def list_conversations(
    user_id: Optional[str] = None,
    limit: int = 100,
    offset: int = 0
):
    """List conversations"""
    try:
        chat_storage = get_chat_storage()
        conversations = chat_storage.list_conversations(user_id, limit, offset)
        
        return {
            "conversations": [conv.to_dict() for conv in conversations],
            "total": len(conversations),
            "limit": limit,
            "offset": offset
        }
    
    except Exception as e:
        logger.error(f"List conversations failed: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={"error": {"message": str(e), "type": ErrorCode.BACKEND_ERROR.value}}
        )


# [ENDPOINT] GET /api/v1/conversations/{id} - Get conversation
@router.get(
    "/conversations/{conversation_id}",
    summary="Get Conversation",
    description="Get conversation by ID with all messages",
    tags=["chat-history"]
)
async def get_conversation(conversation_id: str = PathParam(..., description="Conversation ID")):
    """Get conversation by ID"""
    try:
        chat_storage = get_chat_storage()
        message_storage = get_message_storage()
        
        conversation = chat_storage.get_conversation(conversation_id)
        if not conversation:
            raise HTTPException(
                status_code=status.HTTP_404_NOT_FOUND,
                detail={"error": {"message": "Conversation not found", "type": ErrorCode.INVALID_MODEL.value}}
            )
        
        messages = message_storage.get_conversation_messages(conversation_id)
        
        return {
            "conversation": conversation.to_dict(),
            "messages": [msg.to_dict() for msg in messages],
            "message_count": len(messages)
        }
    
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Get conversation failed: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={"error": {"message": str(e), "type": ErrorCode.BACKEND_ERROR.value}}
        )


# [ENDPOINT] POST /api/v1/conversations/{id}/messages - Add messages to conversation
@router.post(
    "/conversations/{conversation_id}/messages",
    summary="Add Messages",
    description="Add messages to conversation",
    tags=["chat-history"]
)
async def add_messages(
    conversation_id: str = PathParam(..., description="Conversation ID"),
    messages: List[CreateMessageRequest] = Field(..., description="Messages to add")
):
    """Add messages to conversation"""
    try:
        message_storage = get_message_storage()
        
        created_messages = []
        for msg_req in messages:
            msg = message_storage.create_message(
                message_id=msg_req.message_id,
                conversation_id=conversation_id,
                role=msg_req.role,
                content=msg_req.content
            )
            created_messages.append(msg.to_dict())
        
        return {
            "status": "success",
            "messages": created_messages,
            "count": len(created_messages)
        }
    
    except Exception as e:
        logger.error(f"Add messages failed: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={"error": {"message": str(e), "type": ErrorCode.BACKEND_ERROR.value}}
        )


# [ENDPOINT] POST /api/v1/conversations/search - Search conversations
@router.post(
    "/conversations/search",
    summary="Search Conversations",
    description="""
    ## Search conversations by title/topic
    
    Future: Semantic search using conversation embeddings
    """,
    tags=["chat-history"]
)
async def search_conversations(request: SearchConversationsRequest):
    """Search conversations"""
    try:
        chat_storage = get_chat_storage()
        conversations = chat_storage.search_conversations(
            query=request.query,
            user_id=request.user_id,
            limit=request.limit
        )
        
        return {
            "query": request.query,
            "conversations": [conv.to_dict() for conv in conversations],
            "total": len(conversations)
        }
    
    except Exception as e:
        logger.error(f"Search conversations failed: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={"error": {"message": str(e), "type": ErrorCode.BACKEND_ERROR.value}}
        )


# =============================================================================
# SYNC ENDPOINTS (IndexedDB ↔ Server)
# =============================================================================

# [ENDPOINT] POST /api/v1/sync/push - Sync from IndexedDB to server
@router.post(
    "/sync/push",
    summary="Sync Push",
    description="""
    ## Push data from extension IndexedDB to server
    
    **Use when:**
    - User installs native app for first time
    - Reconnecting after offline period
    - Manual sync requested
    
    **What happens:**
    - Extension sends all IndexedDB data
    - Server stores in database
    - Server returns sync status
    - Extension can mark as synced
    
    **Conflict resolution:**
    - Last-write-wins (based on updated_at timestamp)
    - Newer data overwrites older
    """,
    tags=["sync"]
)
async def sync_push(request: SyncPushRequest):
    """Push data from extension to server"""
    try:
        sync_manager = get_sync_manager()
        
        # Sync conversations
        conv_result = sync_manager.push_conversations(request.conversations)
        
        # Sync messages
        msg_result = sync_manager.push_messages(request.messages)
        
        return {
            "status": "success",
            "conversations": conv_result,
            "messages": msg_result,
            "total_synced": conv_result["synced"] + msg_result["synced"]
        }
    
    except Exception as e:
        logger.error(f"Sync push failed: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={"error": {"message": str(e), "type": ErrorCode.BACKEND_ERROR.value}}
        )


# [ENDPOINT] POST /api/v1/sync/pull - Sync from server to IndexedDB
@router.post(
    "/sync/pull",
    summary="Sync Pull",
    description="""
    ## Pull data from server to extension IndexedDB
    
    **Use when:**
    - Extension initializes with native app
    - Periodic sync to get updates
    - Switching devices
    
    **Returns:**
    - All conversations for user
    - All messages per conversation
    - Extension updates IndexedDB
    """,
    tags=["sync"]
)
async def sync_pull(
    user_id: Optional[str] = None,
    since: Optional[int] = None
):
    """Pull data from server to extension"""
    try:
        sync_manager = get_sync_manager()
        
        # Pull conversations
        conversations = sync_manager.pull_conversations(user_id, since)
        
        # Pull messages for each conversation
        all_messages = []
        for conv in conversations:
            messages = sync_manager.pull_messages(conv['id'], since)
            all_messages.extend(messages)
        
        return {
            "status": "success",
            "conversations": conversations,
            "messages": all_messages,
            "counts": {
                "conversations": len(conversations),
                "messages": len(all_messages)
            }
        }
    
    except Exception as e:
        logger.error(f"Sync pull failed: {e}", exc_info=True)
        raise HTTPException(
            status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
            detail={"error": {"message": str(e), "type": ErrorCode.BACKEND_ERROR.value}}
        )

