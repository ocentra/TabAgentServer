# Implementation Plan

- [ ] 1. Fix build issues and set up project dependencies
  - [ ] 1.1 Fix missing dependencies in common crate
    - Add missing tokio and tracing dependencies to common/Cargo.toml
    - Ensure all workspace dependencies are properly configured
    - Validate that existing tests can compile and run
    - _Requirements: 5.1_

  - [ ] 1.2 Add new dependencies for storage engine migration
    - Add libmdbx-rs and rkyv dependencies to workspace Cargo.toml
    - Update storage crate dependencies for new abstractions
    - Configure feature flags for engine selection (sled vs libmdbx)
    - _Requirements: 1.1, 2.1, 3.1_

- [ ] 2. Define StorageEngine trait abstraction
  - [ ] 2.1 Create storage engine trait with core database operations
    - Define StorageEngine trait with get, insert, remove, transaction methods
    - Define Transaction trait for transactional operations
    - Create unified error types for engine abstraction
    - _Requirements: 1.1, 1.2_

  - [ ] 2.2 Implement SledEngine compatibility wrapper
    - Create SledEngine struct wrapping existing sled::Db functionality
    - Implement StorageEngine trait for SledEngine with identical behavior
    - Create SledTransaction wrapper for sled transactions
    - _Requirements: 1.2, 1.4_

  - [ ] 2.3 Write unit tests for StorageEngine abstraction
    - Create generic test suite that works with any StorageEngine implementation
    - Test CRUD operations, transactions, and error handling
    - Validate SledEngine maintains identical behavior to current implementation
    - _Requirements: 1.1, 1.2, 5.1_

- [ ] 3. Refactor StorageManager for generic engine support
  - [ ] 3.1 Make StorageManager generic over StorageEngine trait
    - Modify StorageManager to accept generic E: StorageEngine parameter
    - Update constructor methods to accept engine instances
    - Refactor all database operations to use engine abstraction instead of direct sled calls
    - _Requirements: 1.3, 1.4_

  - [ ] 3.2 Update DatabaseCoordinator for generic engines
    - Modify DatabaseCoordinator to work with generic StorageManager instances
    - Update tier management to support different engine types
    - Maintain existing multi-tier architecture functionality
    - _Requirements: 1.3, 3.4_

  - [ ] 3.3 Preserve existing API surface and functionality
    - Ensure all public methods maintain identical signatures
    - Validate IndexManager integration continues to work
    - Test DatabaseType and TemperatureTier compatibility
    - _Requirements: 1.4, 5.5_

  - [ ] 3.4 Run existing test suite against refactored code
    - Execute all existing storage tests with SledEngine
    - Validate no regressions in functionality
    - Test multi-tier operations and indexing integration
    - _Requirements: 5.1, 5.5_

- [ ] 4. Implement rkyv serialization support
  - [ ] 4.1 Add rkyv derives to core data models
    - Add Archive, Deserialize, Serialize derives to Node enum and all variants
    - Update Edge, Embedding, and ID types with rkyv derives
    - Maintain existing serde derives during transition period
    - _Requirements: 2.1, 2.4, 2.5_

  - [ ] 4.2 Create RkyvMetadata conversion layer
    - Implement RkyvMetadata type as rkyv-compatible alternative to serde_json::Value
    - Create bidirectional conversion between serde_json::Value and RkyvMetadata
    - Update data models to use RkyvMetadata with conversion utilities
    - _Requirements: 2.3, 2.4_

  - [ ] 4.3 Update values crate for rkyv compatibility
    - Add rkyv derives to API data structures in tabagent-values crate
    - Ensure utoipa schema generation continues to work
    - Maintain serde compatibility for external API interfaces
    - _Requirements: 2.2, 2.4_

  - [ ] 4.4 Test serialization compatibility and performance
    - Validate data can be serialized/deserialized with both formats
    - Test conversion between serde and rkyv formats
    - Benchmark serialization performance improvements
    - _Requirements: 2.4, 5.4_

- [ ] 5. Implement MdbxEngine with libmdbx
  - [ ] 5.1 Create MdbxEngine implementation
    - Implement MdbxEngine struct using libmdbx Environment and Database
    - Implement StorageEngine trait with MVCC operations
    - Create MdbxTransaction for transactional operations
    - _Requirements: 3.1, 3.2, 3.4_

  - [ ] 5.2 Implement zero-copy deserialization
    - Add rkyv deserialization support to StorageManager
    - Implement engine-specific serialization detection
    - Create zero-copy read paths for MdbxEngine
    - _Requirements: 3.3, 2.1_

  - [ ] 5.3 Add comprehensive error handling
    - Implement MdbxEngineError with proper error conversion
    - Ensure compatibility with existing DbError types
    - Add proper error propagation for libmdbx operations
    - _Requirements: 3.1, 5.1_

  - [ ] 5.4 Create MdbxEngine test suite
    - Write comprehensive unit tests for MdbxEngine operations
    - Test MVCC concurrency behavior and transaction isolation
    - Validate zero-copy performance characteristics
    - _Requirements: 3.5, 5.1, 5.4_

- [ ] 6. Create data migration tooling
  - [ ] 6.1 Implement migration tool for sled to libmdbx
    - Create standalone migration utility to transfer data between engines
    - Implement data integrity validation during migration
    - Add progress reporting and error handling for large datasets
    - _Requirements: 4.1, 4.2, 4.3_

  - [ ] 6.2 Create A/B testing framework
    - Implement engine switching mechanism for testing
    - Create performance comparison utilities
    - Add rollback capabilities to previous storage implementation
    - _Requirements: 5.2, 5.3, 4.4_

  - [ ] 6.3 Validate migration data integrity
    - Test migration tool with various data sizes and types
    - Verify identical data retrieval between engines
    - Test edge cases and error recovery scenarios
    - _Requirements: 4.2, 4.3, 5.1_

- [ ] 7. Integration testing and performance validation
  - [ ] 7.1 Run complete test suite with MdbxEngine
    - Execute all existing tests using MdbxEngine instead of SledEngine
    - Validate IndexManager integration with new engine
    - Test DatabaseCoordinator multi-tier operations
    - _Requirements: 5.1, 5.5_

  - [ ] 7.2 Performance benchmarking and optimization
    - Create benchmarks comparing sled vs libmdbx performance
    - Measure zero-copy deserialization improvements
    - Test MVCC concurrency performance under load
    - _Requirements: 5.4_

  - [ ] 7.3 Production readiness validation
    - Test system stability under production-like workloads
    - Validate memory usage and resource efficiency
    - Test error recovery and edge case handling
    - _Requirements: 5.1, 5.4_

- [ ] 8. Cleanup and finalization
  - [ ] 8.1 Remove legacy sled dependencies
    - Remove sled dependency from workspace and storage crate
    - Delete SledEngine compatibility wrapper code
    - Update imports and remove unused sled-related code
    - _Requirements: 6.1, 6.2_

  - [ ] 8.2 Update documentation and examples
    - Update storage crate README with new libmdbx architecture
    - Document migration process and performance improvements
    - Update code examples and API documentation
    - _Requirements: 6.3_

  - [ ] 8.3 Final validation and cleanup
    - Run final test suite to ensure no regressions
    - Validate all legacy code has been removed
    - Perform final performance validation
    - _Requirements: 6.4, 5.1_