# Requirements Document

## Introduction

This feature implements native messaging capabilities in Rust to enable secure communication between Chrome extensions and the TabAgent server. The native messaging system will provide 100% functional parity with the existing API and WebRTC implementations, supporting all 36+ endpoints through Chrome's native messaging protocol. The implementation will follow the exact architectural patterns established in the API and WebRTC crates, including the RouteHandler trait system and compile-time enforcement rules.

## Glossary

- **Native_Messaging_Host**: The Rust-based application that handles communication with Chrome extensions via stdin/stdout using Chrome's native messaging protocol
- **Chrome_Extension**: Browser extension that communicates with the Native_Messaging_Host through Chrome's native messaging API
- **Route_Handler**: Component implementing the RouteHandler trait for processing specific endpoint requests (identical to API/WebRTC patterns)
- **Message_Router**: Component responsible for routing incoming messages to appropriate RouteHandler implementations
- **JSON_Protocol**: Chrome's native messaging protocol using length-prefixed JSON messages
- **TabAgent_Server**: The main Rust server application providing indexing, caching, and API services
- **Request_Validator**: Component that validates incoming requests using the same validation rules as API/WebRTC routes
- **Response_Formatter**: Component that formats responses according to Chrome's native messaging protocol

## Requirements

### Requirement 1

**User Story:** As a Chrome extension developer, I want to access all TabAgent API endpoints through native messaging, so that I have complete feature parity with HTTP and WebRTC clients.

#### Acceptance Criteria

1. THE Native_Messaging_Host SHALL support all 36+ endpoints available in the API crate including chat completions, embeddings, models, RAG, reranking, system info, stats, resources, management, sessions, generation control, and parameters
2. THE Native_Messaging_Host SHALL support all WebRTC-equivalent endpoints including video streaming, audio streaming, and media controls
3. THE Native_Messaging_Host SHALL implement identical request/response schemas for each endpoint as defined in the API and WebRTC crates
4. THE Native_Messaging_Host SHALL maintain OpenAI compatibility for endpoints marked as openai_compatible in the API crate
5. THE Native_Messaging_Host SHALL support all endpoint-specific features including streaming, binary data, authentication requirements, and rate limiting tiers

### Requirement 2

**User Story:** As a Chrome extension developer, I want native messaging to follow Chrome's protocol specification, so that my extension can communicate reliably with the TabAgent server.

#### Acceptance Criteria

1. WHEN a Chrome extension sends a message via native messaging, THE Native_Messaging_Host SHALL receive length-prefixed JSON messages through stdin according to Chrome's specification
2. THE Native_Messaging_Host SHALL parse the 4-byte little-endian length header followed by UTF-8 JSON payload
3. THE Native_Messaging_Host SHALL respond through stdout using the same length-prefixed JSON format
4. THE Native_Messaging_Host SHALL handle message framing correctly for messages up to Chrome's maximum size limit
5. IF a malformed message is received, THEN THE Native_Messaging_Host SHALL send a properly formatted error response and continue processing subsequent messages

### Requirement 3

**User Story:** As a developer maintaining the TabAgent codebase, I want the native messaging implementation to follow identical architectural patterns as API and WebRTC crates, so that it integrates seamlessly and maintains code quality standards.

#### Acceptance Criteria

1. THE Native_Messaging_Host SHALL implement the RouteHandler trait for each endpoint with identical metadata, validation, and test case requirements
2. THE Native_Messaging_Host SHALL use the enforce_route_handler! macro to ensure compile-time rule compliance for all routes
3. THE Native_Messaging_Host SHALL implement identical error handling using the same error types and patterns as API/WebRTC crates
4. THE Native_Messaging_Host SHALL use the same logging patterns with request_id tracing for all operations
5. THE Native_Messaging_Host SHALL follow the same crate structure with routes/, error.rs, traits.rs, and route_trait.rs organization

### Requirement 4

**User Story:** As a Chrome extension developer, I want native messaging to provide the same validation and error handling as HTTP API endpoints, so that I receive consistent and reliable error messages.

#### Acceptance Criteria

1. THE Native_Messaging_Host SHALL implement identical request validation for each endpoint using the same ValidationRule implementations
2. THE Native_Messaging_Host SHALL return the same error types and messages as the corresponding API endpoints
3. THE Native_Messaging_Host SHALL enforce the same parameter ranges, required fields, and business logic constraints
4. THE Native_Messaging_Host SHALL provide the same detailed error context including field names and validation messages
5. THE Native_Messaging_Host SHALL maintain the same rate limiting and authentication enforcement as API endpoints

### Requirement 5

**User Story:** As a TabAgent server administrator, I want native messaging to integrate with existing services using the same interfaces, so that Chrome extensions access identical functionality as other clients.

#### Acceptance Criteria

1. THE Native_Messaging_Host SHALL use the same AppStateProvider trait interface for request handling as the API crate
2. THE Native_Messaging_Host SHALL route all requests through tabagent_values::RequestValue and ResponseValue types
3. THE Native_Messaging_Host SHALL maintain identical request/response transformations as implemented in API route handlers
4. THE Native_Messaging_Host SHALL support the same concurrent request handling and state management patterns
5. THE Native_Messaging_Host SHALL integrate with the same backend services (ONNX, GGUF, Python) through the unified handler interface

### Requirement 6

**User Story:** As a Chrome extension user, I want native messaging to be performant and reliable, so that browser interactions with TabAgent are responsive and stable.

#### Acceptance Criteria

1. THE Native_Messaging_Host SHALL process simple requests (health, system info) with response times under 50 milliseconds
2. THE Native_Messaging_Host SHALL handle concurrent requests from multiple Chrome extension instances without blocking
3. THE Native_Messaging_Host SHALL implement proper message queuing and backpressure handling for high-throughput scenarios
4. THE Native_Messaging_Host SHALL gracefully handle Chrome extension disconnections and reconnections
5. THE Native_Messaging_Host SHALL maintain stable operation during extended browser sessions with proper resource cleanup

### Requirement 7

**User Story:** As a security-conscious user, I want native messaging to implement the same security controls as HTTP API endpoints, so that Chrome extensions cannot bypass security measures.

#### Acceptance Criteria

1. THE Native_Messaging_Host SHALL implement the same authentication requirements as specified in route metadata (requires_auth field)
2. THE Native_Messaging_Host SHALL enforce the same rate limiting tiers as defined in API route metadata
3. THE Native_Messaging_Host SHALL validate Chrome extension origins and maintain audit logs of all interactions
4. THE Native_Messaging_Host SHALL implement the same authorization checks for protected endpoints
5. IF security violations are detected, THEN THE Native_Messaging_Host SHALL log security events and reject requests with appropriate error responses