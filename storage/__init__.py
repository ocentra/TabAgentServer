"""
Storage Module
==============

Hybrid storage system for chat history, embeddings, and knowledge graphs.

Storage Strategy:
- No native app: Extension uses IndexedDB only
- With native app: Server database (primary) + IndexedDB (cache)

Storage Backend: **ArangoDB** (Multi-Model Database)
- Document storage (JSON like IndexedDB)
- Graph queries (native AQL traversal)
- Vector search (future ArangoDB 3.12+)
- Mirrors IndexedDB structure for consistency

Components:
- database.py - ArangoDB connection (replaces SQLite)
- chat_storage.py - Conversation management (nodes)
- message_storage.py - Message storage (nodes)
- embedding_storage.py - Vector storage (embeddings collection)
- sync_manager.py - IndexedDB ↔ Server sync
- knowledge_graph/ - Graph utilities, entity extraction, graph RAG

Design Principles:
- ✅ Mirror IndexedDB structure (client/server consistency)
- ✅ Multi-model (document + graph + vector in ONE DB)
- ✅ Flexible schema (JSON properties, no migrations)
- ✅ Production-ready (ArangoDB is battle-tested)
- 🔜 Knowledge graph ready (native graph support)
"""

from .database import get_database, Database
from .chat_storage import ChatStorage, get_chat_storage
from .message_storage import MessageStorage, get_message_storage
from .embedding_storage import EmbeddingStorage, get_embedding_storage
from .sync_manager import SyncManager, get_sync_manager

__all__ = [
    "Database",
    "get_database",
    "ChatStorage",
    "get_chat_storage",
    "MessageStorage", 
    "get_message_storage",
    "EmbeddingStorage",
    "get_embedding_storage",
    "SyncManager",
    "get_sync_manager",
]

