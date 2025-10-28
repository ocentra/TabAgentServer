# Implementation Plan

- [x] 1. Set up native messaging crate structure and core interfaces
  - [x] Create Cargo.toml with dependencies matching API/WebRTC crates (tokio, serde, async-trait, anyhow, thiserror, tracing, uuid, chrono)
  - [x] Create lib.rs with public API exports and crate documentation
  - [x] Create main.rs as binary entry point for native messaging host
  - [x] Define core module structure (config, error, traits, route_trait, protocol, router, middleware, routes)
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

- [x] 2. Implement Chrome native messaging protocol handler


  - [x] 2.1 Create protocol.rs with message parsing and formatting

    - Implement length-prefixed JSON message parsing from stdin
    - Implement length-prefixed JSON message writing to stdout
    - Add message size validation (1MB Chrome limit)
    - Handle protocol-level errors and malformed messages
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_
  
  - [x] 2.2 Define message data structures

    - Create IncomingMessage struct for Chrome extension requests
    - Create OutgoingMessage struct for responses to Chrome extensions
    - Create ErrorResponse struct for error details
    - Add proper serialization/deserialization attributes
    - _Requirements: 2.1, 2.2, 2.4_
  
  - [x] 2.3 Write protocol unit tests

    - Test message parsing with valid length-prefixed JSON
    - Test message formatting to stdout
    - Test error handling for malformed messages
    - Test maximum message size enforcement
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [x] 3. Create error handling system matching API/WebRTC patterns


  - [x] 3.1 Define NativeMessagingError enum

    - Create error hierarchy (Protocol, Validation, RouteNotFound, Internal, Backend)
    - Implement Display and Error traits
    - Add error conversion from ApiError and WebRtcError
    - _Requirements: 4.1, 4.2, 4.3, 4.4_
  
  - [x] 3.2 Implement error response formatting

    - Create consistent error response JSON format
    - Add error code mapping and categorization
    - Implement detailed error context for validation failures
    - _Requirements: 4.1, 4.2, 4.4_
  
  - [x] 3.3 Write error handling tests

    - Test error conversion from backend errors
    - Test error response JSON formatting
    - Test error logging and tracing
    - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [x] 4. Implement route handler trait system with compile-time enforcement


  - [x] 4.1 Create route_trait.rs with NativeMessagingRoute trait

    - Define trait with Request/Response associated types
    - Add metadata(), validate_request(), handle(), and test_cases() methods
    - Create RouteMetadata struct matching API/WebRTC patterns
    - Add compile-time verification methods
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_
  
  - [x] 4.2 Create enforcement macros

    - Implement enforce_native_messaging_route! macro
    - Add compile-time checks for documentation, tests, and validation
    - Create route registration helpers
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_
  
  - [x] 4.3 Define validation rules and common validators

    - Create ValidationRule trait for reusable validation logic
    - Implement common validators (NotEmpty, InRange, VecNotEmpty)
    - Add field-specific validation with context-aware error messages
    - _Requirements: 4.1, 4.2, 4.3, 4.4_
  
  - [x] 4.4 Write trait system tests

    - Test route metadata validation
    - Test compile-time enforcement macros
    - Test validation rule implementations
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

