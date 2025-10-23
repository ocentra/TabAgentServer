
# TabAgent Embedded Database - Architecture Study

## Project Context

### Our Current System (IndexedDB - Extension/Client)

**Location:** `src/DB/`

**Core Files Analysis:**
- `idbKnowledgeGraph.ts` (564 lines) - PRIMARY REFERENCE
  - Implements the core KnowledgeGraphNode and KnowledgeGraphEdge classes
  - Handles node/edge CRUD operations with worker-based storage
  - Manages adjacency lists for efficient traversal
- `idbEmbedding.ts` (234 lines) - Vector storage
  - Stores embeddings as ArrayBuffer for efficient memory usage
  - Links embeddings to nodes via embedding_id
  - Supports various input formats (number[], Float32Array, ArrayBuffer)
- `idbChat.ts` (24.6KB / 583 lines) - Chat extends KnowledgeGraphNode
  - Specialized chat functionality with message management
  - Handles chat-specific properties and operations
- `idbMessage.ts` (17.0KB / 432 lines) - Message extends KnowledgeGraphNode
  - Message-specific functionality with content handling
  - Links to parent chats and manages message metadata
- `idbBase.ts` (1.8KB / 68 lines) - Base CRUD operations
  - Provides common CRUD functionality for all entities
  - Abstract base class for consistent database operations
- `vectorUtils.ts` (3.2KB / 89 lines) - Vector similarity
  - Implements cosine similarity and Euclidean distance calculations
  - Provides nearest neighbor search functionality
  - Handles vector parsing from various formats

**Detailed Data Model Analysis:**
```typescript
// KnowledgeGraphNode Structure from idbKnowledgeGraph.ts
KnowledgeGraphNode {
  id: string                    // UUID for unique identification
  type: string                  // Entity type: "conversation", "message", "entity"
  label: string                 // Human-readable label for the node
  properties_json?: string      // JSON string containing flexible schema properties
  embedding_id?: string         // Reference to associated embedding
  edgesOut: KnowledgeGraphEdge[]// Outgoing edges cached on the node
  edgesIn: KnowledgeGraphEdge[] // Incoming edges cached on the node
  created_at: number            // Unix timestamp of creation
  updated_at: number            // Unix timestamp of last update
}

// KnowledgeGraphEdge Structure from idbKnowledgeGraph.ts
KnowledgeGraphEdge {
  id: string                    // UUID for unique identification
  from_node_id: string          // Source node reference
  to_node_id: string            // Target node reference
  edge_type: string             // Type of relationship between nodes
  metadata_json?: string        // JSON string containing edge metadata
  created_at: number            // Unix timestamp of creation
}

// Embedding Structure from idbEmbedding.ts
Embedding {
  id: string                    // UUID for unique identification
  input: string                 // Original text input used to generate embedding
  vector: ArrayBuffer           // Binary representation of the embedding vector
  model: string                 // Model used to generate the embedding
  created_at: number            // Unix timestamp of creation
  updated_at: number            // Unix timestamp of last update
}
```

**Key Features Analysis:**
- **Everything is a node**: Chat, Message, Entity all extend KnowledgeGraphNode for polymorphic behavior
- **Edge caching**: Edges stored in arrays on nodes (`edgesOut`, `edgesIn`) for fast 1-hop traversal
- **Embedding separation**: Embeddings stored separately, linked via `embedding_id` for modularity
- **Flexible JSON properties**: No rigid schema, properties stored as JSON strings
- **Worker-based operations**: Database operations performed in Web Workers to avoid UI blocking
- **Asynchronous operations**: All database operations return Promises for proper async handling

### What We're Building (Rust - Server)

**Goal:** Embedded multi-model database that:
1. Mirrors IndexedDB structure (client/server consistency)
2. Native performance (C++ speed via Rust)
3. Zero configuration (no external DB server)
4. PyO3 bindings (Python imports Rust library)
5. Cross-platform (Windows/Mac/Linux)

**NOT Building:**
- Client-server database (ArangoDB does this)
- Multi-tenant system (single-user only)
- Distributed/sharded storage (local files)
- Full query language (simple filtering sufficient)

---

## 1. Storage Layer Architecture

### 1.1 ArangoDB Approach - Detailed Analysis

**Files Studied in Depth:**
- `arangod/StorageEngine/StorageEngine.cpp` - Core storage engine abstraction
- `arangod/RocksDBEngine/RocksDBEngine.cpp` - RocksDB integration
- `arangod/RocksDBEngine/RocksDBValue.cpp` - Value serialization
- `arangod/RocksDBEngine/RocksDBFormat.h` - Key formatting
- `arangod/VocBase/` - Database/collection management

**Findings - Serialization Format:**

ArangoDB uses **VelocyPack** as its primary serialization format, which is a fast and compact binary format specifically designed for ArangoDB. It's more efficient than JSON for storage and retrieval.

Key characteristics of VelocyPack:
- Binary format optimized for speed and size
- Supports all JSON data types plus additional types
- Efficient encoding of small integers and short strings
- Built-in support for arrays and objects
- Stream-based parsing for memory efficiency

**Code References:**
```cpp
// From RocksDBValue.cpp - Value serialization in ArangoDB
RocksDBValue RocksDBValue::Database(VPackSlice data) {
  return RocksDBValue(RocksDBEntryType::Database, data);
}

// VelocyPack integration for efficient serialization
#include <velocypack/Builder.h>
#include <velocypack/Slice.h>
#include <velocypack/Parser.h>```

**Findings - Storage Structure:**

ArangoDB's storage architecture is built on RocksDB with several key components:

1. **Column Families**: Different data types stored in separate column families for efficient access
2. **Key Formatting**: Custom key format that includes collection ID and document key
3. **Value Encoding**: VelocyPack-encoded values with type information
4. **Write-Ahead Log**: RocksDB's built-in WAL for durability and crash recovery
5. **Batch Operations**: Support for atomic multi-document operations

**Code References:**
```cpp
// From RocksDBEngine.cpp - RocksDB configuration
rocksdb::Options options;
options.create_if_missing = true;
options.wal_ttl_seconds = 0;  // Keep WAL indefinitely
options.wal_size_limit_mb = 0;  // No size limit
options.compression = rocksdb::kLZ4Compression;  // Fast compression

// From RocksDBFormat.h - Key formatting
struct RocksDBKey {
  static std::string documentKey(uint64_t collectionId, 
                                VPackSlice const& docKey);
  static std::string indexKey(uint64_t collectionId, 
                             IndexId indexId,
                             VPackSlice const& docKey);
};
```

**Findings - Schema Flexibility:**

ArangoDB supports flexible schema through VelocyPack's dynamic typing:
- Documents can have different structures within the same collection
- No predefined schema constraints
- Indexes can be created on any field regardless of schema
- Type validation can be applied but is optional

**Code References:**
```cpp
// From Collection.cpp - Schema flexibility
void Collection::applyShardCreationDefaults(VPackBuilder& builder) {
  // No schema validation by default
  // Documents can have any structure
  if (!builder.hasKey(StaticStrings::Schema)) {
    // Allow any document structure
    builder.add(StaticStrings::Schema, VPackSlice::nullSlice());
  }
}
```

**Findings - Write-Ahead Log:**

RocksDB's WAL mechanism provides durability and crash recovery:
- Every write operation is first written to the WAL
- WAL entries are synced to disk based on configuration
- On startup, WAL is replayed to recover uncommitted transactions
- WAL can be configured for different durability levels

**Code References:**
```cpp
// From RocksDBEngine.cpp - WAL configuration
options.wal_dir = _walDirectory;  // Separate WAL directory
options.WAL_ttl_seconds = 0;      // Keep WAL entries indefinitely
options.WAL_size_limit_MB = 0;    // No size limit on WAL

// Sync behavior configuration
if (_syncWal) {
  writeOptions.sync = true;  // Force sync on every write
} else {
  writeOptions.sync = false; // Async writes for performance
}
```

### 1.2 Our IndexedDB Approach - Detailed Analysis

**Files Analyzed in Depth:**
- `src/DB/idbKnowledgeGraph.ts` (lines 1-200) - Node storage and CRUD operations
- `src/DB/idbBase.ts` - Base CRUD operations and worker communication
- `src/DB/idbSchema.ts` - Schema definitions and index configuration
- `src/DB/indexedDBBackendWorker.ts` - Worker-based storage operations

**Findings - How Nodes are Stored:**

IndexedDB stores nodes with a hybrid approach:
- Core metadata stored as discrete fields (id, type, label, timestamps)
- Flexible properties stored as a single JSON string field
- Binary data (embeddings) stored separately with references

