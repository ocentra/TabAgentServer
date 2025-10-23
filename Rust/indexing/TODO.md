# Indexing Crate - TODO

## ✅ Completed

### Phase 2: Three-Tier Indexing
- [x] StructuralIndex implementation (B-tree on properties)
- [x] GraphIndex implementation (adjacency lists)
- [x] VectorIndex implementation (HNSW for ANN search)
- [x] IndexManager orchestration layer
- [x] Automatic node indexing (12 node types, 12+ properties)
- [x] Automatic edge indexing (bidirectional)
- [x] Automatic embedding indexing
- [x] Query methods (get_nodes_by_property, get_outgoing/incoming_edges, search_vectors)
- [x] 18 unit tests passing
- [x] 9 doc tests passing

### Integration
- [x] Zero circular dependencies (uses common/models)
- [x] Used by storage crate for auto-indexing
- [x] sled::Tree-based persistence for structural/graph indexes
- [x] In-memory HNSW with disk persistence for vector index

## 🔄 In Progress

_Nothing currently in progress_

## 📋 Pending

### Performance & Scalability
- [ ] Batch index updates (update many nodes at once)
- [ ] Index rebuild from scratch (after schema changes)
- [ ] Hot/warm/cold index tiers (Phase 5 - memory hierarchy)
- [ ] Index compression for space efficiency
- [ ] Parallel index updates (tokio for concurrent indexing)

### Vector Index Enhancements
- [ ] Multiple HNSW indexes for different embedding dimensions (384, 768, 1536)
- [ ] Metadata filtering in vector search
- [ ] Hybrid search (combine structural + vector)
- [ ] Vector index persistence path derived from sled db location (currently uses temp)

### Query Features
- [ ] Range queries (timestamp > X, size < Y)
- [ ] Composite queries (chat_id = X AND sender = Y)
- [ ] Fuzzy search on text properties
- [ ] Aggregate queries (count, sum, avg)

### Maintenance
- [ ] Index statistics (size, entry count)
- [ ] Index validation/repair
- [ ] Incremental index optimization

### Phase 3 Integration
- [ ] Query planner integration
- [ ] Cost-based query optimization
- [ ] Multi-index query fusion

## 🚫 Blockers

### Vector Index Persistence
- **Issue**: Vector index currently uses temp path (timestamp-based)
- **Blocker**: `sled::Db` doesn't expose its path in public API
- **Workaround**: Using `std::env::temp_dir()` with unique timestamp
- **TODO**: Either add path parameter to `with_indexing()` or find alternative

## 📊 Progress

- **Phase 2 (Three-Tier Indexing)**: ✅ 100% Complete
- **Query Features**: 🟡 30% (basic queries work, advanced pending)
- **Performance**: 🟡 40% (works well, optimization pending)
- **Overall**: **STABLE** - Production-ready for basic queries, enhancements pending

