# Requirements Document

## Introduction

This document specifies the requirements for integrating Model Context Protocol (MCP) as a fourth transport layer into the existing TabAgent server architecture. MCP will enable AI assistants (Cursor, Claude Desktop, etc.) to directly query and interact with the TabAgent system for debugging, monitoring, and data access.

**Current State Analysis:**
- Server crate orchestrates HTTP API, WebRTC, and Native Messaging transports
- Existing logging infrastructure in `common::logging` with `LogEntry`, `LogLevel`, `LogSource`
- Storage engine migration planned to use `libmdbx`/`rkyv` for zero-copy operations
- AppState provides unified business logic across all transport layers
- Standalone MCP log server exists but needs integration into main architecture

**Target State:**
- MCP integrated as fourth transport layer alongside HTTP, WebRTC, Native Messaging
- MCP tools provide access to logs, database queries, model operations, and system monitoring
- Leverage existing storage layer and logging infrastructure
- Support for multiple MCP servers (logs, models, database, etc.) under unified architecture
- Zero-copy data access using `rkyv` serialization from storage engine migration

## Glossary

- **MCP_Transport**: Model Context Protocol transport layer integrated into the main server
- **MCP_Server**: Individual MCP server instance handling specific tool categories (logs, models, database)
- **MCP_Tool**: Specific function exposed via MCP (query_logs, get_model_info, search_database)
- **MCP_Manager**: Component managing multiple MCP servers and routing requests
- **Transport_Layer**: Communication interface (HTTP, WebRTC, Native Messaging, MCP)
- **AppState**: Unified business logic provider shared across all transport layers
- **Storage_Engine**: Abstract database interface from storage engine migration
- **Zero_Copy_Access**: Direct memory access to serialized data without deserialization overhead

## Requirements

### Requirement 1

**User Story:** As a system architect, I want MCP integrated as a fourth transport layer, so that AI assistants can access TabAgent functionality through a standardized protocol.

#### Acceptance Criteria

1. THE server SHALL support MCP as a fourth transport layer alongside HTTP, WebRTC, and Native Messaging
2. THE MCP_Transport SHALL use the same AppState instance as other transport layers for unified business logic
3. THE server SHALL support running MCP in combination with other transports (e.g., HTTP + MCP, All transports)
4. THE MCP_Transport SHALL be configurable via CLI arguments and server configuration
5. THE server SHALL handle MCP stdio transport for AI assistant integration

### Requirement 2

**User Story:** As an AI assistant, I want to access TabAgent logs through MCP tools, so that I can help users debug issues and analyze system behavior.

#### Acceptance Criteria

1. THE MCP_Server SHALL provide query_logs tool for filtering and retrieving log entries
2. THE MCP_Server SHALL provide get_log_stats tool for log analytics and summaries
3. THE MCP_Server SHALL provide clear_logs tool for log management operations
4. THE log tools SHALL use the existing Storage_Engine to persist logs instead of in-memory buffers
5. THE log tools SHALL support all existing LogLevel, LogSource, and LogQuery filtering capabilities

### Requirement 3

**User Story:** As an AI assistant, I want to access TabAgent's database and models through MCP tools, so that I can provide comprehensive system analysis and troubleshooting.

#### Acceptance Criteria

1. THE MCP_Server SHALL provide search_database tool for querying nodes, edges, and embeddings
2. THE MCP_Server SHALL provide get_model_info tool for retrieving loaded model information
3. THE MCP_Server SHALL provide get_system_stats tool for performance and resource monitoring
4. THE database tools SHALL use the existing Storage_Engine abstraction for data access
5. THE tools SHALL support zero-copy data access when using rkyv serialization

### Requirement 4

**User Story:** As a developer, I want MCP to leverage the existing storage infrastructure, so that logs and data are persisted and accessible across system restarts.

#### Acceptance Criteria

1. THE MCP_Transport SHALL use the existing Storage_Engine from the storage engine migration
2. THE logs SHALL be stored in a dedicated database tree/table for persistence
3. THE MCP tools SHALL support both sled (current) and libmdbx (target) storage engines
4. THE system SHALL maintain log data across server restarts and transport mode changes
5. THE MCP tools SHALL benefit from zero-copy reads when using rkyv serialization

### Requirement 5

**User Story:** As a system administrator, I want MCP to integrate seamlessly with the existing server architecture, so that it doesn't disrupt current functionality or require separate deployment.

#### Acceptance Criteria

1. THE MCP_Transport SHALL be managed by the existing server binary without requiring separate processes
2. THE MCP configuration SHALL be integrated into the existing ServerConfig and CLI arguments
3. THE MCP_Transport SHALL support the same error handling and logging patterns as other transports
4. THE system SHALL maintain backward compatibility with existing transport configurations
5. THE MCP integration SHALL not affect the performance or stability of other transport layers

### Requirement 6

**User Story:** As a quality assurance engineer, I want comprehensive testing and validation for MCP integration, so that the new transport layer maintains system reliability.

#### Acceptance Criteria

1. THE MCP_Transport SHALL pass comprehensive unit and integration tests
2. THE MCP tools SHALL be validated against real AI assistant integrations (Cursor, Claude Desktop)
3. THE system SHALL maintain all existing functionality when MCP is enabled or disabled
4. THE MCP integration SHALL be benchmarked for performance impact on other transport layers
5. THE MCP tools SHALL handle error conditions gracefully and provide meaningful error messages