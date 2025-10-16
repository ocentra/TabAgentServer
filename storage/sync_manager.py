"""
Sync Manager
============

Manages synchronization between extension IndexedDB and server database.

Sync Strategy:
- Extension uses IndexedDB (always works, offline capable)
- When native app available, sync to server database
- Server becomes source of truth when connected
- IndexedDB acts as local cache

Sync Operations:
- Push: IndexedDB → Server (when connecting)
- Pull: Server → IndexedDB (on demand)
- Merge: Resolve conflicts (last-write-wins)

Use Cases:
- User installs native app → Migrate IndexedDB data to server
- Extension offline → Queue changes, sync when reconnected
- Multiple devices → Server syncs across devices (future)
"""

import logging
from typing import Dict, List, Optional, Any
from datetime import datetime

from .chat_storage import get_chat_storage, Conversation
from .message_storage import get_message_storage, Message
from .embedding_storage import get_embedding_storage

logger = logging.getLogger(__name__)


class SyncStatus:
    """Sync status constants"""
    PENDING = "pending"
    SYNCING = "syncing"
    SYNCED = "synced"
    CONFLICT = "conflict"
    ERROR = "error"


class SyncManager:
    """
    Manages IndexedDB ↔ Server database synchronization.
    
    Handles migration, conflict resolution, and continuous sync.
    """
    
    def __init__(self):
        """Initialize sync manager"""
        self.chat_storage = get_chat_storage()
        self.message_storage = get_message_storage()
        self.embedding_storage = get_embedding_storage()
        logger.info("SyncManager initialized")
    
    def push_conversations(self, conversations: List[Dict[str, Any]]) -> Dict[str, Any]:
        """
        Push conversations from extension to server.
        
        Args:
            conversations: List of conversation dicts from IndexedDB
            
        Returns:
            Sync result with counts
        """
        created = 0
        updated = 0
        skipped = 0
        
        for conv_data in conversations:
            try:
                # Check if exists
                existing = self.chat_storage.get_conversation(conv_data['id'])
                
                if existing:
                    # Update if newer
                    if conv_data.get('updated_at', 0) > existing.updated_at:
                        self.chat_storage.update_conversation(
                            conversation_id=conv_data['id'],
                            title=conv_data.get('title'),
                            topic=conv_data.get('topic'),
                            domain=conv_data.get('domain'),
                            is_starred=conv_data.get('isStarred', False)
                        )
                        updated += 1
                    else:
                        skipped += 1
                else:
                    # Create new
                    self.chat_storage.create_conversation(
                        conversation_id=conv_data['id'],
                        user_id=conv_data.get('user_id', 'default'),
                        title=conv_data.get('title', 'Untitled'),
                        topic=conv_data.get('topic'),
                        domain=conv_data.get('domain')
                    )
                    created += 1
            
            except Exception as e:
                logger.error(f"Error syncing conversation {conv_data.get('id')}: {e}")
                continue
        
        logger.info(f"Conversations synced: {created} created, {updated} updated, {skipped} skipped")
        
        return {
            "synced": created + updated,
            "created": created,
            "updated": updated,
            "skipped": skipped,
            "total": len(conversations)
        }
    
    def push_messages(self, messages: List[Dict[str, Any]]) -> Dict[str, Any]:
        """
        Push messages from extension to server.
        
        Args:
            messages: List of message dicts from IndexedDB
            
        Returns:
            Sync result
        """
        # Use batch insert for efficiency
        count = self.message_storage.create_messages_batch(messages)
        
        logger.info(f"Messages synced: {count}")
        
        return {
            "synced": count,
            "total": len(messages)
        }
    
    def pull_conversations(
        self,
        user_id: Optional[str] = None,
        since: Optional[int] = None
    ) -> List[Dict[str, Any]]:
        """
        Pull conversations from server to extension.
        
        Args:
            user_id: Filter by user (optional)
            since: Only get conversations updated after this timestamp
            
        Returns:
            List of conversation dicts
        """
        conversations = self.chat_storage.list_conversations(user_id=user_id)
        
        # Filter by timestamp if provided
        if since:
            conversations = [c for c in conversations if c.updated_at > since]
        
        return [conv.to_dict() for conv in conversations]
    
    def pull_messages(
        self,
        conversation_id: str,
        since: Optional[int] = None
    ) -> List[Dict[str, Any]]:
        """
        Pull messages for a conversation.
        
        Args:
            conversation_id: Conversation ID
            since: Only get messages after this timestamp
            
        Returns:
            List of message dicts
        """
        messages = self.message_storage.get_conversation_messages(conversation_id)
        
        # Filter by timestamp if provided
        if since:
            messages = [m for m in messages if m.created_at > since]
        
        return [msg.to_dict() for msg in messages]


# Global singleton
_sync_manager: Optional[SyncManager] = None


def get_sync_manager() -> SyncManager:
    """Get global sync manager instance"""
    global _sync_manager
    if _sync_manager is None:
        _sync_manager = SyncManager()
    return _sync_manager