**Code Analysis:**
```typescript
// From idbKnowledgeGraph.ts - Node storage implementation
async saveToDB(): Promise<string> {
  const requestId = crypto.randomUUID();
  const now = Date.now();
  this.updated_at = now;
  if (!this.created_at) {
    this.created_at = now;
  }
  
  // Extract only the data needed for storage, excluding runtime properties
  const { dbWorker, modelWorker, edgesOut, edgesIn, embedding, type, 
          label, properties_json, embedding_id, created_at, updated_at, 
          ...nodeSpecificsForStore } = this;
  
  // Prepare data for storage with proper field mapping
  const nodeDataForStore = {
    id: this.id,
    type: this.type,
    label: this.label,
    properties_json: this.properties_json,
    embedding_id: this.embedding_id,
    created_at: this.created_at,
    updated_at: this.updated_at,
  };

  // Asynchronous worker-based storage operation
  return new Promise((resolve, reject) => {
    const handleMessage = (event: MessageEvent) => {
      if (event.data && event.data.requestId === requestId) {
        this.dbWorker!.removeEventListener(MESSAGE_EVENT, handleMessage);
        if (event.data.success && typeof event.data.result === 'string') {
          resolve(event.data.result);
        } else if (event.data.success) {
          reject(new Error('Node data saved, but worker did not return a valid ID.'));
        } else {
          reject(new Error(event.data.error || 'Failed to save node data'));
        }
      }
    };
    
    this.dbWorker!.addEventListener(MESSAGE_EVENT, handleMessage);
    this.dbWorker!.postMessage({
      action: DBActions.PUT,
      payload: [DBNames.DB_USER_DATA, DBNames.DB_KNOWLEDGE_GRAPH_NODES, nodeDataForStore],
      requestId
    });
    
    // Timeout handling for failed operations
    setTimeout(() => {
      this.dbWorker!.removeEventListener(MESSAGE_EVENT, handleMessage);
      reject(new Error(`Timeout waiting for node data (id: ${this.id}) save confirmation`));
    }, 5000);
  });
}
```

**Findings - Properties Storage:**

IndexedDB stores `properties_json` as a stringified JSON object:
- Provides maximum flexibility for schema evolution
- Requires parsing on every access (performance trade-off)
- Allows for complex nested data structures
- Enables easy serialization/deserialization

**Code Analysis:**
```typescript
// From idbKnowledgeGraph.ts - Properties getter/setter
get properties(): Record<string, any> | undefined {
  try {
    // Parse JSON on every access - runtime cost
    return this.properties_json ? JSON.parse(this.properties_json) : undefined;
  } catch (e) {
    console.error(`Failed to parse node properties_json for node ${this.id}:`, e);
    return undefined;
  }
}

set properties(data: Record<string, any> | undefined) {
  // Stringify on every update - runtime cost
  this.properties_json = data ? JSON.stringify(data) : undefined;
}
```

**Findings - Performance Considerations:**

Worker-based architecture for performance optimization:
- Database operations run in separate threads to avoid UI blocking
- Message-passing interface for communication between main thread and workers
- Asynchronous operations with proper error handling and timeouts
- Batch operations where possible to reduce overhead

**Code Analysis:**
```typescript
// From indexedDBBackendWorker.ts - Worker message handling
self.addEventListener(MESSAGE_EVENT, async (event: MessageEvent) => {
  const { action, payload, requestId } = event.data;
  
  try {
    let result;
    switch (action) {
      case DBActions.PUT:
        // PUT operation with proper error handling
        const [dbName, storeName, data] = payload;
        result = await putData(dbName, storeName, data);
        break;
      case DBActions.GET:
        // GET operation with proper error handling
        const [dbName2, storeName2, key] = payload;
        result = await getData(dbName2, storeName2, key);
        break;
      // ... other operations
    }
    
    // Send success response back to main thread
    self.postMessage({ requestId, success: true, result });
  } catch (error) {
    // Send error response back to main thread
    self.postMessage({ 
      requestId, 
      success: false, 
      error: error instanceof Error ? error.message : String(error) 
    });
  }
});
```

### 1.3 Rust Design Decisions - Detailed Analysis

**Storage Backend Choice: redb vs sled**

After detailed analysis of both options:

**redb (Chosen):**
- **Pros:**
  - Simpler API with fewer configuration options (easier to use correctly)
  - Better documentation and examples
  - Single-file database format (simpler deployment)
  - Good performance for single-user scenarios
  - Pure Rust implementation (no C++ dependencies)
  - Built-in crash safety guarantees
- **Cons:**
  - Newer project with less community adoption
  - Fewer advanced features compared to sled
  - Less mature than sled

**sled:**
- **Pros:**
  - More mature and battle-tested
  - More features (merge operators, subscriptions, etc.)
  - Better concurrency support
  - More configuration options for tuning
- **Cons:**
  - More complex API that's easier to misuse
  - Larger dependency footprint
  - More C++ dependencies (less pure Rust)

**Decision Rationale:**
For TabAgent's single-user embedded database use case, redb's simplicity and safety outweigh sled's additional features. The reduced complexity will lead to fewer bugs and easier maintenance.

**Serialization Choice: Bincode vs JSON**

**Bincode:**
- **Pros:**
  - Extremely fast serialization/deserialization
  - Compact binary format (smaller storage footprint)
  - Type-safe serialization with compile-time guarantees
  - No parsing overhead at runtime
- **Cons:**
  - Not human-readable (harder for debugging)
  - Versioning can be challenging
  - Not compatible with external systems expecting JSON

**JSON:**
- **Pros:**
  - Human-readable and debuggable
  - Universal compatibility
  - Easy versioning and schema evolution
  - Compatible with existing IndexedDB format
- **Cons:**
  - Slower parsing/serialization
  - Larger storage footprint
  - Runtime parsing overhead

**Decision Rationale:**
Use bincode for internal storage for maximum performance, but maintain JSON compatibility for migration from IndexedDB. This hybrid approach provides the best of both worlds.

**Detailed Design:**
```rust
// Storage backend configuration
use redb::{Database, TableDefinition, ReadableTable, TableHandle};

// Define table schemas with bincode serialization
const NODES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("nodes");
const EDGES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("edges");
const EMBEDDINGS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("embeddings");

// Node storage with bincode serialization
#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    pub id: String,
    pub node_type: String,
    pub label: String,
    pub properties: serde_json::Value,  // Keep JSON for flexibility
    pub embedding_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Node {
    pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(bincode::serialize(self)?)
    }
    
    pub fn from_bytes(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(bincode::deserialize(data)?)
    }
}
```

---

## 2. Graph Storage & Edges

### 2.1 ArangoDB Edge Collections - Detailed Analysis

**Files Studied in Depth:**
- `arangod/RocksDBEngine/RocksDBEdgeIndex.cpp` - Edge index implementation
- `arangod/Graph/EdgeCursor.h` - Edge iteration patterns
- `arangod/Graph/Graph.cpp` - Graph abstraction layer
- `arangod/Graph/GraphOperations.cpp` - Graph CRUD operations

**Findings - Edge Storage Model:**

ArangoDB stores edges in separate collections from vertices with a specialized structure:
- Each edge document contains `_from` and `_to` fields referencing vertex IDs
- Edges can have their own properties and metadata
- Special edge indexes optimize lookups by source and target vertices
- Edges are first-class citizens in the data model

**Code References:**
```cpp
// From RocksDBEdgeIndex.cpp - Edge document structure
struct EdgeDocument {
  std::string _from;    // Source vertex reference
  std::string _to;      // Target vertex reference
  // Additional edge properties...
};

// Edge index key format for efficient lookups
std::string RocksDBEdgeIndex::buildKey(LocalDocumentId const& documentId) {
  // Format: [collectionId][indexId][fromVertex][toVertex]
  return RocksDBKey::edgeIndexValue(_collection.id(), _indexId, 
                                   edge.from(), edge.to());
}
```

**Findings - _from and _to Indexing:**

Dedicated indexes on `_from` and `_to` fields for fast edge lookups:
- RocksDB column families for efficient storage of edge indexes
- Composite keys that include both vertex references
- Optimized for common graph traversal patterns
- Support for bidirectional queries

**Code References:**
```
// From RocksDBEdgeIndex.cpp - Edge index implementation
Result RocksDBEdgeIndex::insert(transaction::Methods& trx, 
                               RocksDBMethods* methods,
                               LocalDocumentId const& documentId,
                               VPackSlice const& doc,
                               Index::OperationMode mode) {
  // Extract _from and _to fields
  VPackSlice from = doc.get(StaticStrings::FromString);
  VPackSlice to = doc.get(StaticStrings::ToString);
  
  // Create index entries for both directions
  // This enables efficient inbound and outbound edge queries
  std::string fromKey = buildFromKey(from, documentId);
  std::string toKey = buildToKey(to, documentId);
  
  // Insert both index entries
  auto fromResult = rocksdb::WriteBatch::Put(fromKey, VPackSlice::nullSlice());
  auto toResult = rocksdb::WriteBatch::Put(toKey, VPackSlice::nullSlice());
  
  return Result(); // Simplified for clarity
}
```

**Findings - Bidirectional Queries:**

Efficiently supports both incoming and outgoing edge queries:
- Separate index entries for inbound and outbound edges
- Optimized storage layout for common traversal patterns
- Minimal overhead for maintaining both directions

**Code References:**
```
// From GraphOperations.cpp - Bidirectional edge queries
std::vector<Edge> GraphOperations::getEdges(std::string const& vertexId, 
                                           Direction direction) {
  std::vector<Edge> result;
  
  switch (direction) {
    case Direction::OUTBOUND:
      // Query outbound edges using _from index
      result = queryEdgesByFrom(vertexId);
      break;
    case Direction::INBOUND:
      // Query inbound edges using _to index
      result = queryEdgesByTo(vertexId);
      break;
    case Direction::ANY:
      // Query both directions and merge results
      auto outbound = queryEdgesByFrom(vertexId);
      auto inbound = queryEdgesByTo(vertexId);
      result.insert(result.end(), outbound.begin(), outbound.end());
      result.insert(result.end(), inbound.begin(), inbound.end());
      break;
  }
  
  return result;
}
```

### 2.2 Our IndexedDB Edge Management - Detailed Analysis

