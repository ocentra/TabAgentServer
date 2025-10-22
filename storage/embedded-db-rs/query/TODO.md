# Query Crate - TODO

## Phase 3: Converged Query Engine - ✅ COMPLETE

### ✅ Completed

- [x] Create query crate structure
- [x] Define query models (ConvergedQuery, QueryResult, Path, etc.)
- [x] Implement QueryManager core logic
- [x] Implement structural filter pipeline with index integration
- [x] Implement graph filter pipeline with BFS traversal
- [x] Implement semantic search integration with candidate filtering
- [x] Implement find_shortest_path API
- [x] Write unit tests
- [x] Write integration tests
- [x] Write comprehensive documentation

## Future Enhancements

### High Priority

- [ ] **Complex Filter Logic**  
  Support nested AND/OR expressions for structural filters
  - Design filter tree structure
  - Implement evaluation engine
  - Add tests for complex queries

- [ ] **Range Queries**  
  Support comparison operators on indexed fields
  - Implement B-tree index for numeric/timestamp fields
  - Add GreaterThan, LessThan, Between operators
  - Update structural filter evaluation

- [ ] **Query Optimization**  
  Cost-based query planning
  - Collect index statistics
  - Estimate filter selectivity
  - Reorder filter execution for optimal performance

### Medium Priority

- [ ] **Pagination Improvements**  
  Cursor-based pagination for large result sets
  - Design cursor format
  - Implement stateless cursor encoding/decoding
  - Add cursor-based query API

- [ ] **Multi-Hop Graph Patterns**  
  Cypher-like pattern matching
  - Design pattern syntax
  - Implement pattern matcher
  - Support variable-length paths

- [ ] **Result Caching**  
  Cache frequently-used query results
  - Design cache key format
  - Implement LRU eviction policy
  - Invalidate on data changes

### Low Priority

- [ ] **Query Explain Plan**  
  Debugging and optimization tool
  - Show filter execution order
  - Display candidate set sizes at each stage
  - Estimate query cost

- [ ] **Aggregations**  
  COUNT, SUM, AVG, MIN, MAX on result sets
  - Define aggregation functions
  - Implement efficient aggregation
  - Support GROUP BY

- [ ] **Full-Text Search Integration**  
  Combine with vector search for hybrid retrieval
  - Integrate Tantivy or similar
  - Implement BM25 scoring
  - Fuse with semantic scores

## Performance Benchmarks to Add

- [ ] Benchmark: 1M nodes, structural filter → target < 5ms
- [ ] Benchmark: 100K nodes, graph traversal (depth 3) → target < 10ms
- [ ] Benchmark: Converged query (filter + graph + semantic) → target < 15ms
- [ ] Benchmark: Shortest path on dense graph → target < 20ms

## Code Quality

- [x] All public APIs documented
- [x] No `.unwrap()` in production code
- [x] Proper error handling with `QueryError`
- [x] Follows Rust Architecture Guidelines (RAG)
- [ ] Add property-based tests with `proptest`
- [ ] Add fuzzing tests for query parsing
- [ ] Benchmark suite for regression testing

## Known Limitations

1. **Filter Operators**: Currently only `Equals` is efficiently supported. `NotEquals` and comparison operators require different index structures.

2. **Graph Traversal**: Currently uses simple BFS. For very deep or wide graphs, may need optimization (bidirectional search, pruning).

3. **Pagination**: Current offset-based pagination is inefficient for large offsets. Need cursor-based approach.

4. **Filter AND/OR**: All structural filters are AND'd together. No support for complex boolean expressions.

## Notes

- The two-stage pipeline (accurate filtering → semantic ranking) is the core architectural decision and should not be changed without careful consideration.
- Query performance is heavily dependent on the quality of the indexes provided by the `indexing` crate.
- All query operations are read-only; mutations should go through the `storage` crate directly.