- [x] 5. Create message router and dispatch system


  - [x] 5.1 Implement MessageRouter struct

    - Create route registration system using HashMap<String, RouteDispatcher>
    - Add route dispatch logic with request/response transformation
    - Implement concurrent request handling
    - Add request_id tracking and logging
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_
  
  - [x] 5.2 Create RouteDispatcher trait

    - Define trait for type-erased route handling
    - Implement dispatcher for each route type
    - Add error handling and response formatting
    - _Requirements: 5.1, 5.2, 5.3_
  
  - [x] 5.3 Write router tests

    - Test route registration and dispatch
    - Test concurrent request handling
    - Test error propagation and logging
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 6. Implement core system routes (health, system, stats)


  - [x] 6.1 Create health route handler


    - Implement HealthRoute struct with NativeMessagingRoute trait
    - Add identical request/response schemas as API health route
    - Implement validation and error handling
    - Add comprehensive test cases
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [x] 6.2 Create system info route handler

    - Implement SystemRoute struct matching API system route
    - Add system information gathering and response formatting
    - Implement proper validation and error handling
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [x] 6.3 Create stats route handler

    - Implement StatsRoute struct matching API stats route
    - Add performance statistics collection and formatting
    - Implement validation and error handling
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [x] 6.4 Write system route tests

    - Test health route with mock state provider
    - Test system info route response format
    - Test stats route data collection
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [x] 7. Implement AI/ML routes (chat, embeddings, generate)
  - [x] 7.1 Create chat completions route handler
    - Implement ChatRoute struct with identical schemas as API chat route
    - Add OpenAI-compatible request/response handling
    - Implement streaming support for chat completions
    - Add comprehensive validation (model, messages, temperature, max_tokens, top_p)
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [x] 7.2 Create embeddings route handler
    - Implement EmbeddingsRoute struct matching API embeddings route
    - Add text embedding request/response handling
    - Implement validation for input text and model parameters
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [x] 7.3 Create text generation route handler
    - Implement GenerateRoute struct matching API generate route
    - Add OpenAI-compatible completions endpoint
    - Implement validation for generation parameters
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [x] 7.4 Create responses route handler (alternative format)
    - Implement ResponsesRoute struct matching API responses route
    - Add flexible input handling (string or messages array)
    - Implement parameter validation and response formatting
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [x] 7.5 Write AI/ML route tests
    - Test chat route with various parameter combinations
    - Test embeddings route with different input formats
    - Test generation route validation and responses
    - Test responses route flexible input handling
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [x] 8. Implement model management routes
  - [x] 8.1 Create model listing route handler
    - Implement ListModelsRoute struct matching API models route
    - Add OpenAI-compatible model listing response
    - Implement proper model information formatting
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [x] 8.2 Create model loading/unloading route handlers
    - Implement LoadModelRoute and UnloadModelRoute structs
    - Add model loading validation and error handling
    - Implement proper resource management responses
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [ ] 8.3 Create model info route handler
    - Implement ModelInfoRoute struct for individual model details
    - Add model metadata and status information
    - Implement validation for model identifiers
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [ ] 8.4 Write model management tests
    - Test model listing with various states
    - Test model loading/unloading operations
    - Test model info retrieval and formatting
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [ ] 9. Implement RAG and reranking routes
  - [ ] 9.1 Create RAG query route handler
    - Implement RagRoute struct matching API rag route
    - Add document retrieval and augmented generation
    - Implement validation for query parameters and context
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [ ] 9.2 Create reranking route handler
    - Implement RerankRoute struct matching API rerank route
    - Add document reranking request/response handling
    - Implement validation for documents and query parameters
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [ ] 9.3 Create extended RAG routes
    - Implement SemanticSearchRoute, SimilarityRoute, EvaluateEmbeddingsRoute
    - Implement ClusterRoute and RecommendRoute matching API rag_extended routes
    - Add comprehensive validation for each route type
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [ ] 9.4 Write RAG route tests
    - Test RAG query with various document sets
    - Test reranking with different scoring methods
    - Test extended RAG functionality
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [ ] 10. Implement session and parameter management routes
  - [ ] 10.1 Create session management route handlers
    - Implement GetHistoryRoute and SaveMessageRoute structs
    - Add session history retrieval and message saving
    - Implement validation for session IDs and message formats
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [ ] 10.2 Create parameter management route handlers
    - Implement GetParamsRoute and SetParamsRoute structs
    - Add generation parameter retrieval and configuration
    - Implement validation for parameter values and ranges
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [ ] 10.3 Create generation control route handlers
    - Implement StopGenerationRoute and GetHaltStatusRoute structs
    - Add generation stopping and status checking
    - Implement proper control flow and state management
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [ ] 10.4 Write session and parameter tests
    - Test session history operations
    - Test parameter get/set functionality
    - Test generation control operations
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [ ] 11. Implement resource and management routes
  - [ ] 11.1 Create resource monitoring route handlers
    - Implement GetResourcesRoute, EstimateMemoryRoute, CompatibilityRoute structs
    - Add system resource monitoring and estimation
    - Implement validation for resource queries and estimates
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [ ] 11.2 Create model management route handlers
    - Implement PullModelRoute, DeleteModelRoute, GetLoadedModelsRoute structs
    - Implement SelectModelRoute, GetEmbeddingModelsRoute, GetRecipesRoute structs
    - Implement GetRegisteredModelsRoute struct
    - Add comprehensive model lifecycle management
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [ ] 11.3 Write resource and management tests
    - Test resource monitoring and estimation
    - Test model management operations
    - Test model selection and configuration
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [x] 12. Create native messaging host binary and main loop
  - [x] 12.1 Implement main.rs binary entry point
    - Create command-line argument parsing for configuration
    - Initialize logging and tracing infrastructure
    - Set up application state and route registration
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_
  
  - [x] 12.2 Implement main message processing loop
    - Create stdin/stdout message processing loop
    - Add graceful shutdown handling and resource cleanup
    - Implement concurrent request processing with proper backpressure
    - Add error recovery and connection stability
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_
  
  - [x] 12.3 Add configuration and middleware
    - Implement configuration loading from files and environment
    - Add request/response middleware for logging and metrics
    - Implement rate limiting and authentication middleware
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_
  
  - [x] 12.4 Write integration tests
    - Test full message processing workflow
    - Test concurrent request handling
    - Test error recovery and stability
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [x] 13. Add security and authentication features
  - [x] 13.1 Implement Chrome extension origin validation
    - Add origin checking for incoming requests
    - Implement extension ID validation and allowlisting
    - Add security event logging and monitoring
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_
  
  - [x] 13.2 Implement rate limiting system
    - Add rate limiting tiers matching API route metadata
    - Implement per-extension rate limiting and quotas
    - Add rate limit exceeded error responses
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_
  
  - [x] 13.3 Add authentication and authorization
    - Implement authentication mechanisms for protected routes
    - Add authorization checks based on route metadata
    - Implement audit logging for all security events
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_
  
  - [x] 13.4 Write security tests
    - Test origin validation and rejection
    - Test rate limiting enforcement
    - Test authentication and authorization
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