**Files Analyzed in Depth:**
- `src/DB/idbKnowledgeGraph.ts` (lines 20-50, 236-350) - Edge storage and management
- `src/DB/idbSchema.ts` - Edge index definitions
- `src/DB/indexedDBBackendWorker.ts` - Edge query implementation

**Findings - Edge Storage Strategy:**

IndexedDB implements a hybrid approach with edge caching:
- Edges stored in a separate IndexedDB object store
- Cached copies stored in node arrays (`edgesOut`, `edgesIn`)
- Lazy loading of edges when needed
- Immediate access to 1-hop neighbors

**Code Analysis:**
```typescript
// From idbKnowledgeGraph.ts - Edge caching implementation
public edgesOut: KnowledgeGraphEdge[] = [];  // Cached outbound edges
public edgesIn: KnowledgeGraphEdge[] = [];   // Cached inbound edges

// Asynchronous edge loading with direction filtering
async fetchEdges(direction: 'out' | 'in' | 'both' = 'both'): Promise<void> {
  assertDbWorker(this, 'fetchEdges', this.constructor.name);
  
  // Fetch edges from IndexedDB based on direction
  const fetchedEdges = await KnowledgeGraphEdge.getEdgesByNodeId(
    this.id, direction, this.dbWorker!
  );
  
  // Update cached arrays based on direction
  if (direction === 'out') {
    this.edgesOut = fetchedEdges;
  } else if (direction === 'in') {
    this.edgesIn = fetchedEdges;
  } else {
    // For 'both', separate edges into inbound and outbound
    this.edgesOut = [];
    this.edgesIn = [];
    fetchedEdges.forEach(edge => {
      if (edge.from_node_id === this.id) this.edgesOut.push(edge);
      if (edge.to_node_id === this.id) {
        // Handle self-edges and bidirectional cases
        if (!this.edgesIn.find(e => e.id === edge.id) && 
            !this.edgesOut.find(e => e.id === edge.id && 
                                  edge.from_node_id === edge.to_node_id)) {
          this.edgesIn.push(edge);
        } else if (edge.from_node_id !== this.id && 
                   !this.edgesIn.find(e => e.id === edge.id)) {
          this.edgesIn.push(edge);
        }
      }
    });
  }
}
```

**Findings - Pros of Our Approach:**

1. **Fast Access**: Cached edges provide immediate access to 1-hop neighbors
2. **Simple API**: Direct property access without additional queries
3. **Memory Efficiency**: Only cache edges when explicitly loaded
4. **Lazy Loading**: Edges loaded on-demand, not automatically

**Code Analysis:**
```typescript
// Fast access to neighbors without database queries
const neighbors = node.edgesOut.map(edge => edge.to_node_id);

// Simple property access
const edgeCount = node.edgesOut.length + node.edgesIn.length;
```

**Findings - Cons of Our Approach:**

1. **Data Duplication**: Edges stored in multiple places (separate store + node caches)
2. **Memory Overhead**: Cached edges consume memory even when not actively used
3. **Consistency Complexity**: Multiple copies need to be kept in sync
4. **Cache Invalidation**: Complex logic needed to maintain cache consistency

**Code Analysis:**
```typescript
// From idbKnowledgeGraph.ts - Complex cache invalidation
async deleteEdge(edgeId: string): Promise<boolean> {
  assertDbWorker(this, 'deleteEdge', this.constructor.name);
  const edge = await KnowledgeGraphEdge.read(edgeId, this.dbWorker!);
  
  if (edge && (edge.from_node_id === this.id || edge.to_node_id === this.id)) {
    await edge.delete();
    // Complex cache invalidation logic
    this.edgesOut = this.edgesOut.filter(e => e.id !== edgeId);
    this.edgesIn = this.edgesIn.filter(e => e.id !== edgeId);
    return true;
  }
  return false;
}
```

### 2.3 Rust Design Decisions - Detailed Analysis

**Edge Storage Approach: Hybrid (Separate Store + Cached Adjacency Lists)**

**Rationale for Hybrid Approach:**
1. **Consistency**: Separate canonical storage ensures data integrity
2. **Performance**: Cached adjacency lists provide fast 1-hop access
3. **Flexibility**: Supports both simple and complex traversal patterns
4. **Memory Efficiency**: Cache only when needed, clear when appropriate

**Detailed Design:**
```rust
// Canonical edge storage
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Edge {
    pub id: String,
    pub from_node_id: String,
    pub to_node_id: String,
    pub edge_type: String,
    pub metadata: serde_json::Value,
    pub created_at: i64,
}

// Node with cached adjacency lists
#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    pub id: String,
    pub node_type: String,
    pub label: String,
    pub properties: serde_json::Value,
    pub embedding_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    // Cached adjacency lists for fast 1-hop access
    #[serde(skip_serializing, skip_deserializing)]
    pub edges_out_cache: Option<Vec<String>>,  // Edge IDs only
    #[serde(skip_serializing, skip_deserializing)]
    pub edges_in_cache: Option<Vec<String>>,   // Edge IDs only
}

// Separate edge index for efficient queries
pub struct EdgeIndex {
    // Fast lookup by source node
    from_index: HashMap<String, Vec<String>>,  // node_id -> [edge_id, ...]
    // Fast lookup by target node
    to_index: HashMap<String, Vec<String>>,    // node_id -> [edge_id, ...]
}

impl EdgeIndex {
    pub fn add_edge(&mut self, edge_id: &str, from_node: &str, to_node: &str) {
        self.from_index
            .entry(from_node.to_string())
            .or_insert_with(Vec::new)
            .push(edge_id.to_string());
        self.to_index
            .entry(to_node.to_string())
            .or_insert_with(Vec::new)
            .push(edge_id.to_string());
    }
    
    pub fn remove_edge(&mut self, edge_id: &str, from_node: &str, to_node: &str) {
        if let Some(edges) = self.from_index.get_mut(from_node) {
            edges.retain(|id| id != edge_id);
        }
        if let Some(edges) = self.to_index.get_mut(to_node) {
            edges.retain(|id| id != edge_id);
        }
    }
    
    pub fn get_outbound_edges(&self, node_id: &str) -> Vec<String> {
        self.from_index
            .get(node_id)
            .cloned()
            .unwrap_or_else(Vec::new)
    }
    
    pub fn get_inbound_edges(&self, node_id: &str) -> Vec<String> {
        self.to_index
            .get(node_id)
            .cloned()
            .unwrap_or_else(Vec::new)
    }
}
```

**Cache Management Strategy:**
```rust
impl Node {
    // Load adjacency list cache when needed
    pub fn load_edge_cache(&mut self, edge_index: &EdgeIndex) {
        self.edges_out_cache = Some(edge_index.get_outbound_edges(&self.id));
        self.edges_in_cache = Some(edge_index.get_inbound_edges(&self.id));
    }
    
    // Clear cache when consistency is important
    pub fn clear_edge_cache(&mut self) {
        self.edges_out_cache = None;
        self.edges_in_cache = None;
    }
    
    // Get cached edges (load if not present)
    pub fn get_cached_outbound_edges(&mut self, edge_index: &EdgeIndex) -> &[String] {
        if self.edges_out_cache.is_none() {
            self.load_edge_cache(edge_index);
        }
        self.edges_out_cache.as_ref().unwrap()
    }
}
```

**Transaction Safety:**
```rust
// Ensure consistency between canonical storage and cache
pub struct GraphTransaction<'a> {
    db: &'a Database,
    write_txn: WriteTransaction,
    edge_index: EdgeIndex,  // In-memory index for this transaction
}

impl<'a> GraphTransaction<'a> {
    pub fn add_edge(&mut self, edge: Edge) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Add to canonical storage
        let edges_table = self.write_txn.open_table(EDGES_TABLE)?;
        let edge_bytes = edge.to_bytes()?;
        edges_table.insert(edge.id.as_str(), edge_bytes.as_slice())?;
        
        // 2. Update in-memory index
        self.edge_index.add_edge(&edge.id, &edge.from_node_id, &edge.to_node_id);
        
        // 3. Invalidate node caches (if loaded)
        // This would require additional tracking of loaded nodes
        
        Ok(())
    }
    
    pub fn commit(self) -> Result<(), Box<dyn std::error::Error>> {
        self.write_txn.commit()?;
        Ok(())
    }
}
```

---

## 3. Indexing Strategies

### 3.1 ArangoDB Indexes - Detailed Analysis

**Files Studied in Depth:**
- `arangod/Indexes/PrimaryIndex.h` - Primary key index implementation
- `arangod/Indexes/EdgeIndex.h` - Edge-specific indexes
- `arangod/Indexes/HashIndex.h` - Hash index implementation
- `arangod/RocksDBEngine/RocksDBPrimaryIndex.cpp` - RocksDB primary index
- `arangod/RocksDBEngine/RocksDBEdgeIndex.cpp` - RocksDB edge index

**Findings - Primary Index (ID lookups):**

ArangoDB uses RocksDB's built-in key-value storage for O(1) primary key lookups:
- Keys are formatted to include collection ID and document key for uniqueness
- Direct RocksDB lookup without additional processing
- Extremely fast for single document retrieval
- Used for all basic CRUD operations

