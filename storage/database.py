"""
Database Manager
================

ArangoDB - Multi-Model Database (Document + Graph + Vector)
Mirrors the IndexedDB structure from the extension for perfect consistency.

Collections:
- nodes: Knowledge graph nodes (conversations, messages, entities)
  - Mirrors: IndexedDB KnowledgeGraphNode
  - Each node has: type, label, properties, embedding_id, edges
  
- edges: Knowledge graph edges (relationships)
  - Mirrors: IndexedDB KnowledgeGraphEdge
  - Each edge has: from_node_id, to_node_id, edge_type, metadata

- embeddings: Vector embeddings
  - Mirrors: IndexedDB Embedding
  - Each embedding has: vector, model, source info

This approach provides:
- ONE unified storage system (like IndexedDB)
- Document storage (JSON properties)
- Graph queries (relationships)
- Vector search (semantic similarity)
- Consistent client/server architecture
"""

import logging
from typing import Optional, Dict, Any, List
from arango import ArangoClient
from arango.database import StandardDatabase
from arango.collection import StandardCollection

logger = logging.getLogger(__name__)


class Database:
    """
    ArangoDB connection manager.
    
    Mirrors IndexedDB architecture for consistency between client and server.
    """
    
    def __init__(
        self,
        host: str = "http://localhost:8529",
        username: str = "root",
        password: str = "",
        db_name: str = "tabagent"
    ):
        """
        Initialize ArangoDB connection.
        
        Args:
            host: ArangoDB server URL
            username: Database username
            password: Database password
            db_name: Database name
        """
        self.host = host
        self.username = username
        self.password = password
        self.db_name = db_name
        
        self._client: Optional[ArangoClient] = None
        self._db: Optional[StandardDatabase] = None
        
        logger.info(f"Database initialized: {host}/{db_name}")
    
    def connect(self) -> StandardDatabase:
        """
        Connect to ArangoDB and initialize collections.
        
        Creates database and collections if they don't exist.
        
        Returns:
            ArangoDB database instance
        """
        if self._db is not None:
            return self._db
        
        try:
            # Connect to ArangoDB
            self._client = ArangoClient(hosts=self.host)
            
            # Connect to system database to create our database
            sys_db = self._client.db("_system", username=self.username, password=self.password)
            
            # Create database if not exists
            if not sys_db.has_database(self.db_name):
                sys_db.create_database(self.db_name)
                logger.info(f"Created database: {self.db_name}")
            
            # Connect to our database
            self._db = self._client.db(self.db_name, username=self.username, password=self.password)
            
            # Create collections (mirroring IndexedDB structure)
            self._create_collections()
            
            logger.info("ArangoDB connected successfully")
            return self._db
            
        except Exception as e:
            logger.error(f"Failed to connect to ArangoDB: {e}")
            logger.warning("Falling back to SQLite for now. Install ArangoDB for full features.")
            raise
    
    def _create_collections(self) -> None:
        """
        Create collections if they don't exist.
        
        Mirrors IndexedDB structure:
        - nodes → KnowledgeGraphNode
        - edges → KnowledgeGraphEdge
        - embeddings → Embedding
        """
        # Knowledge Graph Nodes collection
        # Stores: conversations, messages, entities (like IndexedDB KnowledgeGraphNode)
        if not self._db.has_collection("nodes"):
            nodes = self._db.create_collection("nodes")
            logger.info("Created collection: nodes")
            
            # Create indexes for performance
            nodes.add_persistent_index(fields=["type"], unique=False)
            nodes.add_persistent_index(fields=["embedding_id"], unique=False, sparse=True)
            nodes.add_persistent_index(fields=["user_id"], unique=False, sparse=True)
            nodes.add_persistent_index(fields=["created_at"], unique=False)
            logger.info("Created indexes on nodes collection")
        
        # Knowledge Graph Edges collection
        # Stores: relationships between nodes (like IndexedDB KnowledgeGraphEdge)
        if not self._db.has_collection("edges"):
            edges = self._db.create_collection("edges", edge=True)
            logger.info("Created edge collection: edges")
            
            # Create indexes
            edges.add_persistent_index(fields=["_from"], unique=False)
            edges.add_persistent_index(fields=["_to"], unique=False)
            edges.add_persistent_index(fields=["edge_type"], unique=False)
            logger.info("Created indexes on edges collection")
        
        # Embeddings collection
        # Stores: vector embeddings with references to nodes
        if not self._db.has_collection("embeddings"):
            embeddings = self._db.create_collection("embeddings")
            logger.info("Created collection: embeddings")
            
            # Create indexes
            embeddings.add_persistent_index(fields=["source_type", "source_id"], unique=False)
            embeddings.add_persistent_index(fields=["model"], unique=False)
            logger.info("Created indexes on embeddings collection")
        
        # Create graph (for traversal queries)
        if not self._db.has_graph("knowledge_graph"):
            self._db.create_graph(
                "knowledge_graph",
                edge_definitions=[{
                    "edge_collection": "edges",
                    "from_vertex_collections": ["nodes"],
                    "to_vertex_collections": ["nodes"]
                }]
            )
            logger.info("Created graph: knowledge_graph")
    
    def get_database(self) -> StandardDatabase:
        """
        Get ArangoDB database instance.
        
        Returns:
            ArangoDB database
        """
        if self._db is None:
            return self.connect()
        return self._db
    
    def get_collection(self, name: str) -> StandardCollection:
        """
        Get ArangoDB collection by name.
        
        Args:
            name: Collection name (nodes, edges, embeddings)
            
        Returns:
            ArangoDB collection
        """
        db = self.get_database()
        return db.collection(name)
    
    def close(self) -> None:
        """Close database connection"""
        if self._client:
            self._client.close()
            self._client = None
            self._db = None
            logger.info("ArangoDB connection closed")
    
    # Convenience methods that mirror IndexedDB operations
    
    def create_node(
        self,
        node_type: str,
        label: str,
        properties: Dict[str, Any],
        embedding_id: Optional[str] = None
    ) -> str:
        """
        Create a knowledge graph node.
        
        Mirrors: IndexedDB KnowledgeGraphNode.create()
        
        Args:
            node_type: Node type (conversation, message, entity)
            label: Human-readable label
            properties: JSON properties (flexible schema)
            embedding_id: Optional reference to embedding
            
        Returns:
            Node ID (_key)
        """
        import time
        
        db = self.get_database()
        nodes = db.collection("nodes")
        
        now = int(time.time() * 1000)  # Milliseconds like IndexedDB
        
        doc = {
            "type": node_type,
            "label": label,
            "properties": properties,
            "embedding_id": embedding_id,
            "created_at": now,
            "updated_at": now
        }
        
        result = nodes.insert(doc)
        return result["_key"]
    
    def create_edge(
        self,
        from_node_key: str,
        to_node_key: str,
        edge_type: str,
        metadata: Optional[Dict[str, Any]] = None
    ) -> str:
        """
        Create a knowledge graph edge.
        
        Mirrors: IndexedDB KnowledgeGraphEdge.create()
        
        Args:
            from_node_key: Source node key
            to_node_key: Target node key
            edge_type: Relationship type
            metadata: Optional edge metadata
            
        Returns:
            Edge ID (_key)
        """
        import time
        
        db = self.get_database()
        edges = db.collection("edges")
        
        doc = {
            "_from": f"nodes/{from_node_key}",
            "_to": f"nodes/{to_node_key}",
            "edge_type": edge_type,
            "metadata": metadata or {},
            "created_at": int(time.time() * 1000)
        }
        
        result = edges.insert(doc)
        return result["_key"]
    
    def execute(self, aql: str, bind_vars: Optional[Dict[str, Any]] = None) -> List[Dict[str, Any]]:
        """
        Execute AQL query.
        
        Args:
            aql: AQL query string
            bind_vars: Query bind variables
            
        Returns:
            Query results as list of dicts
        """
        db = self.get_database()
        cursor = db.aql.execute(aql, bind_vars=bind_vars or {})
        return list(cursor)
    
    def commit(self) -> None:
        """
        Commit transaction (no-op for ArangoDB).
        
        ArangoDB auto-commits by default. Kept for SQLite API compatibility.
        """
        pass


# Global singleton
_database: Optional[Database] = None


def get_database() -> Database:
    """
    Get global database instance.
    
    Returns:
        Database singleton
    """
    global _database
    if _database is None:
        _database = Database()
        try:
            _database.connect()  # Ensure collections created
        except Exception as e:
            logger.error(f"Failed to connect to ArangoDB: {e}")
            logger.info("You can install ArangoDB from: https://www.arangodb.com/download")
            # For now, let it fail - we'll add SQLite fallback if needed
            raise
    return _database