- [x] 14. Add streaming and binary data support
  - [x] 14.1 Implement streaming response handling
    - Add support for streaming chat completions and generation
    - Implement proper flow control and backpressure handling
    - Add streaming error recovery and cleanup
    - _Requirements: 1.5, 6.2, 6.3_
  
  - [x] 14.2 Add binary data support for media routes
    - Implement binary payload handling for video/audio streams
    - Add proper encoding/decoding for binary data in JSON messages
    - Implement size limits and validation for binary payloads
    - _Requirements: 1.5, 6.2, 6.3_
  
  - [x] 14.3 Write streaming and binary tests
    - Test streaming response handling
    - Test binary data encoding/decoding
    - Test flow control and error recovery
    - _Requirements: 1.5, 6.2, 6.3_

- [x] 15. Final integration and testing
  - [x] 15.1 Complete route registration and verification
    - Register all implemented routes with the message router
    - Verify compile-time enforcement for all routes
    - Add route discovery and documentation generation
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  
  - [x] 15.2 Add comprehensive logging and monitoring
    - Implement structured logging with request_id tracing
    - Add performance metrics and monitoring hooks
    - Implement health checks and status reporting
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_
  
  - [x] 15.3 Create Chrome extension manifest and test extension
    - Create native messaging host manifest file
    - Build simple test Chrome extension for validation
    - Add end-to-end testing with real Chrome extension
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_
  
  - [x] 15.4 Write end-to-end tests
    - Test complete workflow from Chrome extension to backend
    - Test all routes with real Chrome extension communication
    - Test error handling and recovery scenarios
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.5_