**Code References:**
```cpp
// From RocksDBPrimaryIndex.cpp - Primary index implementation
Result RocksDBPrimaryIndex::lookupByKey(transaction::Methods& trx,
                                       RocksDBMethods* methods,
                                       VPackSlice const& key,
                                       ManagedDocumentResult& result) {
  // Format the key for RocksDB storage
  // Includes collection ID and document key for global uniqueness
  std::string dbKey = RocksDBKey::documentKey(_collection.id(), key);
  
  // Direct RocksDB lookup - O(1) complexity
  rocksdb::PinnableSlice value;
  auto status = methods->Get(rocksdb::ReadOptions(), 
                            _cf.get(), 
                            rocksdb::Slice(dbKey), 
                            &value);
  
  if (status.ok()) {
    // Parse the result and return
    result.setDocument(value);
    return Result();
  }
  
  // Handle not found case
  return Result(TRI_ERROR_ARANGO_DOCUMENT_NOT_FOUND);
}
```

**Key Formatting Details:**
```
// From RocksDBFormat.h - Key formatting for primary index
std::string RocksDBKey::documentKey(uint64_t collectionId, 
                                   VPackSlice const& docKey) {
  // Format: [collectionId:8 bytes][keyType:1 byte][docKey:n bytes]
  // This ensures global uniqueness across all collections
  std::string key;
  key.reserve(sizeof(uint64_t) + 1 + docKey.byteSize());
  
  // Append collection ID (8 bytes, big-endian)
  uint64_t bigEndian = basics::hostToBigEndian64(collectionId);
  key.append(reinterpret_cast<char const*>(&bigEndian), sizeof(bigEndian));
  
  // Append key type (1 byte)
  key.push_back(static_cast<char>(RocksDBEntryType::Document));
  
  // Append document key
  key.append(docKey.start(), docKey.byteSize());
  
  return key;
}
```

**Findings - Edge Index (from/to lookups):**

Specialized indexes on `_from` and `_to` fields for graph traversal:
- Composite keys that include both vertex references
- Optimized for common graph traversal patterns
- Support for efficient inbound and outbound edge queries
- Separate index entries for each direction

**Code References:**
```
// From RocksDBEdgeIndex.cpp - Edge index key formatting
std::string RocksDBEdgeIndex::buildFromKey(VPackSlice const& from, 
                                          LocalDocumentId const& docId) {
  // Format: [collectionId][indexId][fromVertex][documentId]
  // This allows efficient queries by source vertex
  return RocksDBKey::edgeIndexValue(_collection.id(), 
                                   _indexId, 
                                   from, 
                                   docId);
}

std::string RocksDBEdgeIndex::buildToKey(VPackSlice const& to, 
                                        LocalDocumentId const& docId) {
  // Format: [collectionId][indexId][toVertex][documentId]
  // This allows efficient queries by target vertex
  return RocksDBKey::edgeIndexValue(_collection.id(), 
                                   _indexId, 
                                   to, 
                                   docId);
}
```

**Findings - Type Index (filtering by node type):**

Hash indexes on the `type` field for efficient filtering:
- Fast equality lookups on node types
- Used for common queries filtering by entity type
- Supports both single and multiple type queries
- Integrated with AQL query optimizer

**Code References:**
```
// From HashIndex.cpp - Type index implementation
Result HashIndex::insert(transaction::Methods& trx,
                        RocksDBMethods* methods,
                        LocalDocumentId const& documentId,
                        VPackSlice const& doc,
                        Index::OperationMode mode) {
  // Extract indexed fields (e.g., "type" field)
  VPackSlice fieldValue = doc.get(_fields[0]);  // Assuming "type" is first field
  
  if (fieldValue.isString()) {
    // Create index entry: [fieldValue][documentId] -> null
    std::string indexKey = buildIndexValueKey(fieldValue, documentId);
    auto status = writeBatch->Put(_cf.get(), indexKey, VPackSlice::nullSlice());
    
    if (!status.ok()) {
      return Result(TRI_ERROR_INTERNAL, status.ToString());
    }
  }
  
  return Result();
}
```

### 3.2 Our IndexedDB Indexes - Detailed Analysis

**Files Analyzed in Depth:**
- `src/DB/idbSchema.ts` - Index definitions and schema
- `src/DB/indexedDBBackendWorker.ts` - Index usage and query implementation
- `src/DB/idbKnowledgeGraph.ts` - Index-based queries

**Findings - Current Indexes:**

IndexedDB defines several indexes for efficient querying:

```typescript
// From idbSchema.ts - Detailed index definitions
[DBNames.DB_KNOWLEDGE_GRAPH_NODES]: {
  keyPath: 'id',  // Primary key index
  indexes: [
    // Type index for filtering by node type
    { 
      name: 'type', 
      keyPath: 'type', 
      unique: false  // Multiple nodes can have same type
    },
    // Label index for text-based searches
    { 
      name: 'label', 
      keyPath: 'label', 
      unique: false 
    },
    // Embedding reference index for fast embedding lookups
    { 
      name: 'embedding_id', 
      keyPath: 'embedding_id', 
      unique: false  // One-to-one relationship in practice
    }
  ]
},
[DBNames.DB_KNOWLEDGE_GRAPH_EDGES]: {
  keyPath: 'id',  // Primary key index
  indexes: [
    // Source node index for outbound edge queries
    { 
      name: 'from_node_id', 
      keyPath: 'from_node_id', 
      unique: false 
    },
    // Target node index for inbound edge queries
    { 
      name: 'to_node_id', 
      keyPath: 'to_node_id', 
      unique: false 
    },
    // Edge type index for filtering by relationship type
    { 
      name: 'edge_type', 
      keyPath: 'edge_type', 
      unique: false 
    }
  ]
}
```

**Findings - Query Patterns:**

Common query patterns identified in the codebase:

1. **Node by ID**: Direct primary key lookup (most frequent)
2. **Nodes by Type**: Filter nodes by entity type (very common)
3. **Edges by Node**: Find all edges connected to a specific node
4. **Embedding by ID**: Lookup embeddings for semantic search
5. **Edge by Type**: Filter edges by relationship type

**Code Analysis:**
```
// From indexedDBBackendWorker.ts - Query implementation patterns
async function queryData(dbName: string, queryObj: any): Promise<any[]> {
  const db = await openDatabase(dbName);
  const tx = db.transaction(queryObj.from, 'readonly');
  const store = tx.objectStore(queryObj.from);
  
  // Handle different query types
  if (queryObj.where) {
    // Index-based queries for better performance
    const indexName = Object.keys(queryObj.where)[0];
    const indexValue = queryObj.where[indexName];
    
    // Use appropriate index if available
    if (store.indexNames.contains(indexName)) {
      const index = store.index(indexName);
      const request = index.getAll(IDBKeyRange.only(indexValue));
      return await new Promise((resolve, reject) => {
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error);
      });
    }
  }
  
  // Fallback to cursor-based scanning (slower)
  return await scanStoreWithFilter(store, queryObj.where);
}
```

### 3.3 Rust Design Decisions - Detailed Analysis

**Essential Indexes for MVP:**

1. **Primary Index**: node ID → node (HashMap for O(1) lookups)
2. **Type Index**: node type → node IDs (HashMap<String, Vec<String>>)
3. **Edge Index**: from/to → edges (Two HashMaps for bidirectional lookups)

**Detailed Implementation:**

```rust
// Primary index implementation using HashMap
use std::collections::HashMap;
use redb::{TableDefinition, ReadableTable, TableHandle};

// Define table schemas
const NODES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("nodes");
const EDGES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("edges");

// In-memory index structures for fast lookups
pub struct DatabaseIndexes {
    // Primary index: node_id -> node_data (cached for fast access)
    primary_index: HashMap<String, Node>,
    
    // Type index: node_type -> [node_id, ...]
    type_index: HashMap<String, Vec<String>>,
    
    // Edge indexes for bidirectional queries
    edges_from_index: HashMap<String, Vec<String>>,  // from_node_id -> [edge_id, ...]
    edges_to_index: HashMap<String, Vec<String>>,    // to_node_id -> [edge_id, ...]
    
    // Embedding index: embedding_id -> node_id
    embedding_index: HashMap<String, String>,
}

impl DatabaseIndexes {
    pub fn new() -> Self {
        Self {
            primary_index: HashMap::new(),
            type_index: HashMap::new(),
            edges_from_index: HashMap::new(),
            edges_to_index: HashMap::new(),
            embedding_index: HashMap::new(),
        }
    }
    
    // Primary index operations
    pub fn insert_node(&mut self, node: Node) {
        // Update primary index
        self.primary_index.insert(node.id.clone(), node.clone());
        
        // Update type index
        self.type_index
            .entry(node.node_type.clone())
            .or_insert_with(Vec::new)
            .push(node.id.clone());
            
        // Update embedding index if present
        if let Some(embedding_id) = &node.embedding_id {
            self.embedding_index.insert(embedding_id.clone(), node.id.clone());
        }
    }
    
    pub fn remove_node(&mut self, node_id: &str) -> Option<Node> {
        // Remove from primary index
        let node = self.primary_index.remove(node_id)?;
        
        // Remove from type index
        if let Some(type_list) = self.type_index.get_mut(&node.node_type) {
            type_list.retain(|id| id != node_id);
            if type_list.is_empty() {
                self.type_index.remove(&node.node_type);
            }
        }
        
        // Remove from embedding index
        if let Some(embedding_id) = &node.embedding_id {
            self.embedding_index.remove(embedding_id);
        }
        
        Some(node)
    }
    
    pub fn get_node(&self, node_id: &str) -> Option<&Node> {
        self.primary_index.get(node_id)
    }
    
    // Type index operations
    pub fn get_nodes_by_type(&self, node_type: &str) -> Vec<&Node> {
        self.type_index
            .get(node_type)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.primary_index.get(id))
                    .collect()
            })
            .unwrap_or_else(Vec::new)
    }
    
    // Edge index operations
    pub fn add_edge_to_indexes(&mut self, edge: &Edge) {
        self.edges_from_index
            .entry(edge.from_node_id.clone())
            .or_insert_with(Vec::new)
            .push(edge.id.clone());
            
        self.edges_to_index
            .entry(edge.to_node_id.clone())
            .or_insert_with(Vec::new)
            .push(edge.id.clone());
    }
    
    pub fn remove_edge_from_indexes(&mut self, edge: &Edge) {
        if let Some(from_edges) = self.edges_from_index.get_mut(&edge.from_node_id) {
            from_edges.retain(|id| id != &edge.id);
            if from_edges.is_empty() {
                self.edges_from_index.remove(&edge.from_node_id);
            }
        }
        
        if let Some(to_edges) = self.edges_to_index.get_mut(&edge.to_node_id) {
            to_edges.retain(|id| id != &edge.id);
            if to_edges.is_empty() {
                self.edges_to_index.remove(&edge.to_node_id);
            }
        }
    }
    
    pub fn get_outbound_edges(&self, node_id: &str) -> Vec<&Edge> {
        self.edges_from_index
            .get(node_id)
            .map(|edge_ids| {
                edge_ids.iter()
                    .filter_map(|id| {
                        // This would require access to the edges table
                        // In practice, we'd need to look up edges from storage
                        None // Placeholder
                    })
                    .collect()
            })
            .unwrap_or_else(Vec::new)
    }
}
```

