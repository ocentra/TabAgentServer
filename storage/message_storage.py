"""
Message Storage Manager
=======================

Manages messages as knowledge graph nodes in ArangoDB.
Mirrors IndexedDB Message class for consistency.

Each message is stored as a node with:
- type: "message"
- label: truncated content preview
- properties: {conversation_id, role, content, metadata}
- embedding_id: reference to message embedding

Messages are linked to conversations via:
- Properties reference (conversation_id in properties)
- Optional graph edges (for explicit relationships)
"""

import logging
import time
from typing import List, Dict, Any, Optional
from .database import get_database

logger = logging.getLogger(__name__)


class MessageStorage:
    """
    Message storage manager.
    
    Mirrors IndexedDB Message operations.
    """
    
    def __init__(self):
        self.db = get_database()
    
    def create_message(
        self,
        message_id: str,
        conversation_id: str,
        role: str,
        content: str,
        metadata: Optional[Dict[str, Any]] = None
    ) -> str:
        """
        Create a new message.
        
        Mirrors: IndexedDB Message.create()
        
        Args:
            message_id: Unique message ID
            conversation_id: Parent conversation ID
            role: Message role (user, assistant, system)
            content: Message content
            metadata: Optional additional metadata
            
        Returns:
            Message node key
        """
        try:
            nodes = self.db.get_collection("nodes")
            
            now = int(time.time() * 1000)
            
            # Create label from content preview (first 50 chars)
            label = content[:50] + ("..." if len(content) > 50 else "")
            
            doc = {
                "_key": message_id,
                "type": "message",
                "label": label,
                "properties": {
                    "conversation_id": conversation_id,
                    "role": role,
                    "content": content,
                    "metadata": metadata or {}
                },
                "embedding_id": None,
                "created_at": now,
                "updated_at": now
            }
            
            nodes.insert(doc)
            
            # Update conversation's updated_at timestamp
            self._update_conversation_timestamp(conversation_id, now)
            
            logger.info(f"Created message: {message_id}")
            return message_id
            
        except Exception as e:
            logger.error(f"Failed to create message: {e}")
            raise
    
    def create_messages_bulk(
        self,
        messages: List[Dict[str, Any]]
    ) -> List[str]:
        """
        Create multiple messages in bulk.
        
        Args:
            messages: List of message dicts with id, conversation_id, role, content, metadata
            
        Returns:
            List of created message IDs
        """
        try:
            nodes = self.db.get_collection("nodes")
            now = int(time.time() * 1000)
            
            docs = []
            conversation_ids = set()
            
            for msg in messages:
                label = msg["content"][:50] + ("..." if len(msg["content"]) > 50 else "")
                
                doc = {
                    "_key": msg["id"],
                    "type": "message",
                    "label": label,
                    "properties": {
                        "conversation_id": msg["conversation_id"],
                        "role": msg["role"],
                        "content": msg["content"],
                        "metadata": msg.get("metadata", {})
                    },
                    "embedding_id": None,
                    "created_at": now,
                    "updated_at": now
                }
                
                docs.append(doc)
                conversation_ids.add(msg["conversation_id"])
            
            # Bulk insert
            results = nodes.insert_many(docs)
            
            # Update conversation timestamps
            for conv_id in conversation_ids:
                self._update_conversation_timestamp(conv_id, now)
            
            logger.info(f"Created {len(docs)} messages in bulk")
            return [msg["id"] for msg in messages]
            
        except Exception as e:
            logger.error(f"Failed to create messages in bulk: {e}")
            raise
    
    def get_message(self, message_id: str) -> Optional[Dict[str, Any]]:
        """
        Get message by ID.
        
        Mirrors: IndexedDB Message.read()
        
        Args:
            message_id: Message ID
            
        Returns:
            Message data or None
        """
        try:
            nodes = self.db.get_collection("nodes")
            doc = nodes.get(message_id)
            
            if doc and doc.get("type") == "message":
                return self._format_message(doc)
            
            return None
            
        except Exception as e:
            logger.error(f"Failed to get message: {e}")
            return None
    
    def list_messages(
        self,
        conversation_id: str,
        limit: int = 100,
        offset: int = 0
    ) -> List[Dict[str, Any]]:
        """
        List messages in a conversation.
        
        Args:
            conversation_id: Conversation ID
            limit: Maximum results
            offset: Result offset
            
        Returns:
            List of messages ordered by created_at
        """
        try:
            aql = """
                FOR node IN nodes
                    FILTER node.type == "message"
                    FILTER node.properties.conversation_id == @conversation_id
                    SORT node.created_at ASC
                    LIMIT @offset, @limit
                    RETURN node
            """
            
            results = self.db.execute(aql, {
                "conversation_id": conversation_id,
                "limit": limit,
                "offset": offset
            })
            
            return [self._format_message(doc) for doc in results]
            
        except Exception as e:
            logger.error(f"Failed to list messages: {e}")
            return []
    
    def update_message(
        self,
        message_id: str,
        updates: Dict[str, Any]
    ) -> bool:
        """
        Update message.
        
        Mirrors: IndexedDB Message.update()
        
        Args:
            message_id: Message ID
            updates: Fields to update (content, metadata)
            
        Returns:
            True if successful
        """
        try:
            nodes = self.db.get_collection("nodes")
            doc = nodes.get(message_id)
            
            if not doc or doc.get("type") != "message":
                return False
            
            # Update content
            if "content" in updates:
                doc["properties"]["content"] = updates["content"]
                doc["label"] = updates["content"][:50] + ("..." if len(updates["content"]) > 50 else "")
            
            # Update metadata
            if "metadata" in updates:
                doc["properties"]["metadata"] = updates["metadata"]
            
            # Update timestamp
            doc["updated_at"] = int(time.time() * 1000)
            
            nodes.update(doc)
            logger.info(f"Updated message: {message_id}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to update message: {e}")
            return False
    
    def delete_message(self, message_id: str) -> bool:
        """
        Delete message.
        
        Mirrors: IndexedDB Message.delete()
        
        Args:
            message_id: Message ID
            
        Returns:
            True if successful
        """
        try:
            nodes = self.db.get_collection("nodes")
            nodes.delete(message_id)
            
            logger.info(f"Deleted message: {message_id}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to delete message: {e}")
            return False
    
    def _update_conversation_timestamp(self, conversation_id: str, timestamp: int) -> None:
        """
        Update conversation's updated_at timestamp.
        
        Args:
            conversation_id: Conversation ID
            timestamp: New timestamp
        """
        try:
            nodes = self.db.get_collection("nodes")
            conv = nodes.get(conversation_id)
            
            if conv and conv.get("type") == "conversation":
                conv["updated_at"] = timestamp
                nodes.update(conv)
                
        except Exception as e:
            logger.warning(f"Failed to update conversation timestamp: {e}")
    
    def _format_message(self, doc: Dict[str, Any]) -> Dict[str, Any]:
        """
        Format message document for API response.
        
        Args:
            doc: ArangoDB document
            
        Returns:
            Formatted message
        """
        props = doc.get("properties", {})
        
        return {
            "id": doc["_key"],
            "conversation_id": props.get("conversation_id"),
            "role": props.get("role"),
            "content": props.get("content", ""),
            "embedding_id": doc.get("embedding_id"),
            "created_at": doc.get("created_at"),
            "metadata": props.get("metadata", {})
        }


# Global singleton
_message_storage: Optional[MessageStorage] = None


def get_message_storage() -> MessageStorage:
    """
    Get global message storage instance.
    
    Returns:
        MessageStorage singleton
    """
    global _message_storage
    if _message_storage is None:
        _message_storage = MessageStorage()
    return _message_storage
