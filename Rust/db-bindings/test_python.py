#!/usr/bin/env python3
"""
Simple test script to verify Python bindings work.
"""

import sys
sys.path.insert(0, "../target/wheels")

import embedded_db
import tempfile
import os

def test_basic_operations():
    print("🧪 Testing TabAgent Embedded Database Python Bindings")
    print("=" * 60)
    
    # Create a temporary database
    with tempfile.TemporaryDirectory() as tmpdir:
        db_path = os.path.join(tmpdir, "test.db")
        print(f"\n1. Creating database at: {db_path}")
        
        db = embedded_db.EmbeddedDB(db_path)
        print("   ✓ Database created successfully!")
        
        # Test Chat node insertion
        print("\n2. Inserting a Chat node...")
        chat = {
            "type": "Chat",
            "id": "chat_001",
            "title": "Test Conversation",
            "topic": "Testing Python Bindings",
            "created_at": 1697500000000,
            "updated_at": 1697500000000,
            "message_ids": [],
            "summary_ids": [],
            "metadata": "{}"
        }
        
        chat_id = db.insert_node(chat)
        print(f"   ✓ Chat inserted with ID: {chat_id}")
        
        # Test Chat retrieval
        print("\n3. Retrieving the Chat node...")
        retrieved_chat = db.get_node(chat_id)
        if retrieved_chat:
            print(f"   ✓ Chat retrieved: {retrieved_chat['title']}")
            print(f"     - Topic: {retrieved_chat['topic']}")
        else:
            print("   ✗ Chat not found!")
            return False
        
        # Test Message node insertion
        print("\n4. Inserting a Message node...")
        message = {
            "type": "Message",
            "id": "msg_001",
            "chat_id": chat_id,
            "sender": "user",
            "timestamp": 1697500000000,
            "text_content": "Hello from Python!",
            "attachment_ids": [],
            "metadata": "{}"
        }
        
        msg_id = db.insert_node(message)
        print(f"   ✓ Message inserted with ID: {msg_id}")
        
        # Test Edge insertion
        print("\n5. Creating an edge (Chat -> Message)...")
        edge_id = db.insert_edge(
            from_node=chat_id,
            to_node=msg_id,
            edge_type="CONTAINS",
            metadata=None
        )
        print(f"   ✓ Edge created with ID: {edge_id}")
        
        # Test Edge retrieval
        print("\n6. Retrieving the edge...")
        edge = db.get_edge(edge_id)
        if edge:
            print(f"   ✓ Edge retrieved: {edge['from_node']} -> {edge['to_node']}")
            print(f"     - Type: {edge['edge_type']}")
        else:
            print("   ✗ Edge not found!")
            return False
        
        # Test Embedding insertion
        print("\n7. Inserting an embedding...")
        embedding_id = db.insert_embedding(
            embedding_id="emb_001",
            vector=[0.1, 0.2, 0.3, 0.4] * 96,  # 384-dim vector
            model="test-model"
        )
        print(f"   ✓ Embedding inserted with ID: {embedding_id}")
        
        # Test stats
        print("\n8. Getting database statistics...")
        stats = db.stats()
        print(f"   ✓ Stats: {stats}")
        
        # Test deletion
        print("\n9. Deleting the message...")
        success = db.delete_node(msg_id)
        if success:
            print("   ✓ Message deleted successfully!")
        else:
            print("   ✗ Failed to delete message!")
            return False
        
        print("\n" + "=" * 60)
        print("✅ ALL TESTS PASSED!")
        print("=" * 60)
        return True

if __name__ == "__main__":
    try:
        success = test_basic_operations()
        sys.exit(0 if success else 1)
    except Exception as e:
        print(f"\n❌ TEST FAILED WITH ERROR:")
        print(f"   {type(e).__name__}: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