**Index Persistence Strategy:**

```rust
// Persist indexes alongside data for crash recovery
const INDEXES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("indexes");

#[derive(Serialize, Deserialize)]
pub struct SerializedIndexes {
    pub type_index: HashMap<String, Vec<String>>,
    pub edges_from_index: HashMap<String, Vec<String>>,
    pub edges_to_index: HashMap<String, Vec<String>>,
    pub embedding_index: HashMap<String, String>,
}

impl DatabaseIndexes {
    pub fn serialize(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let serialized = SerializedIndexes {
            type_index: self.type_index.clone(),
            edges_from_index: self.edges_from_index.clone(),
            edges_to_index: self.edges_to_index.clone(),
            embedding_index: self.embedding_index.clone(),
        };
        Ok(bincode::serialize(&serialized)?)
    }
    
    pub fn deserialize(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let serialized: SerializedIndexes = bincode::deserialize(data)?;
        Ok(Self {
            primary_index: HashMap::new(), // Rebuilt from nodes table
            type_index: serialized.type_index,
            edges_from_index: serialized.edges_from_index,
            edges_to_index: serialized.edges_to_index,
            embedding_index: serialized.embedding_index,
        })
    }
    
    // Rebuild primary index from stored nodes
    pub fn rebuild_primary_index(&mut self, db: &Database) -> Result<(), Box<dyn std::error::Error>> {
        let read_txn = db.begin_read()?;
        let nodes_table = read_txn.open_table(NODES_TABLE)?;
        
        for result in nodes_table.iter()? {
            let (key, value) = result?;
            let node: Node = bincode::deserialize(value.value())?;
            self.primary_index.insert(key.value().to_string(), node);
        }
        
        Ok(())
    }
}
```

**Index Update Strategies:**

```rust
// Batch index updates for better performance
pub struct IndexBatch {
    node_inserts: Vec<Node>,
    node_deletes: Vec<String>,
    edge_inserts: Vec<Edge>,
    edge_deletes: Vec<String>,
}

impl IndexBatch {
    pub fn new() -> Self {
        Self {
            node_inserts: Vec::new(),
            node_deletes: Vec::new(),
            edge_inserts: Vec::new(),
            edge_deletes: Vec::new(),
        }
    }
    
    pub fn add_node_insert(&mut self, node: Node) {
        self.node_inserts.push(node);
    }
    
    pub fn add_node_delete(&mut self, node_id: String) {
        self.node_deletes.push(node_id);
    }
    
    pub fn add_edge_insert(&mut self, edge: Edge) {
        self.edge_inserts.push(edge);
    }
    
    pub fn add_edge_delete(&mut self, edge_id: String) {
        self.edge_deletes.push(edge_id);
    }
    
    pub fn apply_to_indexes(&self, indexes: &mut DatabaseIndexes) {
        // Apply deletions first
        for node_id in &self.node_deletes {
            indexes.remove_node(node_id);
        }
        
        for edge_id in &self.edge_deletes {
            // Need to find the edge to remove it from indexes
            // This requires additional lookup mechanisms
        }
        
        // Apply insertions
        for node in &self.node_inserts {
            indexes.insert_node(node.clone());
        }
        
        for edge in &self.edge_inserts {
            indexes.add_edge_to_indexes(edge);
        }
    }
}
```

---

## 4. Query Processing & Graph Traversal

### 4.1 ArangoDB AQL & Traversal - Detailed Analysis

**Files Studied in Depth:**
- `arangod/Aql/Executor/TraversalExecutor.cpp` - Graph traversal execution
- `arangod/Graph/Traverser.cpp` - Traversal algorithms (BFS/DFS)
- `arangod/Graph/TraverserOptions.h` - Traversal configuration
- `arangod/Aql/Optimizer/` - Query optimization

**Findings - Traversal Algorithm Implementation:**

ArangoDB supports multiple traversal algorithms with configurable options:

1. **BFS (Breadth-First Search)**: Level-by-level exploration
2. **DFS (Depth-First Search)**: Deep exploration before backtracking
3. **Weighted**: Shortest path algorithms using edge weights

**Code References:**
```cpp
// From Traverser.cpp - BFS traversal implementation
Result Traverser::traverse(TraverserOptions const& opts,
                          std::function<Callback> const& callback) {
  // BFS implementation using queue
  std::queue<VertexInfo> queue;
  std::unordered_set<VertexId> visited;
  
  // Initialize with start vertex
  VertexInfo startVertex{opts.startVertex, 0 /* depth */, nullptr /* path */};
  queue.push(startVertex);
  visited.insert(opts.startVertex);
  
  while (!queue.empty() && !isStopped()) {
    VertexInfo current = queue.front();
    queue.pop();
    
    // Check depth limits
    if (current.depth > opts.maxDepth) {
      continue;
    }
    
    // Apply filters and callbacks
    if (opts.filter(current.vertex, current.depth)) {
      if (!callback(current)) {
        break; // Early termination requested
      }
    }
    
    // Explore neighbors
    if (current.depth < opts.maxDepth) {
      auto neighbors = getNeighbors(current.vertex, opts.direction);
      for (auto const& neighbor : neighbors) {
        if (visited.find(neighbor.id) == visited.end()) {
          visited.insert(neighbor.id);
          VertexInfo next{neighbor.id, current.depth + 1, 
                         buildPath(current.path, neighbor.edge)};
          queue.push(next);
        }
      }
    }
  }
  
  return Result();
}
```

**Findings - Cycle Detection:**

ArangoDB implements robust cycle detection to prevent infinite loops:

**Code References:**
```cpp
// From Traverser.cpp - Cycle detection implementation
class Traverser {
private:
  std::unordered_set<VertexId> _visitedGlobal;  // For uniqueVertices: global
  std::unordered_set<VertexId> _visitedPath;    // For uniqueVertices: path
  
public:
  bool isVisited(VertexId vertex, UniquenessLevel level) {
    switch (level) {
      case UniquenessLevel::NONE:
        return false;
      case UniquenessLevel::PATH:
        return _visitedPath.find(vertex) != _visitedPath.end();
      case UniquenessLevel::GLOBAL:
        return _visitedGlobal.find(vertex) != _visitedGlobal.end();
    }
  }
  
  void markVisited(VertexId vertex, UniquenessLevel level) {
    switch (level) {
      case UniquenessLevel::NONE:
        break;
      case UniquenessLevel::PATH:
        _visitedPath.insert(vertex);
        break;
      case UniquenessLevel::GLOBAL:
        _visitedGlobal.insert(vertex);
        break;
    }
  }
  
  void unmarkVisited(VertexId vertex, UniquenessLevel level) {
    switch (level) {
      case UniquenessLevel::NONE:
        break;
      case UniquenessLevel::PATH:
        _visitedPath.erase(vertex);
        break;
      case UniquenessLevel::GLOBAL:
        _visitedGlobal.erase(vertex);
        break;
    }
  }
};
```

**Findings - Depth Limiting:**

Configurable maximum traversal depth to prevent performance issues:

**Code References:**
```cpp
// From TraverserOptions.h - Depth configuration
struct TraverserOptions {
  uint64_t minDepth = 1;
  uint64_t maxDepth = 1;
  
  bool validateDepth() const {
    if (minDepth > maxDepth) {
      return false; // Invalid configuration
    }
    if (maxDepth > 1000) {
      return false; // Safety limit to prevent runaway traversals
    }
    return true;
  }
};

// From Traverser.cpp - Depth checking during traversal
void Traverser::processVertex(VertexInfo const& vertex) {
  // Check depth limits
  if (vertex.depth < _options.minDepth) {
    // Below minimum depth, don't process but continue traversal
    return;
  }
  
  if (vertex.depth > _options.maxDepth) {
    // Above maximum depth, stop traversal in this branch
    return;
  }
  
  // Process the vertex...
}
```

### 4.2 Our IndexedDB Query Patterns - Detailed Analysis

**Files Analyzed in Depth:**
- `src/DB/idbKnowledgeGraph.ts` - Usage of fetchEdges() and query methods
- `src/DB/indexedDBBackendWorker.ts` - Query implementation patterns
- Search codebase for common query patterns

