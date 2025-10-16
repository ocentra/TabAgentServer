"""
Chat Storage Manager
====================

Manages conversations as knowledge graph nodes in ArangoDB.
Mirrors IndexedDB Chat class for consistency.

Each conversation is stored as a node with:
- type: "conversation"
- label: conversation title
- properties: {user_id, topic, domain, is_starred, metadata}
- embedding_id: reference to conversation embedding
"""

import logging
import time
from typing import List, Dict, Any, Optional
from .database import get_database

logger = logging.getLogger(__name__)


class ChatStorage:
    """
    Conversation storage manager.
    
    Mirrors IndexedDB Chat operations.
    """
    
    def __init__(self):
        self.db = get_database()
    
    def create_conversation(
        self,
        conversation_id: str,
        user_id: str,
        title: str,
        topic: Optional[str] = None,
        domain: Optional[str] = None,
        metadata: Optional[Dict[str, Any]] = None
    ) -> str:
        """
        Create a new conversation.
        
        Mirrors: IndexedDB Chat.create()
        
        Args:
            conversation_id: Unique conversation ID
            user_id: User who owns this conversation
            title: Conversation title
            topic: Optional topic classification
            domain: Optional domain classification
            metadata: Optional additional metadata
            
        Returns:
            Conversation node key
        """
        try:
            nodes = self.db.get_collection("nodes")
            
            now = int(time.time() * 1000)
            
            doc = {
                "_key": conversation_id,
                "type": "conversation",
                "label": title,
                "properties": {
                    "user_id": user_id,
                    "topic": topic,
                    "domain": domain,
                    "is_starred": False,
                    "metadata": metadata or {}
                },
                "embedding_id": None,
                "created_at": now,
                "updated_at": now
            }
            
            nodes.insert(doc)
            logger.info(f"Created conversation: {conversation_id}")
            return conversation_id
            
        except Exception as e:
            logger.error(f"Failed to create conversation: {e}")
            raise
    
    def get_conversation(self, conversation_id: str) -> Optional[Dict[str, Any]]:
        """
        Get conversation by ID.
        
        Mirrors: IndexedDB Chat.read()
        
        Args:
            conversation_id: Conversation ID
            
        Returns:
            Conversation data or None
        """
        try:
            nodes = self.db.get_collection("nodes")
            doc = nodes.get(conversation_id)
            
            if doc and doc.get("type") == "conversation":
                return self._format_conversation(doc)
            
            return None
            
        except Exception as e:
            logger.error(f"Failed to get conversation: {e}")
            return None
    
    def list_conversations(
        self,
        user_id: str,
        limit: int = 100,
        offset: int = 0
    ) -> List[Dict[str, Any]]:
        """
        List conversations for a user.
        
        Args:
            user_id: User ID
            limit: Maximum results
            offset: Result offset
            
        Returns:
            List of conversations
        """
        try:
            aql = """
                FOR node IN nodes
                    FILTER node.type == "conversation"
                    FILTER node.properties.user_id == @user_id
                    SORT node.updated_at DESC
                    LIMIT @offset, @limit
                    RETURN node
            """
            
            results = self.db.execute(aql, {
                "user_id": user_id,
                "limit": limit,
                "offset": offset
            })
            
            return [self._format_conversation(doc) for doc in results]
            
        except Exception as e:
            logger.error(f"Failed to list conversations: {e}")
            return []
    
    def search_conversations(
        self,
        user_id: str,
        query: str,
        limit: int = 20
    ) -> List[Dict[str, Any]]:
        """
        Search conversations by title or content.
        
        Args:
            user_id: User ID
            query: Search query
            limit: Maximum results
            
        Returns:
            Matching conversations
        """
        try:
            # Simple text search on title
            aql = """
                FOR node IN nodes
                    FILTER node.type == "conversation"
                    FILTER node.properties.user_id == @user_id
                    FILTER CONTAINS(LOWER(node.label), LOWER(@query))
                    SORT node.updated_at DESC
                    LIMIT @limit
                    RETURN node
            """
            
            results = self.db.execute(aql, {
                "user_id": user_id,
                "query": query,
                "limit": limit
            })
            
            return [self._format_conversation(doc) for doc in results]
            
        except Exception as e:
            logger.error(f"Failed to search conversations: {e}")
            return []
    
    def update_conversation(
        self,
        conversation_id: str,
        updates: Dict[str, Any]
    ) -> bool:
        """
        Update conversation.
        
        Mirrors: IndexedDB Chat.update()
        
        Args:
            conversation_id: Conversation ID
            updates: Fields to update (title, topic, domain, is_starred, metadata)
            
        Returns:
            True if successful
        """
        try:
            nodes = self.db.get_collection("nodes")
            doc = nodes.get(conversation_id)
            
            if not doc or doc.get("type") != "conversation":
                return False
            
            # Update allowed fields
            if "title" in updates:
                doc["label"] = updates["title"]
            
            # Update properties
            if "properties" not in doc:
                doc["properties"] = {}
            
            for key in ["topic", "domain", "is_starred", "metadata"]:
                if key in updates:
                    doc["properties"][key] = updates[key]
            
            # Update timestamp
            doc["updated_at"] = int(time.time() * 1000)
            
            nodes.update(doc)
            logger.info(f"Updated conversation: {conversation_id}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to update conversation: {e}")
            return False
    
    def delete_conversation(self, conversation_id: str) -> bool:
        """
        Delete conversation and all related messages.
        
        Mirrors: IndexedDB Chat.delete()
        
        Args:
            conversation_id: Conversation ID
            
        Returns:
            True if successful
        """
        try:
            # Delete all messages in this conversation
            aql = """
                FOR node IN nodes
                    FILTER node.type == "message"
                    FILTER node.properties.conversation_id == @conversation_id
                    REMOVE node IN nodes
            """
            
            self.db.execute(aql, {"conversation_id": conversation_id})
            
            # Delete conversation node
            nodes = self.db.get_collection("nodes")
            nodes.delete(conversation_id)
            
            logger.info(f"Deleted conversation: {conversation_id}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to delete conversation: {e}")
            return False
    
    def _format_conversation(self, doc: Dict[str, Any]) -> Dict[str, Any]:
        """
        Format conversation document for API response.
        
        Args:
            doc: ArangoDB document
            
        Returns:
            Formatted conversation
        """
        props = doc.get("properties", {})
        
        return {
            "id": doc["_key"],
            "user_id": props.get("user_id"),
            "title": doc.get("label", "Untitled"),
            "topic": props.get("topic"),
            "domain": props.get("domain"),
            "is_starred": props.get("is_starred", False),
            "embedding_id": doc.get("embedding_id"),
            "created_at": doc.get("created_at"),
            "updated_at": doc.get("updated_at"),
            "metadata": props.get("metadata", {})
        }


# Global singleton
_chat_storage: Optional[ChatStorage] = None


def get_chat_storage() -> ChatStorage:
    """
    Get global chat storage instance.
    
    Returns:
        ChatStorage singleton
    """
    global _chat_storage
    if _chat_storage is None:
        _chat_storage = ChatStorage()
    return _chat_storage