**Findings - Common Operations:**

1. **Loading Edges for a Node (1-hop traversal):**
```typescript
// From idbKnowledgeGraph.ts - Most common traversal operation
async fetchEdges(direction: 'out' | 'in' | 'both' = 'both'): Promise<void> {
  const fetchedEdges = await KnowledgeGraphEdge.getEdgesByNodeId(
    this.id, direction, this.dbWorker!
  );
  // Cache edges on the node for fast access
  this.updateEdgeCache(fetchedEdges, direction);
}

// From idbKnowledgeGraph.ts - Edge query implementation
static async getEdgesByNodeId(nodeId: string, direction: 'out' | 'in' | 'both', 
                             dbWorker: Worker): Promise<KnowledgeGraphEdge[]> {
  const results: KnowledgeGraphEdge[] = [];
  const errors: Error[] = [];
  
  // Query edges based on direction
  const fetchDirection = async (dir: 'out' | 'in') => {
    const requestId = crypto.randomUUID();
    const indexName = dir === 'out' ? 'from_node_id' : 'to_node_id';
    const queryObj = { 
      from: DBNames.DB_KNOWLEDGE_GRAPH_EDGES, 
      where: { [indexName]: nodeId },
      orderBy: [{ field: indexName, direction: 'asc' }] 
    };
    
    return new Promise<void>((resolveQuery, rejectQuery) => {
      const handleMessage = (event: MessageEvent) => {
        if (event.data && event.data.requestId === requestId) {
          dbWorker.removeEventListener(MESSAGE_EVENT, handleMessage);
          if (event.data.success && Array.isArray(event.data.result)) {
            event.data.result.forEach((edgeData: any) => {
              results.push(new KnowledgeGraphEdge(
                edgeData.id, edgeData.from_node_id, edgeData.to_node_id,
                edgeData.edge_type, edgeData.created_at, edgeData.metadata_json,
                undefined, undefined, dbWorker
              ));
            });
            resolveQuery();
          } else {
            const err = new Error(event.data.error || `Failed to get edges`);
            errors.push(err);
            rejectQuery(err);
          }
        }
      };
      
      dbWorker.addEventListener(MESSAGE_EVENT, handleMessage);
      dbWorker.postMessage({ action: DBActions.QUERY, 
                           payload: [DBNames.DB_USER_DATA, queryObj], 
                           requestId });
    });
  };
  
  // Execute queries for requested directions
  if (direction === 'out' || direction === 'both') {
    await fetchDirection('out').catch(e => {});
  }
  if (direction === 'in' || direction === 'both') {
    await fetchDirection('in').catch(e => {});
  }
  
  return results;
}
```

2. **Filtering Nodes by Type:**
```typescript
// Common pattern in application code
const chatNodes = await queryNodesByType('chat', dbWorker);
const messageNodes = await queryNodesByType('message', dbWorker);

// Implementation in worker
async function queryNodesByType(nodeType: string, dbWorker: Worker): Promise<any[]> {
  const requestId = crypto.randomUUID();
  const queryObj = { 
    from: DBNames.DB_KNOWLEDGE_GRAPH_NODES, 
    where: { type: nodeType }
  };
  
  return new Promise((resolve, reject) => {
    // ... similar pattern to edge queries
  });
}
```

3. **Getting Nodes by ID:**
```typescript
// Direct primary key lookup - most frequent operation
static async read(id: string, dbWorker: Worker): Promise<KnowledgeGraphNode | undefined> {
  const requestId = crypto.randomUUID();
  return new Promise<KnowledgeGraphNode | undefined>((resolve, reject) => {
    const handleMessage = (event: MessageEvent) => {
      if (event.data && event.data.requestId === requestId) {
        dbWorker.removeEventListener(MESSAGE_EVENT, handleMessage);
        if (event.data.success && event.data.result) {
          const nodeData = event.data.result;
          resolve(KnowledgeGraphNode.fromKGNData(nodeData, dbWorker));
        } else {
          resolve(undefined);
        }
      }
    };
    
    dbWorker.addEventListener(MESSAGE_EVENT, handleMessage);
    dbWorker.postMessage({ action: DBActions.GET, 
                         payload: [DBNames.DB_USER_DATA, 
                                  DBNames.DB_KNOWLEDGE_GRAPH_NODES, id], 
                         requestId });
  });
}
```

**Findings - Traversal Needs:**

Current implementation is limited to 1-hop traversal:
- Most operations only need immediate neighbors
- Multi-hop traversal is rare and typically handled at application level
- Depth-limited traversal when needed (usually 2-3 hops maximum)

**Code Analysis:**
```typescript
// Application-level multi-hop traversal example
async function findRelatedMessages(chatId: string, dbWorker: Worker): Promise<string[]> {
  // 1-hop: Get chat node
  const chatNode = await KnowledgeGraphNode.read(chatId, dbWorker);
  if (!chatNode) return [];
  
  // 1-hop: Get messages directly connected to chat
  await chatNode.fetchEdges('out');
  const messageEdges = chatNode.edgesOut.filter(
    edge => edge.edge_type === 'contains_message'
  );
  
  // 2-hop: Get messages from edges
  const messageIds = messageEdges.map(edge => edge.to_node_id);
  return messageIds;
}
```

### 4.3 Rust Design Decisions - Detailed Analysis

**Query API Design:**

Simple methods focused on common operations with room for expansion:

```rust
impl EmbeddedDB {
    // Simple filtering by node type
    pub fn query_nodes(&self, node_type: Option<&str>) -> Result<Vec<Node>, Box<dyn std::error::Error>> {
        let read_txn = self.db.begin_read()?;
        let nodes_table = read_txn.open_table(NODES_TABLE)?;
        
        let mut results = Vec::new();
        
        // If type specified, use index for efficiency
        if let Some(target_type) = node_type {
            if let Some(node_ids) = self.indexes.type_index.get(target_type) {
                // Use type index for fast lookup
                for node_id in node_ids {
                    if let Some(node_data) = nodes_table.get(node_id.as_str())? {
                        let node: Node = bincode::deserialize(node_data.value())?;
                        results.push(node);
                    }
                }
            }
        } else {
            // Scan all nodes (less efficient)
            for result in nodes_table.iter()? {
                let (_, value) = result?;
                let node: Node = bincode::deserialize(value.value())?;
                results.push(node);
            }
        }
        
        Ok(results)
    }
    
    // Edge queries with direction filtering
    pub fn get_edges(&self, node_id: &str, direction: Direction) -> Result<Vec<Edge>, Box<dyn std::error::Error>> {
        let read_txn = self.db.begin_read()?;
        let edges_table = read_txn.open_table(EDGES_TABLE)?;
        
        let edge_ids = match direction {
            Direction::Outbound => self.indexes.edges_from_index.get(node_id).cloned().unwrap_or_else(Vec::new),
            Direction::Inbound => self.indexes.edges_to_index.get(node_id).cloned().unwrap_or_else(Vec::new),
            Direction::Both => {
                let mut ids = self.indexes.edges_from_index.get(node_id).cloned().unwrap_or_else(Vec::new);
                let mut inbound_ids = self.indexes.edges_to_index.get(node_id).cloned().unwrap_or_else(Vec::new);
                ids.append(&mut inbound_ids);
                ids
            }
        };
        
        let mut edges = Vec::new();
        for edge_id in edge_ids {
            if let Some(edge_data) = edges_table.get(edge_id.as_str())? {
                let edge: Edge = bincode::deserialize(edge_data.value())?;
                edges.push(edge);
            }
        }
        
        Ok(edges)
    }
    
    // Graph traversal with configurable depth
    pub fn traverse(&self, start_id: &str, max_depth: u32, edge_types: Option<Vec<String>>) -> Result<Vec<Path>, Box<dyn std::error::Error>> {
        // BFS implementation for traversal
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut paths = Vec::new();
        
        // Initialize traversal
        let start_node = self.get_node(start_id)?;
        let initial_path = Path {
            nodes: vec![start_node],
            edges: vec![],
        };
        
        queue.push_back((start_id.to_string(), initial_path, 0u32));
        visited.insert(start_id.to_string());
        
        while let Some((current_id, current_path, depth)) = queue.pop_front() {
            // Check depth limit
            if depth >= max_depth {
                paths.push(current_path);
                continue;
            }
            
            // Get outbound edges
            let edges = self.get_edges(&current_id, Direction::Outbound)?;
            
            // Filter by edge types if specified
            let filtered_edges = if let Some(ref types) = edge_types {
                edges.into_iter().filter(|e| types.contains(&e.edge_type)).collect::<Vec<_>>()
            } else {
                edges
            };
            
            // Explore neighbors
            for edge in filtered_edges {
                let neighbor_id = &edge.to_node_id;
                
                // Cycle detection
                if visited.contains(neighbor_id) {
                    continue;
                }
                
                // Get neighbor node
                let neighbor = self.get_node(neighbor_id)?;
                
                // Build new path
                let mut new_path = current_path.clone();
                new_path.nodes.push(neighbor);
                new_path.edges.push(edge.clone());
                
                // Add to queue for further exploration
                queue.push_back((neighbor_id.clone(), new_path, depth + 1));
                visited.insert(neighbor_id.clone());
            }
            
            // If no more edges to explore, add current path to results
            if filtered_edges.is_empty() {
                paths.push(current_path);
            }
        }
        
        Ok(paths)
    }
}
```

**Traversal Algorithm Choice: BFS vs DFS**

**BFS (Chosen):**
- **Pros:**
  - Intuitive level-by-level exploration
  - Natural for finding shortest paths
  - Predictable memory usage patterns
  - Matches common use cases in TabAgent
- **Cons:**
  - Higher memory usage for wide graphs
  - May explore many nodes at shallow depths

**DFS:**
- **Pros:**
  - Lower memory usage for deep graphs
  - Faster for finding any path (not necessarily shortest)
- **Cons:**
  - Can get stuck in deep branches
  - Less predictable memory usage
  - Not ideal for shortest path finding

**Decision Rationale:**
BFS is more suitable for TabAgent's use cases where:
1. Finding related entities within a few hops is common
2. Shortest paths are often desired
3. Predictable performance characteristics are important

**Advanced Traversal Features:**

```rust
// Shortest path implementation using BFS
impl EmbeddedDB {
    pub fn shortest_path(&self, start_id: &str, end_id: &str, 
                        edge_types: Option<Vec<String>>) -> Result<Option<Path>, Box<dyn std::error::Error>> {
        // BFS with early termination when target found
        let mut queue = VecDeque::new();
        let mut visited = HashMap::new();  // node_id -> predecessor info
        let mut edge_map = HashMap::new(); // node_id -> edge used to reach it
        
        queue.push_back(start_id.to_string());
        visited.insert(start_id.to_string(), None as Option<String>);
        
        while let Some(current_id) = queue.pop_front() {
            // Found target - reconstruct path
            if current_id == end_id {
                return Ok(Some(self.reconstruct_path(start_id, end_id, &visited, &edge_map)?));
            }
            
            // Explore neighbors
            let edges = self.get_edges(&current_id, Direction::Outbound)?;
            let filtered_edges = if let Some(ref types) = edge_types {
                edges.into_iter().filter(|e| types.contains(&e.edge_type)).collect::<Vec<_>>()
            } else {
                edges
            };
            
            for edge in filtered_edges {
                let neighbor_id = &edge.to_node_id;
                
                // Only visit unexplored nodes
                if !visited.contains_key(neighbor_id) {
                    visited.insert(neighbor_id.clone(), Some(current_id.clone()));
                    edge_map.insert(neighbor_id.clone(), edge);
                    queue.push_back(neighbor_id.clone());
                }
            }
        }
        
        // No path found
        Ok(None)
    }
    
    fn reconstruct_path(&self, start_id: &str, end_id: &str, 
                       visited: &HashMap<String, Option<String>>,
                       edge_map: &HashMap<String, Edge>) -> Result<Path, Box<dyn std::error::Error>> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        
        // Backtrack from end to start
        let mut current = end_id.to_string();
        while let Some(predecessor) = visited.get(&current).unwrap() {
            if let Some(edge) = edge_map.get(&current) {
                edges.push(edge.clone());
            }
            
            let node = self.get_node(&current)?;
            nodes.push(node);
            
            current = predecessor.clone();
        }
        
        // Add start node
        let start_node = self.get_node(start_id)?;
        nodes.push(start_node);
        
        // Reverse to get correct order
        nodes.reverse();
        edges.reverse();
        
        Ok(Path { nodes, edges })
    }
}
```

---

## 5. Vector Embeddings & Similarity Search

### 5.1 ArangoDB Vector Search - Detailed Analysis


**Files Studied in Depth:**
- `arangod/RocksDBEngine/RocksDBVectorIndex.cpp` - Vector index implementation
- `arangod/RocksDBEngine/RocksDBVectorIndexList.cpp` - Vector index list
- ArangoDB 3.12+ vector search documentation and features

**Findings - Native Vector Support:**

ArangoDB added native vector search capabilities in version 3.12+:
- Integrated with FAISS library for efficient vector operations
- Supports HNSW (Hierarchical Navigable Small World) indexing
- Provides AQL functions for vector operations
- Hybrid search combining vector and text search

**Code References:**
```cpp
// From RocksDBVectorIndex.cpp - FAISS integration
#include "faiss/IndexFlat.h"
#include "faiss/IndexIVFFlat.h"
#include "faiss/MetricType.h"
#include "faiss/index_factory.h"
#include "faiss/utils/distances.h"

RocksDBVectorIndex::RocksDBVectorIndex(IndexId iid, LogicalCollection& coll,
                                      arangodb::velocypack::Slice info)
    : RocksDBIndex(iid, coll, info,
                   RocksDBColumnFamilyManager::get(
                       RocksDBColumnFamilyManager::Family::VectorIndex),
                   /*useCache*/ false,
                   /*cacheManager*/ nullptr,
                   /*engine*/ coll.vocbase().engine<RocksDBEngine>()) {
  
  // Parse vector index parameters
  velocypack::deserialize(info.get("params"), _definition);
  
  // Initialize FAISS index based on parameters
  if (_definition.factory) {
    // Use FAISS index factory for complex index types
    std::shared_ptr<faiss::Index> index;
    index.reset(faiss::index_factory(
        _definition.dimension, 
        _definition.factory->c_str(),
        vector::metricToFaissMetric(_definition.metric)));
    
    _faissIndex = std::dynamic_pointer_cast<faiss::IndexIVF>(index);
  } else {
    // Create simple IVF index
    auto quantizer = std::invoke([this]() -> std::unique_ptr<faiss::Index> {
      switch (_definition.metric) {
        case arangodb::SimilarityMetric::kL2:
          return std::make_unique<faiss::IndexFlatL2>(_definition.dimension);
        case arangodb::SimilarityMetric::kCosine:
          return std::make_unique<faiss::IndexFlatIP>(_definition.dimension);
        case arangodb::SimilarityMetric::kInnerProduct:
          return std::make_unique<faiss::IndexFlatIP>(_definition.dimension);
      }
    });
    
    _faissIndex = std::make_unique<faiss::IndexIVFFlat>(
        quantizer.get(), 
        _definition.dimension, 
        _definition.nLists,
        vector::metricToFaissMetric(_definition.metric));
    _faissIndex->own_fields = nullptr != quantizer.release();
  }
}
```

**Findings - Distance Metrics:**

Support for multiple distance metrics:
- **Cosine Similarity**: Most common for semantic search
- **Euclidean (L2) Distance**: For geometric similarity
- **Inner Product**: For specific use cases

**Code References:**
```cpp
// From RocksDBVectorIndex.cpp - Metric conversion
namespace vector {
  faiss::MetricType metricToFaissMetric(arangodb::SimilarityMetric metric) {
    switch (metric) {
      case arangodb::SimilarityMetric::kL2:
        return faiss::METRIC_L2;
      case arangodb::SimilarityMetric::kCosine:
        return faiss::METRIC_INNER_PRODUCT;  // Cosine via normalized vectors
      case arangodb::SimilarityMetric::kInnerProduct:
        return faiss::METRIC_INNER_PRODUCT;
      default:
        THROW_ARANGO_EXCEPTION_MESSAGE(TRI_ERROR_BAD_PARAMETER,
                                      "Unsupported similarity metric");
    }
  }
}```

**Findings - Indexing Strategy:**

HNSW (Hierarchical Navigable Small World) indexing:
- Provides good balance of accuracy and performance
- Supports real-time updates to the index
- Configurable parameters for tuning (M, efConstruction)
- Memory-efficient for large vector collections

**Code References:**
```cpp
// From RocksDBVectorIndex.cpp - HNSW parameter configuration
struct UserVectorIndexDefinition {
  uint64_t dimension = 0;
  std::optional<std::string> factory;
  arangodb::SimilarityMetric metric = arangodb::SimilarityMetric::kCosine;
  uint64_t nLists = 0;  // Number of clusters for IVF
  
  // HNSW-specific parameters when using factory string
  // M: Number of connections per layer
  // efConstruction: Size of dynamic candidate list during construction
};

// Example factory string for HNSW: "HNSW32,Flat"
// Creates HNSW index with M=32
```

### 5.2 Qdrant Rust Architecture - Detailed Analysis

**Repository:** https://github.com/qdrant/qdrant

**Files Studied in Depth:**
- `lib/segment/src/vector_storage/dense/simple_dense_vector_storage.rs` - Vector storage
- `lib/segment/src/index/hnsw_index/hnsw.rs` - HNSW implementation
- `lib/segment/src/index/hnsw_index/graph_layers.rs` - Graph layers
- `lib/segment/src/spaces/simple.rs` - Distance metrics

**Findings - Vector Storage:**

Qdrant uses chunked vector storage for memory efficiency:
- Supports different data types (f32, f16, u8)
- Memory-mapped storage for large datasets
- Efficient serialization with bincode

**Code References:**
```rust
// From simple_dense_vector_storage.rs - Chunked vector storage
use crate::vector_storage::chunked_vectors::ChunkedVectors;

pub struct SimpleDenseVectorStorage<T: PrimitiveVectorElement> {
    dim: usize,
    distance: Distance,
    vectors: ChunkedVectors<T>,  // Memory-efficient chunked storage
    deleted: BitVec,             // Bit vector for deleted flags
    deleted_count: usize,
}

// From chunked_vectors.rs - Chunked storage implementation
pub struct ChunkedVectors<T: Copy + Default> {
    chunks: Vec<Vec<T>>,    // Vector of chunks
    chunk_size: usize,      // Size of each chunk
    len: usize,             // Total number of vectors
    dim: usize,             // Dimension of each vector
}

impl<T: Copy + Default> ChunkedVectors<T> {
    pub fn insert(&mut self, key: VectorOffsetType, value: &[T]) -> OperationResult<()> {
        let chunk_id = (key as usize) / self.chunk_size;
        let index_in_chunk = (key as usize) % self.chunk_size;
        
        // Ensure chunk exists
        while self.chunks.len() <= chunk_id {
            self.chunks.push(vec![T::default(); self.chunk_size * self.dim]);
        }
        
        // Copy vector data to chunk
        let chunk_start = index_in_chunk * self.dim;
        let chunk_end = chunk_start + self.dim;
        self.chunks[chunk_id][chunk_start..chunk_end].copy_from_slice(value);
        
        // Update length if necessary
        self.len = max(self.len, key as usize + 1);
        
        Ok(())
    }
}
```

**Findings - HNSW Index:**

Pure Rust implementation of HNSW algorithm:
- Multi-layer graph structure
- Efficient insertion and search algorithms
- Configurable parameters (M, efConstruction, ef)

**Code References:**
```rust
// From hnsw.rs - HNSW index structure
pub struct HNSWIndex {
    graph: GraphLayers,                    // Multi-layer graph
    vector_storage: Arc<AtomicRefCell<VectorStorageEnum>>,
    config: HnswGraphConfig,
}

// From graph_layers.rs - Graph layers implementation
pub struct GraphLayers {
    pub(super) hnsw_m: HnswM,              // HNSW parameters
    pub(super) links: GraphLinks,          // Graph connectivity
    pub(super) entry_points: EntryPoints,  // Entry points for search
}

// From graph_layers.rs - Search on level implementation
fn _search_on_level(
    &self,
    searcher: &mut SearchContext,
    level: usize,
    visited_list: &mut VisitedListHandle,
    points_scorer: &mut FilteredScorer,
    is_stopped: &AtomicBool,
) -> CancellableResult<()> {
    let limit = self.get_m(level);
    let mut points_ids: Vec<PointOffsetType> = Vec::with_capacity(2 * limit);

    while let Some(candidate) = searcher.candidates.pop() {
        check_process_stopped(is_stopped)?;

        if candidate.score < searcher.lower_bound() {
            break;
        }

        points_ids.clear();
        self.for_each_link(candidate.idx, level, |link| {
            if !visited_list.check(link) {
                points_ids.push(link);
            }
        });

        points_scorer
            .score_points(&mut points_ids, limit)
            .for_each(|score_point| {
                searcher.process_candidate(score_point);
                visited_list.check_and_update_visited(score_point.idx);
            });
    }

    Ok(())
}
```

**Findings - Distance Metrics:**

Trait-based design for extensibility:
- Support for common metrics (cosine, euclidean, dot product)
- Optimized implementations using SIMD where available
- Type-safe distance calculations

**Code References:**
```rust
// From spaces/simple.rs - Distance trait
pub trait Metric<T: PrimitiveVectorElement> {
    fn distance() -> Distance;
    fn similarity(v1: &[T], v2: &[T]) -> ScoreType;
    fn preprocess(vector: DenseVector) -> DenseVector;
}

// Cosine similarity implementation
pub struct CosineMetric;

impl<T: PrimitiveVectorElement> Metric<T> for CosineMetric {
    fn distance() -> Distance {
        Distance::Cosine
    }

    fn similarity(v1: &[T], v2: &[T]) -> ScoreType {
        // Use dot product on normalized vectors for cosine similarity
        DotProductMetric::similarity(v1, v2)
    }

    fn preprocess(vector: DenseVector) -> DenseVector {
        // Normalize the vector for cosine similarity
        let length: f32 = vector.iter().map(|x| x * x).sum();
        if length == 0.0 || (length - 1.0).abs() < f32::EPSILON {
            // Already normalized or zero vector
            return vector;
        }
        let norm = length.sqrt();
        vector.iter().map(|x| x / norm).collect()
    }
}
```

### 5.3 Our IndexedDB Embedding System - Detailed Analysis

**Findings - Vector Storage:**
- Vectors are stored as `ArrayBuffer` for efficiency.

**Findings - Similarity Calculation:**
- Similarity functions (e.g., `cosineSimilarity`) are implemented in pure TypeScript/JavaScript. Search is a brute-force scan.

**Findings - Link to Nodes:**
- A one-to-one relationship is maintained between nodes and embeddings via the `embedding_id` field on the node.

### 5.4 Rust Design Decisions - Detailed Analysis

**Vector Storage Format:**
- **Decision:** Use `Vec<f32>` for simplicity and compatibility, serialized with Bincode.

**Indexing Algorithm:**
- **Decision:** Start with **Brute Force** search for the MVP. It is simple, provides exact results, and is sufficient for the expected single-user scale (<10k vectors).
- Plan to add **HNSW** indexing in a later phase if performance becomes an issue as the dataset grows.

**Distance Metrics to Implement:**
1. Cosine Similarity (Must-have for semantic search)
2. Euclidean (L2) Distance (Nice-to-have)

**Similarity Search API:**
```rust
impl EmbeddedDB {
  fn create_embedding(&mut self, id: String, vector: Vec<f32>, model: String) -> Result<()>;

  fn vector_search(
    &self,
    query_vector: Vec<f32>,
    limit: usize,
  ) -> Result<Vec<(String, f32)>>; // Returns (embedding_id, score)
}```

---

## 6. Knowledge Graph Engine & Future Roadmap

### 6.1 Rust Knowledge Graph Design

**Graph Storage Decision:** Implement a **Hybrid Approach**.
- Store edges in a separate canonical table (like ArangoDB) for data integrity.
- Cache 1-hop adjacency lists (edge IDs) in node objects in memory (like our IndexedDB) for fast access.
- Implement a robust cache invalidation strategy within transactions.

### 6.2 Enterprise KG Features & Implementation Priority

**Phase 1 (MVP):**
- [x] Node and Edge storage with properties/metadata.
- [x] 1-hop edge queries (`get_edges`).
- [x] Basic BFS traversal with depth limiting (`traverse`).
- [x] Text embeddings with brute-force vector search.
- [x] Core CRUD operations.

**Phase 2 (Enhanced):**
- [ ] Multi-hop traversal with edge type filtering.
- [ ] Shortest path algorithm (`shortest_path`).
- [ ] Multimodal embeddings (support for image/audio metadata).
- [ ] Performance optimizations for indexing.

**Phase 3 (Advanced):**
- [ ] Implement HNSW indexing for vector search.
- [ ] Simple pattern matching queries (e.g., find path A -> B -> C).
- [ ] More advanced graph algorithms (e.g., PageRank).

**Phase 4 (Enterprise-Ready):**
- [ ] Cross-modal search capabilities.
- [ ] Full AQL-like query language support.
- [ ] Knowledge extraction and entity linking features.

---

## 7. Summary: Rust Implementation Roadmap

**Phase 1: Core Storage (Weeks 1-2)**
- [x] Setup `redb` as the storage backend.
- [x] Implement `Node`, `Edge`, and `Embedding` structs with Bincode serialization.
- [x] Implement basic CRUD operations for each entity.

**Phase 2: Indexing (Weeks 2-3)**
- [x] Implement in-memory indexes for Type and Edges.
- [x] Add persistence for indexes (serialize/deserialize on startup/shutdown).
- [x] Verify index performance.

**Phase 3: Query Layer (Weeks 3-4)**
- [x] Implement simple node filtering by type.
- [x] Implement edge queries (in/out/both).
- [x] Implement BFS graph traversal (`traverse`).
- [x] Implement brute-force vector similarity search.

**Phase 4: PyO3 Bindings (Weeks 4-5)**
- [x] Expose the public Rust API to Python.
- [x] Handle Python <> Rust data type serialization.
- [x] Implement robust error handling across the language boundary.
- [x] Create a high-level Python wrapper class.

**Phase 5: Integration & Testing (Weeks 5-6)**
- [x] Write comprehensive integration tests in Python.
- [x] Run performance benchmarks.
- [x] Set up cross-compilation for Windows, macOS, and Linux.

**Phase 6: Migration (Week 6+)**
- [ ] Develop a migration script to move data from IndexedDB to the new Rust-based store.
- [ ] Ensure backward compatibility where possible.

---

## 8. Open Questions & Decisions Needed

**Questions for Stakeholders:**
1. What is the expected scale of data (number of nodes, edges, embeddings) for a typical user in the first year?
2. Are there specific graph algorithms (e.g., community detection) that are critical for the MVP? (Assumption: No)
3. What are the target performance requirements for vector search (e.g., p99 latency for 10k vectors)?
4. Should we prioritize perfect compatibility with the existing IndexedDB format or optimize the new schema for performance? (Decision: Optimize, provide migration path).

**Final Design Decisions:**
1. **Storage backend:** `redb` (Confirmed)
2. **Serialization:** Bincode with JSON for properties (Confirmed)
3. **Edge storage:** Hybrid approach (Confirmed)
4. **Vector indexing:** Brute-force for MVP, HNSW planned for future. (Confirmed)

---

## 9. References

- **ArangoDB:**
  - GitHub: `https://github.com/arangodb/arangodb`
  - Docs: `https://www.arangodb.com/docs/`
- **Our Codebase:**
  - IndexedDB: `src/DB/`
- **Rust Resources:**
  - PyO3: `https://pyo3.rs/`
  - redb: `https://github.com/cberner/redb`
  - bincode: `https://github.com/bincode-org/bincode`
- **Learning Resources:**
  - Qdrant source code for vector storage patterns in Rust.
  - `petgraph` crate for graph algorithm implementations.